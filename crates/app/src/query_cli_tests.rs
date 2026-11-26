//! `codeskimr symbols`, `usages`, `impact`, `context` and `unused` over a real, scanned project: the JSON a
//! script reads (`schema`, `stale`, the failure with its code) and the text a person reads.

use std::fs;
use std::path::PathBuf;

use codeskimr_cli::{
    ContextQuery, GlobalOptions, ImpactQuery, QueryAction, QueryTarget, SymbolsQuery, UnusedQuery,
    UsagesQuery,
};
use codeskimr_codes::ErrorCode;
use codeskimr_i18n::Messages;
use serde_json::Value;

use crate::annotations_cli::Surroundings;
use crate::annotations_cli_tests::scanned;
use crate::locations::Locations;
use crate::query_cli;
use crate::scan::Session;

const TARGET: &str = "namespace Demo;\n\npublic class Target\n{\n    public static int Do(int x)\n    {\n        return x + 1;\n    }\n}\n";
const CALLER: &str = "namespace Demo;\n\npublic class Caller\n{\n    public int Run()\n    {\n        return Target.Do(1);\n    }\n}\n";
const SYMBOL: &str = "Demo.Target.Do";

struct World {
    _root: tempfile::TempDir,
    folder: PathBuf,
    data: Locations,
    global: GlobalOptions,
    messages: Messages,
    _session: Session,
}

impl World {
    fn new() -> Self {
        let done = scanned(&[("Target.cs", TARGET), ("Caller.cs", CALLER)]);
        Self {
            _root: done.root,
            folder: done.folder,
            data: done.data,
            global: done.global,
            messages: Messages::load().unwrap(),
            _session: done.session,
        }
    }

    fn run(&self, action: &QueryAction, language: &str) -> Result<String, query_cli::QueryFailure> {
        let around = Surroundings {
            global: &self.global,
            locations: &self.data,
            messages: &self.messages,
            language,
        };
        query_cli::run(action, &around, &self.folder)
    }

    fn json(&self, action: &QueryAction) -> Value {
        serde_json::from_str(&self.run(action, "en").unwrap()).unwrap()
    }
}

fn target(json: bool) -> QueryTarget {
    QueryTarget {
        project: None,
        json,
    }
}

fn context(json: bool) -> QueryAction {
    QueryAction::Context(ContextQuery {
        symbol: SYMBOL.to_owned(),
        budget: None,
        target: target(json),
    })
}

#[test]
fn every_json_answer_names_its_schema_and_says_the_scan_is_current() {
    let world = World::new();
    let search = QueryAction::Symbols(SymbolsQuery {
        query: "Do".to_owned(),
        kind: None,
        limit: None,
        target: target(true),
    });
    let found = world.json(&search);
    assert_eq!(found["schema"], 1);
    assert_eq!(found["stale"], false);
    assert!(
        found["hits"]
            .as_array()
            .unwrap()
            .iter()
            .any(|hit| hit["qualified_name"] == SYMBOL)
    );
}

#[test]
fn the_context_holds_the_callers_the_affected_files_and_no_source() {
    let world = World::new();
    let answer = world.json(&context(true));
    assert_eq!(answer["symbol"]["qualified_name"], SYMBOL);
    assert_eq!(answer["callers"]["total"], 1);
    assert_eq!(
        answer["callers"]["items"][0]["from"]["qualified_name"],
        "Demo.Caller.Run"
    );
    let files: Vec<&str> = answer["affected_files"]["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|file| file.as_str().unwrap())
        .collect();
    assert!(files.contains(&"Caller.cs"));
    assert!(!answer.to_string().contains("return x + 1"));
}

#[test]
fn usages_and_impact_answer_like_their_tools() {
    let world = World::new();
    let usages = QueryAction::Usages(UsagesQuery {
        symbol: SYMBOL.to_owned(),
        direction: None,
        depth: None,
        include_tests: false,
        limit: None,
        target: target(true),
    });
    assert_eq!(world.json(&usages)["total"], 1);
    let impact = QueryAction::Impact(ImpactQuery {
        symbol: SYMBOL.to_owned(),
        limit: Some(5),
        target: target(true),
    });
    assert_eq!(
        world.json(&impact)["symbols"][0]["qualified_name"],
        "Demo.Caller.Run"
    );
}

#[test]
fn a_file_changed_after_the_scan_makes_the_answer_stale_and_the_text_says_so() {
    let world = World::new();
    fs::write(
        world.folder.join("Target.cs"),
        TARGET.replace("x + 1", "x + 2"),
    )
    .unwrap();
    assert_eq!(world.json(&context(true))["stale"], true);
    let english = world.run(&context(false), "en").unwrap();
    assert!(english.contains("Callers (1 of 1):"), "{english}");
    assert!(english.contains("may be out of date"), "{english}");
    let german = world.run(&context(false), "de").unwrap();
    assert!(german.contains("Aufrufer (1 von 1):"), "{german}");
    assert!(german.contains("veraltet"), "{german}");
}

#[test]
fn a_file_that_went_away_or_came_new_makes_the_answer_stale_too() {
    let world = World::new();
    assert_eq!(world.json(&context(true))["stale"], false);
    fs::write(
        world.folder.join("Extra.cs"),
        TARGET.replace("Target", "Extra"),
    )
    .unwrap();
    assert_eq!(world.json(&context(true))["stale"], true);
    fs::remove_file(world.folder.join("Extra.cs")).unwrap();
    assert_eq!(world.json(&context(true))["stale"], false);
    fs::remove_file(world.folder.join("Caller.cs")).unwrap();
    assert_eq!(world.json(&context(true))["stale"], true);
}

#[test]
fn an_unknown_symbol_fails_with_its_registered_code_as_json() {
    let world = World::new();
    let action = QueryAction::Context(ContextQuery {
        symbol: "Demo.Missing".to_owned(),
        budget: None,
        target: target(true),
    });
    let failure = world.run(&action, "en").err().unwrap();
    assert_eq!(failure.code(), ErrorCode::SymbolNotFound);
    let json: Value = serde_json::from_str(&failure.json()).unwrap();
    assert_eq!(json["schema"], 1);
    assert_eq!(json["error"]["code"], "MCP-202");
    assert_eq!(json["error"]["params"]["symbol"], "Demo.Missing");
}

#[test]
fn the_project_can_be_named_by_id_or_by_folder() {
    let world = World::new();
    for project in ["1".to_owned(), world.folder.to_string_lossy().into_owned()] {
        let action = QueryAction::Context(ContextQuery {
            symbol: SYMBOL.to_owned(),
            budget: Some("short".to_owned()),
            target: QueryTarget {
                project: Some(project),
                json: true,
            },
        });
        assert_eq!(world.json(&action)["budget"], "short");
    }
}

fn unused(json: bool) -> QueryAction {
    QueryAction::Unused(UnusedQuery {
        kind: Some("functions".to_owned()),
        language: None,
        assessment: None,
        include_tests: false,
        limit: None,
        target: target(json),
    })
}

#[test]
fn unused_lists_what_nobody_calls_with_the_reason_and_leaves_out_what_is_called() {
    let world = World::new();
    let answer = world.json(&unused(true));
    assert_eq!(answer["schema"], 1);
    let names: Vec<&str> = answer["unused"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["qualified_name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"Demo.Caller.Run"), "{names:?}");
    assert!(!names.contains(&SYMBOL), "{names:?}");
    assert!(answer["unused"][0]["reason"].is_string());
    assert_eq!(answer["left_out"], serde_json::json!([]));
    let text = world.run(&unused(false), "en").unwrap();
    assert!(text.contains("Demo.Caller.Run"), "{text}");
}
