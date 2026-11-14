use std::error::Error;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use codeskimr_application::cancel::CancelToken;
use codeskimr_application::config::Config;
use codeskimr_application::error::CodedError;
use codeskimr_application::level2;
use codeskimr_application::ports::{
    ConsentRecord, ConsentStore, IndexJob, IndexLimits, IndexRunner, IndexerSpec, IndexerStatus,
    RunPermit, ToolIssue, ToolSpec, ToolVersion,
};
use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_domain::{
    FolderPath, LanguageId, Principal, Project, ProjectId, ProjectName, ProjectSource,
};
use codeskimr_indexer::{FailureReason, IndexerError, ProcessIndexRunner};
use sysinfo::{Pid, ProcessesToUpdate, System};
use tempfile::TempDir;

const SECRET: &str = "CANARY-SECRET-VALUE";

fn mask(text: &str) -> String {
    text.replace(SECRET, "[redacted]")
}

fn fake_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_fake_indexer"))
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_default()
}

fn spec(mode: &str) -> IndexerSpec {
    IndexerSpec {
        name: "fake".to_owned(),
        indexer: ToolSpec {
            program: "fake_indexer".to_owned(),
            version_args: vec!["--version".to_owned()],
            min_version: ToolVersion::parse("1.0.0").unwrap(),
        },
        toolchain: Vec::new(),
        args: vec![
            mode.to_owned(),
            "{output}".to_owned(),
            "{project}/child.pid".to_owned(),
        ],
        env_allow: vec!["EXTRA_ALLOWED".to_owned(), "EXTRA_TOKEN".to_owned()],
        env: vec![("FIXED_BY_PACK".to_owned(), "yes".to_owned())],
        install_hint: "install it".to_owned(),
    }
}

fn limits() -> IndexLimits {
    IndexLimits {
        timeout: Duration::from_secs(60),
        max_memory_bytes: 2 * 1024 * 1024 * 1024,
        max_output_bytes: 64 * 1024 * 1024,
    }
}

fn source_env(dir: &Path, inherit_path: bool) -> Vec<(OsString, OsString)> {
    let mut source: Vec<(OsString, OsString)> = std::env::vars_os()
        .filter(|(name, _)| !name.to_string_lossy().eq_ignore_ascii_case("PATH"))
        .collect();
    let real_path = std::env::var_os("PATH").unwrap_or_default();
    let mut paths = vec![dir.to_path_buf()];
    if inherit_path {
        paths.extend(std::env::split_paths(&real_path));
    }
    source.push(("PATH".into(), std::env::join_paths(paths).unwrap()));
    for (name, value) in [
        ("GITHUB_TOKEN", SECRET),
        ("CODESKIMR_MASTER", SECRET),
        ("MY_PASSWORD", SECRET),
        ("AWS_SECRET_ACCESS_KEY", SECRET),
        ("ANTHROPIC_API_KEY", SECRET),
        ("UNLISTED_VALUE", SECRET),
        ("EXTRA_TOKEN", SECRET),
        ("EXTRA_ALLOWED", "fine"),
    ] {
        source.push((name.into(), value.into()));
    }
    source
}

fn project_record(folder: &Path) -> Project {
    Project {
        id: ProjectId::new(1),
        name: ProjectName::new("app").unwrap(),
        source: ProjectSource::Folder(FolderPath::new(folder.to_str().unwrap()).unwrap()),
        layers: Vec::new(),
    }
}

/// A consent store that keeps the two flags in memory.
#[derive(Default)]
struct Flags(Mutex<(bool, bool)>);

impl ConsentStore for Flags {
    fn consent(&self, _: ProjectId) -> Result<ConsentRecord, CodedError> {
        let (enabled, confirmed) = *self.0.lock().unwrap();
        Ok(ConsentRecord { enabled, confirmed })
    }

    fn store_consent(&self, _: ProjectId, record: ConsentRecord) -> Result<(), CodedError> {
        *self.0.lock().unwrap() = (record.enabled, record.confirmed);
        Ok(())
    }
}

fn confirmed_store() -> Arc<dyn ConsentStore> {
    let store: Arc<dyn ConsentStore> = Arc::new(Flags::default());
    level2::confirm(&Principal::local_owner(), store.as_ref(), ProjectId::new(1)).unwrap();
    store
}

struct Fixture {
    store: Arc<dyn ConsentStore>,
    permit: RunPermit,
    project: TempDir,
    data: TempDir,
    runner: ProcessIndexRunner,
    language: LanguageId,
}

impl Fixture {
    fn new() -> Self {
        Self::build(TempDir::new().unwrap(), &fake_dir(), true)
    }

    /// `inherit_path` adds the `PATH` of the test process after `dir`. A test that needs `dir` to be
    /// the only place with the indexer must not: on Windows cargo puts `target/debug` into the
    /// `PATH` of every test process, and the real fake indexer lives there.
    fn build(project: TempDir, dir: &Path, inherit_path: bool) -> Self {
        let data = TempDir::new().unwrap();
        let source = source_env(dir, inherit_path);
        let runner =
            ProcessIndexRunner::new(data.path().to_path_buf(), mask).with_environment(source);
        let store = confirmed_store();
        let config = Config::defaults().unwrap();
        let permit = level2::authorize_run(
            &Principal::local_owner(),
            &config.level2,
            &store,
            &project_record(project.path()),
            &FolderPath::new("unused").unwrap(),
        )
        .unwrap();
        Self {
            store,
            permit,
            project,
            data,
            runner,
            language: LanguageId::new("csharp").unwrap(),
        }
    }

    fn run_with(
        &self,
        spec: &IndexerSpec,
        limits: IndexLimits,
        cancel: &CancelToken,
    ) -> Result<codeskimr_application::ports::IndexArtifact, CodedError> {
        let job = IndexJob {
            language: &self.language,
            spec,
            limits,
            permit: &self.permit,
        };
        self.runner.run(&job, cancel)
    }

    fn run(&self, mode: &str) -> Result<codeskimr_application::ports::IndexArtifact, CodedError> {
        self.run_with(&spec(mode), limits(), &CancelToken::new())
    }
}

fn failure_reason(error: &CodedError) -> Option<FailureReason> {
    match error.source()?.downcast_ref::<IndexerError>()? {
        IndexerError::Failed { reason, .. } => Some(reason.clone()),
        _ => None,
    }
}

/// The problem the check reports for a copy of the indexer that wants version 9.
fn too_old_issue(fixture: &Fixture) -> ToolIssue {
    let mut old = spec("ok");
    old.indexer.min_version = ToolVersion::parse("9.0.0").unwrap();
    let IndexerStatus::Unavailable(problems) = fixture.runner.check(&old) else {
        panic!("ready");
    };
    problems.into_iter().next().expect("a problem").issue
}

fn code_of<T>(result: Result<T, CodedError>) -> ErrorCode {
    result.err().map(|error| error.code()).expect("an error")
}

fn data_dir_entries(fixture: &Fixture) -> usize {
    std::fs::read_dir(fixture.data.path()).unwrap().count()
}

#[test]
fn the_index_is_returned_and_its_folder_goes_with_the_artifact() {
    let fixture = Fixture::new();
    let artifact = fixture.run("ok").unwrap();
    assert_eq!(std::fs::read(artifact.path()).unwrap(), b"index");
    assert!(artifact.path().starts_with(fixture.data.path()));
    let path = artifact.path().to_path_buf();
    drop(artifact);
    assert!(!path.exists());
    assert_eq!(data_dir_entries(&fixture), 0);
}

#[test]
fn the_environment_holds_no_secret_and_nothing_unlisted() {
    let fixture = Fixture::new();
    let artifact = fixture.run("dump-env").unwrap();
    let text = std::fs::read_to_string(artifact.path()).unwrap();
    assert!(!text.contains(SECRET), "{text}");
    assert!(text.contains("EXTRA_ALLOWED=fine"));
    assert!(text.contains("FIXED_BY_PACK=yes"));
    assert!(text.contains("PATH="));
    for name in [
        "GITHUB_TOKEN",
        "CODESKIMR_MASTER",
        "UNLISTED_VALUE",
        "EXTRA_TOKEN",
    ] {
        assert!(!text.contains(name), "{name} leaked");
    }
    let temp = text
        .lines()
        .find(|line| {
            line.to_uppercase().starts_with("TMPDIR=") || line.to_uppercase().starts_with("TEMP=")
        })
        .expect("a temporary folder");
    assert!(temp.contains(&fixture.data.path().to_string_lossy().into_owned()));
}

fn is_gone(pid: u32) -> bool {
    let mut system = System::new().unwrap();
    system.refresh_processes(ProcessesToUpdate::All, true);
    system.process(Pid::from_u32(pid)).is_none_or(|process| {
        format!("{:?}", process.status())
            .to_lowercase()
            .contains("zombie")
    })
}

fn wait_until_gone(pid: u32) -> bool {
    let until = Instant::now() + Duration::from_secs(15);
    while Instant::now() < until {
        if is_gone(pid) {
            return true;
        }
        thread::sleep(Duration::from_millis(100));
    }
    false
}

fn child_pid(fixture: &Fixture) -> u32 {
    let file = fixture.project.path().join("child.pid");
    let until = Instant::now() + Duration::from_secs(10);
    while !file.is_file() && Instant::now() < until {
        thread::sleep(Duration::from_millis(50));
    }
    std::fs::read_to_string(file)
        .unwrap()
        .trim()
        .parse()
        .unwrap()
}

#[test]
fn a_timeout_kills_the_indexer_and_the_child_it_started() {
    let fixture = Fixture::new();
    let short = IndexLimits {
        timeout: Duration::from_secs(2),
        ..limits()
    };
    let started = Instant::now();
    let result = fixture.run_with(&spec("hang-with-child"), short, &CancelToken::new());
    assert_eq!(code_of(result), ErrorCode::IndexerTimeout);
    assert!(started.elapsed() < Duration::from_secs(30));
    assert!(wait_until_gone(child_pid(&fixture)), "the child survived");
    assert_eq!(data_dir_entries(&fixture), 0, "the output folder stayed");
}

#[test]
fn cancelling_kills_the_indexer_and_the_child_it_started() {
    let fixture = Fixture::new();
    let token = CancelToken::new();
    let other = token.clone();
    let canceller = thread::spawn(move || {
        thread::sleep(Duration::from_millis(1500));
        other.cancel();
    });
    let result = fixture.run_with(&spec("hang-with-child"), limits(), &token);
    canceller.join().unwrap();
    assert_eq!(code_of(result), ErrorCode::ScanCancelled);
    assert!(wait_until_gone(child_pid(&fixture)), "the child survived");
}

#[test]
fn too_much_console_output_stops_the_indexer() {
    let fixture = Fixture::new();
    let small = IndexLimits {
        max_output_bytes: 1024 * 1024,
        ..limits()
    };
    let result = fixture.run_with(&spec("flood"), small, &CancelToken::new());
    assert_eq!(code_of(result), ErrorCode::IndexerOutputTooLarge);
}

#[test]
fn too_large_files_in_the_output_folder_stop_the_indexer() {
    let fixture = Fixture::new();
    let small = IndexLimits {
        max_output_bytes: 1024 * 1024,
        ..limits()
    };
    let result = fixture.run_with(&spec("big-file"), small, &CancelToken::new());
    assert_eq!(code_of(result), ErrorCode::IndexerOutputTooLarge);
}

#[test]
fn too_much_memory_stops_the_indexer() {
    let fixture = Fixture::new();
    let small = IndexLimits {
        max_memory_bytes: 100 * 1024 * 1024,
        ..limits()
    };
    let result = fixture.run_with(&spec("memory"), small, &CancelToken::new());
    assert_eq!(code_of(result), ErrorCode::IndexerMemoryLimit);
}

#[test]
fn a_failing_indexer_is_reported_with_its_redacted_output() {
    let fixture = Fixture::new();
    let error = fixture.run("fail").unwrap_err();
    assert_eq!(error.code(), ErrorCode::IndexerFailed);
    let cause = error
        .source()
        .and_then(|s| s.downcast_ref::<IndexerError>());
    let Some(IndexerError::Failed { reason, excerpt }) = cause else {
        panic!("not a failure: {cause:?}");
    };
    assert_eq!(*reason, FailureReason::Exit(Some(3)));
    assert!(excerpt.contains("restore failed"), "{excerpt}");
    assert!(!excerpt.contains(SECRET), "{excerpt}");
    assert!(!format!("{error:?}").contains(SECRET));
}

#[test]
fn an_indexer_that_writes_no_index_fails() {
    let fixture = Fixture::new();
    let error = fixture.run("no-index").unwrap_err();
    assert_eq!(failure_reason(&error), Some(FailureReason::NoIndex));
}

#[test]
fn a_missing_indexer_is_unavailable_and_nothing_runs() {
    let fixture = Fixture::new();
    let mut missing = spec("ok");
    missing.indexer.program = "no_such_indexer_for_codeskimr".to_owned();
    let status = fixture.runner.check(&missing);
    let IndexerStatus::Unavailable(problems) = status else {
        panic!("ready");
    };
    assert_eq!(problems.len(), 1);
    assert_eq!(
        problems.first().map(|p| &p.issue),
        Some(&ToolIssue::Missing)
    );
    let result = fixture.run_with(&missing, limits(), &CancelToken::new());
    assert_eq!(code_of(result), ErrorCode::IndexerUnavailable);
}

#[test]
fn an_indexer_older_than_required_is_unavailable() {
    let fixture = Fixture::new();
    assert!(matches!(
        too_old_issue(&fixture),
        ToolIssue::TooOld { found, .. } if found.contains("1.2.3")
    ));
    assert_eq!(fixture.runner.check(&spec("ok")), IndexerStatus::Ready);
}

#[test]
fn a_program_inside_the_project_is_never_used() {
    let project = TempDir::new().unwrap();
    let bin = project.path().join("bin");
    std::fs::create_dir(&bin).unwrap();
    let exe = PathBuf::from(env!("CARGO_BIN_EXE_fake_indexer"));
    std::fs::copy(&exe, bin.join(exe.file_name().unwrap())).unwrap();
    // The only fake indexer on the path lives inside the project, as a hostile repository would
    // have it.
    let fixture = Fixture::build(project, &bin, false);
    assert_eq!(code_of(fixture.run("ok")), ErrorCode::IndexerUnavailable);
}

#[test]
fn too_deeply_nested_output_counts_as_too_large() {
    let fixture = Fixture::new();
    let small = IndexLimits {
        max_output_bytes: 512 * 1024 * 1024,
        ..limits()
    };
    let result = fixture.run_with(&spec("deep-dir"), small, &CancelToken::new());
    assert_eq!(code_of(result), ErrorCode::IndexerOutputTooLarge);
}

#[test]
fn a_secret_across_the_cut_of_the_excerpt_is_masked_before_it_is_cut() {
    let fixture = Fixture::new();
    let error = fixture.run("fail-long").unwrap_err();
    let cause = error
        .source()
        .and_then(|s| s.downcast_ref::<IndexerError>());
    let Some(IndexerError::Failed { excerpt, .. }) = cause else {
        panic!("not a failure: {cause:?}");
    };
    assert!(excerpt.len() <= 2048, "{}", excerpt.len());
    assert!(
        !excerpt.contains("CANARY") && !excerpt.contains("SECRET"),
        "{excerpt}"
    );
}

#[test]
fn the_version_output_in_a_problem_is_short_and_redacted() {
    let fixture = Fixture::new();
    let ToolIssue::TooOld { found, .. } = too_old_issue(&fixture) else {
        panic!("not too old");
    };
    assert!(found.len() <= 200);
}

#[test]
fn consent_withdrawn_after_the_permit_stops_the_run_before_it_starts() {
    let fixture = Fixture::new();
    level2::revoke(
        &Principal::local_owner(),
        fixture.store.as_ref(),
        ProjectId::new(1),
    )
    .unwrap();
    let result = fixture.run("ok");
    assert_eq!(code_of(result), ErrorCode::IndexerNotConfirmed);
    assert_eq!(data_dir_entries(&fixture), 0, "the output folder stayed");
}

#[cfg(unix)]
#[test]
fn a_link_in_place_of_the_index_is_not_an_index() {
    let fixture = Fixture::new();
    let error = fixture.run("link-index").unwrap_err();
    assert_eq!(failure_reason(&error), Some(FailureReason::NoIndex));
}
