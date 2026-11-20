//! The HTML export through the same path the server takes.

use std::num::NonZeroU32;

use codeskimr_domain::{FolderPath, Project, ProjectId, ProjectName, ProjectSource};
use codeskimr_storage::SqliteStore;
use serde_json::{Value, json};

use std::fs;

use codeskimr_codes::{Coded, ErrorCode};

use super::export::{Embedded, page_from};
use super::tests::Fixture;
use crate::principal;

const PAGE: &str = "<html><script id=\"codeskimr-data\" type=\"application/json\">__CODESKIMR_EXPORT_DATA__</script></html>";
const DATA_START: &str = "type=\"application/json\">";
const DATA_END: &str = "</script>";

impl Fixture {
    /// The data block of the page, read back as JSON.
    fn data(&self, project: u64) -> Value {
        let out = self
            .call("export.html", json!({ "project": project }))
            .unwrap();
        let html = out["html"].as_str().unwrap();
        let start = html.find(DATA_START).unwrap() + DATA_START.len();
        let length = html[start..].find(DATA_END).unwrap();
        serde_json::from_str(&html[start..start + length]).unwrap()
    }
}

#[test]
fn the_export_is_offered_on_every_surface_that_lists_commands() {
    let fixture = Fixture::installed(Some(PAGE));
    let listed = fixture.api.list(&principal::resolve());
    let info = listed.iter().find(|c| c.id == "export.html").unwrap();
    assert!(info.read_only);
}

#[test]
fn the_page_holds_the_symbols_and_the_usages_of_the_scan() {
    let fixture = Fixture::installed(Some(PAGE));
    let project = fixture.scanned();
    let data = fixture.data(project);
    assert_eq!(data["format"], 1);
    assert_eq!(data["project"]["name"], "app");
    let symbols = data["symbols"].as_object().unwrap();
    let names: Vec<_> = symbols
        .values()
        .map(|s| s["name"].as_str().unwrap())
        .collect();
    assert!(
        names.contains(&"Calculator") && names.contains(&"Add"),
        "{names:?}"
    );
    let usages = data["usages"].as_object().unwrap();
    assert_eq!(usages.len(), symbols.len());
    let some = usages.values().next().unwrap();
    assert_eq!(some["answer"]["level2"], "unavailable");
    assert!(some["summary_without_tests"]["unchecked"].is_number());
}

#[test]
fn the_page_holds_no_source_text() {
    let fixture = Fixture::installed(Some(PAGE));
    let project = fixture.scanned();
    let out = fixture
        .call("export.html", json!({ "project": project }))
        .unwrap();
    let html = out["html"].as_str().unwrap();
    assert!(!html.contains("return a + b"));
    assert!(!html.contains(&fixture.root.path().to_string_lossy().into_owned()));
    assert_eq!(out["symbols"], 2);
}

#[test]
fn without_the_built_page_the_export_fails_with_its_code() {
    let fixture = Fixture::installed(None);
    let project = fixture.scanned();
    let result = fixture.call("export.html", json!({ "project": project }));
    assert_eq!(result, Err("EXP-201"));
}

#[test]
fn a_project_that_was_not_scanned_or_not_registered_is_refused() {
    let fixture = Fixture::installed(Some(PAGE));
    let store = SqliteStore::open(&fixture.locations.database(), NonZeroU32::MIN).unwrap();
    let project = Project {
        id: ProjectId::new(1),
        name: ProjectName::new("app").unwrap(),
        source: ProjectSource::Folder(FolderPath::new(fixture.folder()).unwrap()),
        layers: Vec::new(),
    };
    store.save_project(&project).unwrap();
    assert_eq!(
        fixture.call("export.html", json!({ "project": 1 })),
        Err("MCP-203")
    );
    assert_eq!(
        fixture.call("export.html", json!({ "project": 99 })),
        Err("PRJ-201")
    );
}

const PAGE_NAME: &str = "export/export.html";
const PAGE_TEXT: &str = "<html>__CODESKIMR_EXPORT_DATA__</html>";
const EMBEDDED: Embedded = &[("index.html", b"x"), (PAGE_NAME, PAGE_TEXT.as_bytes())];

#[test]
fn without_a_ui_folder_the_page_comes_from_the_program() {
    assert_eq!(page_from(None, EMBEDDED).unwrap(), PAGE_TEXT);
    let error = page_from(None, &[("index.html", b"x")]).unwrap_err();
    assert_eq!(error.code(), ErrorCode::ExportBundleMissing);
}

#[test]
fn an_existing_ui_folder_wins_over_the_program() {
    let dir = tempfile::tempdir().unwrap();
    let folder = dir.path().join("export");
    fs::create_dir_all(&folder).unwrap();
    fs::write(folder.join("export.html"), "from the folder").unwrap();
    assert_eq!(
        page_from(Some(dir.path()), EMBEDDED).unwrap(),
        "from the folder"
    );
    let empty = tempfile::tempdir().unwrap();
    assert!(page_from(Some(empty.path()), EMBEDDED).is_err());
}

#[test]
fn the_hits_name_their_places_by_id_and_the_walk_is_direct_only() {
    let fixture = Fixture::installed(Some(PAGE));
    let caller = "namespace Demo;\n\npublic class Use\n{\n    public int Run(Calculator c) { return c.Add(1, 2); }\n}\n";
    std::fs::write(fixture.root.path().join("app/Use.cs"), caller).unwrap();
    let project = fixture.scanned();
    let data = fixture.data(project);
    let places = data["places"].as_object().unwrap();
    let hits: Vec<_> = data["usages"]
        .as_object()
        .unwrap()
        .values()
        .flat_map(|u| ["incoming", "outgoing"].map(|way| u["answer"][way]["hits"].clone()))
        .filter_map(|hits| hits.as_array().cloned())
        .flatten()
        .collect();
    assert!(!hits.is_empty(), "the scan has references");
    for hit in &hits {
        assert_eq!(hit["depth"], 1);
        for end in ["from", "to"] {
            let id = hit[end].as_u64().expect("a place is named by its id");
            assert!(places.contains_key(&id.to_string()));
        }
    }
}
