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
    ("Other.cs", "namespace Demo;\n\npublic class Other\n{\n}\n"),
    (
        "Base.cs",
        "namespace Demo;\n\npublic class Base\n{\n    public void Hello() { }\n}\n",
    ),
    (
        "Derived.cs",
        "namespace Demo;\n\npublic class Derived : Other\n{\n}\n",
    ),
    (
        "UsesDerived.cs",
        "namespace Demo;\n\npublic class UsesDerived\n{\n    public void Go()\n    {\n        Derived d = new Derived();\n        d.Hello();\n    }\n}\n",
    ),
    (
        "Util.cs",
        "namespace Demo;\n\npublic class Util\n{\n    public void Work()\n    {\n        Helper.Go();\n        Target.Do();\n    }\n}\n",
    ),
];

/// A project of Python files: a caller of a class, a helper function called in its own file.
const PYTHON_BASE: &[(&str, &str)] = &[
    (
        "caller.py",
        "from target import Target\n\n\ndef run():\n    item = Target()\n    item.do()\n",
    ),
    (
        "target.py",
        "class Target:\n    def do(self):\n        pass\n",
    ),
    (
        "util.py",
        "def helper():\n    return 1\n\n\ndef work():\n    return helper()\n",
    ),
];

/// A change to the project: a file written, or `None` for a file removed.
type Change = (&'static str, Option<&'static str>);

fn write_project(folder: &Path, base: &[(&str, &str)], changes: &[Change]) {
    fs::create_dir_all(folder).unwrap();
    for (path, text) in base {
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

/// The references after a full scan of `base` with all `steps` applied, and after a full scan of
/// `base` followed by one incremental scan per step.
fn both_ways(base: &[(&str, &str)], steps: &[&[Change]]) -> (Vec<String>, Vec<String>) {
    let root = tempfile::tempdir().unwrap();
    let full_folder = root.path().join("full");
    let all: Vec<Change> = steps.iter().flat_map(|step| step.iter().copied()).collect();
    write_project(&full_folder, base, &all);
    let full_data = locations(&root.path().join("full-data"));
    session(&full_folder, &full_data)
        .scan_quick(ScanMode::Full)
        .unwrap();

    let step_folder = root.path().join("step");
    write_project(&step_folder, base, &[]);
    let step_data = locations(&root.path().join("step-data"));
    let step = session(&step_folder, &step_data);
    step.scan_quick(ScanMode::Full).unwrap();
    for changes in steps {
        apply(&step_folder, changes);
        step.scan_quick(ScanMode::Incremental).unwrap();
    }
    assert_eq!(
        latest_mode(&step_data),
        ScanMode::Incremental,
        "the steps must not fall back to a full scan"
    );
    (references_of(&full_data), references_of(&step_data))
}

/// The mode of the newest snapshot.
fn latest_mode(locations: &Locations) -> ScanMode {
    let config = config_files::load(locations, &GlobalOptions::default(), None).unwrap();
    let store = open_store(locations, &config).unwrap();
    store.latest_snapshot(PROJECT).unwrap().unwrap().mode
}

fn assert_same(changes: &[Change]) {
    let (full, incremental) = both_ways(BASE, &[changes]);
    assert_eq!(incremental, full);
}

fn assert_same_in_steps(steps: &[&[Change]]) {
    let (full, incremental) = both_ways(BASE, steps);
    assert_eq!(incremental, full);
}

fn assert_python_same(steps: &[&[Change]]) {
    let (full, incremental) = both_ways(PYTHON_BASE, steps);
    assert_eq!(incremental, full);
}

#[test]
fn the_base_project_resolves_the_calls_of_both_files() {
    let (full, incremental) = both_ways(BASE, &[&[]]);
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

const MISSING: &str =
    "namespace Demo;\n\npublic class Missing\n{\n    public static void Thing() { }\n}\n";
const GOAL: &str = "namespace Demo;\n\npublic class Goal\n{\n    public static void Do() { }\n}\n";

#[test]
fn a_new_base_class_gives_the_unchanged_caller_the_inherited_member() {
    // `UsesDerived` calls `Hello` on a `Derived`; only `Derived.cs` changes, to derive from `Base`.
    assert_same(&[(
        "Derived.cs",
        Some("namespace Demo;\n\npublic class Derived : Base\n{\n}\n"),
    )]);
}

#[test]
fn a_changed_base_class_changes_what_the_unchanged_derived_class_inherits() {
    assert_same(&[(
        "Base.cs",
        Some("namespace Demo;\n\npublic class Base\n{\n    public void Hi() { }\n}\n"),
    )]);
}

#[test]
fn the_names_read_in_one_incremental_scan_serve_the_next() {
    // Step one writes `Missing.cs` and `Goal`; step two changes both again and deletes a file.
    assert_same_in_steps(&[
        &[("Missing.cs", Some(MISSING)), ("Target.cs", Some(GOAL))],
        &[
            (
                "Missing.cs",
                Some("namespace Demo;\n\npublic class Missing\n{\n}\n"),
            ),
            ("Helper.cs", None),
        ],
    ]);
}

#[test]
fn a_change_that_is_undone_in_the_next_scan_leaves_no_trace() {
    let original = BASE[1].1;
    assert_same_in_steps(&[
        &[("Target.cs", Some(GOAL))],
        &[("Target.cs", Some(original))],
    ]);
}

#[test]
fn a_python_project_gives_the_same_references_after_a_renamed_class() {
    assert_python_same(&[&[(
        "target.py",
        Some("class Goal:\n    def do(self):\n        pass\n"),
    )]]);
}

#[test]
fn a_python_project_gives_the_same_references_after_a_second_helper_and_a_deleted_file() {
    assert_python_same(&[
        &[("other.py", Some("def helper():\n    return 2\n"))],
        &[("util.py", None)],
    ]);
}

#[test]
fn an_edit_inside_a_body_gives_the_same_result() {
    // The declarations stay, so the unchanged files resolve only what pointed into the file.
    assert_same(&[(
        "Target.cs",
        Some(
            "namespace Demo;\n\npublic class Target\n{\n    public static void Do() { Helper.Go(); }\n}\n",
        ),
    )]);
}

#[test]
fn an_edit_that_only_adds_a_comment_gives_the_same_result() {
    assert_same_in_steps(&[
        &[(
            "Helper.cs",
            Some(
                "namespace Demo;\n\n// first\npublic class Helper\n{\n    public static void Go() { }\n}\n",
            ),
        )],
        &[(
            "Helper.cs",
            Some(
                "namespace Demo;\n\n// first\n// second\npublic class Helper\n{\n    public static void Go() { }\n}\n",
            ),
        )],
    ]);
}

#[test]
fn a_name_that_an_import_makes_visible_follows_an_edit_of_the_imported_file() {
    assert_same(&[(
        "Lib.cs",
        Some(
            "namespace Demo.Lib;\n\npublic class Tool\n{\n    public static void Run() { Helper.Go(); }\n}\n",
        ),
    )]);
}

#[test]
fn a_name_that_an_import_makes_visible_follows_a_rename_and_a_second_candidate() {
    assert_same_in_steps(&[
        &[(
            "Lib.cs",
            Some(
                "namespace Demo.Lib;\n\npublic class Gadget\n{\n    public static void Run() { }\n}\n",
            ),
        )],
        &[(
            "Elsewhere.cs",
            Some(
                "namespace Demo.Elsewhere;\n\npublic class Tool\n{\n    public static void Run() { }\n}\n",
            ),
        )],
    ]);
}

#[test]
fn a_file_with_only_an_import_follows_an_edit_of_the_imported_file() {
    let lib = "namespace Demo.Lib;\n\npublic class Tool\n{\n    public static void Run() { }\n}\n";
    let importer = "using Demo.Lib;\n\nnamespace Elsewhere;\n\npublic class Importer\n{\n}\n";
    let edited = "namespace Demo.Lib;\n\npublic class Tool\n{\n    public static void Run() { int x = 1; }\n}\n";
    let (full, incremental) = both_ways(
        BASE,
        &[
            &[("Lib.cs", Some(lib)), ("Importer.cs", Some(importer))],
            &[("Lib.cs", Some(edited))],
        ],
    );
    assert!(
        full.iter().any(|line| line.starts_with("Importer.cs")),
        "{full:?}"
    );
    assert_eq!(incremental, full);
}

#[test]
fn a_python_file_with_only_an_import_follows_an_edit_of_the_imported_file() {
    let importer = ("importer.py", Some("from target import Target\n"));
    let (full, incremental) = both_ways(
        PYTHON_BASE,
        &[
            &[importer],
            &[(
                "target.py",
                Some("class Target:\n    def do(self):\n        return 1\n"),
            )],
        ],
    );
    assert!(
        full.iter().any(|line| line.contains("importer.py")),
        "{full:?}"
    );
    assert_eq!(incremental, full);
}

/// A file of tests next to the code it tests: its uses are `test` references (issue #406).
const TEST_FILE: Change = (
    "test_target.py",
    Some("from target import Target\n\n\ndef test_do():\n    Target().do()\n"),
);

#[test]
fn uses_in_a_test_file_are_test_references_in_both_ways_of_scanning() {
    let (full, incremental) = both_ways(PYTHON_BASE, &[&[TEST_FILE]]);
    assert_eq!(incremental, full);
    let in_test: Vec<&String> = full
        .iter()
        .filter(|line| line.starts_with("test_target.py"))
        .collect();
    assert!(
        in_test.iter().any(|line| line.contains("} test ")),
        "{full:?}"
    );
    assert!(
        !in_test.iter().any(|line| line.contains("} call ")),
        "{full:?}"
    );
    // The code under test keeps its calls.
    assert!(
        full.iter()
            .any(|line| line.starts_with("caller.py") && line.contains(" call "))
    );
}
