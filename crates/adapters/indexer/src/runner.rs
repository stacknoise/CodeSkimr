//! The adapter behind the [`IndexRunner`] port.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use codeskimr_application::cancel::CancelToken;
use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{
    IndexArtifact, IndexJob, IndexRunner, IndexerSpec, IndexerStatus, PLACEHOLDER_OUTPUT,
    PLACEHOLDER_PROJECT, ToolProblem,
};
use codeskimr_codes::Coded;
use tempfile::TempDir;

use crate::consts::{
    EXCERPT_BYTES, HOME_DIR_NAME, INDEX_FILE_NAME, LOG_CODE, LOG_EXCERPT, LOG_INDEXER,
    OUTPUT_DIR_PREFIX, PATH_ENV, SCRATCH_DIR_NAME, SURVIVED_CODE,
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

    fn problems(spec: &IndexerSpec, probe: &Probe<'_>) -> Vec<ToolProblem> {
        std::iter::once(&spec.indexer)
            .chain(&spec.toolchain)
            .filter_map(|tool| check_tool(tool, probe))
            .collect()
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
        let problems = Self::problems(job.spec, &probe);
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
            Ok(finished) => return self.finish(&finished, output),
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
        finished: &Finished,
        output: OutputDir,
    ) -> Result<IndexArtifact, IndexerError> {
        let index = output.index();
        if !finished.success {
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

fn launch_of(
    job: &IndexJob<'_>,
    environment: &Environment,
    output: &OutputDir,
) -> Result<Launch, IndexerError> {
    let path = environment
        .iter()
        .find(|(name, _)| name.to_string_lossy().eq_ignore_ascii_case(PATH_ENV))
        .map(|(_, value)| value.clone())
        .unwrap_or_default();
    let program =
        resolve(&job.spec.indexer.program, &path, job.permit.project_root()).ok_or_else(|| {
            IndexerError::Unavailable(vec![ToolProblem {
                program: job.spec.indexer.program.clone(),
                issue: codeskimr_application::ports::ToolIssue::Missing,
            }])
        })?;
    let root = job.permit.project_root().to_string_lossy();
    let index = output.index();
    let index_text = index.to_string_lossy();
    let replacements = [
        (PLACEHOLDER_PROJECT, root.as_ref()),
        (PLACEHOLDER_OUTPUT, index_text.as_ref()),
    ];
    let args = job
        .spec
        .args
        .iter()
        .map(|arg| OsString::from(substitute(arg, &replacements)))
        .collect();
    Ok(Launch {
        program,
        args,
        env: environment.clone(),
        cwd: job.permit.project_root().to_path_buf(),
        limits: Limits {
            timeout: job.limits.timeout,
            max_memory_bytes: job.limits.max_memory_bytes,
            max_output_bytes: job.limits.max_output_bytes,
        },
        watch_dir: Some(output.dir.path().to_path_buf()),
        note_dir: Some(output.dir.path().to_path_buf()),
    })
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
        let problems = Self::problems(spec, &probe);
        if problems.is_empty() {
            IndexerStatus::Ready
        } else {
            IndexerStatus::Unavailable(problems)
        }
    }

    fn run(&self, job: &IndexJob<'_>, cancel: &CancelToken) -> Result<IndexArtifact, CodedError> {
        self.execute(job, cancel).map_err(|error| {
            Self::log(job.spec, &error);
            CodedError::caused_by(error)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::substitute;

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
