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

/// The status line the server answers to `request`, or none if it closes without an answer.
#[cfg(test)]
async fn status_of(stream: &mut TcpStream, request: &str) -> Option<String> {
    stream.write_all(request.as_bytes()).await.unwrap();
    let mut buffer = vec![0_u8; 512];
    let read = stream.read(&mut buffer).await.unwrap_or_default();
    let text = String::from_utf8_lossy(buffer.get(..read).unwrap_or_default()).into_owned();
    text.lines().next().map(str::to_owned)
}

/// What the server sends until it closes the connection, or none if it does not within `limit`.
#[cfg(test)]
async fn until_closed(stream: &mut TcpStream, limit: Duration) -> Option<Vec<u8>> {
    let mut rest = Vec::new();
    timeout(limit, stream.read_to_end(&mut rest))
        .await
        .ok()?
        .ok()?;
    Some(rest)
}

#[cfg(test)]
async fn open_stream(server: &Running) -> (TcpStream, Option<String>) {
    let mut stream = TcpStream::connect(server.address).await.unwrap();
    let request = format!("{UPGRADE}Host: {}\r\n\r\n", server.address);
    let status = status_of(&mut stream, &request).await;
    (stream, status)
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
    let (mut stream, status) = open_stream(&server).await;
    assert!(status.unwrap().contains("101"));
    let rest = until_closed(&mut stream, Duration::from_secs(300)).await;
    assert_ne!(rest.unwrap().len(), 0, "it pinged before it gave up");
}

#[tokio::test(start_paused = true)]
async fn a_connection_without_a_single_byte_is_closed() {
    let server = start(owner()).await;
    let mut stream = TcpStream::connect(server.address).await.unwrap();
    let closed = until_closed(&mut stream, Duration::from_secs(60)).await;
    assert!(
        closed.is_some(),
        "the server gave up on a silent connection"
    );
}

#[tokio::test(start_paused = true)]
async fn a_head_that_never_ends_closes_the_connection() {
    let server = start(owner()).await;
    let mut stream = TcpStream::connect(server.address).await.unwrap();
    stream
        .write_all(b"GET /api/v1/commands HTTP/1.1\r\nHost: ")
        .await
        .unwrap();
    let closed = until_closed(&mut stream, Duration::from_secs(60)).await;
    assert!(closed.is_some(), "the server gave up on the head");
}

#[tokio::test(start_paused = true)]
async fn a_body_that_never_ends_is_answered_with_a_timeout_code() {
    let server = start(owner()).await;
    let mut stream = TcpStream::connect(server.address).await.unwrap();
    let head = format!(
        "POST /api/v1/commands/test.wipe HTTP/1.1\r\nHost: {}\r\nAuthorization: Bearer {TOKEN}\r\nContent-Type: application/json\r\nContent-Length: 100\r\n\r\n{{\"",
        server.address
    );
    stream.write_all(head.as_bytes()).await.unwrap();
    let mut answer = Vec::new();
    let finished = timeout(Duration::from_secs(120), async {
        let mut buffer = [0_u8; 1024];
        loop {
            let read = stream.read(&mut buffer).await.unwrap_or_default();
            answer.extend_from_slice(buffer.get(..read).unwrap_or_default());
            if read == 0
                || answer.windows(4).any(|part| part == b"\r\n\r\n") && answer.ends_with(b"}")
            {
                break;
            }
        }
    })
    .await;
    assert!(finished.is_ok());
    let text = String::from_utf8_lossy(&answer);
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
