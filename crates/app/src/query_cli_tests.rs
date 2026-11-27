//! `codeskimr symbols`, `usages`, `impact`, `context` and `unused` over a real, scanned project:
//! the JSON a script reads (`schema`, `stale`, the failure with its code) and the text a person
//! reads.

use std::fs;
use std::path::PathBuf;

use codeskimr_cli::{
    ContextQuery, GlobalOptions, ImpactQuery, QueryAction, QueryTarget, RenameQuery, RulesQuery,
    ScopeArgs, SymbolsQuery, UnusedQuery, UsagesQuery,
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

pub(crate) struct World {
    _root: tempfile::TempDir,
    pub(crate) folder: PathBuf,
    data: Locations,
    global: GlobalOptions,
    messages: Messages,
    pub(crate) session: Session,
}

impl World {
    pub(crate) fn new() -> Self {
        let done = scanned(&[("Target.cs", TARGET), ("Caller.cs", CALLER)]);
        Self {
            _root: done.root,
            folder: done.folder,
            data: done.data,
            global: done.global,
            messages: Messages::load().unwrap(),
            session: done.session,
        }
    }

    pub(crate) fn run(
        &self,
        action: &QueryAction,
        language: &str,
    ) -> Result<String, query_cli::QueryFailure> {
        let around = Surroundings {
            global: &self.global,
            locations: &self.data,
            messages: &self.messages,
            language,
        };
        query_cli::run(action, &around, &self.folder).map(|answered| answered.text)
    }

    fn answered(&self, action: &QueryAction) -> query_cli::Answered {
        let around = Surroundings {
            global: &self.global,
            locations: &self.data,
            messages: &self.messages,
            language: "en",
        };
        query_cli::run(action, &around, &self.folder).unwrap()
    }

    pub(crate) fn json(&self, action: &QueryAction) -> Value {
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
fn a_new_file_in_a_folder_the_scope_leaves_out_does_not_make_the_answer_stale() {
    let world = World::new();
    fs::create_dir_all(world.folder.join("legacy")).unwrap();
    fs::write(
        world.folder.join("legacy/Old.cs"),
        TARGET.replace("Target", "Old"),
    )
    .unwrap();
    assert_eq!(world.json(&context(true))["stale"], true);
    let around = Surroundings {
        global: &world.global,
        locations: &world.data,
        messages: &world.messages,
        language: "en",
    };
    let args = ScopeArgs {
        project: world.folder.to_string_lossy().into_owned(),
        without_folders: vec!["legacy".to_owned()],
        ..ScopeArgs::default()
    };
    crate::scope_cli::run_scope(&args, &around).unwrap();
    assert_eq!(world.json(&context(true))["stale"], false);
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
        reason: None,
        include_tests: false,
        limit: None,
        target: target(json),
    })
}

fn unused_names(answer: &Value) -> Vec<&str> {
    answer["unused"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["qualified_name"].as_str().unwrap())
        .collect()
}

#[test]
fn unused_lists_what_nobody_calls_with_the_reason_and_leaves_out_what_is_called() {
    let world = World::new();
    let answer = world.json(&unused(true));
    assert_eq!(answer["schema"], 1);
    let names = unused_names(&answer);
    assert!(names.contains(&"Demo.Caller.Run"), "{names:?}");
    assert!(!names.contains(&SYMBOL), "{names:?}");
    assert!(answer["unused"][0]["reason"].is_string());
    assert_eq!(answer["left_out"], serde_json::json!([]));
    let text = world.run(&unused(false), "en").unwrap();
    assert!(text.contains("Demo.Caller.Run"), "{text}");
}

#[test]
fn an_entry_of_the_exception_list_moves_the_symbol_to_left_out_with_its_reason() {
    let world = World::new();
    let list = "[[unused.ignore]]\nsymbols = [\"*.Run\"]\nreason = \"called by the host\"\n";
    fs::write(world.folder.join(".codeskimr.toml"), list).unwrap();
    let answer = world.json(&unused(true));
    let names = unused_names(&answer);
    assert!(!names.contains(&"Demo.Caller.Run"), "{names:?}");
    assert_eq!(answer["left_out"][0]["qualified_name"], "Demo.Caller.Run");
    assert_eq!(
        answer["left_out"][0]["rule"]["reason"],
        "called by the host"
    );
    assert_eq!(answer["counts"]["left_out"], 1);
    let text = world.run(&unused(false), "en").unwrap();
    assert!(text.contains("because: called by the host"), "{text}");
}

#[test]
fn a_broken_exception_list_fails_the_command_with_its_code() {
    let world = World::new();
    let file = world.folder.join(".codeskimr.toml");
    for (text, code) in [
        (
            "[[unused.ignore]]\nsymbols = [\"*.Run\"]\n",
            ErrorCode::ConfigInvalidValue,
        ),
        ("[[unused.ignore]\n", ErrorCode::ConfigParseFailed),
    ] {
        fs::write(&file, text).unwrap();
        let failure = world.run(&unused(true), "en").unwrap_err();
        assert_eq!(failure.code(), code, "{text}");
        assert!(failure.json().contains(code.id()), "{text}");
    }
}

#[test]
fn every_assessment_and_kind_the_command_line_offers_is_accepted_by_the_tool() {
    let world = World::new();
    for assessment in ["likely", "public", "approximate"] {
        for kind in ["all", "types", "functions"] {
            let action = QueryAction::Unused(UnusedQuery {
                kind: Some(kind.to_owned()),
                language: None,
                assessment: Some(assessment.to_owned()),
                reason: None,
                include_tests: false,
                limit: None,
                target: target(true),
            });
            assert!(world.run(&action, "en").is_ok(), "{assessment} {kind}");
        }
    }
}

fn rename(new_name: &str, json: bool) -> QueryAction {
    QueryAction::Rename(RenameQuery {
        symbol: SYMBOL.to_owned(),
        new_name: new_name.to_owned(),
        limit: None,
        target: target(json),
    })
}

#[test]
fn the_rename_preview_lists_the_declaration_and_the_usages_and_says_what_it_cannot_know() {
    let world = World::new();
    let answer = world.json(&rename("Calculate", true));
    assert_eq!(answer["schema"], 1);
    assert_eq!(answer["new_name"], "Calculate");
    let places: Vec<(&str, &str)> = answer["sites"]["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|site| {
            (
                site["path"].as_str().unwrap(),
                site["kind"].as_str().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        places,
        [("Caller.cs", "usage"), ("Target.cs", "declaration")]
    );
    assert_eq!(answer["conflicts"], serde_json::json!([]));
    let text = world.run(&rename("Calculate", false), "en").unwrap();
    assert!(text.contains("CodeSkimr changes nothing"), "{text}");
    assert!(
        text.contains("does not say whether the code compiles"),
        "{text}"
    );
}

#[test]
fn the_rename_preview_finds_the_old_name_in_files_the_index_does_not_know() {
    let world = World::new();
    fs::write(
        world.folder.join("notes.md"),
        "Call Do in the host.\nDoes not match: Done.\n",
    )
    .unwrap();
    let answer = world.json(&rename("Calculate", true));
    let texts = &answer["text_hits"];
    assert_eq!(texts["searched"], true);
    assert_eq!(texts["total"], 1, "{texts}");
    assert_eq!(texts["items"][0]["path"], "notes.md");
    assert_eq!(texts["items"][0]["kind"], "documentation");
    let text = world.run(&rename("Calculate", false), "en").unwrap();
    assert!(text.contains("notes.md:1  documentation"), "{text}");
}

#[test]
fn the_rename_preview_finds_a_conflict_in_the_same_scope_and_refuses_a_bad_name() {
    let world = World::new();
    let answer = world.json(&rename("Run", true));
    assert_eq!(answer["conflicts"], serde_json::json!([]));
    let clash = r"namespace Demo;

public class Target
{
    public static int Do(int x) { return x; }
    public static int Calculate(int x) { return x; }
}
";
    let session = scanned(&[("Target.cs", clash), ("Caller.cs", CALLER)]);
    let other = World {
        _root: session.root,
        folder: session.folder,
        data: session.data,
        global: session.global,
        messages: Messages::load().unwrap(),
        session: session.session,
    };
    let found = other.json(&rename("Calculate", true));
    assert_eq!(
        found["conflicts"][0]["qualified_name"],
        "Demo.Target.Calculate"
    );
    assert_eq!(found["conflicts"][0]["same_signature"], true);
    let failure = world.run(&rename("not a name", true), "en").unwrap_err();
    assert_eq!(failure.code(), ErrorCode::ToolArgumentsInvalid);
}

fn rules(json: bool) -> QueryAction {
    QueryAction::Rules(RulesQuery {
        limit: None,
        hold: false,
        target: target(json),
    })
}

fn hold_rules(json: bool) -> QueryAction {
    QueryAction::Rules(RulesQuery {
        limit: None,
        hold: true,
        target: target(json),
    })
}

const FORBID: &str = "[[rules.forbid]]\nfrom = \"Demo.Caller\"\nto = \"Demo.Target\"\n";
const HELD: &str = "held_on = \"2026-11-25\"\n\n[[entries]]\nrule = \"forbid:Demo.Caller>Demo.Target\"\nfrom = \"Demo.Caller.Run\"\nto = \"Demo.Target.Do\"\n\n[[entries]]\nrule = \"forbid:Demo.Caller>Demo.Target\"\nfrom = \"Demo.Caller.Run\"\nto = \"Demo.Target\"\n";

#[test]
fn a_new_violation_of_a_rule_is_listed_with_its_place_and_fails_the_run() {
    let world = World::new();
    fs::write(world.folder.join(".codeskimr.toml"), FORBID).unwrap();
    let answered = world.answered(&rules(true));
    assert!(answered.failing);
    let answer: Value = serde_json::from_str(&answered.text).unwrap();
    // The class and its method are two references of the one call.
    assert_eq!(answer["new"], 2);
    let violation = &answer["rules"][0]["violations"][1];
    assert_eq!(violation["from"], "Demo.Caller.Run");
    assert_eq!(violation["to"], "Demo.Target.Do");
    assert_eq!(violation["places"][0]["path"], "Caller.cs");
    let text = world.answered(&rules(false)).text;
    assert!(text.contains("Demo.Caller.Run -> Demo.Target.Do"), "{text}");
    assert!(text.contains("NEW"), "{text}");
}

#[test]
fn a_held_violation_is_an_old_burden_and_does_not_fail_the_run() {
    let world = World::new();
    fs::write(world.folder.join(".codeskimr.toml"), FORBID).unwrap();
    fs::write(world.folder.join(".codeskimr-baseline.toml"), HELD).unwrap();
    let answered = world.answered(&rules(true));
    assert!(!answered.failing);
    let answer: Value = serde_json::from_str(&answered.text).unwrap();
    assert_eq!(answer["old"], 2);
    assert_eq!(answer["new"], 0);
    let text = world.answered(&rules(false)).text;
    assert!(text.contains("held"), "{text}");
}

#[test]
fn a_project_without_rules_says_so_and_does_not_fail() {
    let world = World::new();
    let answered = world.answered(&rules(false));
    assert!(!answered.failing);
    assert!(
        answered.text.contains("no architecture rules"),
        "{}",
        answered.text
    );
}

#[test]
fn a_broken_rule_or_held_state_fails_the_command_with_its_code() {
    let world = World::new();
    let rules_file = world.folder.join(".codeskimr.toml");
    let held_file = world.folder.join(".codeskimr-baseline.toml");
    fs::write(
        &rules_file,
        "[[rules.forbid]]\nfrom = \"Demo\"\nto = \"Demo.Target\"\n",
    )
    .unwrap();
    let failure = world.run(&rules(true), "en").unwrap_err();
    assert_eq!(failure.code(), ErrorCode::ConfigInvalidValue);
    fs::write(&rules_file, FORBID).unwrap();
    fs::write(&held_file, "[[entries\n").unwrap();
    let failure = world.run(&rules(true), "en").unwrap_err();
    assert!(failure.json().contains(failure.code().id()));
}

#[test]
fn holding_the_state_from_the_command_line_makes_every_violation_an_old_burden() {
    let world = World::new();
    fs::write(world.folder.join(".codeskimr.toml"), FORBID).unwrap();
    assert!(world.answered(&rules(true)).failing);
    let held = world.answered(&hold_rules(true));
    assert!(!held.failing);
    let answer: Value = serde_json::from_str(&held.text).unwrap();
    assert_eq!(answer["entries"], 2);
    let file = fs::read_to_string(world.folder.join(".codeskimr-baseline.toml")).unwrap();
    assert!(file.contains(answer["held_on"].as_str().unwrap()), "{file}");
    assert!(!world.answered(&rules(true)).failing);
    let text = world.answered(&hold_rules(false)).text;
    assert!(
        text.contains("2 violations now count as old burdens"),
        "{text}"
    );
}
