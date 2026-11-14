//! The HTTP guard of the running app (blueprint S-04) and the canary check of its event stream
//! (S-02): token per call, `Host` and `Origin`, CORS, response headers and the WebSocket.

use std::io::{ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use serde_json::{Value, json};

use crate::consts::URL_SCHEME;
use crate::security_tests::{CREDENTIAL_CANARY, assert_answers_clean, command, folders, run};

const EVENTS: &str = "/api/v1/events";
const COMMANDS: &str = "/api/v1/commands";
const FOREIGN: &str = "evil.example";
/// How long a stream may be silent before the listener looks at its stop flag.
const QUIET: Duration = Duration::from_millis(300);
/// The time events need to reach a listener after the last command answered.
const AFTER_TRAFFIC: Duration = Duration::from_millis(800);
/// How long a request may take on a loaded machine; the server closes the connection sooner.
const PATIENCE: Duration = Duration::from_secs(60);

/// Opens a connection and sends one raw request: `headers` follow the request line.
fn connect(url: &str, request: &str, headers: &[String], body: &str) -> TcpStream {
    let address = url.trim_start_matches(URL_SCHEME);
    let mut lines = vec![request.to_owned()];
    if !headers.iter().any(|line| line.starts_with("Host:")) {
        lines.push(format!("Host: {address}"));
    }
    lines.extend(headers.iter().cloned());
    lines.push(format!("Content-Length: {}", body.len()));
    let text = format!("{}\r\n\r\n{body}", lines.join("\r\n"));
    let mut stream = TcpStream::connect(address).unwrap();
    stream.write_all(text.as_bytes()).unwrap();
    stream
}

/// One raw request; the answer is read to the end of the connection.
fn raw(url: &str, request: &str, headers: &[String], body: &str) -> String {
    let mut stream = connect(url, request, headers, body);
    stream.set_read_timeout(Some(PATIENCE)).unwrap();
    let mut bytes = Vec::new();
    let _ = stream.read_to_end(&mut bytes);
    String::from_utf8_lossy(&bytes).into_owned()
}

/// Everything a connection sends until `stop` is set and it has been quiet for a moment.
fn listen(mut stream: TcpStream, stop: &AtomicBool) -> String {
    stream.set_read_timeout(Some(QUIET)).unwrap();
    let mut bytes = Vec::new();
    let mut chunk = [0_u8; 4096];
    loop {
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(count) => bytes.extend_from_slice(&chunk[..count]),
            Err(error) if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {
                if stop.load(Ordering::SeqCst) {
                    break;
                }
            }
            Err(_) => break,
        }
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

/// Reads from the connection until the text holds `needle`, the connection ends or `PATIENCE`
/// passes without a byte.
fn read_until(stream: &mut TcpStream, needle: &str) -> String {
    stream.set_read_timeout(Some(PATIENCE)).unwrap();
    let mut bytes = Vec::new();
    let mut chunk = [0_u8; 4096];
    while !String::from_utf8_lossy(&bytes).contains(needle) {
        match stream.read(&mut chunk) {
            Ok(0) | Err(_) => break,
            Ok(count) => bytes.extend_from_slice(&chunk[..count]),
        }
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

fn status(answer: &str) -> u16 {
    answer.split_whitespace().nth(1).unwrap().parse().unwrap()
}

fn command_request(url: &str, id: &str, headers: &[String], body: &Value) -> String {
    let mut all = vec![
        "Content-Type: application/json".to_owned(),
        "Connection: close".to_owned(),
    ];
    all.extend_from_slice(headers);
    raw(
        url,
        &format!("POST {COMMANDS}/{id} HTTP/1.1"),
        &all,
        &body.to_string(),
    )
}

fn bearer(token: &str) -> String {
    format!("Authorization: Bearer {token}")
}

/// Calls that change something or serve source are refused one by one without the right token.
fn token_per_call(url: &str, token: &str, source: &str) {
    let body = json!({ "source": source });
    for wrong in [None, Some("csk_wrong"), Some("")] {
        let headers: Vec<String> = wrong.map(bearer).into_iter().collect();
        let answer = command_request(url, "projects.add", &headers, &body);
        assert_eq!(status(&answer), 401, "token {wrong:?}");
        assert!(answer.contains("API-401"));
        // Source text needs the token although it only reads (ADR 0032).
        let input = json!({ "project": 1, "symbol": 1 });
        let answer = command_request(url, "explorer.source", &headers, &input);
        assert_eq!(status(&answer), 401, "source, token {wrong:?}");
        let answer = command_request(url, "usages.code", &headers, &input);
        assert_eq!(status(&answer), 401, "code of hits, token {wrong:?}");
    }
    let rpc = json!({"jsonrpc": "2.0", "id": 1, "method": "tools/list", "params": {}}).to_string();
    for wrong in [None, Some("csk_wrong")] {
        let mut headers = vec![
            "Content-Type: application/json".to_owned(),
            "Connection: close".to_owned(),
        ];
        headers.extend(wrong.map(bearer));
        let answer = raw(url, "POST /mcp HTTP/1.1", &headers, &rpc);
        assert_eq!(status(&answer), 401, "mcp token {wrong:?}");
    }
    // The controls: the right token passes, on REST and on MCP.
    assert_eq!(
        status(&command_request(
            url,
            "projects.add",
            &[bearer(token)],
            &body
        )),
        200
    );
    let headers = [
        "Content-Type: application/json".to_owned(),
        "Connection: close".to_owned(),
        "Accept: application/json, text/event-stream".to_owned(),
        bearer(token),
    ];
    assert_eq!(status(&raw(url, "POST /mcp HTTP/1.1", &headers, &rpc)), 200);
}

/// Storing and deleting credentials needs the token, and a body that does not parse never echoes
/// the secret in it.
fn credentials_need_the_token(url: &str, token: &str) -> Vec<String> {
    let stored = json!({ "id": "work", "user": "anton", "secret": CREDENTIAL_CANARY });
    let bad_body = json!({ "id": "work", "user": "anton", "secret": [CREDENTIAL_CANARY] });
    let delete = json!({ "id": "work" });
    for wrong in [None, Some("csk_wrong"), Some("")] {
        let headers: Vec<String> = wrong.map(bearer).into_iter().collect();
        for (id, body) in [
            ("credentials.set", &stored),
            ("credentials.delete", &delete),
        ] {
            let answer = command_request(url, id, &headers, body);
            assert_eq!(status(&answer), 401, "{id}, token {wrong:?}");
            assert!(answer.contains("API-401"), "{id}");
        }
    }
    let right = [bearer(token)];
    let bad = command_request(url, "credentials.set", &right, &bad_body);
    assert_eq!(status(&bad), 400);
    assert!(bad.contains("API-101"));
    // The controls: the right token stores and deletes.
    let set = command_request(url, "credentials.set", &right, &stored);
    let gone = command_request(url, "credentials.delete", &right, &delete);
    assert_eq!(status(&set), 200);
    assert_eq!(status(&gone), 200);
    vec![bad, set, gone]
}

/// A foreign `Host` or `Origin` is refused with `API-402`, whatever the token.
fn host_and_origin(url: &str, token: &str, source: &str) {
    let address = url.trim_start_matches(URL_SCHEME);
    let body = json!({ "source": source });
    let refused = |headers: Vec<String>, what: &str| {
        let answer = command_request(url, "projects.add", &headers, &body);
        assert!(answer.contains("API-402"), "{what}");
        assert_eq!(status(&answer), 403, "{what}");
    };
    refused(
        vec![bearer(token), format!("Host: {FOREIGN}")],
        "foreign host",
    );
    refused(
        vec![bearer(token), format!("Origin: http://{FOREIGN}")],
        "foreign origin",
    );
    refused(
        vec![bearer(token), "Origin: null".to_owned()],
        "null origin",
    );
    // A foreign host with the right origin, and the other way round, is as bad.
    let own = format!("Origin: http://{address}");
    refused(
        vec![bearer(token), own.clone(), format!("Host: {FOREIGN}")],
        "host only",
    );
    // The control: the same request with our own origin passes.
    assert_eq!(
        status(&command_request(
            url,
            "projects.add",
            &[bearer(token), own],
            &body
        )),
        200
    );
}

/// No answer allows a cross-site read, and a preflight from a foreign site is refused.
fn cors_and_headers(url: &str, token: &str) -> Vec<String> {
    let preflight = raw(
        url,
        &format!("OPTIONS {COMMANDS}/projects.add HTTP/1.1"),
        &[
            format!("Origin: http://{FOREIGN}"),
            "Access-Control-Request-Method: POST".to_owned(),
            "Connection: close".to_owned(),
        ],
        "",
    );
    let normal = raw(
        url,
        &format!("GET {COMMANDS} HTTP/1.1"),
        &[bearer(token), "Connection: close".to_owned()],
        "",
    );
    let foreign = raw(
        url,
        &format!("GET {COMMANDS} HTTP/1.1"),
        &[
            bearer(token),
            format!("Origin: http://{FOREIGN}"),
            "Connection: close".to_owned(),
        ],
        "",
    );
    let answers = vec![preflight, normal, foreign];
    for answer in &answers {
        let lower = answer.to_ascii_lowercase();
        assert!(!lower.contains("access-control-allow-origin"));
        assert!(lower.contains("content-security-policy:"));
        assert!(lower.contains("frame-ancestors 'none'"));
        assert!(lower.contains("x-content-type-options: nosniff"));
        assert!(lower.contains("cache-control:"));
    }
    assert!(answers[0].contains("API-402"));
    assert_eq!(status(&answers[1]), 200);
    assert!(answers[2].contains("API-402"));
    answers
}

fn upgrade_headers(origin: String) -> Vec<String> {
    vec![
        "Connection: Upgrade".to_owned(),
        "Upgrade: websocket".to_owned(),
        "Sec-WebSocket-Version: 13".to_owned(),
        "Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==".to_owned(),
        origin,
    ]
}

fn upgrade(url: &str, origin: String) -> TcpStream {
    connect(
        url,
        &format!("GET {EVENTS} HTTP/1.1"),
        &upgrade_headers(origin),
        "",
    )
}

/// The event stream refuses a foreign origin; for our own it streams, and nothing it sends is a
/// secret.
fn event_stream(url: &str, token: &str, source: &str) -> (String, String) {
    let address = url.trim_start_matches(URL_SCHEME);
    let foreign = read_until(
        &mut upgrade(url, format!("Origin: http://{FOREIGN}")),
        "API-402",
    );
    assert!(foreign.contains("API-402"));
    assert!(!foreign.contains("101 Switching"));
    // Listen on one connection while the traffic runs on others.
    let stop = Arc::new(AtomicBool::new(false));
    let mut stream = upgrade(url, format!("Origin: http://{address}"));
    // The server subscribes before it answers the handshake, so nothing is missed after this.
    let head = read_until(&mut stream, "\r\n\r\n");
    let flag = Arc::clone(&stop);
    let listener = std::thread::spawn(move || listen(stream, &flag));
    let secret = format!("https://user:{CREDENTIAL_CANARY}@127.0.0.1:9/r.git");
    let _ = command(
        url,
        Some(token),
        "projects.add",
        &json!({ "source": secret }),
    );
    let _ = command(
        url,
        Some(token),
        "projects.add",
        &json!({ "source": source }),
    );
    std::thread::sleep(AFTER_TRAFFIC);
    stop.store(true, Ordering::SeqCst);
    (foreign, head + &listener.join().unwrap())
}

#[test]
fn bad_tokens_hosts_and_origins_are_refused_and_the_event_stream_leaks_nothing() {
    let folders = folders();
    let source = folders.project.display().to_string();
    let outcome = run(&folders, move |url, token| {
        token_per_call(url, token, &source);
        host_and_origin(url, token, &source);
        let mut answers = credentials_need_the_token(url, token);
        answers.extend(cors_and_headers(url, token));
        let (foreign, stream) = event_stream(url, token, &source);
        answers.push(foreign);
        answers.push(stream);
        answers
    });
    let stream = outcome.answers.last().unwrap();
    // The control: the handshake succeeded and events came after it.
    assert!(stream.contains("101 Switching"));
    let body = stream.split_once("\r\n\r\n").map_or("", |parts| parts.1);
    assert!(!body.is_empty(), "no event arrived");
    assert_answers_clean(
        &outcome.answers,
        &[CREDENTIAL_CANARY, outcome.token.as_str()],
    );
}
