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
use crate::locations::Locations;
use crate::principal;

const SOURCE: &str = "namespace Demo;\n\npublic class Calculator\n{\n    public int Add(int a, int b) { return a + b; }\n}\n";

struct Fixture {
    root: TempDir,
    locations: Locations,
    api: Arc<CommandApi>,
}

impl Fixture {
    fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir_all(root.path().join("app")).unwrap();
        fs::write(root.path().join("app/Calculator.cs"), SOURCE).unwrap();
        let locations = Locations {
            data_dir: root.path().join("data"),
            languages_dir: Path::new(env!("CARGO_MANIFEST_DIR")).join("../../languages"),
            user_config: None,
            bases: Vec::new(),
        };
        let api = command_api(&GlobalOptions::default(), &locations, EventBus::new()).unwrap();
        Self {
            root,
            locations,
            api: Arc::new(api),
        }
    }

    /// Gives the project these layers, through a second connection to the same database.
    fn set_layers(&self, project: u64, layers: &[(&str, &str)]) {
        let store = SqliteStore::open(&self.locations.database(), NonZeroU32::MIN).unwrap();
        let mut stored = store.project(ProjectId::new(project)).unwrap();
        stored.layers = layers
            .iter()
            .map(|(name, namespace)| Layer {
                name: LayerName::new(*name).unwrap(),
                namespaces: vec![QualifiedName::new(*namespace).unwrap()],
            })
            .collect();
        store.save_project(&stored).unwrap();
    }

    /// Registers the folder without scanning it, so the project has no snapshot.
    fn register_unscanned(&self) -> u64 {
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

    fn folder(&self) -> String {
        self.root.path().join("app").to_string_lossy().into_owned()
    }

    fn call(&self, id: &str, input: Value) -> Result<Value, &'static str> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        runtime
            .block_on(self.api.run(principal::resolve(), id, input))
            .map_err(|error| error.code().entry().id)
    }
}

#[test]
fn the_commands_are_offered() {
    let fixture = Fixture::new();
    let ids: Vec<_> = fixture
        .api
        .list(&principal::resolve())
        .into_iter()
        .map(|info| info.id)
        .collect();
    assert_eq!(
        ids,
        [
            "explorer.outline",
            "explorer.source",
            "explorer.symbol",
            "projects.add",
            "projects.list",
            "projects.rescan",
            "search.query",
            "status.info"
        ]
    );
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
    assert_eq!(namespace["name"], "Calculator.Demo");
    assert_eq!(namespace["types"][0]["name"], "Calculator");
    assert_eq!(namespace["types"][0]["kind"], "class");
}

#[test]
fn a_symbol_comes_with_breadcrumb_and_members() {
    let fixture = Fixture::new();
    let project = added(&fixture);
    let outline = fixture
        .call("explorer.outline", json!({ "project": project }))
        .unwrap();
    let id = outline["layers"][0]["namespaces"][0]["types"][0]["id"].clone();
    let shown = fixture
        .call(
            "explorer.symbol",
            json!({ "project": project, "symbol": id }),
        )
        .unwrap();
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
    fixture.set_layers(
        project,
        &[("Core", "Calculator.Demo"), ("Other", "Nowhere")],
    );
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
fn the_outline_names_the_limits_behind_the_colours() {
    let fixture = Fixture::new();
    let project = added(&fixture);
    let outline = fixture
        .call("explorer.outline", json!({ "project": project }))
        .unwrap();
    assert_eq!(outline["thresholds"]["complexity_good_max"], 5);
    assert_eq!(outline["thresholds"]["maintainability_medium_min"], 35);
}

#[test]
fn a_token_without_the_read_right_is_refused() {
    let fixture = Fixture::new();
    let project = added(&fixture);
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let nobody = principal::nobody();
    for id in [
        "explorer.outline",
        "explorer.source",
        "explorer.symbol",
        "search.query",
        "status.info",
    ] {
        let refused = runtime
            .block_on(
                fixture
                    .api
                    .run(nobody, id, json!({ "project": project, "symbol": 1 })),
            )
            .map_err(|error| error.code().entry().id);
        assert_eq!(refused.unwrap_err(), "AUTH-401", "{id}");
    }
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
fn the_search_of_a_project_without_a_scan_or_without_text_is_empty() {
    let fixture = Fixture::new();
    let project = fixture.register_unscanned();
    let found = fixture
        .call("search.query", json!({ "project": project, "query": "a" }))
        .unwrap();
    assert_eq!(found, json!({ "snapshot": null, "hits": [] }));
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
