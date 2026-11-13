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

use super::command_api;
use crate::locations::Locations;
use crate::principal;

const SOURCE: &str = "namespace Demo;\n\npublic class Calculator\n{\n    public int Add(int a, int b) { return a + b; }\n}\n";

struct Fixture {
    root: TempDir,
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
            api: Arc::new(api),
        }
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
            "explorer.symbol",
            "projects.add",
            "projects.list",
            "projects.rescan"
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
