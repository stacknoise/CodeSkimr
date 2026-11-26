//! The adapter behind the [`IndexRunner`] port.

use std::collections::{BTreeSet, HashMap};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use codeskimr_application::cancel::CancelToken;
use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{
    IndexArtifact, IndexJob, IndexRunner, IndexerSpec, IndexerStatus, PLACEHOLDER_OUTPUT,
    PLACEHOLDER_PROJECT, ToolProblem, ToolSeen,
};
use codeskimr_codes::Coded;
use codeskimr_domain::RelativePath;
use tempfile::TempDir;

use crate::consts::{
    BYTES_PER_MB, EXCERPT_BYTES, HEAP_SHARE_PERCENT, HOME_DIR_NAME, INDEX_FILE_NAME, LOG_CODE,
    LOG_EXCERPT, LOG_INDEXER, OUTPUT_DIR_PREFIX, PATH_ENV, PATH_SEPARATOR, PERCENT,
    PLACEHOLDER_HEAP_MB, PROJECT_FILE_WILDCARD, SCRATCH_DIR_NAME, SURVIVED_CODE,
};
use crate::env::{self, Environment, Folders};
use crate::error::{FailureReason, IndexerError};
use crate::orphans;
use crate::process::{Finished, Launch, Limits, RunError, Stop, run_guarded};
use crate::redact::{Redact, excerpt};
use crate::tools::{Probe, check_tool, resolve};

/// Runs indexers as child processes. The temporary output of every run lives in a folder inside
/// `data_dir` that is removed when the returned [`IndexArtifact`] is dropped.
pub struct ProcessIndexRunner {
    data_dir: PathBuf,
    redact: Redact,
    /// The environment of the application, read once; the allow list picks from it.
    source_env: Vec<(OsString, OsString)>,
    /// Set once the folders that an earlier, killed application left behind were cleaned up.
    swept: OnceLock<()>,
}

impl ProcessIndexRunner {
    /// A runner that keeps its temporary folders in `data_dir` and passes everything the indexer
    /// printed through `redact` before it is logged or put in an error.
    #[must_use]
    pub fn new(data_dir: PathBuf, redact: Redact) -> Self {
        Self {
            data_dir,
            redact,
            source_env: std::env::vars_os().collect(),
            swept: OnceLock::new(),
        }
    }

    /// The same runner, taking the variables to pick from out of `source_env` instead of the
    /// environment of this process (for tests and for a composition root that has its own).
    #[must_use]
    pub fn with_environment(mut self, source_env: Vec<(OsString, OsString)>) -> Self {
        self.source_env = source_env;
        self
    }

    fn environment(&self, spec: &IndexerSpec, folders: &Folders<'_>) -> Environment {
        env::build(self.source_env.iter().cloned(), spec, folders)
    }

    /// Asks every tool of `spec` once: what is wrong, and the version of every tool that is fine.
    fn examine(spec: &IndexerSpec, probe: &Probe<'_>) -> (Vec<ToolProblem>, Vec<ToolSeen>) {
        let mut problems = Vec::new();
        let mut found = Vec::new();
        for tool in std::iter::once(&spec.indexer).chain(&spec.toolchain) {
            match check_tool(tool, probe) {
                Ok(seen) => found.push(seen),
                Err(problem) => problems.push(problem),
            }
        }
        (problems, found)
    }

    fn excerpt(&self, finished: &Finished) -> String {
        let console = [finished.stdout_text(), finished.stderr_text()].concat();
        excerpt(&console, EXCERPT_BYTES, self.redact)
    }

    fn execute(
        &self,
        job: &IndexJob<'_>,
        cancel: &CancelToken,
    ) -> Result<IndexArtifact, IndexerError> {
        // Once per start: what an application that was killed left running and in the data folder.
        self.swept.get_or_init(|| orphans::sweep(&self.data_dir));
        let output = OutputDir::create(&self.data_dir)?;
        let environment = self.environment(
            job.spec,
            &Folders {
                scratch: &output.scratch,
                home: &output.home,
                project_root: job.permit.project_root(),
            },
        );
        let probe = Probe {
            env: &environment,
            redact: self.redact,
            neutral_dir: &output.scratch,
            project_root: job.permit.project_root(),
        };
        let (problems, _) = Self::examine(job.spec, &probe);
        if !problems.is_empty() {
            return Err(IndexerError::Unavailable(problems));
        }
        let launch = launch_of(job, &environment, &output)?;
        // Consent is read again right before the process starts: it may have been withdrawn, or
        // the project pointed elsewhere, since the permit was issued.
        job.permit.recheck().map_err(IndexerError::NotConfirmed)?;
        self.run_launched(job, &launch, cancel, output)
    }

    /// Runs the process and turns its end into the result; a leader that outlived the kill keeps
    /// the run folder for the sweep.
    fn run_launched(
        &self,
        job: &IndexJob<'_>,
        launch: &Launch,
        cancel: &CancelToken,
        output: OutputDir,
    ) -> Result<IndexArtifact, IndexerError> {
        let (error, survived) = match run_guarded(launch, cancel) {
            Ok(finished) => return self.finish(job.spec, &finished, output),
            Err(RunError::Io(cause)) => (io_failure(&cause), false),
            Err(RunError::Watch { cause, survived }) => (io_failure(&cause), survived),
            Err(RunError::Stopped { stop, survived }) => (stopped(stop), survived),
        };
        if survived {
            Self::keep_for_sweep(&job.spec.name, output);
        }
        Err(error)
    }

    /// The leader outlived the kill: the folder with the notes stays, so that the next start finds
    /// the process and ends it. Nothing of the indexer's output or of paths goes into the log.
    fn keep_for_sweep(name: &str, output: OutputDir) {
        drop(output.dir.keep());
        tracing::warn!({ LOG_INDEXER } = %name, { LOG_CODE } = SURVIVED_CODE);
    }

    fn finish(
        &self,
        spec: &IndexerSpec,
        finished: &Finished,
        output: OutputDir,
    ) -> Result<IndexArtifact, IndexerError> {
        let index = output.index();
        if !finished.success {
            if ran_out_of_memory(spec, finished) {
                return Err(IndexerError::MemoryLimit);
            }
            return Err(failed(
                FailureReason::Exit(finished.code),
                self.excerpt(finished),
            ));
        }
        // A link that the indexer put in place of the index is not an index.
        if !index.symlink_metadata().is_ok_and(|meta| meta.is_file()) {
            return Err(failed(FailureReason::NoIndex, self.excerpt(finished)));
        }
        Ok(IndexArtifact::new(index, Box::new(output.dir)))
    }

    fn log(spec: &IndexerSpec, error: &IndexerError) {
        let excerpt = match error {
            IndexerError::Failed { excerpt, .. } => excerpt.as_str(),
            _ => Default::default(),
        };
        // A cancel by the user or a missing tool is routine, not a warning.
        if matches!(
            error,
            IndexerError::Cancelled | IndexerError::Unavailable(_)
        ) {
            tracing::info!({ LOG_INDEXER } = %spec.name, { LOG_CODE } = error.code().id());
            return;
        }
        tracing::warn!({ LOG_INDEXER } = %spec.name, { LOG_CODE } = error.code().id(), { LOG_EXCERPT } = %excerpt);
    }
}

/// Whether the console of a failed run says that the indexer used up its own memory budget.
fn ran_out_of_memory(spec: &IndexerSpec, finished: &Finished) -> bool {
    let console = [finished.stdout_text(), finished.stderr_text()].concat();
    spec.memory_markers
        .iter()
        .any(|marker| console.contains(marker.as_str()))
}

fn io_failure(cause: &std::io::Error) -> IndexerError {
    failed(FailureReason::Io(cause.to_string()), String::new())
}

fn failed(reason: FailureReason, excerpt: String) -> IndexerError {
    IndexerError::Failed { reason, excerpt }
}

const fn stopped(stop: Stop) -> IndexerError {
    match stop {
        Stop::Timeout => IndexerError::Timeout,
        Stop::Memory => IndexerError::MemoryLimit,
        Stop::Output => IndexerError::OutputTooLarge,
        Stop::Cancelled => IndexerError::Cancelled,
    }
}

/// The temporary output folder of one run: the index goes into it, the indexer's own temporary
/// files go into `scratch` below it, and `home` is the empty home of the indexer.
struct OutputDir {
    dir: TempDir,
    scratch: PathBuf,
    home: PathBuf,
}

impl OutputDir {
    fn create(data_dir: &Path) -> Result<Self, IndexerError> {
        let io = |cause: std::io::Error| {
            failed(FailureReason::OutputDir(cause.to_string()), String::new())
        };
        std::fs::create_dir_all(data_dir).map_err(io)?;
        let dir = tempfile::Builder::new()
            .prefix(OUTPUT_DIR_PREFIX)
            .tempdir_in(data_dir)
            .map_err(io)?;
        let scratch = dir.path().join(SCRATCH_DIR_NAME);
        std::fs::create_dir(&scratch).map_err(io)?;
        let home = dir.path().join(HOME_DIR_NAME);
        std::fs::create_dir(&home).map_err(io)?;
        // Without the note another instance would take the folder for abandoned after the grace
        // period and end the run; so a run that cannot be noted does not start.
        orphans::note_owner(dir.path()).map_err(io)?;
        Ok(Self { dir, scratch, home })
    }

    fn index(&self) -> PathBuf {
        self.dir.path().join(INDEX_FILE_NAME)
    }
}

/// The program of the indexer, looked up in the `PATH` of its environment.
fn program_of(job: &IndexJob<'_>, environment: &Environment) -> Result<PathBuf, IndexerError> {
    let path = environment
        .iter()
        .find(|(name, _)| name.to_string_lossy().eq_ignore_ascii_case(PATH_ENV))
        .map(|(_, value)| value.clone())
        .unwrap_or_default();
    resolve(&job.spec.indexer.program, &path, job.permit.project_root()).ok_or_else(|| {
        IndexerError::Unavailable(vec![ToolProblem {
            program: job.spec.indexer.program.clone(),
            issue: codeskimr_application::ports::ToolIssue::Missing,
        }])
    })
}

/// The arguments of the indexer with the project folder and the index file filled in.
fn args_of(workdir: &Path, output: &OutputDir, job: &IndexJob<'_>) -> Vec<OsString> {
    let root = workdir.to_string_lossy();
    let index = output.index();
    let index_text = index.to_string_lossy();
    let replacements = [
        (PLACEHOLDER_PROJECT, root.as_ref()),
        (PLACEHOLDER_OUTPUT, index_text.as_ref()),
    ];
    job.spec
        .args
        .iter()
        .map(|arg| OsString::from(substitute(arg, &replacements)))
        .collect()
}

fn launch_of(
    job: &IndexJob<'_>,
    environment: &Environment,
    output: &OutputDir,
) -> Result<Launch, IndexerError> {
    let workdir = workdir_of(job)?;
    Ok(Launch {
        program: program_of(job, environment)?,
        args: args_of(&workdir, output, job),
        env: with_memory_env(job, environment),
        cwd: workdir,
        limits: Limits {
            timeout: job.limits.timeout,
            max_memory_bytes: job.limits.max_memory_bytes,
            max_output_bytes: job.limits.max_output_bytes,
        },
        watch_dir: Some(output.dir.path().to_path_buf()),
        note_dir: Some(output.dir.path().to_path_buf()),
    })
}

/// The folder the indexer runs in: the project root, or the workspace below it (ADR 0099). A
/// workspace that is no folder inside the project (a link that leads out of it, or a folder that
/// is gone) is refused, so the indexer never runs outside the project.
fn workdir_of(job: &IndexJob<'_>) -> Result<PathBuf, IndexerError> {
    let root = job.permit.project_root();
    let Some(workspace) = job.workspace else {
        return Ok(root.to_path_buf());
    };
    let outside = || failed(FailureReason::Workspace, String::new());
    let folder = std::fs::canonicalize(root.join(workspace.as_str())).map_err(|_| outside())?;
    let base = std::fs::canonicalize(root).map_err(|_| outside())?;
    if folder.starts_with(&base) && folder.is_dir() {
        Ok(folder)
    } else {
        Err(outside())
    }
}

/// The heap share of a memory limit, in megabytes.
fn heap_mb(max_memory_bytes: u64) -> u64 {
    (max_memory_bytes / BYTES_PER_MB).saturating_mul(HEAP_SHARE_PERCENT) / PERCENT
}

/// The environment with the variables that the pack derives from the memory limit of the run.
fn with_memory_env(job: &IndexJob<'_>, environment: &Environment) -> Environment {
    let heap = heap_mb(job.limits.max_memory_bytes).to_string();
    let mut result = environment.clone();
    for (name, value) in &job.spec.memory_env {
        result.retain(|(existing, _)| !existing.to_string_lossy().eq_ignore_ascii_case(name));
        let text = substitute(value, &[(PLACEHOLDER_HEAP_MB, heap.as_str())]);
        result.push((OsString::from(name), OsString::from(text)));
    }
    result
}

/// Replaces the placeholders in one pass, so that a value that holds the text of a placeholder
/// (a project folder named `{output}`) is not replaced again.
fn substitute(arg: &str, replacements: &[(&str, &str)]) -> String {
    let mut result = String::new();
    let mut rest = arg;
    while let Some((at, placeholder, value)) = replacements
        .iter()
        .filter_map(|(placeholder, value)| {
            rest.find(placeholder).map(|at| (at, *placeholder, *value))
        })
        .min_by_key(|(at, _, _)| *at)
    {
        result.push_str(rest.get(..at).unwrap_or_default());
        result.push_str(value);
        rest = rest
            .get(at.saturating_add(placeholder.len())..)
            .unwrap_or_default();
    }
    result.push_str(rest);
    result
}

impl IndexRunner for ProcessIndexRunner {
    fn check(&self, spec: &IndexerSpec) -> IndexerStatus {
        let neutral = if self.data_dir.is_dir() {
            self.data_dir.clone()
        } else {
            std::env::temp_dir()
        };
        let root = neutral.clone();
        let environment = self.environment(
            spec,
            &Folders {
                scratch: &neutral,
                home: &neutral,
                project_root: &root,
            },
        );
        let probe = Probe {
            env: &environment,
            redact: self.redact,
            neutral_dir: &neutral,
            project_root: &root,
        };
        let (problems, found) = Self::examine(spec, &probe);
        if problems.is_empty() {
            IndexerStatus::Ready { found }
        } else {
            IndexerStatus::Unavailable { problems, found }
        }
    }

    fn locate(&self, program: &str) -> Option<String> {
        let path = self
            .source_env
            .iter()
            .find(|(name, _)| name.to_string_lossy().eq_ignore_ascii_case(PATH_ENV))
            .map(|(_, value)| value.clone())?;
        // No project here: the data folder stands in for the folder `PATH` must not lead into.
        resolve(program, &path, &self.data_dir).map(|found| found.to_string_lossy().into_owned())
    }

    fn has_project_file(&self, spec: &IndexerSpec, root: &Path) -> bool {
        spec.project_files.is_empty() || project_file_near(&spec.project_files, root)
    }

    fn workspaces(
        &self,
        spec: &IndexerSpec,
        root: &Path,
        files: &[&RelativePath],
    ) -> Vec<Option<RelativePath>> {
        if self.has_project_file(spec, root) {
            return vec![None];
        }
        outermost_project_folders(&spec.project_files, root, files)
            .into_iter()
            .map(Some)
            .collect()
    }

    fn run(&self, job: &IndexJob<'_>, cancel: &CancelToken) -> Result<IndexArtifact, CodedError> {
        self.execute(job, cancel).map_err(|error| {
            Self::log(job.spec, &error);
            CodedError::caused_by(error)
        })
    }
}

/// The outermost folders below `root` that hold one of the `wanted` files and have a source file
/// among `files` (ADR 0099). A folder inside such a folder is part of its workspace: the indexer
/// finds the members from the outer manifest. Sorted, without duplicates.
fn outermost_project_folders(
    wanted: &[String],
    root: &Path,
    files: &[&RelativePath],
) -> Vec<RelativePath> {
    let mut checked: HashMap<String, bool> = HashMap::new();
    let mut found: BTreeSet<RelativePath> = BTreeSet::new();
    for file in files {
        let folders: Vec<&str> = file.as_str().split(PATH_SEPARATOR).collect();
        let Some((_, parents)) = folders.split_last() else {
            continue;
        };
        for depth in 1..=parents.len() {
            let folder = parents.iter().take(depth).copied().collect::<Vec<_>>();
            let key = folder.join(PATH_SEPARATOR);
            let holds = *checked
                .entry(key.clone())
                .or_insert_with(|| folder_holds(wanted, &root.join(&key)));
            if holds {
                found.extend(RelativePath::new(key).ok());
                break;
            }
        }
    }
    found.into_iter().collect()
}

/// Whether `folder` itself (not a folder above it) holds one of the `wanted` files.
fn folder_holds(wanted: &[String], folder: &Path) -> bool {
    std::fs::read_dir(folder).is_ok_and(|entries| {
        entries
            .filter_map(Result::ok)
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .any(|name| wanted.iter().any(|want| matches_project_file(want, &name)))
    })
}

/// Whether one of the `wanted` files lies in `root` or in a folder above it (the indexers look
/// upwards, too). A folder that cannot be listed counts as a find: the run then reports what is
/// wrong.
fn project_file_near(wanted: &[String], root: &Path) -> bool {
    root.ancestors().any(|dir| {
        std::fs::read_dir(dir).map_or(true, |entries| {
            entries
                .filter_map(Result::ok)
                .map(|entry| entry.file_name().to_string_lossy().into_owned())
                .any(|name| wanted.iter().any(|want| matches_project_file(want, &name)))
        })
    })
}

/// Whether the file name `name` is the one `wanted` names; `*` at the start matches any prefix.
fn matches_project_file(wanted: &str, name: &str) -> bool {
    wanted
        .strip_prefix(PROJECT_FILE_WILDCARD)
        .map_or_else(|| wanted == name, |suffix| name.ends_with(suffix))
}

#[cfg(test)]
mod tests {
    use super::{
        heap_mb, matches_project_file, outermost_project_folders, project_file_near, substitute,
    };

    fn wanted(names: &[&str]) -> Vec<String> {
        names.iter().map(|name| (*name).to_owned()).collect()
    }

    fn paths(names: &[&str]) -> Vec<codeskimr_domain::RelativePath> {
        names
            .iter()
            .map(|name| codeskimr_domain::RelativePath::new(*name).expect("path"))
            .collect()
    }

    #[test]
    fn the_outermost_folder_with_a_project_file_is_the_workspace() {
        let dir = tempfile::tempdir().expect("temporary folder");
        let root = dir.path();
        for folder in ["cli/crates/inner", "tools/gen", "plain"] {
            std::fs::create_dir_all(root.join(folder)).expect("folders");
        }
        std::fs::write(root.join("cli").join("Cargo.toml"), "").expect("file");
        std::fs::write(root.join("cli/crates/inner").join("Cargo.toml"), "").expect("file");
        std::fs::write(root.join("tools/gen").join("Cargo.toml"), "").expect("file");
        let files = paths(&[
            "cli/crates/inner/src/lib.rs",
            "cli/src/main.rs",
            "tools/gen/main.rs",
            "plain/loose.rs",
            "top.rs",
        ]);
        let refs: Vec<&codeskimr_domain::RelativePath> = files.iter().collect();
        let found = outermost_project_folders(&wanted(&["Cargo.toml"]), root, &refs);
        assert_eq!(found, paths(&["cli", "tools/gen"]));
    }

    #[test]
    fn no_project_file_below_the_root_gives_no_workspace() {
        let dir = tempfile::tempdir().expect("temporary folder");
        std::fs::create_dir_all(dir.path().join("src")).expect("folders");
        let files = paths(&["src/a.rs"]);
        let refs: Vec<&codeskimr_domain::RelativePath> = files.iter().collect();
        assert!(outermost_project_folders(&wanted(&["Cargo.toml"]), dir.path(), &refs).is_empty());
    }

    #[test]
    fn a_project_file_in_a_subfolder_does_not_count() {
        let dir = tempfile::tempdir().expect("temporary folder");
        let root = dir.path().join("project");
        std::fs::create_dir_all(root.join("cli")).expect("folders");
        std::fs::write(root.join("cli").join("Cargo.toml"), "").expect("file");
        assert!(!project_file_near(&wanted(&["Cargo.toml"]), &root));
        std::fs::write(root.join("Cargo.toml"), "").expect("file");
        assert!(project_file_near(&wanted(&["Cargo.toml"]), &root));
    }

    #[test]
    fn a_project_file_is_found_by_suffix_and_above_the_folder() {
        let dir = tempfile::tempdir().expect("temporary folder");
        let root = dir.path().join("work").join("project");
        std::fs::create_dir_all(&root).expect("folders");
        assert!(!project_file_near(&wanted(&["*.sln"]), &root));
        std::fs::write(dir.path().join("work").join("App.sln"), "").expect("file");
        assert!(project_file_near(&wanted(&["*.sln"]), &root));
    }

    #[test]
    fn the_heap_is_three_quarters_of_the_limit() {
        assert_eq!(heap_mb(16 * 1024 * 1024 * 1024), 12_288);
        assert_eq!(heap_mb(4096 * 1024 * 1024), 3072);
    }

    #[test]
    fn an_empty_folder_has_no_project_file() {
        let dir = tempfile::tempdir().expect("temporary folder");
        assert!(!project_file_near(
            &wanted(&["Cargo.toml", "*.csproj"]),
            dir.path()
        ));
    }

    #[test]
    fn a_project_file_matches_by_name_or_suffix() {
        assert!(matches_project_file("Cargo.toml", "Cargo.toml"));
        assert!(!matches_project_file("Cargo.toml", "cargo.lock"));
        assert!(matches_project_file("*.sln", "App.sln"));
        assert!(!matches_project_file("*.sln", "App.slnx"));
    }

    #[test]
    fn placeholders_are_replaced_in_one_pass() {
        let replacements = [("{project}", "/work/{output}"), ("{output}", "/out/index")];
        assert_eq!(
            substitute("--in={project}:{output}", &replacements),
            "--in=/work/{output}:/out/index"
        );
        assert_eq!(substitute("plain", &replacements), "plain");
    }
}
