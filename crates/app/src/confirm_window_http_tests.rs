//! The route and the commands of the level 2 switch in the running app, end to end over TCP, with
//! a window that only counts (ADR 0071). No test here can open a real window.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::channel;

use codeskimr_application::config::Config;
use codeskimr_application::error::CodedError;
use codeskimr_application::events::EventBus;
use codeskimr_application::level2::Launch;
use codeskimr_application::ports::{ConfirmationWindow, WindowOutcome};
use codeskimr_cli::GlobalOptions;
use codeskimr_domain::ProjectId;
use codeskimr_storage::SqliteStore;
use serde_json::json;
use tokio::sync::oneshot;

use crate::consts::URL_SCHEME;
use crate::locations::{Locations, UserConfig};
use crate::mcp::http_service;
use crate::mcp_http_tests::{post, rpc, token_store};
use crate::running::RunningScans;
use crate::serve::{Parts, serve_for_test, test_ui};
use crate::ui_commands::{Window, command_api_over};
use crate::{config_files, wiring};

/// Counts the requests for a window and opens nothing.
struct Counting(Arc<AtomicUsize>);

impl ConfirmationWindow for Counting {
    fn open(&self, _project: ProjectId) -> Result<WindowOutcome, CodedError> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Ok(WindowOutcome::Opened)
    }
}

fn call(url: &str, request: (&str, &str), headers: &[&str], body: &str) -> (u16, String) {
    let address = url.trim_start_matches(URL_SCHEME);
    let mut lines = vec![format!("{} {} HTTP/1.1", request.0, request.1)];
    if !headers.iter().any(|h| h.starts_with("Host:")) {
        lines.push(format!("Host: {address}"));
    }
    lines.extend(headers.iter().map(|h| (*h).to_owned()));
    lines.push(format!("Content-Length: {}", body.len()));
    lines.push("Connection: close".to_owned());
    let text = format!("{}\r\n\r\n{body}", lines.join("\r\n"));
    let mut stream = TcpStream::connect(address).unwrap();
    stream.write_all(text.as_bytes()).unwrap();
    let mut answer = String::new();
    stream.read_to_string(&mut answer).unwrap();
    (
        answer.split_whitespace().nth(1).unwrap().parse().unwrap(),
        answer,
    )
}

const JSON: &str = "Content-Type: application/json";
const WINDOW: (&str, &str) = ("POST", "/api/v1/level2/confirm-window");

/// Everything the guard refuses, before any window is asked for.
fn refusals(url: &str, token: &str, opened: &AtomicUsize) {
    let bearer = format!("Authorization: Bearer {token}");
    let body = json!({ "project_id": 1 }).to_string();
    let foreign_origin = "Origin: http://evil.example";
    let cross_site = "Sec-Fetch-Site: cross-site";
    let form = "Content-Type: application/x-www-form-urlencoded";
    let refused = [
        (vec![JSON], 401, "API-401"),
        (
            vec![JSON, "Authorization: Bearer csk_wrong"],
            401,
            "API-401",
        ),
        (vec![JSON, bearer.as_str(), foreign_origin], 403, "API-402"),
        (vec![JSON, bearer.as_str(), cross_site], 403, "API-402"),
        (vec![form, bearer.as_str()], 400, "API-101"),
        (
            vec![JSON, bearer.as_str(), "Host: evil.example"],
            403,
            "API-402",
        ),
    ];
    for (headers, status, code) in refused {
        let (got, answer) = call(url, WINDOW, &headers, &body);
        assert_eq!(
            (got, answer.contains(code)),
            (status, true),
            "{headers:?}: {answer}"
        );
    }
    assert_eq!(opened.load(Ordering::SeqCst), 0);
}

/// The status reads without a token, the revoke needs it.
fn status_and_revoke(url: &str) {
    let status = json!({ "project": 1 }).to_string();
    let (got, answer) = call(
        url,
        ("POST", "/api/v1/commands/level2.status"),
        &[JSON],
        &status,
    );
    assert_eq!(got, 200, "{answer}");
    assert!(answer.contains("\"state\":\"off\""), "{answer}");
    let revoke = ("POST", "/api/v1/commands/level2.revoke");
    assert_eq!(call(url, revoke, &[JSON], &status).0, 401);
    assert_eq!(
        call(
            url,
            revoke,
            &[JSON, "Authorization: Bearer csk_wrong"],
            &status
        )
        .0,
        401
    );
}

/// The commands of the HTTP layer run as the browser client: with the token the revoke goes
/// through, while the same server's MCP endpoint refuses it (`mcp_has_no_tool`, ADR 0072).
fn http_runs_as_the_browser_client(url: &str, token: &str) {
    let bearer = format!("Authorization: Bearer {token}");
    let status = json!({ "project": 1 }).to_string();
    let revoke = ("POST", "/api/v1/commands/level2.revoke");
    let (got, answer) = call(url, revoke, &[JSON, bearer.as_str()], &status);
    assert_eq!(got, 200, "{answer}");
    assert!(answer.contains("\"confirmed\":false"), "{answer}");
}

/// One click opens one window and confirms nothing; the second is inside the pause.
fn one_click_and_the_pause(url: &str, token: &str, opened: &AtomicUsize) {
    let bearer = format!("Authorization: Bearer {token}");
    let body = json!({ "project_id": 1 }).to_string();
    let status = json!({ "project": 1 }).to_string();
    let address = url.trim_start_matches(URL_SCHEME);
    let own = format!("Origin: http://{address}");
    let ok = [
        JSON,
        bearer.as_str(),
        own.as_str(),
        "Sec-Fetch-Site: same-origin",
    ];
    let (got, answer) = call(url, WINDOW, &ok, &body);
    assert_eq!(got, 200, "{answer}");
    assert!(answer.ends_with("{\"launch\":\"window\"}"), "{answer}");
    assert_eq!(opened.load(Ordering::SeqCst), 1);
    let (got, answer) = call(url, WINDOW, &ok, &body);
    assert_eq!(got, 429, "{answer}");
    assert!(answer.contains("API-504") && answer.to_lowercase().contains("retry-after:"));
    assert_eq!(opened.load(Ordering::SeqCst), 1);
    let (_, answer) = call(
        url,
        ("POST", "/api/v1/commands/level2.status"),
        &[JSON],
        &status,
    );
    assert!(answer.contains("\"confirmed\":false") && answer.contains("\"state\":\"off\""));
}

/// The MCP service has no tool for any of it.
fn mcp_has_no_tool(url: &str, token: &str) {
    let (code, listed) = post(url, Some(token), &rpc(1, "tools/list", &json!({})));
    assert_eq!(code, 200);
    let names: Vec<&str> = listed["result"]["tools"]
        .as_array()
        .map_or_else(Vec::new, |tools| {
            tools.iter().filter_map(|t| t["name"].as_str()).collect()
        });
    assert_ne!(names, [] as [&str; 0]);
    assert!(
        names.iter().all(|name| !name.contains("level2")),
        "{names:?}"
    );
    let revoke_as_tool = json!({ "name": "level2.revoke", "arguments": { "project": 1 } });
    let (_, answer) = post(url, Some(token), &rpc(2, "tools/call", &revoke_as_tool));
    assert!(
        answer.get("error").is_some() || answer["result"]["isError"] == true,
        "{answer}"
    );
}

/// A configuration with the switch on, and a catalog that holds project 1.
fn switched_on(data: &std::path::Path) -> (Locations, GlobalOptions, Config, Arc<SqliteStore>) {
    let config_file = data.join("user.toml");
    std::fs::write(&config_file, "[level2]\nenabled = true\n").unwrap();
    let mut locations: Locations = crate::mcp_http_tests::locations(&data.join("state"));
    locations.user_config = Some(UserConfig {
        path: config_file,
        required: true,
    });
    let global = GlobalOptions::default();
    let config = config_files::load(&locations, &global, None).unwrap();
    let catalog = wiring::open_store(&locations, &config).unwrap();
    {
        use codeskimr_domain::{FolderPath, Project, ProjectName, ProjectSource};
        catalog
            .save_project(&Project {
                id: ProjectId::new(1),
                name: ProjectName::new("app").unwrap(),
                source: ProjectSource::Folder(FolderPath::new("/work/app").unwrap()),
                layers: Vec::new(),
            })
            .unwrap();
    }
    (locations, global, config, catalog)
}

/// A window that only counts how often it was opened.
fn counting_window(opened: &Arc<AtomicUsize>) -> Window {
    Window {
        port: Arc::new(Counting(Arc::clone(opened))),
        launch: Launch::Window,
    }
}

#[test]
fn the_window_route_and_the_commands_of_the_switch_follow_the_guard_end_to_end() {
    let data = tempfile::tempdir().unwrap();
    let (locations, global, config, catalog) = switched_on(data.path());
    let opened = Arc::new(AtomicUsize::new(0));
    let events = EventBus::new();
    let secrets = crate::memory_keystore::secrets();
    let window = counting_window(&opened);
    let (api, desk) = command_api_over(
        &catalog,
        &config,
        (&global, &locations),
        (events.clone(), Arc::clone(&secrets)),
        ((None, RunningScans::default()), window),
    )
    .unwrap();
    let parts = Parts {
        ui: test_ui(),
        api,
        confirmation: Some(desk),
        mcp: Some(http_service(&config, &locations, (events.clone(), &secrets)).unwrap()),
        events,
        token_lock: None,
        purge: None,
    };
    let (store, token) = token_store();
    let (stop, stopped) = oneshot::channel::<()>();
    let (found, url) = channel::<String>();
    let seen = Arc::clone(&opened);
    let client = std::thread::spawn(move || {
        let url = url.recv().unwrap();
        refusals(&url, &token, &seen);
        status_and_revoke(&url);
        one_click_and_the_pause(&url, &token, &seen);
        http_runs_as_the_browser_client(&url, &token);
        mcp_has_no_tool(&url, &token);
        stop.send(()).unwrap();
    });
    serve_for_test(&store, parts, &found, stopped).unwrap();
    client.join().unwrap();
}
