//! The commands of the project list through the same path the server takes.

use std::fs;
use std::path::Path;
use std::sync::Arc;

use codeskimr_application::events::EventBus;
use codeskimr_cli::GlobalOptions;
use codeskimr_codes::Coded;
use codeskimr_server::CommandApi;
use serde_json::{Value, json};
use tempfile::TempDir;

use std::num::NonZeroU32;

use codeskimr_application::ports::ProjectRepository;
use codeskimr_domain::{
    FolderPath, Layer, LayerName, Project, ProjectId, ProjectName, ProjectSource, QualifiedName,
};
use codeskimr_storage::SqliteStore;

use super::command_api;
use crate::locations::{Locations, UserConfig};
use crate::principal;
use crate::scan::open_on;
use crate::source_arg::classify;
use crate::wiring::Secrets;
use codeskimr_domain::ScanMode;

/// Name of the user configuration file a fixture may hold.
pub(crate) const USER_CONFIG_FILE: &str = "user-config.toml";

const SOURCE: &str = "namespace Demo;\n\npublic class Calculator\n{\n    public int Add(int a, int b) { return a + b; }\n}\n";

pub(crate) struct Fixture {
    pub(crate) root: TempDir,
    pub(crate) locations: Locations,
    pub(crate) api: Arc<CommandApi>,
    pub(crate) secrets: Arc<Secrets>,
    pub(crate) events: EventBus,
}

impl Fixture {
    pub(super) fn new() -> Self {
        Self::installed(None)
    }

    /// An installation whose UI folder holds `page` as the export page, or no UI folder at all.
    pub(crate) fn installed(page: Option<&str>) -> Self {
        Self::installed_with(page, None)
    }

    /// Like [`Fixture::installed`], with a user configuration file that holds `config`.
    pub(crate) fn installed_with(page: Option<&str>, config: Option<&str>) -> Self {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir_all(root.path().join("app")).unwrap();
        fs::write(root.path().join("app/Calculator.cs"), SOURCE).unwrap();
        let ui = root.path().join("ui");
        if let Some(page) = page {
            fs::create_dir_all(ui.join("export")).unwrap();
            fs::write(ui.join("export/export.html"), page).unwrap();
        }
        let locations = Locations {
            data_dir: root.path().join("data"),
            languages_dir: Path::new(env!("CARGO_MANIFEST_DIR")).join("../../languages"),
            user_config: config.map(|text| {
                let path = root.path().join(USER_CONFIG_FILE);
                fs::write(&path, text).unwrap();
                UserConfig {
                    path,
                    required: true,
                }
            }),
            bases: Vec::new(),
            program_dir: None,
        };
        let global = GlobalOptions {
            overrides: vec![("ui.dir".to_owned(), format!("'{}'", ui.display()))],
            ..GlobalOptions::default()
        };
        let secrets = crate::memory_keystore::secrets();
        let events = EventBus::new();
        let api = command_api(&global, &locations, events.clone(), Arc::clone(&secrets)).unwrap();
        Self {
            root,
            locations,
            api: Arc::new(api),
            secrets,
            events,
        }
    }

    /// Gives the project these layers, through a second connection to the same database.
    fn set_layers(&self, project: u64, layers: &[(&str, &str)]) {
        let store = SqliteStore::open(&self.locations.database(), NonZeroU32::MIN).unwrap();
        let mut stored = store.project(ProjectId::new(project)).unwrap();
        stored.layers = layers
            .iter()
            .map(|(name, namespace)| Layer {
                may_use: None,
                name: LayerName::new(*name).unwrap(),
                namespaces: vec![QualifiedName::new(*namespace).unwrap()],
            })
            .collect();
        store.save_project(&stored).unwrap();
    }

    /// Registers the folder without scanning it, so the project has no snapshot.
    pub(super) fn register_unscanned(&self) -> u64 {
        let store = SqliteStore::open(&self.locations.database(), NonZeroU32::MIN).unwrap();
        let source = ProjectSource::Folder(FolderPath::new(self.folder()).unwrap());
        let project = Project {
            id: ProjectId::new(1),
            name: ProjectName::new("app").unwrap(),
            source,
            layers: Vec::new(),
        };
        store.save_project(&project).unwrap();
        project.id.get()
    }

    /// Registers and scans the demo project and returns its id.
    pub(crate) fn scanned(&self) -> u64 {
        let added = self
            .call("projects.add", json!({ "source": self.folder() }))
            .unwrap();
        added["project"].as_u64().unwrap()
    }

    pub(crate) fn folder(&self) -> String {
        self.root.path().join("app").to_string_lossy().into_owned()
    }

    pub(crate) fn call(&self, id: &str, input: Value) -> Result<Value, &'static str> {
        self.call_as(principal::resolve(), id, input)
    }

    /// The same call for another caller.
    pub(crate) fn call_as(
        &self,
        caller: codeskimr_domain::Principal,
        id: &str,
        input: Value,
    ) -> Result<Value, &'static str> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        runtime
            .block_on(self.api.run(caller, id, input))
            .map_err(|error| error.code().entry().id)
    }
}

/// Every command of the registry, in the order the API lists them.
const COMMAND_IDS: [&str; 45] = [
    "alerts.dismiss",
    "alerts.dismiss_all",
    "alerts.list",
    "annotations.add_note",
    "annotations.add_task",
    "annotations.create",
    "annotations.delete",
    "annotations.list",
    "annotations.mark_critical",
    "annotations.summary",
    "annotations.update",
    "credentials.delete",
    "credentials.list",
    "credentials.set",
    "explorer.outline",
    "explorer.source",
    "explorer.symbol",
    "export.html",
    "graph.layers",
    "layers.get",
    "layers.set",
    "level2.revoke",
    "level2.status",
    "preferences.get",
    "preferences.set",
    "projects.add",
    "projects.cancel",
    "projects.folder_dialog",
    "projects.list",
    "projects.pick_folder",
    "projects.remove",
    "projects.rename",
    "projects.rescan",
    "projects.suggest_name",
    "projects.tools",
    "scope.get",
    "scope.set",
    "search.query",
    "settings.effective",
    "stats.overview",
    "stats.ranking",
    "status.info",
    "tools.indexers",
    "usages.code",
    "usages.query",
];

#[test]
fn the_commands_are_offered() {
    let fixture = Fixture::new();
    let ids: Vec<_> = fixture
        .api
        .list(&principal::resolve())
        .into_iter()
        .map(|info| info.id)
        .collect();
    assert_eq!(ids, COMMAND_IDS);
}

#[test]
fn the_indexers_are_listed_without_any_project() {
    let fixture = Fixture::new();
    let view = fixture.call("tools.indexers", Value::Null).unwrap();
    let tools: Vec<&str> = view["tools"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|tool| tool["tool"].as_str())
        .collect();
    assert!(tools.contains(&"scip-dotnet"), "{tools:?}");
    assert!(view["checked_at"].is_i64());
}

#[test]
fn a_new_installation_has_no_projects() {
    let fixture = Fixture::new();
    assert_eq!(fixture.call("projects.list", Value::Null), Ok(json!([])));
}

#[test]
fn an_added_folder_is_scanned_and_listed_with_its_language() {
    let fixture = Fixture::new();
    let scanned = fixture
        .call("projects.add", json!({ "source": fixture.folder() }))
        .unwrap();
    assert_eq!(scanned["files"], 1);
    let list = fixture.call("projects.list", Value::Null).unwrap();
    let row = &list[0];
    assert_eq!(row["name"], "app");
    assert_eq!(row["kind"], "folder");
    assert_eq!(row["types"], 1);
    assert_eq!(row["scan"]["mode"], "full");
    assert_eq!(row["languages"][0]["language"], "csharp");
    assert_eq!(row["languages"][0]["tier"], "full");
    assert_eq!(row["languages"][0]["symbols"], scanned["symbols"]);
    assert_eq!(row["running"], false);
}

#[test]
fn adding_the_same_folder_again_keeps_one_project() {
    let fixture = Fixture::new();
    let input = json!({ "source": fixture.folder() });
    let first = fixture.call("projects.add", input.clone()).unwrap();
    let second = fixture.call("projects.add", input).unwrap();
    assert_eq!(first["project"], second["project"]);
    let list = fixture.call("projects.list", Value::Null).unwrap();
    assert_eq!(list.as_array().unwrap().len(), 1);
}

#[test]
fn a_relative_path_and_a_missing_folder_are_refused() {
    let fixture = Fixture::new();
    for source in ["app", "/does/not/exist"] {
        let result = fixture.call("projects.add", json!({ "source": source }));
        assert_eq!(result, Err("PRJ-101"), "{source}");
    }
}

#[test]
fn a_git_address_with_a_password_is_refused() {
    let fixture = Fixture::new();
    let result = fixture.call(
        "projects.add",
        json!({ "source": "https://user:secret@example.org/repo.git" }),
    );
    assert_eq!(result, Err("PRJ-101"));
}

#[test]
fn a_project_is_scanned_again_incrementally() {
    let fixture = Fixture::new();
    let scanned = fixture
        .call("projects.add", json!({ "source": fixture.folder() }))
        .unwrap();
    let again = fixture
        .call("projects.rescan", json!({ "project": scanned["project"] }))
        .unwrap();
    assert_eq!(again["project"], scanned["project"]);
    let list = fixture.call("projects.list", Value::Null).unwrap();
    assert_eq!(list[0]["scan"]["mode"], "incremental");
}

#[test]
fn a_project_is_scanned_completely_when_the_caller_asks_for_it() {
    let fixture = Fixture::new();
    let scanned = fixture
        .call("projects.add", json!({ "source": fixture.folder() }))
        .unwrap();
    fixture
        .call(
            "projects.rescan",
            json!({ "project": scanned["project"], "full": true }),
        )
        .unwrap();
    let list = fixture.call("projects.list", Value::Null).unwrap();
    assert_eq!(list[0]["scan"]["mode"], "full");
}

#[test]
fn a_scan_of_the_browser_says_why_level_2_did_not_start_and_stores_the_report() {
    let fixture = Fixture::new();
    let scanned = fixture
        .call("projects.add", json!({ "source": fixture.folder() }))
        .unwrap();
    // The switch is off in this configuration and nothing is confirmed.
    assert_eq!(
        scanned["level2"],
        json!({ "state": "not_started", "reason": "not_confirmed" })
    );
    let list = fixture.call("projects.list", Value::Null).unwrap();
    let report = &list[0]["report"];
    assert_eq!(report["last_scan"]["origin"], "browser");
    assert_eq!(report["last_scan"]["level"], "1");
    let languages = report["level2"]["languages"].as_array().unwrap();
    assert!(
        languages
            .iter()
            .all(|line| line["outcome"] == "skipped" && line["reason"] == "not_confirmed")
    );
}

#[test]
fn a_caller_that_asks_for_level_1_only_gets_no_attempt() {
    let fixture = Fixture::new();
    let scanned = fixture
        .call(
            "projects.add",
            json!({ "source": fixture.folder(), "level2": false }),
        )
        .unwrap();
    assert_eq!(
        scanned["level2"],
        json!({ "state": "not_requested", "reason": null })
    );
}

#[test]
fn a_client_without_the_right_to_run_indexers_is_told_so_and_level_1_is_stored() {
    let fixture = Fixture::new();
    let id = added(&fixture);
    let result = fixture
        .call_as(
            principal::remote_client(),
            "projects.rescan",
            json!({ "project": id }),
        )
        .unwrap();
    assert_eq!(
        result["level2"],
        json!({ "state": "not_started", "reason": "not_permitted" })
    );
    assert_eq!(result["project"], id);
}

#[test]
fn an_unknown_project_cannot_be_scanned() {
    let fixture = Fixture::new();
    let result = fixture.call("projects.rescan", json!({ "project": 99 }));
    assert_eq!(result, Err("PRJ-201"));
}

fn added(fixture: &Fixture) -> u64 {
    let scanned = fixture
        .call("projects.add", json!({ "source": fixture.folder() }))
        .unwrap();
    scanned["project"].as_u64().unwrap()
}

#[test]
fn the_outline_sorts_the_types_into_a_namespace() {
    let fixture = Fixture::new();
    let project = added(&fixture);
    let outline = fixture
        .call("explorer.outline", json!({ "project": project }))
        .unwrap();
    let namespace = &outline["layers"][0]["namespaces"][0];
    assert_eq!(namespace["name"], "Demo");
    assert_eq!(namespace["types"][0]["name"], "Calculator");
    assert_eq!(namespace["types"][0]["kind"], "class");
}

/// The answer of `explorer.symbol` for the first type of the outline.
fn shown_type(fixture: &Fixture, project: u64) -> serde_json::Value {
    let outline = fixture
        .call("explorer.outline", json!({ "project": project }))
        .unwrap();
    let id = outline["layers"][0]["namespaces"][0]["types"][0]["id"].clone();
    fixture
        .call(
            "explorer.symbol",
            json!({ "project": project, "symbol": id }),
        )
        .unwrap()
}

#[test]
fn a_symbol_comes_with_breadcrumb_and_members() {
    let fixture = Fixture::new();
    let project = added(&fixture);
    let shown = shown_type(&fixture, project);
    assert_eq!(shown["name"], "Calculator");
    assert_eq!(shown["first_line"], 3);
    assert_eq!(shown["members"][0]["name"], "Add");
    assert_eq!(shown["members"][0]["complexity"], 1.0);
    assert_eq!(shown["members"][0]["complexity_rating"], "good");
    assert_eq!(shown["breadcrumb"][1]["name"], "Demo");
}

#[test]
fn an_unknown_project_or_symbol_is_refused() {
    let fixture = Fixture::new();
    assert_eq!(
        fixture.call("explorer.outline", json!({ "project": 99 })),
        Err("PRJ-201")
    );
    let project = added(&fixture);
    assert_eq!(
        fixture.call(
            "explorer.symbol",
            json!({ "project": project, "symbol": 99999 })
        ),
        Err("MCP-202")
    );
}

#[test]
fn a_project_without_a_scan_has_an_empty_outline_and_no_symbols() {
    let fixture = Fixture::new();
    let project = fixture.register_unscanned();
    let outline = fixture
        .call("explorer.outline", json!({ "project": project }))
        .unwrap();
    assert_eq!(outline["snapshot"], Value::Null);
    assert_eq!(outline["layers"], json!([]));
    assert_eq!(
        fixture.call(
            "explorer.symbol",
            json!({ "project": project, "symbol": 1 })
        ),
        Err("MCP-203")
    );
}

#[test]
fn layers_come_first_and_types_of_no_layer_last() {
    let fixture = Fixture::new();
    let project = added(&fixture);
    fixture.set_layers(project, &[("Core", "Demo"), ("Other", "Nowhere")]);
    let outline = fixture
        .call("explorer.outline", json!({ "project": project }))
        .unwrap();
    assert_eq!(outline["layers"].as_array().unwrap().len(), 2);
    assert_eq!(outline["layers"][0]["name"], "Core");
    assert_eq!(outline["layers"][1]["namespaces"], json!([]));

    fixture.set_layers(project, &[("Other", "Nowhere")]);
    let outline = fixture
        .call("explorer.outline", json!({ "project": project }))
        .unwrap();
    assert_eq!(outline["layers"][1]["name"], "");
    assert_eq!(
        outline["layers"][1]["namespaces"][0]["types"][0]["name"],
        "Calculator"
    );
}

#[test]
fn a_symbol_names_the_layer_that_owns_it() {
    let fixture = Fixture::new();
    let project = added(&fixture);
    assert!(shown_type(&fixture, project)["layer"].is_null());
    fixture.set_layers(project, &[("Core", "Demo")]);
    assert_eq!(shown_type(&fixture, project)["layer"], "Core");
}

#[test]
fn the_outline_names_the_limits_behind_the_colours() {
    let fixture = Fixture::new();
    let project = added(&fixture);
    let outline = fixture
        .call("explorer.outline", json!({ "project": project }))
        .unwrap();
    assert_eq!(outline["thresholds"]["complexity_good_max"], 5);
    assert_eq!(outline["thresholds"]["maintainability_medium_min"], 35);
}

/// Runs a command as `who`, as the API does after it has resolved the caller; the registered
/// error id of a refusal comes back as the error.
pub(super) fn run_as(
    fixture: &Fixture,
    who: codeskimr_domain::Principal,
    id: &str,
    input: Value,
) -> Result<Value, &'static str> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    runtime
        .block_on(fixture.api.run(who, id, input))
        .map_err(|error| error.code().entry().id)
}

/// Writes `source` as `Checkout.cs`, scans the project and returns the input of a usages query
/// for `Calculator.Add`.
fn add_usages_input(fixture: &Fixture, source: &str, way: &str, depth: u32) -> Value {
    fs::write(fixture.root.path().join("app/Checkout.cs"), source).unwrap();
    let project = added(fixture);
    let add = member_id(fixture, project, "Add");
    json!({ "project": project, "symbol": add, "way": way, "depth": depth, "include_tests": true })
}

#[test]
fn a_token_without_the_read_right_is_refused() {
    let fixture = Fixture::new();
    let project = added(&fixture);
    let nobody = principal::nobody();
    for id in [
        "explorer.outline",
        "explorer.source",
        "explorer.symbol",
        "graph.layers",
        "search.query",
        "stats.overview",
        "stats.ranking",
        "status.info",
        "usages.code",
        "usages.query",
    ] {
        let refused = run_as(
            &fixture,
            nobody,
            id,
            json!({ "project": project, "symbol": 1 }),
        );
        assert_eq!(refused.unwrap_err(), "AUTH-401", "{id}");
    }
}

#[test]
fn a_caller_who_may_read_but_not_read_source_gets_no_code() {
    let fixture = Fixture::new();
    let input = add_usages_input(&fixture, CALLER, "both", 2);
    let run = |id: &str| run_as(&fixture, principal::reader(), id, input.clone());
    // The index is open to the reader, the lines of code are not.
    assert!(run("usages.query").is_ok());
    assert_eq!(run("usages.code").unwrap_err(), "AUTH-401");
    assert_eq!(run("explorer.source").unwrap_err(), "AUTH-401");
}

#[test]
fn the_export_and_the_credential_names_need_read_source_like_the_source_text() {
    let fixture = Fixture::new();
    for (id, input) in [
        ("export.html", json!({ "project": 1 })),
        ("credentials.list", Value::Null),
    ] {
        let refused = run_as(&fixture, principal::reader(), id, input);
        assert_eq!(refused.unwrap_err(), "AUTH-401", "{id}");
    }
}

#[test]
fn several_hits_in_one_file_get_their_lines_from_one_read() {
    let fixture = Fixture::new();
    let twice = CALLER.replace(
        "        return calculator.Add(1, 2);\n",
        "        var one = calculator.Add(1, 2);\n        return calculator.Add(one, 3);\n",
    );
    let input = add_usages_input(&fixture, &twice, "incoming", 1);
    let code = fixture.call("usages.code", input).unwrap();
    let texts: Vec<&str> = code["lines"]
        .as_array()
        .unwrap()
        .iter()
        .map(|line| line["text"].as_str().unwrap())
        .collect();
    assert_eq!(
        texts,
        [
            "var one = calculator.Add(1, 2);",
            "return calculator.Add(one, 3);"
        ]
    );
}

#[test]
fn the_search_finds_a_member_by_a_part_of_its_name_and_names_its_type() {
    let fixture = Fixture::new();
    let project = added(&fixture);
    let found = fixture
        .call(
            "search.query",
            json!({ "project": project, "query": "add" }),
        )
        .unwrap();
    assert!(found["snapshot"].is_u64());
    assert!(found["entries"].as_u64().unwrap() >= 2);
    let hit = &found["hits"][0];
    assert_eq!(hit["name"], "Add");
    assert_eq!(hit["kind"], "method");
    assert_eq!(hit["path"], "Calculator.cs");
    assert!(hit["parent"].is_u64());
    let kinds: Vec<_> = fixture
        .call(
            "search.query",
            json!({ "project": project, "query": "calc" }),
        )
        .unwrap()["hits"]
        .as_array()
        .unwrap()
        .iter()
        .map(|hit| hit["kind"].clone())
        .collect();
    assert!(kinds.contains(&json!("class")));
}

#[test]
fn the_search_finds_the_files_whose_path_matches_and_names_their_first_type() {
    let fixture = Fixture::new();
    let project = added(&fixture);
    let found = fixture
        .call(
            "search.query",
            json!({ "project": project, "query": "calcu" }),
        )
        .unwrap();
    let file = &found["files"][0];
    assert_eq!(file["path"], "Calculator.cs");
    assert_eq!(file["first_type"], type_id(&fixture, project));
    assert!(file["loc"].as_u64().unwrap() > 0);
    assert_eq!(file["project"], project);
    let none = fixture
        .call(
            "search.query",
            json!({ "project": project, "query": "zzz" }),
        )
        .unwrap();
    assert_eq!(none["files"], json!([]));
}

#[test]
fn a_search_over_all_projects_names_the_project_of_every_hit_and_skips_unscanned_ones() {
    let fixture = Fixture::new();
    let project = added(&fixture);
    fixture.register_unscanned();
    let found = fixture
        .call(
            "search.query",
            json!({ "project": 0, "query": "add", "all_projects": true }),
        )
        .unwrap();
    let hit = &found["hits"][0];
    assert_eq!(hit["name"], "Add");
    assert_eq!(hit["project"], project);
    assert_ne!(hit["project_name"].as_str().unwrap(), "");
    let own = fixture
        .call(
            "search.query",
            json!({ "project": project, "query": "add" }),
        )
        .unwrap();
    assert_eq!(own["hits"], found["hits"]);
}

#[test]
fn the_search_of_a_project_without_a_scan_or_without_text_is_empty() {
    let fixture = Fixture::new();
    let project = fixture.register_unscanned();
    let found = fixture
        .call("search.query", json!({ "project": project, "query": "a" }))
        .unwrap();
    assert_eq!(
        found,
        json!({ "snapshot": null, "entries": null, "hits": [], "files": [] })
    );
    let scanned = added(&fixture);
    let blank = fixture
        .call("search.query", json!({ "project": scanned, "query": "" }))
        .unwrap();
    assert_eq!(blank["hits"], json!([]));
    assert_eq!(
        fixture.call("search.query", json!({ "project": 99, "query": "a" })),
        Err("PRJ-201")
    );
}

#[test]
fn the_status_reports_the_memory_of_the_program() {
    let fixture = Fixture::new();
    let status = fixture.call("status.info", Value::Null).unwrap();
    assert!(status["memory_mb"].as_u64().unwrap() > 0);
    assert!(status.get("startup_ms").is_some());
    assert!(status["workers"].as_u64().unwrap() >= 1);
    assert_eq!(status["version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(status["budgets"]["cold_start"], 300);
    assert_eq!(status["budgets"]["incremental_rescan"], 300);
}

#[test]
fn cancelling_without_a_running_scan_says_so() {
    let fixture = Fixture::new();
    let project = added(&fixture);
    assert_eq!(
        fixture.call("projects.cancel", json!({ "project": project })),
        Ok(json!({ "cancelled": false }))
    );
}

#[test]
fn a_cancelled_scan_ends_with_its_own_code_and_writes_no_snapshot() {
    let fixture = Fixture::new();
    let spec = classify(&fixture.folder(), None).unwrap();
    let session = open_on(
        principal::resolve(),
        spec,
        &GlobalOptions::default(),
        &fixture.locations,
        (EventBus::new(), &fixture.secrets),
    )
    .unwrap();
    let result = session.scan_with(ScanMode::Full, |token| token.cancel());
    assert_eq!(result.unwrap_err().code().entry().id, "PRJ-303");
    let rows = fixture.call("projects.list", Value::Null).unwrap();
    assert_eq!(rows[0]["scan"], Value::Null);
}

fn type_id(fixture: &Fixture, project: u64) -> Value {
    fixture
        .call("explorer.outline", json!({ "project": project }))
        .unwrap()["layers"][0]["namespaces"][0]["types"][0]["id"]
        .clone()
}

#[test]
fn the_source_of_a_type_and_of_a_member_comes_from_the_project_folder() {
    let fixture = Fixture::new();
    let project = added(&fixture);
    let id = type_id(&fixture, project);
    let shown = fixture
        .call(
            "explorer.source",
            json!({ "project": project, "symbol": id }),
        )
        .unwrap();
    assert_eq!(shown["available"], true);
    assert_eq!(shown["path"], "Calculator.cs");
    assert_eq!(shown["first_line"], 3);
    assert_eq!(shown["lines"][0], "public class Calculator");
    assert_eq!(shown["lines"].as_array().unwrap().len(), 4);
    assert_eq!(shown["truncated"], false);
    let tokens = shown["tokens"].as_array().unwrap();
    assert!(tokens.iter().any(|token| token["line"] == 0
        && token["start"] == 0
        && token["end"] == 6
        && token["class"] == "keyword"));
    let member = fixture
        .call(
            "explorer.symbol",
            json!({ "project": project, "symbol": id }),
        )
        .unwrap()["members"][0]["id"]
        .clone();
    let line = fixture
        .call(
            "explorer.source",
            json!({ "project": project, "symbol": member }),
        )
        .unwrap();
    assert_eq!(
        line["lines"],
        json!(["    public int Add(int a, int b) { return a + b; }"])
    );
}

#[test]
fn the_source_is_refused_for_an_unknown_symbol_a_missing_scan_and_a_vanished_file() {
    let fixture = Fixture::new();
    let project = added(&fixture);
    assert_eq!(
        fixture.call(
            "explorer.source",
            json!({ "project": project, "symbol": 99999 })
        ),
        Err("MCP-202")
    );
    let id = type_id(&fixture, project);
    fs::remove_file(fixture.root.path().join("app/Calculator.cs")).unwrap();
    assert_eq!(
        fixture.call(
            "explorer.source",
            json!({ "project": project, "symbol": id })
        ),
        Err("SRC-201")
    );
}

#[test]
fn the_source_of_a_project_without_a_scan_is_refused() {
    let fixture = Fixture::new();
    let project = fixture.register_unscanned();
    assert_eq!(
        fixture.call(
            "explorer.source",
            json!({ "project": project, "symbol": 1 })
        ),
        Err("MCP-203")
    );
}

const CALLER: &str = "namespace Demo;\n\npublic class Checkout\n{\n    public int Total(Calculator calculator)\n    {\n        return calculator.Add(1, 2);\n    }\n}\n";

fn member_id(fixture: &Fixture, project: u64, name: &str) -> Value {
    let id = type_id(fixture, project);
    fixture
        .call(
            "explorer.symbol",
            json!({ "project": project, "symbol": id }),
        )
        .unwrap()["members"]
        .as_array()
        .unwrap()
        .iter()
        .find(|member| member["name"] == name)
        .unwrap()["id"]
        .clone()
}

fn usages_of_add(fixture: &Fixture, project: u64, way: &str) -> Value {
    let add = member_id(fixture, project, "Add");
    fixture
        .call(
            "usages.query",
            json!({ "project": project, "symbol": add, "way": way, "depth": 2, "include_tests": true }),
        )
        .unwrap()
}

/// The usages answer for `Add` in a project where `Checkout.Total` calls it.
fn answer_with_caller() -> Value {
    let fixture = Fixture::new();
    fs::write(fixture.root.path().join("app/Checkout.cs"), CALLER).unwrap();
    let project = added(&fixture);
    usages_of_add(&fixture, project, "both")
}

#[test]
fn the_usages_of_a_method_name_the_caller_and_mark_every_hit_unchecked() {
    let answer = answer_with_caller();
    let hits = answer["incoming"]["hits"].as_array().unwrap();
    assert_eq!(hits.len(), 1);
    assert!(
        hits[0]["from"]["qualified_name"]
            .as_str()
            .unwrap()
            .ends_with("Checkout.Total")
    );
    assert_eq!(hits[0]["kind"], "call");
    assert_eq!(hits[0]["line"], 7);
    assert_eq!(hits[0]["verified"], false);
    assert_eq!(answer["summary"]["places"], 1);
    assert_eq!(answer["summary"]["unchecked"], 1);
    assert_eq!(answer["summary"]["verified"], 0);
    assert_eq!(answer["level2"], "unavailable");
    assert_eq!(answer["impact"]["total"], 1);
    assert_eq!(answer["impact"]["public_interfaces"], 1);
    assert_eq!(answer["summary"]["probable"], 0);
    assert_eq!(answer["summary"]["files"], 1);
    assert_eq!(answer["outgoing"]["total"], 0);
}

#[test]
fn the_code_of_a_hit_is_the_trimmed_line_from_the_project_folder() {
    let fixture = Fixture::new();
    let input = add_usages_input(&fixture, CALLER, "both", 2);
    let code = fixture.call("usages.code", input.clone()).unwrap();
    assert_eq!(code["available"], true);
    assert_eq!(
        code["lines"],
        json!([{ "path": "Checkout.cs", "line": 7, "text": "return calculator.Add(1, 2);" }])
    );
    fs::remove_file(fixture.root.path().join("app/Checkout.cs")).unwrap();
    let gone = fixture.call("usages.code", input).unwrap();
    assert_eq!(gone["lines"], json!([]));
}

#[test]
fn the_code_of_hits_is_refused_for_a_missing_scan() {
    let fixture = Fixture::new();
    let project = fixture.register_unscanned();
    let input = json!({ "project": project, "symbol": 1, "way": "both", "depth": 2, "include_tests": true });
    assert_eq!(fixture.call("usages.code", input), Err("MCP-203"));
}

#[test]
fn the_impact_counts_unchecked_symbols_apart() {
    let answer = answer_with_caller();
    assert_eq!(answer["impact"]["verified"], 0);
    assert_eq!(answer["impact"]["unchecked"], 1);
}

#[test]
fn a_direction_leaves_out_the_other_walk() {
    let fixture = Fixture::new();
    fs::write(fixture.root.path().join("app/Checkout.cs"), CALLER).unwrap();
    let project = added(&fixture);
    let upward = usages_of_add(&fixture, project, "incoming");
    assert_eq!(upward["outgoing"], Value::Null);
    let downward = usages_of_add(&fixture, project, "outgoing");
    assert_eq!(downward["incoming"], Value::Null);
    assert_eq!(downward["summary"]["places"], 1);
}

#[test]
fn the_usages_of_an_unknown_symbol_or_a_project_without_a_scan_are_refused() {
    let fixture = Fixture::new();
    let project = added(&fixture);
    let input = |project: u64, symbol: u64| json!({ "project": project, "symbol": symbol, "way": "both", "depth": 0, "include_tests": false });
    assert_eq!(
        fixture.call("usages.query", input(project, 999_999)),
        Err("MCP-202")
    );
    let unscanned = Fixture::new();
    let id = unscanned.register_unscanned();
    assert_eq!(unscanned.call("usages.query", input(id, 1)), Err("MCP-203"));
}

const LAYERED_UI: &str = "using Demo.Core;\n\nnamespace Demo.Ui;\n\npublic class Screen\n{\n    public int Show() { Engine engine = new Engine(); return engine.Run(); }\n}\n";
const LAYERED_CORE: &str =
    "namespace Demo.Core;\n\npublic class Engine\n{\n    public int Run() { return 1; }\n}\n";
const LAYER_RULES: &str = "[[layers]]\nname = \"ui\"\nnamespaces = [\"Demo.Ui\"]\n\n[[layers]]\nname = \"core\"\nnamespaces = [\"Demo.Core\"]\n";

/// A layered two-project fixture and the answer of `graph.layers` with the given extra input.
fn layered_graph(extra: &serde_json::Value) -> serde_json::Value {
    let fixture = Fixture::new();
    let app = fixture.root.path().join("app");
    fs::write(app.join("Screen.cs"), LAYERED_UI).unwrap();
    fs::write(app.join("Engine.cs"), LAYERED_CORE).unwrap();
    fs::write(app.join(".codeskimr.toml"), LAYER_RULES).unwrap();
    let mut input = json!({ "project": added(&fixture) });
    input
        .as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    fixture.call("graph.layers", input).unwrap()
}

#[test]
fn the_graph_places_the_declared_layers_top_down_with_their_dependencies() {
    let graph = layered_graph(&json!({}));
    assert!(graph["snapshot"].is_u64());
    let boxes = graph["boxes"].as_array().unwrap();
    let names: Vec<_> = boxes.iter().map(|b| b["name"].as_str().unwrap()).collect();
    assert_eq!(names, ["ui", "core"]);
    assert!(boxes[0]["y"].as_u64() < boxes[1]["y"].as_u64());
    assert_eq!(boxes[0]["width"], 160);
    assert!(graph["width"].as_u64() >= boxes[0]["width"].as_u64());
    assert!(graph["height"].as_u64() > boxes[1]["y"].as_u64());
}

#[test]
fn the_layer_map_has_a_band_behind_every_layer_row() {
    let graph = layered_graph(&json!({}));
    let bands = graph["bands"].as_array().unwrap();
    let names: Vec<_> = bands.iter().map(|b| b["name"].as_str().unwrap()).collect();
    assert_eq!(names, ["ui", "core"]);
    let boxes = graph["boxes"].as_array().unwrap();
    assert!(bands[0]["y"].as_u64() <= boxes[0]["y"].as_u64());
    assert!(bands[0]["height"].as_u64() > boxes[0]["height"].as_u64());
    let end =
        |band: &serde_json::Value| band["y"].as_u64().unwrap() + band["height"].as_u64().unwrap();
    assert!(end(&bands[0]) <= bands[1]["y"].as_u64().unwrap());
    assert!(end(&bands[1]) <= graph["height"].as_u64().unwrap());
}

#[test]
fn the_namespace_level_puts_one_namespace_in_the_middle_with_its_neighbours() {
    let graph = layered_graph(&json!({ "level": "namespaces" }));
    let boxes = graph["boxes"].as_array().unwrap();
    let found: Vec<_> = boxes
        .iter()
        .map(|b| (b["name"].as_str().unwrap(), b["types"].as_u64().unwrap()))
        .collect();
    let typed = |suffix: &str| found.iter().find(|(name, _)| name.ends_with(suffix));
    assert_eq!(typed("Demo.Core").map(|found| found.1), Some(1));
    assert_eq!(typed("Demo.Ui").map(|found| found.1), Some(1));
    assert!(found.iter().all(|(_, types)| *types > 0));
    assert!(
        boxes
            .iter()
            .all(|b| !b["label"].as_str().unwrap().is_empty())
    );
    assert!(graph["focus"]["name"].is_string());
    assert!(graph["focus"]["symbol"].is_null());
    let links = graph["links"].as_array().unwrap();
    assert_eq!(links.len(), 1);
    assert!(links[0]["from"].as_str().unwrap().ends_with("Demo.Ui"));
    assert!(links[0]["to"].as_str().unwrap().ends_with("Demo.Core"));
}

#[test]
fn the_namespace_in_the_middle_can_be_chosen_and_its_neighbours_sit_above_and_below() {
    let ui = layered_graph(&json!({ "level": "namespaces", "namespace": "Demo.Ui" }));
    let core = layered_graph(&json!({ "level": "namespaces", "namespace": "Demo.Core" }));
    let row_of = |graph: &serde_json::Value, suffix: &str| {
        graph["boxes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|b| b["name"].as_str().unwrap().ends_with(suffix))
            .map(|b| b["rank"].as_u64().unwrap())
    };
    assert!(ui["focus"]["name"].as_str().unwrap().ends_with("Demo.Ui"));
    assert!(row_of(&ui, "Demo.Ui") < row_of(&ui, "Demo.Core"));
    assert!(
        core["focus"]["name"]
            .as_str()
            .unwrap()
            .ends_with("Demo.Core")
    );
    assert!(row_of(&core, "Demo.Core") > row_of(&core, "Demo.Ui"));
    assert_eq!(core["bands"].as_array().unwrap().len(), 0);
}

#[test]
fn a_depth_merges_the_namespaces_and_leaves_no_dependency_to_itself() {
    let all = layered_graph(&json!({ "level": "namespaces" }));
    let top = layered_graph(&json!({ "level": "namespaces", "depth": 1 }));
    let names = |graph: &serde_json::Value| -> Vec<String> {
        graph["boxes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|b| b["name"].as_str().unwrap().to_owned())
            .collect()
    };
    // The C# names carry a prefix (#137), so the first segment is not yet the namespace root; the
    // merge itself is tested on the domain graph.
    assert!(names(&top).iter().all(|name| !name.contains('.')));
    assert!(names(&top).len() <= names(&all).len());
    for link in top["links"].as_array().unwrap() {
        assert_ne!(link["from"], link["to"]);
    }
    assert_ne!(names(&top), Vec::<String>::new());
}

/// The id of the type `name` in the explorer outline of the project.
fn type_named(fixture: &Fixture, project: u64, name: &str) -> Value {
    let outline = fixture
        .call("explorer.outline", json!({ "project": project }))
        .unwrap();
    outline["layers"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|layer| layer["namespaces"].as_array().unwrap())
        .flat_map(|space| space["types"].as_array().unwrap())
        .find(|found| found["name"] == name)
        .unwrap()["id"]
        .clone()
}

/// The map of the type level around `focus`.
fn type_map(fixture: &Fixture, project: u64, focus: &Value) -> Value {
    fixture
        .call(
            "graph.layers",
            json!({ "project": project, "level": "types", "focus": focus }),
        )
        .unwrap()
}

#[test]
fn the_type_level_puts_the_type_in_the_middle_between_its_users_and_what_it_uses() {
    let fixture = Fixture::new();
    fs::write(fixture.root.path().join("app/Checkout.cs"), CALLER).unwrap();
    let project = added(&fixture);
    let calculator = type_named(&fixture, project, "Calculator");
    let map = type_map(&fixture, project, &calculator);
    assert_eq!(map["focus"]["symbol"], calculator);
    assert_eq!(map["focus"]["users_more"], 0);
    let boxes = map["boxes"].as_array().unwrap();
    assert_eq!(boxes.len(), 2);
    let middle = boxes.iter().find(|b| b["symbol"] == calculator).unwrap();
    let user = boxes.iter().find(|b| b["symbol"] != calculator).unwrap();
    assert!(user["name"].as_str().unwrap().ends_with("Checkout"));
    assert!(user["y"].as_u64() < middle["y"].as_u64());
    assert_eq!(map["links"][0]["from"], user["name"]);
    assert_eq!(map["links"][0]["to"], middle["name"]);
    assert_eq!(map["links"][0]["count"], 2);
    assert_eq!(map["bands"], json!([]));
}

#[test]
fn the_type_level_refuses_a_missing_focus_and_something_that_is_no_type() {
    let fixture = Fixture::new();
    let project = added(&fixture);
    let member = member_id(&fixture, project, "Add");
    for focus in [json!(null), json!(99999), member] {
        assert_eq!(
            fixture.call(
                "graph.layers",
                json!({ "project": project, "level": "types", "focus": focus })
            ),
            Err("MCP-202")
        );
    }
}

#[test]
fn the_graph_of_a_project_without_layers_or_without_a_scan_is_empty() {
    let fixture = Fixture::new();
    let project = added(&fixture);
    let graph = fixture
        .call("graph.layers", json!({ "project": project }))
        .unwrap();
    assert_eq!(graph["boxes"], json!([]));
    assert_eq!(graph["links"], json!([]));
    let other = Fixture::new();
    let unscanned = other.register_unscanned();
    let graph = other
        .call("graph.layers", json!({ "project": unscanned }))
        .unwrap();
    assert_eq!(graph["snapshot"], Value::Null);
    assert_eq!(graph["boxes"], json!([]));
}

#[test]
fn the_graph_of_an_unknown_project_is_refused() {
    let fixture = Fixture::new();
    assert_eq!(
        fixture.call("graph.layers", json!({ "project": 99 })),
        Err("PRJ-201")
    );
}

/// A number of an answer as an index.
fn index(value: &Value) -> usize {
    usize::try_from(value.as_u64().unwrap()).unwrap()
}

#[test]
fn the_source_names_the_places_where_a_symbol_is_used() {
    let fixture = Fixture::new();
    fs::write(fixture.root.path().join("app/Checkout.cs"), CALLER).unwrap();
    let project = added(&fixture);
    let checkout = type_named(&fixture, project, "Checkout");
    let total = fixture
        .call(
            "explorer.symbol",
            json!({ "project": project, "symbol": checkout }),
        )
        .unwrap()["members"][0]["id"]
        .clone();
    let add = member_id(&fixture, project, "Add");
    let shown = fixture
        .call(
            "explorer.source",
            json!({ "project": project, "symbol": total }),
        )
        .unwrap();
    let lines = shown["lines"].as_array().unwrap();
    let call = shown["spans"]
        .as_array()
        .unwrap()
        .iter()
        .find(|span| span["symbol"] == add)
        .expect("the call of Add is a place");
    let line = lines[index(&call["line"])].as_str().unwrap();
    let chars: Vec<char> = line.chars().collect();
    let start = index(&call["start"]);
    let end = index(&call["end"]);
    assert_eq!(chars[start..end].iter().collect::<String>(), "Add");
}

#[test]
fn the_preferences_are_kept_on_the_server_and_checked() {
    let fixture = Fixture::new();
    let none = fixture.call("preferences.get", json!(null)).unwrap();
    assert_eq!(none, json!({"theme": null, "language": null}));
    let chosen = json!({"theme": "petrol", "language": "de"});
    assert_eq!(
        fixture.call("preferences.set", chosen.clone()).unwrap(),
        chosen
    );
    assert_eq!(
        fixture.call("preferences.get", json!(null)).unwrap(),
        chosen
    );
    let bad = fixture.call(
        "preferences.set",
        json!({"theme": "../x", "language": null}),
    );
    assert_eq!(bad, Err("API-101"));
    assert_eq!(
        fixture.call("preferences.get", json!(null)).unwrap(),
        chosen
    );
    let cleared = json!({"theme": null, "language": null});
    fixture.call("preferences.set", cleared.clone()).unwrap();
    assert_eq!(
        fixture.call("preferences.get", json!(null)).unwrap(),
        cleared
    );
}

#[test]
fn only_a_manager_may_change_the_preferences() {
    let fixture = Fixture::new();
    let chosen = json!({"theme": "dunkel", "language": "en"});
    let as_reader = |id: &str, input: Value| run_as(&fixture, principal::reader(), id, input);
    assert!(as_reader("preferences.get", json!(null)).is_ok());
    assert_eq!(
        as_reader("preferences.set", chosen.clone()).unwrap_err(),
        "AUTH-401"
    );
    let nobody = run_as(
        &fixture,
        principal::nobody(),
        "preferences.get",
        json!(null),
    );
    assert_eq!(nobody.unwrap_err(), "AUTH-401");
    assert_eq!(
        fixture.call("preferences.get", json!(null)).unwrap(),
        json!({"theme": null, "language": null})
    );
}

#[test]
fn a_rescan_in_the_browser_shows_in_the_usages_of_the_running_server_at_once() {
    let fixture = Fixture::new();
    let input = add_usages_input(&fixture, CALLER, "incoming", 1);
    let places = |answer: &Value| answer["summary"]["places"].as_u64().unwrap();
    let before = fixture.call("usages.query", input.clone()).unwrap();
    // An unchanged file: a new snapshot that carries everything.
    fixture
        .call("projects.rescan", json!({ "project": input["project"] }))
        .unwrap();
    let same = fixture.call("usages.query", input.clone()).unwrap();
    assert_eq!(places(&same), places(&before));
    // A changed file: the new snapshot, not the one the server asked about before.
    let twice = CALLER.replace(
        "        return calculator.Add(1, 2);\n",
        "        var one = calculator.Add(1, 2);\n        return calculator.Add(one, 3);\n",
    );
    fs::write(fixture.root.path().join("app/Checkout.cs"), twice).unwrap();
    fixture
        .call("projects.rescan", json!({ "project": input["project"] }))
        .unwrap();
    let after = fixture.call("usages.query", input).unwrap();
    assert!(places(&after) > places(&before));
    assert!(after["snapshot"].as_u64() > before["snapshot"].as_u64());
}
