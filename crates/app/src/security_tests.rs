//! The canary suite S-02 for the surfaces of the running app (blueprint 12.6): REST, MCP over
//! HTTP and the data folder.
//!
//! The keystore holds two secrets: the installation token and a credential. Whatever is asked of
//! the server, with the token, with a wrong token, with malformed input and with paths that
//! leave the project, no answer and no file in the data folder may contain either of them. The
//! token is the one secret the caller legitimately knows, so it is never expected back.

use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::sync::mpsc::channel;

use codeskimr_application::ports::{Credential, CredentialStore};
use codeskimr_cli::GlobalOptions;
use codeskimr_domain::CredentialId;
use secrecy::SecretString;
use serde_json::{Value, json};
use tokio::sync::oneshot;

use crate::mcp::http_service;
use crate::mcp_http_tests::{locations, rpc, send};
use crate::memory_keystore::secrets as memory_secrets;
use crate::serve::{Parts, serve_for_test, test_ui};
use crate::ui_commands::command_api;
use crate::{config_files, installation_token};

pub(crate) const CREDENTIAL_CANARY: &str = "ghp_CANARY0c4e7a91d2b84f6e9a30cc11";
/// A class name, so that a scan that followed the link would index it as a symbol.
const FILE_CANARY: &str = "CanaryOutside6b1d";
/// The same, inside the project: the control that the search finds what is really there.
const INSIDE: &str = "CanaryInside6b1d";
const COMMANDS: &str = "/api/v1/commands";

/// Everything one run of the server leaves behind for the checks.
pub(crate) struct Run {
    pub(crate) token: String,
    pub(crate) answers: Vec<String>,
}

/// A project folder, an outside file that must stay unreachable, and a data folder.
pub(crate) struct Folders {
    pub(crate) data: tempfile::TempDir,
    pub(crate) project: std::path::PathBuf,
}

pub(crate) fn folders() -> Folders {
    let data = tempfile::tempdir().unwrap();
    let project = data.path().join("project");
    fs::create_dir_all(project.join("src")).unwrap();
    let class = |name: &str| format!("namespace A; public class {name} {{ }}");
    fs::write(project.join("src/a.cs"), class(INSIDE)).unwrap();
    fs::write(data.path().join("outside.cs"), class(FILE_CANARY)).unwrap();
    // A link inside the project that points out of it (blueprint S-05).
    #[cfg(unix)]
    std::os::unix::fs::symlink(data.path().join("outside.cs"), project.join("src/link.cs"))
        .unwrap();
    Folders { data, project }
}

/// Starts the real server (REST and MCP, both with the token) over an in-memory keystore that
/// holds the canary credential, runs `traffic` against it and returns what was answered.
pub(crate) fn run(
    folders: &Folders,
    traffic: impl FnOnce(&str, &str) -> Vec<String> + Send + 'static,
) -> Run {
    let locations = locations(&folders.data.path().join("state"));
    let config = config_files::load(&locations, &GlobalOptions::default(), None).unwrap();
    let secrets = memory_secrets();
    let id = CredentialId::new("work").unwrap();
    let canary = Credential::new(
        SecretString::from("anton"),
        SecretString::from(CREDENTIAL_CANARY),
    );
    secrets.store(&id, &canary).unwrap();
    installation_token::load_or_create(&*secrets).unwrap();
    let token = installation_token::stored_token(&*secrets);
    let events = codeskimr_application::events::EventBus::new();
    let parts = Parts {
        ui: test_ui(),
        api: command_api(
            &GlobalOptions::default(),
            &locations,
            events.clone(),
            Arc::clone(&secrets),
        )
        .unwrap(),
        mcp: Some(http_service(&config, &locations, (events.clone(), &secrets)).unwrap()),
        events,
        token_lock: None,
    };
    let (stop, stopped) = oneshot::channel::<()>();
    let (found, url) = channel::<String>();
    let for_traffic = token.clone();
    let client = std::thread::spawn(move || {
        let answers = traffic(&url.recv().unwrap(), &for_traffic);
        stop.send(()).unwrap();
        answers
    });
    serve_for_test(&*secrets, parts, &found, stopped).unwrap();
    Run {
        token,
        answers: client.join().unwrap(),
    }
}

pub(crate) fn command(url: &str, token: Option<&str>, id: &str, input: &Value) -> String {
    send(
        url,
        ("POST", &format!("{COMMANDS}/{id}")),
        token,
        Some(input),
    )
    .1
}

fn mcp(url: &str, token: Option<&str>, id: u64, method: &str, params: &Value) -> String {
    send(url, ("POST", "/mcp"), token, Some(&rpc(id, method, params))).1
}

/// Every file below `dir`, read as bytes, with its path.
pub(crate) fn files(dir: &Path) -> Vec<(std::path::PathBuf, Vec<u8>)> {
    let mut found = Vec::new();
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            found.extend(files(&path));
        } else {
            found.push((path.clone(), fs::read(path).unwrap()));
        }
    }
    found
}

pub(crate) fn contains(haystack: &[u8], needle: &str) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle.as_bytes())
}

/// The traffic of the suite: valid calls, refused calls, hostile paths and malformed input, on
/// REST and on MCP, with the right token, a wrong one and none.
fn traffic(folder: &Path) -> impl FnOnce(&str, &str) -> Vec<String> + Send + 'static {
    let source = folder.display().to_string();
    move |url, token| {
        let mut answers = Vec::new();
        let add = json!({ "source": source });
        answers.push(command(url, Some(token), "projects.add", &add));
        answers.push(command(url, Some(token), "projects.list", &json!({})));
        // Secrets in a Git address, a path that leaves the project, unknown ids, bad input.
        let secret_url = format!("https://user:{CREDENTIAL_CANARY}@example.com/r.git");
        for input in [
            json!({ "source": secret_url }),
            json!({ "source": "../outside.cs" }),
            json!({ "source": "/etc/passwd" }),
            json!({ "source": 7 }),
        ] {
            answers.push(command(url, Some(token), "projects.add", &input));
        }
        // The symlink and everything a symbol id can reach.
        let search = json!({ "project": 1, "query": "Canary" });
        answers.push(command(url, Some(token), "search.query", &search));
        for symbol in 1..=12 {
            let input = json!({ "project": 1, "symbol": symbol });
            for id in ["explorer.source", "explorer.symbol", "usages.code"] {
                answers.push(command(url, Some(token), id, &input));
                answers.push(command(url, None, id, &input));
                answers.push(command(url, Some("csk_wrong"), id, &input));
            }
        }
        answers.extend(more_traffic(url, token));
        answers
    }
}

/// The remaining REST and MCP calls of the traffic.
fn more_traffic(url: &str, token: &str) -> Vec<String> {
    let mut answers = vec![
        command(
            url,
            Some(token),
            "explorer.outline",
            &json!({ "project": 1 }),
        ),
        command(url, Some(token), "status.info", &json!({})),
    ];
    answers.push(mcp(url, Some(token), 1, "tools/list", &json!({})));
    answers.extend(mcp_tool_traffic(url, token));
    let call = json!({ "name": "get_architecture", "arguments": { "project": 99 } });
    answers.push(mcp(url, Some(token), 2, "tools/call", &call));
    answers.push(mcp(url, Some("csk_wrong"), 3, "tools/list", &json!({})));
    answers.push(mcp(url, None, 4, "tools/list", &json!({})));
    answers
}

/// Every tool of the real MCP service on the registered project, the same way an agent calls it,
/// among them `rescan`, which reads the project folder again, and `check_conventions`, which
/// reads files by path. The paths aim at the file outside the project and at the link to it.
fn mcp_tool_traffic(url: &str, token: &str) -> Vec<String> {
    let calls = [
        ("search_symbols", json!({ "project": 1, "query": "Canary" })),
        (
            "get_symbol",
            json!({ "project": 1, "symbol": "A.CanaryInside6b1d" }),
        ),
        (
            "find_usages",
            json!({ "project": 1, "symbol": "A.CanaryInside6b1d" }),
        ),
        (
            "get_dependencies",
            json!({ "project": 1, "symbol": "A.CanaryInside6b1d" }),
        ),
        ("find_constants", json!({ "project": 1, "query": "Canary" })),
        ("get_architecture", json!({ "project": 1 })),
        ("rescan", json!({ "project": 1 })),
        (
            "check_conventions",
            json!({ "project": 1, "paths": ["src/a.cs", "src/link.cs", "../outside.cs"] }),
        ),
        ("search_symbols", json!({ "project": 1, "query": "Canary" })),
    ];
    calls
        .iter()
        .zip(10_u64..)
        .map(|((name, arguments), id)| {
            let call = json!({ "name": name, "arguments": arguments });
            mcp(url, Some(token), id, "tools/call", &call)
        })
        .collect()
}

#[test]
fn no_canary_leaves_the_running_app() {
    let folders = folders();
    let outcome = run(&folders, traffic(&folders.project));

    assert!(outcome.answers.len() > 20);
    // The controls: the project was registered, the class inside it is found and its source is
    // served, so that the missing outside class means something.
    let found = |text: &str| outcome.answers.iter().any(|answer| answer.contains(text));
    assert!(found("200 OK") && found(INSIDE) && found("public class"));
    // ... and `rescan` over MCP really scanned the project (it answers with its duration).
    assert!(found("duration_ms"));
    // Calls without the token or with a wrong one are refused (S-04).
    assert!(found("401 Unauthorized"));
    // The messages name the position, never the secret: a failing run must not log it.
    for (n, answer) in outcome.answers.iter().enumerate() {
        for (kind, secret) in [CREDENTIAL_CANARY, FILE_CANARY, outcome.token.as_str()]
            .into_iter()
            .enumerate()
        {
            assert!(!answer.contains(secret), "secret {kind} in answer {n}");
        }
    }
    // The database and every other file of the data folder.
    for (path, bytes) in files(folders.data.path()) {
        for (kind, secret) in [CREDENTIAL_CANARY, outcome.token.as_str()]
            .into_iter()
            .enumerate()
        {
            assert!(
                !contains(&bytes, secret),
                "secret {kind} in {}",
                path.display()
            );
        }
    }
}

/// No answer holds any of `secrets`. The messages name the position, never the secret: a failing
/// run must not log it.
pub(crate) fn assert_answers_clean(answers: &[String], secrets: &[&str]) {
    for (kind, secret) in secrets.iter().enumerate() {
        for (n, answer) in answers.iter().enumerate() {
            assert!(!answer.contains(secret), "secret {kind} in answer {n}");
        }
    }
}
