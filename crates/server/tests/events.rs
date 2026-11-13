//! The event stream: JSON messages over a WebSocket, with the same Origin protection.

mod common;

use std::time::Duration;

use codeskimr_application::events::{Event, FilesChanged, ScanProgressed, ScanStopped, StopReason};
use codeskimr_application::stage::StageKind;
use codeskimr_codes::ErrorCode;
use codeskimr_domain::{PermissionSet, Principal, ProjectId};
use common::{Call, Running, owner, start};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;

const UPGRADE: &str = "GET /api/v1/events HTTP/1.1\r\nConnection: Upgrade\r\nUpgrade: websocket\r\nSec-WebSocket-Version: 13\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n";

#[cfg(test)]
/// Opens the stream and returns the connection and the status line of the answer.
async fn open(server: &Running, extra: &str) -> (TcpStream, String) {
    let mut stream = TcpStream::connect(server.address).await.unwrap();
    let request = format!("{UPGRADE}Host: {}\r\n{extra}\r\n", server.address);
    stream.write_all(request.as_bytes()).await.unwrap();
    let mut head = Vec::new();
    while !head.ends_with(b"\r\n\r\n") {
        let mut byte = [0_u8; 1];
        stream.read_exact(&mut byte).await.unwrap();
        head.push(byte[0]);
    }
    let text = String::from_utf8(head).unwrap();
    (stream, text.lines().next().unwrap().to_owned())
}

#[cfg(test)]
/// Reads one unmasked text frame of a server.
async fn frame(stream: &mut TcpStream) -> String {
    let mut head = [0_u8; 2];
    timeout(Duration::from_secs(5), stream.read_exact(&mut head))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(head[0], 0x81, "a final text frame");
    let mut payload = vec![0_u8; usize::from(head[1])];
    assert!(head[1] < 126);
    stream.read_exact(&mut payload).await.unwrap();
    String::from_utf8(payload).unwrap()
}

#[tokio::test]
async fn events_arrive_as_json_in_order() {
    let server = start(owner()).await;
    let (mut stream, status) = open(&server, "").await;
    assert!(status.contains("101"), "{status}");
    let project = ProjectId::new(7);
    server
        .events
        .publish(&Event::ScanProgressed(ScanProgressed {
            project,
            stage: StageKind::Analyze,
            done: 3,
            total: 10,
        }));
    server.events.publish(&Event::FilesChanged(FilesChanged {
        project,
        files: 2,
        rescan: true,
    }));
    server.events.publish(&Event::ScanStopped(ScanStopped {
        project,
        reason: StopReason::Failed(ErrorCode::ProjectNotFound),
    }));
    server.events.publish(&Event::ScanStopped(ScanStopped {
        project,
        reason: StopReason::Cancelled,
    }));
    let seen: Vec<serde_json::Value> = vec![
        serde_json::from_str(&frame(&mut stream).await).unwrap(),
        serde_json::from_str(&frame(&mut stream).await).unwrap(),
        serde_json::from_str(&frame(&mut stream).await).unwrap(),
        serde_json::from_str(&frame(&mut stream).await).unwrap(),
    ];
    assert_eq!(
        seen,
        [
            serde_json::json!({"type":"scan_progressed","project":7,"stage":"analyze","done":3,"total":10}),
            serde_json::json!({"type":"files_changed","project":7,"files":2,"rescan":true}),
            serde_json::json!({"type":"scan_stopped","project":7,"reason":"failed","code":"PRJ-201"}),
            serde_json::json!({"type":"scan_stopped","project":7,"reason":"cancelled","code":null}),
        ]
    );
}

#[tokio::test]
async fn a_foreign_origin_cannot_open_the_stream() {
    let server = start(owner()).await;
    for origin in ["http://evil.example", "null"] {
        let (_, status) = open(&server, &format!("Origin: {origin}\r\n")).await;
        assert!(status.contains("403"), "{origin}: {status}");
    }
    let own = format!("Origin: http://{}\r\n", server.address);
    let (_, status) = open(&server, &own).await;
    assert!(status.contains("101"), "{status}");
}

#[tokio::test]
async fn a_caller_without_read_gets_no_stream() {
    let server = start(Principal::limited(PermissionSet::none())).await;
    let (_, status) = open(&server, "").await;
    assert!(status.contains("403"), "{status}");
}

#[tokio::test]
async fn a_closed_stream_does_not_leave_a_subscriber_behind_for_ever() {
    let server = start(owner()).await;
    for _ in 0..5 {
        let (stream, status) = open(&server, "").await;
        assert!(status.contains("101"));
        drop(stream);
    }
    // The next events reach nobody, and the server stays healthy.
    for _ in 0..3 {
        server.events.publish(&Event::FilesChanged(FilesChanged {
            project: ProjectId::new(1),
            files: 1,
            rescan: false,
        }));
    }
    let reply = Call::get("/api/v1/commands").send(&server).await;
    assert_eq!(reply.status, 200);
}
