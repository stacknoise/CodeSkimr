//! Limits against clients that hold resources: stream cap, connection cap and read timeouts.

mod common;

use std::time::Duration;

use common::{Running, TOKEN, owner, start};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;

const UPGRADE: &str = "GET /api/v1/events HTTP/1.1\r\nConnection: Upgrade\r\nUpgrade: websocket\r\nSec-WebSocket-Version: 13\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n";
const STREAM_CAP: usize = 16;
const CONNECTION_CAP: usize = 256;
const SETTLE_GRACE: Duration = Duration::from_millis(200);
/// How far the paused clock moves before the server gets real time to react (see [`collect_stepped`]).
const CLOCK_STEP: Duration = Duration::from_secs(5);
/// How long a stalled body may stay unanswered, in paused-clock time.
const STALL_LIMIT: Duration = Duration::from_secs(120);

/// Gives the server a moment of real time to see what the client just did.
///
/// The tests with a paused clock jump ahead as soon as the runtime has nothing to do. On macOS the
/// operating system can report a fresh connection or fresh bytes on a loopback socket a moment
/// after the client side has finished with them, so the paused clock would already have run out a
/// deadline (the test's own or the server's) before the server had seen anything. A blocking task
/// keeps the clock from jumping meanwhile.
#[cfg(test)]
async fn settle() {
    tokio::task::spawn_blocking(|| std::thread::sleep(SETTLE_GRACE))
        .await
        .unwrap();
}

/// Connects and lets the server accept the connection (see [`settle`]).
#[cfg(test)]
async fn connect(server: &Running) -> TcpStream {
    let stream = TcpStream::connect(server.address).await.unwrap();
    settle().await;
    stream
}

/// The status line the server answers to `request`, or none if it closes without an answer.
#[cfg(test)]
async fn status_of(stream: &mut TcpStream, request: &str) -> Option<String> {
    stream.write_all(request.as_bytes()).await.unwrap();
    status_line(stream).await
}

/// Like [`status_of`], but lets the server read the request before the answer is awaited (see
/// [`settle`]), for the tests with a paused clock.
#[cfg(test)]
async fn status_of_settled(stream: &mut TcpStream, request: &str) -> Option<String> {
    stream.write_all(request.as_bytes()).await.unwrap();
    settle().await;
    status_line(stream).await
}

/// The first line of what the server sends next, or none if it sends nothing.
#[cfg(test)]
async fn status_line(stream: &mut TcpStream) -> Option<String> {
    let mut buffer = vec![0_u8; 512];
    let read = stream.read(&mut buffer).await.unwrap_or_default();
    let text = String::from_utf8_lossy(buffer.get(..read).unwrap_or_default()).into_owned();
    text.lines().next().map(str::to_owned)
}

/// What the server sends until it closes the connection, or none if it does not within `limit`
/// of paused-clock time (see [`collect_stepped`]).
#[cfg(test)]
async fn until_closed(stream: &mut TcpStream, limit: Duration) -> Option<Vec<u8>> {
    collect_stepped(stream, limit, |_| false).await
}

/// Reads what the server sends while the paused clock moves in steps of `CLOCK_STEP`.
///
/// A single long wait on a paused clock lets the runtime jump to the deadline as soon as it is
/// idle, even if the operating system has not yet reported the server's answer or its close on the
/// loopback socket (macOS, issue #528). After every step the server gets real time to act and to
/// be seen (see [`settle`]). The result is the bytes received once the server closed the
/// connection or `is_complete` accepted them, or none if `limit` of clock time passed before.
#[cfg(test)]
async fn collect_stepped(
    stream: &mut TcpStream,
    limit: Duration,
    is_complete: fn(&[u8]) -> bool,
) -> Option<Vec<u8>> {
    let mut collected = Vec::new();
    let mut buffer = [0_u8; 1024];
    let mut waited = Duration::ZERO;
    loop {
        match stream.try_read(&mut buffer) {
            Ok(read) if read > 0 => {
                collected.extend_from_slice(buffer.get(..read).unwrap_or_default());
                if is_complete(&collected) {
                    return Some(collected);
                }
                continue;
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {}
            Ok(_) | Err(_) => return Some(collected),
        }
        if waited >= limit {
            return None;
        }
        tokio::time::advance(CLOCK_STEP).await;
        waited += CLOCK_STEP;
        settle().await;
    }
}

#[cfg(test)]
async fn open_stream(server: &Running) -> (TcpStream, Option<String>) {
    let mut stream = TcpStream::connect(server.address).await.unwrap();
    let request = format!("{UPGRADE}Host: {}\r\n\r\n", server.address);
    let status = status_of(&mut stream, &request).await;
    (stream, status)
}

/// The `Retry-After` header the server sends when no stream place is free.
#[cfg(test)]
async fn retry_after(server: &Running) -> Option<String> {
    let mut stream = TcpStream::connect(server.address).await.ok()?;
    let request = format!("{UPGRADE}Host: {}\r\n\r\n", server.address);
    stream.write_all(request.as_bytes()).await.ok()?;
    let mut buffer = vec![0_u8; 1024];
    let read = stream.read(&mut buffer).await.ok()?;
    let text = String::from_utf8_lossy(buffer.get(..read)?).to_ascii_lowercase();
    let line = text.lines().find(|line| line.starts_with("retry-after:"))?;
    line.split_once(':')
        .map(|(_, value)| value.trim().to_owned())
}

#[tokio::test]
async fn the_stream_after_the_last_free_place_is_refused_and_a_closed_one_frees_its_place() {
    let server = start(owner()).await;
    let mut open = Vec::new();
    for _ in 0..STREAM_CAP {
        let (stream, status) = open_stream(&server).await;
        assert!(status.unwrap().contains("101"));
        open.push(stream);
    }
    let (_, status) = open_stream(&server).await;
    let status = status.unwrap();
    assert!(status.contains("503"), "{status}");
    assert!(retry_after(&server).await.is_some_and(|value| value == "5"));
    drop(open.pop());
    let mut freed = None;
    for _ in 0..50 {
        let (stream, status) = open_stream(&server).await;
        if status.is_some_and(|line| line.contains("101")) {
            freed = Some(stream);
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert!(freed.is_some(), "a closed stream gives its place back");
}

/// Holds about 512 sockets at once (the clients and the accepted ends), so a machine whose default
/// limit of open files is 256 (macOS) needs `ulimit -n 4096` first; CI does that (issue #320).
#[tokio::test]
async fn a_client_that_is_silent_is_not_answered_beyond_the_connection_cap() {
    let server = start(owner()).await;
    let mut idle = Vec::new();
    for _ in 0..CONNECTION_CAP {
        idle.push(TcpStream::connect(server.address).await.unwrap());
    }
    let mut late = TcpStream::connect(server.address).await.unwrap();
    let request = format!(
        "GET /api/v1/commands HTTP/1.1\r\nHost: {}\r\n\r\n",
        server.address
    );
    let answered = timeout(Duration::from_millis(500), status_of(&mut late, &request)).await;
    assert!(answered.is_err(), "no slot is free, so nobody answers");
    drop(idle);
    let answered = timeout(Duration::from_secs(5), late.read_u8()).await;
    assert!(answered.is_ok(), "a freed slot lets the waiting client in");
}

#[tokio::test(start_paused = true)]
async fn a_stream_whose_client_never_answers_the_pings_is_closed() {
    let server = start(owner()).await;
    let mut stream = connect(&server).await;
    let request = format!("{UPGRADE}Host: {}\r\n\r\n", server.address);
    let status = status_of_settled(&mut stream, &request).await;
    assert!(status.unwrap().contains("101"));
    let rest = until_closed(&mut stream, Duration::from_secs(300)).await;
    assert_ne!(rest.unwrap().len(), 0, "it pinged before it gave up");
}

#[tokio::test(start_paused = true)]
async fn a_connection_without_a_single_byte_is_closed() {
    let server = start(owner()).await;
    let mut stream = connect(&server).await;
    let closed = until_closed(&mut stream, Duration::from_secs(60)).await;
    assert!(
        closed.is_some(),
        "the server gave up on a silent connection"
    );
}

#[tokio::test(start_paused = true)]
async fn a_head_that_never_ends_closes_the_connection() {
    let server = start(owner()).await;
    let mut stream = connect(&server).await;
    stream
        .write_all(b"GET /api/v1/commands HTTP/1.1\r\nHost: ")
        .await
        .unwrap();
    let closed = until_closed(&mut stream, Duration::from_secs(60)).await;
    assert!(closed.is_some(), "the server gave up on the head");
}

/// The answer to a POST to `path` whose body stops after two bytes and never ends.
#[cfg(test)]
async fn answer_to_stalled_body(server: &Running, path: &str) -> String {
    let mut stream = connect(server).await;
    let head = format!(
        "POST {path} HTTP/1.1\r\nHost: {}\r\nAuthorization: Bearer {TOKEN}\r\nContent-Type: application/json\r\nContent-Length: 100\r\n\r\n{{\"",
        server.address
    );
    stream.write_all(head.as_bytes()).await.unwrap();
    settle().await;
    let answer = collect_stepped(&mut stream, STALL_LIMIT, |answer| {
        answer.windows(4).any(|part| part == b"\r\n\r\n") && answer.ends_with(b"}")
    })
    .await;
    assert!(answer.is_some(), "the server answered the stalled body");
    String::from_utf8_lossy(&answer.unwrap_or_default()).into_owned()
}

#[tokio::test(start_paused = true)]
async fn a_body_that_never_ends_is_answered_with_a_timeout_code() {
    let server = start(owner()).await;
    let text = answer_to_stalled_body(&server, "/api/v1/commands/test.wipe").await;
    assert!(text.starts_with("HTTP/1.1 408"), "{text}");
    assert!(text.contains("API-503"), "{text}");
    assert_eq!(server.wipes.load(std::sync::atomic::Ordering::SeqCst), 0);
}

#[tokio::test]
async fn an_open_event_stream_does_not_hold_up_the_shutdown() {
    use std::sync::Arc;

    use codeskimr_application::events::EventBus;
    use codeskimr_application::registry::Registry;
    use codeskimr_server::{CommandApi, InstallationToken, Server};

    let server = Server::new(
        CommandApi::new(Arc::new(Registry::new())),
        EventBus::new(),
        common::owner(),
        InstallationToken::from_token(TOKEN),
        common::ui_options(),
    );
    let bound = server.bind(([127, 0, 0, 1], 0).into()).await.unwrap();
    let address = bound.address().unwrap();
    let (stop, task) = common::serve_until_told(bound);
    let mut client = TcpStream::connect(address).await.unwrap();
    let request = format!("{UPGRADE}Host: {address}\r\n\r\n");
    let status = status_of(&mut client, &request).await;
    assert!(status.unwrap().contains("101"));
    stop.send(()).unwrap();
    let ended = timeout(Duration::from_secs(3), task).await;
    assert!(ended.is_ok(), "the stream ended with the server");
}

#[tokio::test(start_paused = true)]
async fn an_mcp_body_that_never_ends_is_answered_with_a_timeout_code() {
    let server = common::start_mcp(owner()).await;
    let text = answer_to_stalled_body(&server, "/mcp").await;
    assert!(text.starts_with("HTTP/1.1 408"), "{text}");
    assert!(text.contains("API-503"), "{text}");
}

#[tokio::test(start_paused = true)]
async fn an_mcp_body_without_the_token_is_not_read_at_all() {
    let server = common::start_mcp(owner()).await;
    let mut stream = connect(&server).await;
    let head = format!(
        "POST /mcp HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: 100\r\n\r\n{{\"",
        server.address
    );
    let status = status_of_settled(&mut stream, &head).await.unwrap();
    assert!(status.contains("401"), "{status}");
}

/// What `server` answers to a POST to `path` that announces `length` bytes and sends `sent`, and
/// then leaves if `leave` is set.
#[cfg(test)]
async fn post_answer(
    server: &Running,
    path: &str,
    (length, sent): (usize, &str),
    leave: bool,
) -> String {
    let mut stream = connect(server).await;
    let head = format!(
        "POST {path} HTTP/1.1\r\nHost: {}\r\nAuthorization: Bearer {TOKEN}\r\nContent-Type: application/json\r\nContent-Length: {length}\r\n\r\n{sent}",
        server.address
    );
    stream.write_all(head.as_bytes()).await.unwrap();
    if leave {
        stream.shutdown().await.unwrap();
    }
    let mut answer = Vec::new();
    let _ = timeout(Duration::from_secs(1), stream.read_to_end(&mut answer)).await;
    String::from_utf8_lossy(&answer).into_owned()
}

const MCP_PATH: &str = "/mcp";
const REST_PATH: &str = "/api/v1/commands/test.wipe";

#[tokio::test]
async fn a_body_over_the_limit_is_too_large_on_both_kinds_of_route() {
    let body = "x".repeat(1024 * 1024 + 1);
    let server = common::start_mcp(owner()).await;
    for path in [MCP_PATH, REST_PATH] {
        let text = post_answer(&server, path, (body.len(), &body), false).await;
        assert!(text.starts_with("HTTP/1.1 413"), "{path}: {text}");
    }
}

#[tokio::test]
async fn a_client_that_leaves_in_the_middle_is_an_invalid_request_not_too_large() {
    let server = common::start_mcp(owner()).await;
    for path in [MCP_PATH, REST_PATH] {
        let text = post_answer(&server, path, (100, "{\""), true).await;
        // The status line only: the headers carry the random port of the server, and a port such
        // as 41357 contains "413".
        let status = text.lines().next().unwrap_or_default();
        assert!(
            status.is_empty() || status.starts_with("HTTP/1.1 400"),
            "{path}: {text}"
        );
    }
}
