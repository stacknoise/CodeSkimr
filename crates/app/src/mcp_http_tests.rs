//! The MCP route of the running app, end to end over TCP (WP-48).

use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;

use codeskimr_cli::GlobalOptions;
use codeskimr_secrets::KeystoreCredentials;
use serde_json::{Value, json};
use tokio::sync::oneshot;

use crate::consts::URL_SCHEME;
use crate::locations::Locations;
use crate::mcp::http_service;
use crate::memory_keystore::Memory;
use crate::serve::{bare_parts, serve_for_test, test_ui};
use crate::{config_files, installation_token};

const ACCEPT: &str = "application/json, text/event-stream";
const LANGUAGES_PATH: &str = "../../languages";

pub(crate) fn locations(data: &Path) -> Locations {
    Locations {
        data_dir: data.to_path_buf(),
        languages_dir: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(LANGUAGES_PATH),
        user_config: None,
        bases: Vec::new(),
    }
}

/// One request with an optional bearer token and JSON body; the status and the whole answer
/// (headers and body) as text.
pub(crate) fn send(
    url: &str,
    (method, path): (&str, &str),
    token: Option<&str>,
    body: Option<&Value>,
) -> (u16, String) {
    let address = url.trim_start_matches(URL_SCHEME);
    let text = body.map(Value::to_string).unwrap_or_default();
    let auth = token.map_or_else(String::new, |t| format!("Authorization: Bearer {t}\r\n"));
    let request = format!(
        "{method} {path} HTTP/1.1\r\nHost: {address}\r\n{auth}Accept: {ACCEPT}\r\n\
         Content-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{text}",
        text.len()
    );
    let mut stream = TcpStream::connect(address).unwrap();
    stream.write_all(request.as_bytes()).unwrap();
    let mut answer = String::new();
    stream.read_to_string(&mut answer).unwrap();
    let status = answer.split_whitespace().nth(1).unwrap().parse().unwrap();
    (status, answer)
}

/// One POST to `/mcp` with an optional token; the status and the JSON body of the answer.
pub(crate) fn post(url: &str, token: Option<&str>, body: &Value) -> (u16, Value) {
    let (status, answer) = send(url, ("POST", "/mcp"), token, Some(body));
    let payload = answer.split_once("\r\n\r\n").unwrap().1;
    let json = payload
        .lines()
        .map(|line| line.trim_start_matches("data:").trim())
        .find_map(|line| serde_json::from_str(line).ok())
        .unwrap_or(Value::Null);
    (status, json)
}

pub(crate) fn rpc(id: u64, method: &str, params: &Value) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params})
}

fn client(url: &str, token: &str) {
    let hello = rpc(
        1,
        "initialize",
        &json!({"protocolVersion": "2025-03-26", "capabilities": {},
                "clientInfo": {"name": "test", "version": "0"}}),
    );
    let (status, answer) = post(url, Some(token), &hello);
    assert_eq!(status, 200, "{answer}");
    assert_eq!(answer["result"]["serverInfo"]["name"], "codeskimr");
    let (status, answer) = post(url, Some(token), &rpc(2, "tools/list", &json!({})));
    assert_eq!(status, 200, "{answer}");
    let names: Vec<&str> = answer["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|tool| tool["name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"search_symbols") && names.contains(&"rescan"));
    // No default project in the app: a call that names none is refused, one that names an
    // unknown project says so.
    let call = |id, arguments: &Value| {
        let params = json!({"name": "get_architecture", "arguments": arguments});
        post(url, Some(token), &rpc(id, "tools/call", &params)).1["result"].clone()
    };
    let unscoped = call(3, &json!({}));
    assert_eq!(unscoped["isError"], true);
    assert_eq!(unscoped["structuredContent"]["code"], "MCP-101");
    assert_eq!(unscoped["structuredContent"]["params"]["detail"], "project");
    let unknown = call(4, &json!({"project": 99}));
    assert_eq!(unknown["structuredContent"]["code"], "MCP-201");
    // The same call without the token, or with another one, never reaches the tools.
    for wrong in [None, Some("csk_wrong")] {
        let (status, answer) = post(url, wrong, &rpc(5, "tools/list", &json!({})));
        assert_eq!(status, 401);
        assert_eq!(answer["code"], "API-401");
    }
}

/// A keystore with a fresh installation token, and that token.
pub(crate) fn token_store() -> (KeystoreCredentials<Memory>, String) {
    let store = KeystoreCredentials::new(Memory::default());
    installation_token::load_or_create(&store).unwrap();
    let token = installation_token::stored_token(&store);
    (store, token)
}

#[test]
fn the_running_app_answers_mcp_at_slash_mcp_with_the_token_only() {
    let data = tempfile::tempdir().unwrap();
    let locations = locations(data.path());
    let config = config_files::load(&locations, &GlobalOptions::default(), None).unwrap();
    let (store, token) = token_store();
    let mut parts = bare_parts(test_ui());
    parts.mcp = Some(
        http_service(
            &config,
            &locations,
            (parts.events.clone(), &crate::memory_keystore::secrets()),
        )
        .unwrap(),
    );
    let (stop, stopped) = oneshot::channel::<()>();
    let (found, url) = channel::<String>();
    let requests = std::thread::spawn(move || {
        let url = url.recv().unwrap();
        client(&url, &token);
        stop.send(()).unwrap();
    });
    serve_for_test(&store, parts, &found, stopped).unwrap();
    requests.join().unwrap();
}
