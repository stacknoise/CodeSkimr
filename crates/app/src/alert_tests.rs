//! ADR 0068 over a real project: the content hashes of a full and of an incremental scan are the
//! same, and a critical symbol raises an alert only when its tokens change.

use std::fs;
use std::path::Path;
use std::sync::Arc;

use codeskimr_application::error::CodedError;
use codeskimr_application::events::EventBus;
use codeskimr_application::pipeline::{Job, JobResult};
use codeskimr_application::ports::{AlertFilter, SymbolStates};
use codeskimr_application::ports::{AnalyzeRequest, FileAnalyzer, FileFacts};
use codeskimr_application::ports::{AnnotationCreation, AnnotationStore, ProjectRepository};
use codeskimr_cli::{GlobalOptions, ScanArgs};
use codeskimr_codes::ErrorCode;
use codeskimr_domain::{
    AlertChange, AlertKind, AlertState, AlertTrigger, AnnotationContent, AnnotationKind,
    AnnotationLimits, AnnotationTarget, Author, LanguageId, NewAnnotation, ProjectId,
    QualifiedName, RelativePath, ScanMode, SymbolKind, Timestamp,
};
use codeskimr_storage::SqliteStore;

use crate::config_files;
use crate::locations::Locations;
use crate::mcp_http_tests::locations;
use crate::memory_keystore::secrets;
use crate::principal;
use crate::scan::{self, Session};
use crate::wiring::{build_analysis, open_store, scanner_over};

const PROJECT: ProjectId = ProjectId::new(1);

const TARGET: &str = "namespace Demo;\n\npublic class Target\n{\n    public static int Do(int x)\n    {\n        return x + 1;\n    }\n}\n";
const OTHER: &str =
    "namespace Demo;\n\npublic class Other\n{\n    public int Keep() { return 1; }\n}\n";

fn write(folder: &Path, path: &str, text: &str) {
    fs::create_dir_all(folder).unwrap();
    fs::write(folder.join(path), text).unwrap();
}

pub(crate) fn session(folder: &Path, locations: &Locations) -> Session {
    let args = ScanArgs {
        source: folder.to_string_lossy().into_owned(),
        branch: None,
        credential: None,
        json: false,
        watch: false,
        name: None,
    };
    scan::open(
        scan::prepare(&args).unwrap(),
        &GlobalOptions::default(),
        locations,
        &secrets(),
    )
    .unwrap()
}

fn store_of(locations: &Locations) -> std::sync::Arc<SqliteStore> {
    let config = config_files::load(locations, &GlobalOptions::default(), None).unwrap();
    open_store(locations, &config).unwrap()
}

/// Every type and member of the newest snapshot with its hash, sorted.
fn hashes_of(locations: &Locations) -> Vec<(String, String)> {
    let store = store_of(locations);
    let state = store.latest_state(PROJECT).unwrap();
    let names: Vec<QualifiedName> = state
        .symbols
        .iter()
        .map(|symbol| symbol.qualified_name.clone())
        .collect();
    let snapshot = state.snapshot.unwrap();
    let mut found: Vec<(String, String)> = store
        .states(snapshot, &names)
        .unwrap()
        .into_iter()
        .filter_map(|state| {
            Some((
                state.name.to_string(),
                state.content_hash?.as_str().to_owned(),
            ))
        })
        .collect();
    found.sort();
    found
}

pub(crate) fn critical(store: &SqliteStore, kind: SymbolKind, name: &str) {
    let limits = AnnotationLimits {
        max_text_chars: 100,
        max_title_chars: 20,
    };
    store
        .create_annotation(&AnnotationCreation {
            annotation: NewAnnotation {
                project: PROJECT,
                target: AnnotationTarget {
                    kind,
                    name: QualifiedName::new(name).unwrap(),
                    parameters: Vec::new(),
                },
                author: Author::new("anton").unwrap(),
                content: AnnotationContent {
                    kind: AnnotationKind::Critical,
                    title: None,
                    text: Some(limits.text("pays the bills").unwrap()),
                    status: None,
                    due: None,
                    assignee: None,
                },
            },
            now: Timestamp(1),
        })
        .unwrap();
}

pub(crate) fn open_alerts(store: &SqliteStore) -> Vec<codeskimr_domain::Alert> {
    store
        .list_alerts(&AlertFilter {
            project: PROJECT,
            state: Some(AlertState::Open),
        })
        .unwrap()
}

/// A project of two files under `root` with a critical annotation on `name`, scanned twice.
pub(crate) fn watched(
    root: &tempfile::TempDir,
    kind: SymbolKind,
    name: &str,
) -> (std::path::PathBuf, Session, std::sync::Arc<SqliteStore>) {
    let folder = root.path().join("project");
    write(&folder, "Target.cs", TARGET);
    write(&folder, "Other.cs", OTHER);
    let data = locations(&root.path().join("data"));
    let session = session(&folder, &data);
    session.scan_quick(ScanMode::Full).unwrap();
    let store = store_of(&data);
    critical(&store, kind, name);
    session.scan_quick(ScanMode::Incremental).unwrap();
    (folder, session, store)
}

#[test]
fn an_incremental_scan_stores_the_hashes_a_full_scan_of_the_same_files_stores() {
    let root = tempfile::tempdir().unwrap();
    let full_folder = root.path().join("full");
    let step_folder = root.path().join("step");
    let changed = TARGET.replace("x + 1", "x + 2");
    for folder in [&full_folder, &step_folder] {
        write(folder, "Target.cs", TARGET);
        write(folder, "Other.cs", OTHER);
    }
    write(&full_folder, "Target.cs", &changed);
    let full_data = locations(&root.path().join("full-data"));
    session(&full_folder, &full_data)
        .scan_quick(ScanMode::Full)
        .unwrap();

    let step_data = locations(&root.path().join("step-data"));
    let step = session(&step_folder, &step_data);
    step.scan_quick(ScanMode::Full).unwrap();
    let before = hashes_of(&step_data);
    write(&step_folder, "Target.cs", &changed);
    step.scan_quick(ScanMode::Incremental).unwrap();

    let after = hashes_of(&step_data);
    assert_eq!(after, hashes_of(&full_data));
    assert!(after.len() >= 4, "{after:?}");
    // Only what the edit touched has another hash; the file that was carried over keeps its own.
    let differing: Vec<&str> = after
        .iter()
        .zip(&before)
        .filter(|(now, was)| now != was)
        .map(|(now, _)| now.0.as_str())
        .collect();
    assert_eq!(differing, ["Demo.Target", "Demo.Target.Do"]);
}

#[test]
fn a_critical_method_raises_an_alert_for_a_change_of_code_and_not_for_a_cosmetic_edit() {
    let root = tempfile::tempdir().unwrap();
    let (folder, session, store) = watched(&root, SymbolKind::Method, "Demo.Target.Do");
    assert_eq!(open_alerts(&store).len(), 0);

    // Comments and layout: no alert.
    let tidy = "namespace Demo;\n\n// the target\npublic class Target\n{\n\n    public static int Do( int x )\n    {\n        // add one\n        return   x + 1 ;\n    }\n\n}\n";
    write(&folder, "Target.cs", tidy);
    session.scan_quick(ScanMode::Incremental).unwrap();
    assert_eq!(open_alerts(&store).len(), 0);

    // A change of the code: one alert, from the watcher.
    write(&folder, "Target.cs", &TARGET.replace("x + 1", "x + 2"));
    session.scan_quick(ScanMode::Incremental).unwrap();
    let alerts = open_alerts(&store);
    assert_eq!(alerts.len(), 1);
    assert_eq!(alerts[0].sighting.kind, AlertKind::Changed);
    assert_eq!(alerts[0].sighting.changes, [AlertChange::Content]);
    assert_eq!(alerts[0].sighting.trigger, AlertTrigger::Watcher);

    // A save elsewhere adds nothing; another change of the symbol counts.
    write(&folder, "Other.cs", &OTHER.replace("return 1", "return 2"));
    session.scan_quick(ScanMode::Incremental).unwrap();
    assert_eq!(open_alerts(&store)[0].counter, 1);
    write(&folder, "Target.cs", &TARGET.replace("x + 1", "x + 3"));
    session.scan_quick(ScanMode::Incremental).unwrap();
    assert_eq!(open_alerts(&store)[0].counter, 2);
}

#[test]
fn a_deleted_critical_file_raises_a_removed_alert_and_a_full_scan_agrees() {
    let root = tempfile::tempdir().unwrap();
    let (folder, session, store) = watched(&root, SymbolKind::Class, "Demo.Target");
    fs::remove_file(folder.join("Target.cs")).unwrap();
    session.scan_with(ScanMode::Full, |_| {}).unwrap();
    let alerts = open_alerts(&store);
    assert_eq!(alerts.len(), 1);
    assert_eq!(alerts[0].sighting.kind, AlertKind::Removed);
    assert_eq!(alerts[0].sighting.trigger, AlertTrigger::Scan);
}

/// The real analyzer, except that `path` runs into the time limit.
struct TimesOut {
    inner: Arc<dyn FileAnalyzer>,
    path: &'static str,
}

impl FileAnalyzer for TimesOut {
    fn analyze(&self, request: &AnalyzeRequest<'_>) -> Result<FileFacts, CodedError> {
        if request.file.path.as_str() == self.path {
            return Err(CodedError::new(ErrorCode::ParseTimeout));
        }
        self.inner.analyze(request)
    }

    fn naming_rules(&self) -> String {
        self.inner.naming_rules()
    }

    fn is_test_file(&self, language: &LanguageId, path: &RelativePath) -> bool {
        self.inner.is_test_file(language, path)
    }
}

/// A full scan with the real adapters in which `path` runs into the time limit.
fn full_scan_timing_out(
    data: &Locations,
    store: &Arc<SqliteStore>,
    path: &'static str,
) -> JobResult {
    let config = config_files::load(data, &GlobalOptions::default(), None).unwrap();
    let mut analysis = build_analysis(&config, data).unwrap();
    analysis.analyzer = Arc::new(TimesOut {
        inner: analysis.analyzer,
        path,
    });
    let scanner = scanner_over(
        &analysis,
        (store, &config, data),
        (EventBus::new(), &secrets()),
    )
    .unwrap();
    let job = Job::new(PROJECT, ScanMode::Full).without_level2();
    scanner.run(&principal::resolve(), job)
}

#[test]
fn a_full_scan_that_times_out_on_the_file_of_a_critical_symbol_raises_no_alert() {
    let root = tempfile::tempdir().unwrap();
    let (folder, _session, store) = watched(&root, SymbolKind::Method, "Demo.Target.Do");
    let data = locations(&root.path().join("data"));
    write(&folder, "Other.cs", &OTHER.replace("return 1", "return 2"));

    let result = full_scan_timing_out(&data, &store, "Target.cs");

    let JobResult::Finished(job) = result else {
        panic!("the scan did not finish");
    };
    let left_out = &job.paths_report().left_out;
    assert_eq!(left_out.len(), 1);
    assert!(left_out[0].kept, "the former state of the file stays");
    assert_eq!(open_alerts(&store).len(), 0);
    let state = store.latest_state(PROJECT).unwrap();
    let mut paths: Vec<&str> = state.files.iter().map(|file| file.path.as_str()).collect();
    paths.sort_unstable();
    assert_eq!(paths, ["Other.cs", "Target.cs"]);
    let names: Vec<&str> = state
        .symbols
        .iter()
        .map(|symbol| symbol.qualified_name.as_str())
        .collect();
    assert!(names.contains(&"Demo.Target.Do"), "{names:?}");
    assert!(names.contains(&"Demo.Other.Keep"), "{names:?}");
}
