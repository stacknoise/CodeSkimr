//! Acceptance of ADR 0033, decision 8: a project whose indexer is not installed shows what is
//! missing in all three places, the UI command, the summary of `codeskimr scan` and the MCP tool
//! `get_project`. The tools are started through a probe that says which exist; no process runs.

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::sync::mpsc::channel;

use codeskimr_application::error::CodedError;
use codeskimr_application::events::EventBus;
use codeskimr_application::ports::{ProbeOutcome, ProjectToolCheck, ToolProbe};
use codeskimr_cli::{GlobalOptions, ScanArgs, scan_finished, scan_json};
use codeskimr_codes::ErrorCode;
use codeskimr_domain::{ProjectId, ScanMode, ToolReport};
use codeskimr_i18n::Messages;
use serde_json::{Value, json};
use tokio::sync::oneshot;

use crate::locations::Locations;
use crate::mcp::http_service_probing;
use crate::mcp_http_tests::{locations, post, rpc, token_store};
use crate::memory_keystore::secrets;
use crate::serve::{bare_parts, serve_for_test, test_ui};
use crate::wiring::{open_store, tool_checker};
use crate::{config_files, principal, scan, ui_commands};

const SOURCE: &str = "namespace Demo;\n\npublic class Calculator\n{\n    public int Add(int a, int b) { return a + b; }\n}\n";
/// The project asks for SDK 9.0.100 and the machine has 8.0.404; the indexer is not installed.
const GLOBAL_JSON: &str = r#"{ "sdk": { "version": "9.0.100" } }"#;
const INSTALLED_SDK: &str = "8.0.404\n";

/// What the machine has: the SDK, but no indexer.
struct Machine(HashMap<&'static str, &'static str>);

impl ToolProbe for Machine {
    fn probe(&self, command: &str, _args: &[String]) -> ProbeOutcome {
        self.0
            .get(command)
            .map_or(ProbeOutcome::NotFound, |output| {
                ProbeOutcome::Ran((*output).to_owned())
            })
    }
}

fn machine() -> Arc<dyn ToolProbe> {
    Arc::new(Machine(HashMap::from([("dotnet", INSTALLED_SDK)])))
}

/// A project folder and an installation, both in one temporary folder.
struct World {
    root: tempfile::TempDir,
    locations: Locations,
}

impl World {
    fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        let project = root.path().join("app");
        fs::create_dir_all(&project).unwrap();
        fs::write(project.join("Calculator.cs"), SOURCE).unwrap();
        fs::write(project.join("global.json"), GLOBAL_JSON).unwrap();
        let locations = locations(&root.path().join("data"));
        Self { root, locations }
    }

    fn project(&self) -> String {
        self.root.path().join("app").to_string_lossy().into_owned()
    }

    fn api(&self) -> Arc<codeskimr_server::CommandApi> {
        let api = ui_commands::command_api_probing(
            &GlobalOptions::default(),
            &self.locations,
            (EventBus::new(), secrets()),
            Some(machine()),
        )
        .unwrap();
        Arc::new(api)
    }
}

fn ask(api: &Arc<codeskimr_server::CommandApi>, id: &str, input: Value) -> Value {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let api = Arc::clone(api);
    let call = api.run(principal::resolve(), id, input);
    runtime.block_on(call).unwrap()
}

fn tool<'a>(tools: &'a Value, name: &str) -> &'a Value {
    tools
        .as_array()
        .unwrap()
        .iter()
        .find(|tool| tool["tool"] == name)
        .unwrap()
}

fn assert_indexer_is_reported(tools: &Value) {
    let indexer = tool(tools, "scip-dotnet");
    assert_eq!(indexer["state"], "missing");
    assert_eq!(indexer["role"], "indexer");
    assert_eq!(indexer["languages"], json!(["csharp"]));
    let sdk = tool(tools, "dotnet");
    assert_eq!(sdk["state"], "too_old");
    assert_eq!(sdk["found"], "8.0.404");
    assert_eq!(sdk["required"], "9.0.100");
}

#[test]
fn the_ui_commands_show_the_missing_tools_when_a_project_is_added_listed_and_opened() {
    let world = World::new();
    let api = world.api();
    let added = ask(&api, "projects.add", json!({ "source": world.project() }));
    assert_indexer_is_reported(&added["tools"]["tools"]);
    let indexer = tool(&added["tools"]["tools"], "scip-dotnet");
    assert_eq!(indexer["purpose_key"], "tool-scip-dotnet-purpose");
    assert_eq!(indexer["install_key"], "tool-scip-dotnet-install");
    assert_eq!(added["tools"]["languages"][0]["level2_ready"], false);

    let listed = ask(&api, "projects.list", json!(null));
    assert_indexer_is_reported(&listed[0]["tools"]["tools"]);

    let opened = ask(
        &api,
        "projects.tools",
        json!({ "project": added["project"] }),
    );
    assert_indexer_is_reported(&opened["tools"]);
}

#[test]
fn a_project_whose_languages_have_no_level_2_shows_no_tools() {
    let world = World::new();
    let go = world.root.path().join("go-app");
    fs::create_dir_all(&go).unwrap();
    fs::write(go.join("main.go"), "package main\n\nfunc main() {}\n").unwrap();
    let added = ask(
        &world.api(),
        "projects.add",
        json!({ "source": go.to_string_lossy() }),
    );
    assert_eq!(added["tools"]["tools"], json!([]));
    assert_eq!(added["tools"]["languages"], json!([]));
    assert!(
        added["tools"]["checked_at"].is_number(),
        "it was checked, there was nothing to find"
    );
}

/// A check that cannot store its result.
struct BrokenCheck;

impl ProjectToolCheck for BrokenCheck {
    fn check(&self, _project: ProjectId) -> Result<ToolReport, CodedError> {
        Err(CodedError::new(ErrorCode::DatabaseFailed))
    }

    fn stored(&self, _project: ProjectId) -> Result<Option<ToolReport>, CodedError> {
        Ok(None)
    }
}

#[test]
fn a_check_that_fails_never_fails_the_scan() {
    let world = World::new();
    let args = ScanArgs {
        source: world.project(),
        branch: None,
        credential: None,
        json: false,
        watch: false,
    };
    let session = scan::open(
        scan::prepare(&args).unwrap(),
        &GlobalOptions::default(),
        &world.locations,
        &secrets(),
    )
    .unwrap();
    let summary = session
        .with_tools(Arc::new(BrokenCheck))
        .scan(ScanMode::Full)
        .unwrap();
    assert_eq!(summary.files, 1);
    assert_eq!(summary.tools.checked_at, None);
}

fn summary_of(world: &World) -> codeskimr_cli::ScanSummary {
    let global = GlobalOptions::default();
    let args = ScanArgs {
        source: world.project(),
        branch: None,
        credential: None,
        json: false,
        watch: false,
    };
    let config = config_files::load(&world.locations, &global, None).unwrap();
    let store = open_store(&world.locations, &config).unwrap();
    let checker = tool_checker(&store, &config, (&world.locations, machine())).unwrap();
    let spec = scan::prepare(&args).unwrap();
    scan::open(spec, &global, &world.locations, &secrets())
        .unwrap()
        .with_tools(checker)
        .scan(ScanMode::Full)
        .unwrap()
}

#[test]
fn the_scan_summary_names_the_missing_tool_its_purpose_and_how_to_install_it() {
    let world = World::new();
    let summary = summary_of(&world);
    assert_indexer_is_reported(&serde_json::to_value(&summary.tools.tools).unwrap());
    let messages = Messages::load().unwrap();

    let english = scan_finished(&messages, "en", &summary).unwrap();
    let missing = english
        .lines()
        .find(|line| line.starts_with("Tool missing for csharp: scip-dotnet"))
        .unwrap();
    assert!(missing.contains("exact usages of C# code"), "{missing}");
    assert!(
        missing.contains("dotnet tool install --global scip-dotnet"),
        "{missing}"
    );
    let old = english
        .lines()
        .find(|line| line.contains("Tool too old"))
        .unwrap();
    assert!(old.contains("8.0.404") && old.contains("9.0.100"), "{old}");

    let german = scan_finished(&messages, "de", &summary).unwrap();
    assert!(
        german.contains("Werkzeug fehlt für csharp: scip-dotnet"),
        "{german}"
    );
    assert!(german.contains("Installation mit"), "{german}");

    let machine_readable = scan_json(&summary).unwrap();
    assert!(
        machine_readable.contains("\"state\":\"missing\""),
        "{machine_readable}"
    );
}

#[test]
fn a_scan_whose_tools_are_all_there_prints_no_tool_line() {
    let world = World::new();
    let global = GlobalOptions::default();
    let config = config_files::load(&world.locations, &global, None).unwrap();
    let store = open_store(&world.locations, &config).unwrap();
    let everything = Machine(HashMap::from([
        ("dotnet", "9.0.101"),
        ("scip-dotnet", "0.2.1"),
    ]));
    let checker = tool_checker(&store, &config, (&world.locations, Arc::new(everything))).unwrap();
    let args = ScanArgs {
        source: world.project(),
        branch: None,
        credential: None,
        json: false,
        watch: false,
    };
    let summary = scan::open(
        scan::prepare(&args).unwrap(),
        &global,
        &world.locations,
        &secrets(),
    )
    .unwrap()
    .with_tools(checker)
    .scan(ScanMode::Full)
    .unwrap();
    let text = scan_finished(&Messages::load().unwrap(), "en", &summary).unwrap();
    assert_eq!(text.lines().count(), 2, "{text}");
    assert!(summary.tools.languages[0].level2_ready);
}

fn call_get_project(url: &str, token: &str, project: u64) -> Value {
    let params = json!({"name": "get_project", "arguments": {"project": project}});
    post(url, Some(token), &rpc(2, "tools/call", &params)).1["result"].clone()
}

#[test]
fn the_mcp_tool_get_project_names_the_missing_tool_and_how_to_install_it() {
    let world = World::new();
    let added = ask(
        &world.api(),
        "projects.add",
        json!({ "source": world.project() }),
    );
    let project = added["project"].as_u64().unwrap();
    let config = config_files::load(&world.locations, &GlobalOptions::default(), None).unwrap();
    let (store, token) = token_store();
    let mut parts = bare_parts(test_ui());
    parts.mcp = Some(
        http_service_probing(
            &config,
            &world.locations,
            (parts.events.clone(), &secrets()),
            Some(machine()),
        )
        .unwrap(),
    );
    let (stop, stopped) = oneshot::channel::<()>();
    let (found, url) = channel::<String>();
    let client = std::thread::spawn(move || {
        let url = url.recv().unwrap();
        let answer = call_get_project(&url, &token, project);
        stop.send(()).unwrap();
        answer
    });
    serve_for_test(&store, parts, &found, stopped).unwrap();
    let answer = client.join().unwrap();
    assert_get_project_answer(&answer["structuredContent"]);
}

fn assert_get_project_answer(content: &Value) {
    assert_eq!(content["languages"][0]["language"], "csharp");
    assert_eq!(content["languages"][0]["level2_ready"], false);
    assert_indexer_is_reported(&content["tools"]);
    let indexer = tool(&content["tools"], "scip-dotnet");
    let purpose = indexer["purpose"].as_str().unwrap();
    assert!(purpose.contains("exact usages"));
    let install = indexer["install"].as_str().unwrap();
    assert!(install.contains("dotnet tool install"));
}

/// Every message a language pack names must exist in both languages, or the user would see a key.
#[test]
fn every_tool_text_the_language_packs_name_exists_in_german_and_english() {
    let packs = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../languages");
    let catalog = codeskimr_parsing::PackTools::load(&packs).unwrap();
    let messages = Messages::load().unwrap();
    let mut checked = 0;
    for name in [
        "csharp",
        "typescript",
        "javascript",
        "tsx",
        "python",
        "rust",
        "java",
        "kotlin",
    ] {
        let language = codeskimr_domain::LanguageId::new(name).unwrap();
        for spec in codeskimr_application::ports::ToolCatalog::tools_of(&catalog, &language) {
            for key in [&spec.purpose_key, &spec.install_key] {
                for text_language in ["en", "de"] {
                    let text = messages.format(text_language, key, None).unwrap();
                    assert_ne!(&text, key);
                    checked += 1;
                }
            }
        }
    }
    assert_eq!(checked, 8 * 2 * 2 * 2);
}
