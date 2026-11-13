//! The HTTP guard of the running app (blueprint S-04) and the canary check of its event stream
//! (S-02): token per call, `Host` and `Origin`, CORS, response headers and the WebSocket.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use serde_json::{Value, json};

use crate::consts::URL_SCHEME;
use crate::security_tests::{CREDENTIAL_CANARY, command, folders, run};

const EVENTS: &str = "/api/v1/events";
const COMMANDS: &str = "/api/v1/commands";
const FOREIGN: &str = "evil.example";
/// How long the event stream is listened to after the traffic.
const LISTEN: Duration = Duration::from_secs(3);
/// How long a request may take on a loaded machine; the server closes the connection sooner.
const PATIENCE: Duration = Duration::from_secs(60);

/// One raw request: `lines` are the headers after the request line, and the answer is read to
/// the end of the connection (or to `LISTEN` for a stream).
fn raw(url: &str, request: &str, headers: &[String], body: &str) -> String {
    let wait = if request.contains(EVENTS) {
        LISTEN
    } else {
        PATIENCE
    };
    let address = url.trim_start_matches(URL_SCHEME);
    let mut lines = vec![request.to_owned()];
    if !headers.iter().any(|line| line.starts_with("Host:")) {
        lines.push(format!("Host: {address}"));
    }
    lines.extend(headers.iter().cloned());
    lines.push(format!("Content-Length: {}", body.len()));
    let text = format!("{}\r\n\r\n{body}", lines.join("\r\n"));
    let mut stream = TcpStream::connect(address).unwrap();
    stream.set_read_timeout(Some(wait)).unwrap();
    stream.write_all(text.as_bytes()).unwrap();
    let mut bytes = Vec::new();
    let _ = stream.read_to_end(&mut bytes);
    String::from_utf8_lossy(&bytes).into_owned()
}

fn status(answer: &str) -> u16 {
    answer.split_whitespace().nth(1).unwrap().parse().unwrap()
}

fn add_request(url: &str, headers: &[String], body: &Value) -> String {
    let mut all = vec![
        "Content-Type: application/json".to_owned(),
        "Connection: close".to_owned(),
    ];
    all.extend_from_slice(headers);
    raw(
        url,
        &format!("POST {COMMANDS}/projects.add HTTP/1.1"),
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
        let answer = add_request(url, &headers, &body);
        assert_eq!(status(&answer), 401, "token {wrong:?}");
        assert!(answer.contains("API-401"));
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
    // The control: the right token passes.
    assert_eq!(status(&add_request(url, &[bearer(token)], &body)), 200);
}

/// A foreign `Host` or `Origin` is refused with `API-402`, whatever the token.
fn host_and_origin(url: &str, token: &str, source: &str) {
    let address = url.trim_start_matches(URL_SCHEME);
    let body = json!({ "source": source });
    let refused = |headers: Vec<String>, what: &str| {
        let answer = add_request(url, &headers, &body);
        assert!(answer.contains("API-402"), "{what}");
        assert!(matches!(status(&answer), 400 | 403), "{what}");
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
    assert_eq!(status(&add_request(url, &[bearer(token), own], &body)), 200);
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
    let answers = vec![preflight, normal];
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
    answers
}

fn upgrade(url: &str, origin: Option<String>) -> String {
    let mut headers = vec![
        "Connection: Upgrade".to_owned(),
        "Upgrade: websocket".to_owned(),
        "Sec-WebSocket-Version: 13".to_owned(),
        "Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==".to_owned(),
    ];
    headers.extend(origin);
    raw(url, &format!("GET {EVENTS} HTTP/1.1"), &headers, "")
}

/// The event stream refuses a foreign origin; for our own it streams, and nothing it sends is a
/// secret.
fn event_stream(url: &str, token: &str, source: &str) -> (String, String) {
    let address = url.trim_start_matches(URL_SCHEME);
    let foreign = upgrade(url, Some(format!("Origin: http://{FOREIGN}")));
    assert!(foreign.contains("API-402"));
    assert!(!foreign.contains("101 Switching"));
    // Listen on one connection while the traffic runs on others.
    let url_for_listener = url.to_owned();
    let own = format!("Origin: http://{address}");
    let listener = std::thread::spawn(move || upgrade(&url_for_listener, Some(own)));
    std::thread::sleep(Duration::from_millis(300));
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
    (foreign, listener.join().unwrap())
}

#[test]
fn the_guard_holds_on_every_call_and_the_event_stream_leaks_nothing() {
    let folders = folders();
    let source = folders.project.display().to_string();
    let outcome = run(&folders, move |url, token| {
        token_per_call(url, token, &source);
        host_and_origin(url, token, &source);
        let mut answers = cors_and_headers(url, token);
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
    for (kind, secret) in [CREDENTIAL_CANARY, outcome.token.as_str()]
        .into_iter()
        .enumerate()
    {
        for (n, answer) in outcome.answers.iter().enumerate() {
            assert!(!answer.contains(secret), "secret {kind} in answer {n}");
        }
    }
}
