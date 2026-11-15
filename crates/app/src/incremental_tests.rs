//! Issue #49: an incremental scan gives the references a full scan of the same files gives. The
//! full scan is the oracle; each case changes a project and compares both ways to the result.

use std::fs;
use std::path::Path;

use codeskimr_application::ports::ProjectRepository;
use codeskimr_cli::{GlobalOptions, ScanArgs};
use codeskimr_domain::{ProjectId, ScanMode};

use crate::config_files;
use crate::locations::Locations;
use crate::mcp_http_tests::locations;
use crate::memory_keystore::secrets;
use crate::scan::{self, Session};
use crate::wiring::open_store;

const PROJECT: ProjectId = ProjectId::new(1);

/// The files of the project before any change: a caller in `Caller.cs` of a static method in
/// `Target.cs` and of one in `Helper.cs`, and a name in `Util.cs` that nothing declares.
const BASE: &[(&str, &str)] = &[
    (
        "Caller.cs",
        "namespace Demo;\n\npublic class Caller\n{\n    public void Run()\n    {\n        Target.Do();\n        Helper.Go();\n        Missing.Thing();\n    }\n}\n",
    ),
    (
        "Target.cs",
        "namespace Demo;\n\npublic class Target\n{\n    public static void Do() { }\n}\n",
    ),
    (
        "Helper.cs",
        "namespace Demo;\n\npublic class Helper\n{\n    public static void Go() { }\n}\n",
    ),
    (
        "Util.cs",
        "namespace Demo;\n\npublic class Util\n{\n    public void Work()\n    {\n        Helper.Go();\n        Target.Do();\n    }\n}\n",
    ),
];

/// A change to the project: a file written, or `None` for a file removed.
type Change = (&'static str, Option<&'static str>);

fn write_project(folder: &Path, changes: &[Change]) {
    fs::create_dir_all(folder).unwrap();
    for (path, text) in BASE {
        fs::write(folder.join(path), text).unwrap();
    }
    apply(folder, changes);
}

fn apply(folder: &Path, changes: &[Change]) {
    for (path, text) in changes {
        match text {
            Some(text) => fs::write(folder.join(path), text).unwrap(),
            None => fs::remove_file(folder.join(path)).unwrap(),
        }
    }
}

fn session(folder: &Path, locations: &Locations) -> Session {
    let args = ScanArgs {
        source: folder.to_string_lossy().into_owned(),
        branch: None,
        credential: None,
        json: false,
        watch: false,
    };
    scan::open(
        scan::prepare(&args).unwrap(),
        &GlobalOptions::default(),
        locations,
        &secrets(),
    )
    .unwrap()
}

/// Every reference of the newest snapshot as text: file, place, kind, confidence, from and to.
fn references_of(locations: &Locations) -> Vec<String> {
    let config = config_files::load(locations, &GlobalOptions::default(), None).unwrap();
    let store = open_store(locations, &config).unwrap();
    let state = store.latest_state(PROJECT).unwrap();
    let name_of = |id| {
        state
            .index
            .symbols
            .iter()
            .find(|symbol| symbol.id == id)
            .map(|symbol| symbol.qualified_name.as_str().to_owned())
            .unwrap()
    };
    let path_of = |id| {
        state
            .files
            .iter()
            .find(|file| file.id == id)
            .map(|file| file.path.as_str().to_owned())
            .unwrap()
    };
    let mut lines: Vec<String> = state
        .index
        .references
        .iter()
        .map(|r| {
            format!(
                "{} {:?} {} {} {} -> {}",
                path_of(r.file),
                r.span,
                r.kind.label(),
                r.confidence.label(),
                name_of(r.from),
                name_of(r.to)
            )
        })
        .collect();
    lines.sort();
    lines
}

/// The references after a full scan of the project with `changes`, and after a full scan of the
/// base followed by an incremental scan of the same changes.
fn both_ways(changes: &[Change]) -> (Vec<String>, Vec<String>) {
    let root = tempfile::tempdir().unwrap();
    let full_folder = root.path().join("full");
    write_project(&full_folder, changes);
    let full_data = locations(&root.path().join("full-data"));
    session(&full_folder, &full_data)
        .scan_quick(ScanMode::Full)
        .unwrap();

    let step_folder = root.path().join("step");
    write_project(&step_folder, &[]);
    let step_data = locations(&root.path().join("step-data"));
    let step = session(&step_folder, &step_data);
    step.scan_quick(ScanMode::Full).unwrap();
    apply(&step_folder, changes);
    let summary = step.scan_quick(ScanMode::Incremental).unwrap();
    assert!(summary.references > 0);
    (references_of(&full_data), references_of(&step_data))
}

fn assert_same(changes: &[Change]) {
    let (full, incremental) = both_ways(changes);
    assert_eq!(incremental, full);
}

#[test]
fn the_base_project_resolves_the_calls_of_both_files() {
    let (full, incremental) = both_ways(&[]);
    assert_eq!(incremental, full);
    assert!(full.iter().any(|line| line.ends_with("-> Demo.Target.Do")));
}

#[test]
fn a_renamed_target_leaves_the_callers_without_it() {
    assert_same(&[(
        "Target.cs",
        Some("namespace Demo;\n\npublic class Goal\n{\n    public static void Do() { }\n}\n"),
    )]);
}

#[test]
fn a_deleted_file_takes_its_usages_with_it() {
    assert_same(&[("Helper.cs", None)]);
}

#[test]
fn a_new_target_for_a_name_that_matched_nothing_is_found() {
    assert_same(&[(
        "Missing.cs",
        Some("namespace Demo;\n\npublic class Missing\n{\n    public static void Thing() { }\n}\n"),
    )]);
}

#[test]
fn a_second_candidate_makes_the_usages_of_unchanged_files_probable() {
    assert_same(&[(
        "Other.cs",
        Some("namespace Other;\n\npublic class Helper\n{\n    public static void Go() { }\n}\n"),
    )]);
}

#[test]
fn an_added_overload_is_found_by_the_unchanged_callers() {
    assert_same(&[(
        "Target.cs",
        Some(
            "namespace Demo;\n\npublic class Target\n{\n    public static void Do() { }\n    public static void Do(int x) { }\n}\n",
        ),
    )]);
}

#[test]
fn several_changes_at_once_give_the_same_result() {
    assert_same(&[
        ("Helper.cs", None),
        (
            "Missing.cs",
            Some(
                "namespace Demo;\n\npublic class Missing\n{\n    public static void Thing() { }\n}\n",
            ),
        ),
        (
            "Target.cs",
            Some("namespace Demo;\n\npublic class Goal\n{\n    public static void Do() { }\n}\n"),
        ),
    ]);
}
