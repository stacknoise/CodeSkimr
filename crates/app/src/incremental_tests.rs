//! Issue #49: an incremental scan gives the references a full scan of the same files gives. The
//! full scan is the oracle; each case changes a project and compares both ways to the result.

use std::fs;
use std::path::Path;

use codeskimr_application::ports::{
    CodeIndex, PreviousState, ProjectCatalog, ProjectRepository, SymbolKey,
};
use codeskimr_cli::GlobalOptions;
use codeskimr_domain::{ProjectId, ScanMode};
use codeskimr_storage::SqliteStore;

use crate::alert_tests::session;
use crate::config_files;
use crate::locations::Locations;
use crate::mcp_http_tests::locations;
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
        write_file(folder, path, text);
    }
    apply(folder, changes);
}

fn write_file(folder: &Path, path: &str, text: &str) {
    let file = folder.join(path);
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(file, text).unwrap();
}

fn apply(folder: &Path, changes: &[Change]) {
    for (path, text) in changes {
        match text {
            Some(text) => write_file(folder, path, text),
            None => fs::remove_file(folder.join(path)).unwrap(),
        }
    }
}

/// What both ways of scanning must agree on, as text: every reference of the newest snapshot
/// (file, place, kind, confidence, from and to), then coupling, then the warnings.
fn references_of(locations: &Locations) -> Vec<String> {
    let config = config_files::load(locations, &GlobalOptions::default(), None).unwrap();
    let store = open_store(locations, &config).unwrap();
    let state = store.latest_state(PROJECT).unwrap();
    let references = store.latest_index(PROJECT).unwrap().references;
    let name_of = |id| {
        state
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
    let mut lines: Vec<String> = references
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
    lines.extend(coupling_of(&store, &state));
    lines.extend(warnings_stored(&store));
    lines
}

/// Coupling and instability of every symbol of the newest snapshot as text, sorted. They come
/// from the references of the whole snapshot (ADR 0037), so both ways of scanning must agree on
/// them too (ADR 0086).
fn coupling_of(store: &SqliteStore, state: &PreviousState) -> Vec<String> {
    let snapshot = state.snapshot.unwrap();
    let mut lines = Vec::new();
    for symbol in &state.symbols {
        let detail = store
            .symbol(snapshot, &SymbolKey::Id(symbol.id))
            .unwrap()
            .unwrap();
        for metric in detail.metrics.iter().filter(|m| m.kind.is_per_snapshot()) {
            lines.push(format!(
                "{} {} {}",
                symbol.qualified_name.as_str(),
                metric.kind.label(),
                metric.value.get()
            ));
        }
    }
    lines.sort();
    lines
}

/// The warnings the newest snapshot stores, as text.
fn warnings_stored(store: &SqliteStore) -> Vec<String> {
    let overviews = store.overviews().unwrap();
    let project = overviews.iter().find(|o| o.project.id == PROJECT).unwrap();
    project
        .warnings
        .iter()
        .map(|warning| format!("{} {}", warning.code.id(), warning.count))
        .collect()
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

/// The result after two steps with one change each is the one of a full scan.
fn assert_two_steps(first: Change, second: Change) {
    assert_same_in_steps(&[&[first], &[second]]);
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

/// The warnings of a scan as `(code, count)`.
fn warnings_of(summary: &codeskimr_cli::ScanSummary) -> Vec<(String, u64)> {
    summary
        .warnings
        .iter()
        .map(|line| (line.code.clone(), line.count))
        .collect()
}

#[test]
fn places_with_several_candidates_are_one_warning_in_both_ways_of_scanning() {
    let root = tempfile::tempdir().unwrap();
    let folder = root.path().join("project");
    write_project(&folder, BASE, &[]);
    let data = locations(&root.path().join("data"));
    let session = session(&folder, &data);
    let first = session.scan_quick(ScanMode::Full).unwrap();
    assert_eq!(warnings_of(&first), Vec::<(String, u64)>::new());
    // `Target.Do()` in `Caller.cs` and in `Util.cs` now matches two overloads.
    let overloads = "namespace Demo;\n\npublic class Target\n{\n    public static void Do() { }\n    public static void Do(int n) { }\n}\n";
    apply(&folder, &[("Target.cs", Some(overloads))]);
    let incremental = session.scan_quick(ScanMode::Incremental).unwrap();
    let expected = vec![("RES-301".to_owned(), 2)];
    assert_eq!(warnings_of(&incremental), expected);
    let full = session.scan_quick(ScanMode::Full).unwrap();
    assert_eq!(warnings_of(&full), expected);
}

/// The stored count of ambiguous places is carried from scan to scan (ADR 0086): it rises with a
/// second overload, stays over an edit elsewhere and over an edit of one caller, and falls when
/// the overload goes. After every step it is the one of a full scan.
#[test]
fn the_count_of_ambiguous_places_follows_a_full_scan_over_several_scans() {
    let overloads = "namespace Demo;\n\npublic class Target\n{\n    public static void Do() { }\n    public static void Do(int n) { }\n}\n";
    let one = "namespace Demo;\n\npublic class Target\n{\n    public static void Do() { }\n}\n";
    let other = "namespace Demo;\n\npublic class Other\n{\n    public void New() { }\n}\n";
    let util = "namespace Demo;\n\npublic class Util\n{\n    public void Work()\n    {\n        Target.Do();\n        Target.Do();\n    }\n}\n";
    let steps: [&[Change]; 5] = [
        &[("Target.cs", Some(overloads))],
        &[("Other.cs", Some(other))],
        &[("Util.cs", Some(util))],
        &[("Util.cs", None)],
        &[("Target.cs", Some(one))],
    ];
    for upto in 1..=steps.len() {
        assert_same_in_steps(&steps[..upto]);
    }
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

const HELPER_FIRST: &str =
    "namespace Demo;\n\n// first\npublic class Helper\n{\n    public static void Go() { }\n}\n";
const HELPER_SECOND: &str = "namespace Demo;\n\n// first\n// second\npublic class Helper\n{\n    public static void Go() { }\n}\n";

#[test]
fn an_edit_that_only_adds_a_comment_gives_the_same_result() {
    assert_two_steps(
        ("Helper.cs", Some(HELPER_FIRST)),
        ("Helper.cs", Some(HELPER_SECOND)),
    );
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

const GADGET: &str =
    "namespace Demo.Lib;\n\npublic class Gadget\n{\n    public static void Run() { }\n}\n";
const TOOL: &str =
    "namespace Demo.Elsewhere;\n\npublic class Tool\n{\n    public static void Run() { }\n}\n";

#[test]
fn a_name_that_an_import_makes_visible_follows_a_rename_and_a_second_candidate() {
    assert_two_steps(("Lib.cs", Some(GADGET)), ("Elsewhere.cs", Some(TOOL)));
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

// Issue #408: a changed declaration resolves again only the names that the resolver looks up by a
// changed symbol; each case compares with the full scan, as every case above.

/// The files of a TypeScript project: an import of a class, a call of its method.
const TYPESCRIPT_BASE: &[(&str, &str)] = &[
    (
        "caller.ts",
        "import { Target } from \"./target\";\n\nexport function run(): void {\n  const item = new Target();\n  item.act();\n  helper();\n}\n",
    ),
    ("target.ts", "export class Target {\n  act(): void {}\n}\n"),
    (
        "util.ts",
        "export function helper(): number {\n  return 1;\n}\n",
    ),
];

/// A Java project: a caller of a class in the same package, one in another package.
const JAVA_BASE: &[(&str, &str)] = &[
    (
        "demo/Caller.java",
        "package demo;\n\npublic class Caller {\n    public void run() {\n        Target t = new Target();\n        t.act();\n        Helper.go();\n    }\n}\n",
    ),
    (
        "demo/Target.java",
        "package demo;\n\npublic class Target {\n    public void act() { }\n}\n",
    ),
    (
        "demo/Helper.java",
        "package demo;\n\npublic class Helper {\n    public static void go() { }\n}\n",
    ),
];

/// A Kotlin project: a caller of a class and of a top-level function.
const KOTLIN_BASE: &[(&str, &str)] = &[
    (
        "Caller.kt",
        "package demo\n\nclass Caller {\n    fun run() {\n        val t = Target()\n        t.act()\n        helper()\n    }\n}\n",
    ),
    (
        "Target.kt",
        "package demo\n\nclass Target {\n    fun act() { }\n}\n",
    ),
    ("Util.kt", "package demo\n\nfun helper() { }\n"),
];

/// A Rust project: modules that call each other.
const RUST_BASE: &[(&str, &str)] = &[
    (
        "caller.rs",
        "use crate::target::Target;\n\npub fn run() {\n    let item = Target::new();\n    item.act();\n    helper();\n}\n",
    ),
    (
        "target.rs",
        "pub struct Target;\n\nimpl Target {\n    pub fn new() -> Self {\n        Self\n    }\n\n    pub fn act(&self) {}\n}\n",
    ),
    ("util.rs", "pub fn helper() {}\n"),
];

fn assert_same_for(base: &[(&str, &str)], steps: &[&[Change]]) {
    let (full, incremental) = both_ways(base, steps);
    assert_eq!(incremental, full);
}

#[test]
fn a_new_method_elsewhere_changes_nothing_for_unrelated_callers() {
    assert_same(&[(
        "Other.cs",
        Some("namespace Demo;\n\npublic class Other\n{\n    public void Added() { }\n}\n"),
    )]);
}

#[test]
fn a_new_class_with_the_name_of_a_receiver_changes_the_unchanged_members() {
    // `Derived d` in `UsesDerived`: a second `Derived` in another namespace is a new candidate.
    assert_same(&[(
        "Elsewhere.cs",
        Some("namespace Other;\n\npublic class Derived\n{\n    public void Hello() { }\n}\n"),
    )]);
}

/// The text of a file that lives as long as the test binary, which the steps need.
fn leaked(text: String) -> &'static str {
    Box::leak(text.into_boxed_str())
}

/// `Tool.cs`: a class `Tool` with one static method, in `namespace`.
fn tool_in(namespace: &str) -> &'static str {
    leaked(format!(
        "namespace {namespace};\n\npublic class Tool\n{{\n    public static void Run() {{ }}\n}}\n"
    ))
}

/// `Importer.cs`: a class that imports `namespace` and calls `Tool.Run()`.
fn importer_of(namespace: &str) -> &'static str {
    leaked(format!(
        "using {namespace};\n\nnamespace Elsewhere;\n\npublic class Importer\n{{\n    public void Go() {{ Tool.Run(); }}\n}}\n"
    ))
}

#[test]
fn a_moved_namespace_changes_the_files_that_import_it() {
    let (lib, importer, moved) = (
        tool_in("Demo.Lib"),
        importer_of("Demo.Lib"),
        tool_in("Demo.Moved"),
    );
    assert_same_for(
        BASE,
        &[
            &[("Lib.cs", Some(lib)), ("Importer.cs", Some(importer))],
            &[("Lib.cs", Some(moved))],
            &[("Lib.cs", Some(lib))],
        ],
    );
}

#[test]
fn a_namespace_that_an_import_waited_for_is_found_when_it_appears() {
    let (importer, late) = (importer_of("Demo.Late"), tool_in("Demo.Late"));
    assert_same_for(
        BASE,
        &[
            &[("Importer.cs", Some(importer))],
            &[("Late.cs", Some(late))],
        ],
    );
}

/// The steps of the rename scenario for typescript.
const TYPESCRIPT_RENAME_STEPS: &[&[Change]] = &[
    &[(
        "target.ts",
        Some("export class Target {\n  act(): void {}\n  more(): void {}\n}\n"),
    )],
    &[(
        "other.ts",
        Some(
            "export class Target {\n  act(): void {}\n}\n\nexport function helper(): number {\n  return 2;\n}\n",
        ),
    )],
    &[(
        "target.ts",
        Some("export class Goal {\n  act(): void {}\n}\n"),
    )],
    &[("util.ts", None)],
];

/// The steps of the rename scenario for java.
const JAVA_RENAME_STEPS: &[&[Change]] = &[
    &[(
        "other/Helper.java",
        Some("package other;\n\npublic class Helper {\n    public static void go() { }\n}\n"),
    )],
    &[(
        "demo/Target.java",
        Some("package demo;\n\npublic class Goal {\n    public void act() { }\n}\n"),
    )],
    &[("demo/Helper.java", None)],
];

/// The steps of the rename scenario for kotlin.
const KOTLIN_RENAME_STEPS: &[&[Change]] = &[
    &[(
        "Target.kt",
        Some("package demo\n\nclass Target {\n    fun act() { }\n    fun more() { }\n}\n"),
    )],
    &[("Other.kt", Some("package other\n\nfun helper() { }\n"))],
    &[(
        "Target.kt",
        Some("package demo\n\nclass Goal {\n    fun act() { }\n}\n"),
    )],
];

/// The steps of the rename scenario for rust.
const RUST_RENAME_STEPS: &[&[Change]] = &[
    &[(
        "other.rs",
        Some(
            "pub fn helper() {}\n\npub struct Target;\n\nimpl Target {\n    pub fn act(&self) {}\n}\n",
        ),
    )],
    &[(
        "target.rs",
        Some("pub struct Goal;\n\nimpl Goal {\n    pub fn act(&self) {}\n}\n"),
    )],
    &[("util.rs", None)],
];

#[test]
fn each_language_follows_a_renamed_type_a_second_candidate_and_a_change() {
    let scenarios = [
        (TYPESCRIPT_BASE, TYPESCRIPT_RENAME_STEPS),
        (JAVA_BASE, JAVA_RENAME_STEPS),
        (KOTLIN_BASE, KOTLIN_RENAME_STEPS),
        (RUST_BASE, RUST_RENAME_STEPS),
    ];
    for (base, steps) in scenarios {
        assert_same_for(base, steps);
    }
}

#[test]
fn a_scan_restricted_to_paths_leaves_the_rest_of_the_snapshot_as_it_was() {
    let root = tempfile::tempdir().unwrap();
    let folder = root.path().join("live");
    write_project(&folder, PYTHON_BASE, &[]);
    let data = locations(&root.path().join("live-data"));
    let live = session(&folder, &data);
    live.scan_quick(ScanMode::Full).unwrap();
    let before = references_of(&data);
    // One file below the paths changes, one elsewhere changes and one elsewhere is gone.
    apply(
        &folder,
        &[
            (
                "util.py",
                Some(
                    "def helper():\n    return 1\n\n\ndef work():\n    return helper()\n# a note\n",
                ),
            ),
            ("caller.py", Some("def run():\n    pass\n")),
            ("target.py", None),
        ],
    );
    live.scan_paths(&["util.py".to_owned()]).unwrap();
    assert_eq!(latest_mode(&data), ScanMode::Incremental);
    assert_eq!(
        references_of(&data),
        before,
        "the changes outside the paths are not seen yet"
    );
    // The next scan of the whole project sees them and gives what a full scan gives.
    live.scan_quick(ScanMode::Incremental).unwrap();
    assert_ne!(references_of(&data), before);
    assert_eq!(
        references_of(&data),
        full_scan_of(&folder, root.path(), "full")
    );
}

/// A small generator of C# projects, so that the rule of issue #408 meets changes nobody wrote
/// down: the same generator makes the file of every step, and the full scan decides.
mod random {
    use std::fmt::Write;

    /// Number of files, namespaces, classes and methods the generator draws from. Few of each, so
    /// that names meet: the same class name in two namespaces, a call that one file can answer.
    const SLOTS: usize = 6;
    const NAMESPACES: u64 = 3;
    const CLASSES: u64 = 4;
    const METHODS: u64 = 3;
    /// Free functions a file may declare, in the languages that have them.
    const FREES: u64 = 3;

    /// A fixed sequence of numbers for a seed (a linear congruential generator).
    pub struct Dice(u64);

    impl Dice {
        pub fn new(seed: u64) -> Self {
            Self(seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1))
        }

        pub fn below(&mut self, limit: u64) -> u64 {
            self.0 = self
                .0
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            (self.0 >> 33) % limit
        }

        fn chance(&mut self, percent: u64) -> bool {
            self.below(100) < percent
        }
    }

    /// Adds `value` to `list`, or takes the first one out if it holds it already.
    fn toggle(list: &mut Vec<u64>, value: u64) {
        if let Some(at) = list.iter().position(|held| *held == value) {
            list.remove(at);
        } else {
            list.push(value);
        }
    }

    /// One file: a class in a namespace with methods, maybe a base class, imports and calls.
    #[derive(Clone)]
    pub struct Spec {
        namespace: u64,
        class: u64,
        base: Option<u64>,
        methods: Vec<u64>,
        usings: Vec<u64>,
        calls: Vec<(u64, u64)>,
        /// Whether the class declares `M0` first: two classes that both do, one the base of the
        /// other, make an override.
        core: bool,
        /// Free functions the file declares, and those it calls.
        frees: Vec<u64>,
        free_calls: Vec<u64>,
    }

    impl Spec {
        pub fn draw(dice: &mut Dice) -> Self {
            Self {
                namespace: dice.below(NAMESPACES),
                class: dice.below(CLASSES),
                base: dice.chance(30).then(|| dice.below(CLASSES)),
                methods: (0..dice.below(METHODS + 1))
                    .map(|_| dice.below(METHODS))
                    .collect(),
                usings: (0..dice.below(3)).map(|_| dice.below(NAMESPACES)).collect(),
                calls: (0..dice.below(4))
                    .map(|_| (dice.below(CLASSES), dice.below(METHODS)))
                    .collect(),
                core: dice.chance(70),
                frees: (0..dice.below(3)).map(|_| dice.below(FREES)).collect(),
                free_calls: (0..dice.below(3)).map(|_| dice.below(FREES)).collect(),
            }
        }

        /// A small change of one part, or a new draw.
        pub fn tweak(&self, dice: &mut Dice) -> Self {
            let mut next = self.clone();
            match dice.below(11) {
                0 => next.namespace = dice.below(NAMESPACES),
                1 => next.class = dice.below(CLASSES),
                2 => next.base = dice.chance(60).then(|| dice.below(CLASSES)),
                3 => next.methods.push(dice.below(METHODS)),
                4 => {
                    next.methods.pop();
                }
                5 => next.usings.push(dice.below(NAMESPACES)),
                6 => next.rename_method(dice),
                7 => toggle(&mut next.frees, dice.below(FREES)),
                8 => toggle(&mut next.free_calls, dice.below(FREES)),
                9 => next.core = !next.core,
                _ => return Self::draw(dice),
            }
            next
        }

        /// Gives one method of the class another name: the declaration moves, its callers stay.
        fn rename_method(&mut self, dice: &mut Dice) {
            if self.methods.is_empty() {
                self.core = !self.core;
                return;
            }
            let at = usize::try_from(dice.below(self.methods.len() as u64)).unwrap();
            self.methods[at] = (self.methods[at] + 1 + dice.below(METHODS - 1)) % METHODS;
        }

        /// The methods the class declares: `M0` first if it has a core, then its own.
        fn declared(&self) -> Vec<u64> {
            let core = self.core.then_some(0);
            core.into_iter()
                .chain(self.methods.iter().copied())
                .collect()
        }

        /// The lines that call the free functions the file calls.
        fn free_call_lines(&self, indent: &str, call: impl Fn(u64) -> String) -> String {
            let mut text = String::new();
            for free in &self.free_calls {
                writeln!(text, "{indent}{}", call(*free)).unwrap();
            }
            text
        }

        /// A class that holds the free functions of the file as static methods, for the languages
        /// that have no free functions; nothing if the file declares none.
        fn free_class(&self, head: &str, declare: impl Fn(u64) -> String) -> String {
            if self.frees.is_empty() {
                return String::new();
            }
            format!("\n{head}\n{}}}\n", self.free_declarations(declare))
        }

        /// The declarations of the free functions, one text per function.
        fn free_declarations(&self, declare: impl Fn(u64) -> String) -> String {
            self.frees.iter().map(|free| declare(*free)).collect()
        }

        /// The text of the file in the syntax of `lang`.
        pub fn render(&self, lang: Lang) -> String {
            match lang {
                Lang::CSharp => self.csharp(),
                Lang::Java => self.java(),
                Lang::Kotlin => self.kotlin(),
                Lang::TypeScript => self.typescript(),
                Lang::Rust => self.rust(),
                Lang::Python => self.python(),
            }
        }

        /// The lines of the calls: a call on the class itself, then one on a new value, and so on.
        fn call_lines(
            &self,
            indent: &str,
            direct: impl Fn(u64, u64) -> String,
            local: impl Fn(usize, u64, u64) -> String,
        ) -> String {
            let mut text = String::new();
            for (index, (class, method)) in self.calls.iter().enumerate() {
                let lines = if index % 2 == 0 {
                    direct(*class, *method)
                } else {
                    local(index, *class, *method)
                };
                for line in lines.lines() {
                    writeln!(text, "{indent}{line}").unwrap();
                }
            }
            text
        }

        /// One line per file the file imports.
        fn using_lines(&self, line: impl Fn(u64) -> String) -> String {
            self.usings.iter().fold(String::new(), |mut text, using| {
                writeln!(text, "{}", line(*using)).unwrap();
                text
            })
        }

        /// The declaration of every method of the class, one text per method.
        fn method_lines(&self, declare: impl Fn(u64) -> String) -> String {
            self.declared().into_iter().map(declare).collect()
        }

        /// The body of `run`: the calls, then the calls of the free functions.
        fn run_lines(
            &self,
            indent: &str,
            calls: (
                impl Fn(u64, u64) -> String,
                impl Fn(usize, u64, u64) -> String,
                impl Fn(u64) -> String,
            ),
        ) -> String {
            let (direct, local, free) = calls;
            self.call_lines(indent, direct, local) + &self.free_call_lines(indent, free)
        }

        /// What the class says about its base class, nothing if it has none.
        fn base_clause(&self, say: impl Fn(u64) -> String) -> String {
            self.base.map(say).unwrap_or_default()
        }

        /// `with` where the class has a base class and its methods override, `without` otherwise.
        fn overriding<'a>(&self, with: &'a str, without: &'a str) -> &'a str {
            if self.base.is_some() { with } else { without }
        }

        fn own(&self) -> u64 {
            self.calls.len() as u64 % METHODS
        }

        fn csharp(&self) -> String {
            let mut text = String::new();
            for using in &self.usings {
                writeln!(text, "using Demo.N{using};").unwrap();
            }
            writeln!(text, "\nnamespace Demo.N{};\n", self.namespace).unwrap();
            let base = self.base.map(|b| format!(" : K{b}")).unwrap_or_default();
            writeln!(text, "public class K{}{base}\n{{", self.class).unwrap();
            let keyword = if self.base.is_some() {
                "override"
            } else {
                "virtual"
            };
            for method in self.declared() {
                writeln!(text, "    public {keyword} void M{method}() {{ }}").unwrap();
            }
            writeln!(text, "    public void Run()\n    {{").unwrap();
            text += &self.call_lines(
                "        ",
                |c, m| format!("K{c}.M{m}();"),
                |i, c, m| format!("K{c} v{i} = new K{c}();\nv{i}.M{m}();"),
            );
            text += &self.free_call_lines("        ", |f| format!("Free.F{f}();"));
            writeln!(text, "        this.M{}();\n    }}\n}}", self.own()).unwrap();
            text += &self.free_class("public static class Free\n{", |f| {
                format!("    public static void F{f}() {{ }}\n")
            });
            text
        }

        fn java(&self) -> String {
            let mut text = format!("package demo.n{};\n\n", self.namespace);
            for using in &self.usings {
                writeln!(text, "import demo.n{using}.*;").unwrap();
            }
            let base = self
                .base
                .map(|b| format!(" extends K{b}"))
                .unwrap_or_default();
            writeln!(text, "\nclass K{}{base} {{", self.class).unwrap();
            for method in self.declared() {
                if self.base.is_some() {
                    writeln!(text, "    @Override").unwrap();
                }
                writeln!(text, "    public void m{method}() {{ }}").unwrap();
            }
            writeln!(text, "    public void run() {{").unwrap();
            text += &self.call_lines(
                "        ",
                |c, m| format!("K{c}.m{m}();"),
                |i, c, m| format!("K{c} v{i} = new K{c}();\nv{i}.m{m}();"),
            );
            text += &self.free_call_lines("        ", |f| format!("Free.f{f}();"));
            writeln!(text, "        this.m{}();\n    }}\n}}", self.own()).unwrap();
            text += &self.free_class("class Free {", |f| {
                format!("    public static void f{f}() {{ }}\n")
            });
            text
        }

        fn kotlin(&self) -> String {
            let mut text = format!("package demo.n{}\n\n", self.namespace);
            text += &self.using_lines(|using| format!("import demo.n{using}.*"));
            let base = self.base_clause(|b| format!(" : K{b}()"));
            writeln!(text, "\nopen class K{}{base} {{", self.class).unwrap();
            let keyword = self.overriding("override", "open");
            text += &self.method_lines(|method| format!("    {keyword} fun m{method}() {{ }}\n"));
            writeln!(text, "    fun run() {{").unwrap();
            text += &self.run_lines(
                "        ",
                (
                    |c, m| format!("K{c}().m{m}()"),
                    |i, c, m| format!("val v{i} = K{c}()\nv{i}.m{m}()"),
                    |f| format!("f{f}()"),
                ),
            );
            writeln!(text, "        this.m{}()\n    }}\n}}", self.own()).unwrap();
            text += &self.free_declarations(|f| format!("\nfun f{f}() {{ }}\n"));
            text
        }

        // codeskimr: allow CHK-102 assembles the file text of one language from the shared parts
        fn typescript(&self) -> String {
            let mut text =
                self.using_lines(|using| format!("import {{ K{using} }} from \"./F{using}\";"));
            let base = self.base_clause(|b| format!(" extends K{b}"));
            writeln!(text, "\nexport class K{}{base} {{", self.class).unwrap();
            let keyword = self.overriding("override ", "");
            text += &self.method_lines(|method| format!("  {keyword}m{method}(): void {{}}\n"));
            writeln!(text, "  run(): void {{").unwrap();
            text += &self.run_lines(
                "    ",
                (
                    |c, m| format!("new K{c}().m{m}();"),
                    |i, c, m| format!("const v{i} = new K{c}();\nv{i}.m{m}();"),
                    |f| format!("f{f}();"),
                ),
            );
            writeln!(text, "    this.m{}();\n  }}\n}}", self.own()).unwrap();
            text += &self.free_declarations(|f| format!("\nexport function f{f}(): void {{}}\n"));
            text
        }

        fn rust(&self) -> String {
            let mut text = String::new();
            for using in &self.usings {
                writeln!(text, "use crate::f{using}::*;").unwrap();
            }
            writeln!(text, "\npub struct K{};", self.class).unwrap();
            if let Some(base) = self.base {
                writeln!(text, "\nimpl K{base} for K{} {{}}", self.class).unwrap();
            }
            writeln!(text, "\nimpl K{} {{", self.class).unwrap();
            for method in self.declared() {
                writeln!(text, "    pub fn m{method}(&self) {{}}").unwrap();
            }
            writeln!(text, "    pub fn run(&self) {{").unwrap();
            text += &self.call_lines(
                "        ",
                |c, m| format!("K{c}::m{m}();"),
                |i, c, m| format!("let v{i} = K{c};\nv{i}.m{m}();"),
            );
            text += &self.free_call_lines("        ", |f| format!("f{f}();"));
            writeln!(text, "        self.m{}();\n    }}\n}}", self.own()).unwrap();
            text += &self.free_declarations(|f| format!("\npub fn f{f}() {{}}\n"));
            text
        }

        fn python(&self) -> String {
            let mut text = self.using_lines(|using| format!("from f{using} import *"));
            let base = self.base.map(|b| format!("(K{b})")).unwrap_or_default();
            writeln!(text, "\n\nclass K{}{base}:", self.class).unwrap();
            text +=
                &self.method_lines(|method| format!("    def m{method}(self):\n        pass\n\n"));
            writeln!(text, "    def run(self):").unwrap();
            text += &self.call_lines(
                "        ",
                |c, m| format!("K{c}().m{m}()"),
                |i, c, m| format!("v{i} = K{c}()\nv{i}.m{m}()"),
            );
            text += &self.free_call_lines("        ", |f| format!("f{f}()"));
            writeln!(text, "        self.m{}()", self.own()).unwrap();
            text += &self.free_declarations(|f| format!("\n\ndef f{f}():\n    pass\n"));
            text
        }
    }

    /// The languages the generator writes.
    #[derive(Clone, Copy, Debug)]
    pub enum Lang {
        CSharp,
        Java,
        Kotlin,
        TypeScript,
        Rust,
        Python,
    }

    impl Lang {
        pub const ALL: [Lang; 6] = [
            Lang::CSharp,
            Lang::Java,
            Lang::Kotlin,
            Lang::TypeScript,
            Lang::Rust,
            Lang::Python,
        ];

        /// The path of a slot: one file per slot, named so that imports can find it.
        pub fn path_of(self, slot: usize) -> String {
            match self {
                Lang::CSharp => format!("F{slot}.cs"),
                Lang::Java => format!("F{slot}.java"),
                Lang::Kotlin => format!("F{slot}.kt"),
                Lang::TypeScript => format!("F{slot}.ts"),
                Lang::Rust => format!("f{slot}.rs"),
                Lang::Python => format!("f{slot}.py"),
            }
        }
    }

    /// The files of the project: a slot is a file or empty.
    pub fn slots() -> usize {
        SLOTS
    }
}

/// The references of a full scan of the files in `folder`, in a data folder of its own.
fn full_scan_of(folder: &Path, root: &Path, name: &str) -> Vec<String> {
    let copy = root.join(name);
    fs::create_dir_all(&copy).unwrap();
    for entry in fs::read_dir(folder).unwrap() {
        let entry = entry.unwrap();
        fs::copy(entry.path(), copy.join(entry.file_name())).unwrap();
    }
    let data = locations(&root.join(format!("{name}-data")));
    session(&copy, &data).scan_quick(ScanMode::Full).unwrap();
    references_of(&data)
}

/// One to three random changes: a file deleted, tweaked or newly written.
fn change_some_files(
    lang: random::Lang,
    folder: &Path,
    files: &mut [Option<random::Spec>],
    dice: &mut random::Dice,
) {
    for _ in 0..=dice.below(2) {
        let slot = usize::try_from(dice.below(files.len() as u64)).unwrap();
        let path = lang.path_of(slot);
        files[slot] = match (&files[slot], dice.below(5)) {
            (Some(_), 0) => {
                fs::remove_file(folder.join(&path)).unwrap();
                None
            }
            (Some(spec), _) => Some(spec.tweak(dice)),
            (None, _) => Some(random::Spec::draw(dice)),
        };
        if let Some(spec) = &files[slot] {
            write_file(folder, &path, &spec.render(lang));
        }
    }
}

fn random_project_stays_equal(lang: random::Lang, seed: u64, steps: usize) {
    use random::{Dice, Spec, slots};
    let mut dice = Dice::new(seed);
    let root = tempfile::tempdir().unwrap();
    let folder = root.path().join("live");
    let mut files: Vec<Option<Spec>> = (0..slots())
        .map(|_| (dice.below(4) != 0).then(|| Spec::draw(&mut dice)))
        .collect();
    let path_of = |slot: usize| lang.path_of(slot);
    let text: Vec<(String, String)> = files
        .iter()
        .enumerate()
        .filter_map(|(slot, spec)| spec.as_ref().map(|s| (path_of(slot), s.render(lang))))
        .collect();
    let base: Vec<(&str, &str)> = text.iter().map(|(p, t)| (p.as_str(), t.as_str())).collect();
    write_project(&folder, &base, &[]);
    let data = locations(&root.path().join("live-data"));
    let live = session(&folder, &data);
    live.scan_quick(ScanMode::Full).unwrap();
    let mut with_references = 0;
    for step in 0..steps {
        change_some_files(lang, &folder, &mut files, &mut dice);
        live.scan_quick(ScanMode::Incremental).unwrap();
        let full = full_scan_of(&folder, root.path(), &format!("full{step}"));
        assert_eq!(
            references_of(&data),
            full,
            "{lang:?}, seed {seed}, step {step}"
        );
        with_references += usize::from(!full.is_empty());
    }
    // An oracle that compares two empty lists proves nothing: most steps must have references.
    assert!(
        with_references * 2 >= steps,
        "{lang:?}, seed {seed}: only {with_references} of {steps} steps have references"
    );
}

#[test]
fn random_changes_of_a_csharp_project_give_the_references_of_a_full_scan() {
    for seed in 1..=3 {
        random_project_stays_equal(random::Lang::CSharp, seed, 8);
    }
}

#[test]
fn random_changes_of_projects_in_other_languages_give_the_references_of_a_full_scan() {
    for lang in random::Lang::ALL.into_iter().skip(1) {
        for seed in 1..=2 {
            random_project_stays_equal(lang, seed, 6);
        }
    }
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

/// The python project with its test file already in the first snapshot, so that later scans keep it.
fn python_with_tests() -> Vec<(&'static str, &'static str)> {
    let mut base = PYTHON_BASE.to_vec();
    base.push((TEST_FILE.0, TEST_FILE.1.unwrap_or_default()));
    base
}

#[test]
fn a_kept_test_file_keeps_its_test_references_when_other_files_change() {
    let util: Change = ("util.py", Some("def helper():\n    return 2\n"));
    let (full, incremental) = both_ways(&python_with_tests(), &[&[util]]);
    assert_eq!(incremental, full);
    assert!(
        full.iter()
            .any(|line| line.starts_with("test_target.py") && line.contains("} test "))
    );
}

#[test]
fn a_kept_test_file_loses_its_test_reference_when_the_target_goes() {
    let renamed: Change = (
        "target.py",
        Some("class Target:\n    def renamed(self):\n        pass\n"),
    );
    let (full, incremental) = both_ways(&python_with_tests(), &[&[renamed]]);
    assert_eq!(incremental, full);
    assert!(
        !full
            .iter()
            .any(|line| line.starts_with("test_target.py") && line.contains("-> target.Target.do"))
    );
}

/// Python files whose `close` has two candidates and a caller of a receiver of unknown type;
/// the project file lowers `scan.max_candidates` to 2 (ADR 0084, ADR 0040).
const BOUNDED_BASE: &[(&str, &str)] = &[
    (".codeskimr.toml", "[scan]\nmax_candidates = 2\n"),
    ("a.py", "class A:\n    def close(self):\n        pass\n"),
    ("b.py", "class B:\n    def close(self):\n        pass\n"),
    ("caller.py", "def run(item):\n    item.close()\n"),
];

const THIRD_CLOSER: Change = (
    "c.py",
    Some("class C:\n    def close(self):\n        pass\n"),
);

#[test]
fn a_third_candidate_in_a_new_file_makes_the_unchanged_caller_give_the_place_up() {
    let (full, incremental) = both_ways(BOUNDED_BASE, &[&[THIRD_CLOSER]]);
    assert_eq!(incremental, full);
    assert!(!full.iter().any(|line| line.contains("caller.py")));
}

#[test]
fn removing_the_third_candidate_gives_the_place_back_to_the_unchanged_caller() {
    let with_third: Vec<(&str, &str)> = BOUNDED_BASE
        .iter()
        .copied()
        .chain([("c.py", "class C:\n    def close(self):\n        pass\n")])
        .collect();
    let (full, incremental) = both_ways(&with_third, &[&[("c.py", None)]]);
    assert_eq!(incremental, full);
    assert_eq!(
        full.iter()
            .filter(|line| line.contains("caller.py"))
            .count(),
        2
    );
}
