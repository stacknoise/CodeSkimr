//! The event stream: JSON messages over a WebSocket, with the same Origin protection.

mod common;

use std::time::Duration;

use codeskimr_application::events::{
    AlertsChanged, Event, FilesChanged, Level2Finished, Level2Progressed, ScanProgressed,
    ScanStopped, StopReason,
};
use codeskimr_application::stage::{PersistStep, StageKind};
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
    let length = if head[1] == 126 {
        // Sixteen-bit length for payloads of 126 bytes and more.
        let mut extended = [0_u8; 2];
        stream.read_exact(&mut extended).await.unwrap();
        usize::from(u16::from_be_bytes(extended))
    } else {
        assert!(head[1] < 126);
        usize::from(head[1])
    };
    let mut payload = vec![0_u8; length];
    stream.read_exact(&mut payload).await.unwrap();
    String::from_utf8(payload).unwrap()
}

/// A report of the analyze stage: three of ten files after a quarter of a second.
fn progressed(project: ProjectId) -> Event {
    Event::ScanProgressed(ScanProgressed {
        project,
        stage: StageKind::Analyze,
        done: 3,
        total: 10,
        elapsed_ms: 250,
        step: None,
    })
}

#[tokio::test]
async fn a_report_inside_a_step_of_the_write_names_the_step_and_its_place() {
    let server = start(owner()).await;
    let (mut stream, _) = open(&server, "").await;
    server
        .events
        .publish(&Event::ScanProgressed(ScanProgressed {
            project: ProjectId::new(7),
            stage: StageKind::Persist,
            done: 0,
            total: 1,
            elapsed_ms: 9,
            step: Some(PersistStep::ReferenceIndexes),
        }));
    let json: serde_json::Value = serde_json::from_str(&frame(&mut stream).await).unwrap();
    assert_eq!(json["step"], "reference_indexes");
    assert_eq!(json["step_number"], PersistStep::ReferenceIndexes.number());
    assert_eq!(json["steps"], PersistStep::count());
}

#[tokio::test]
async fn events_arrive_as_json_in_order() {
    let server = start(owner()).await;
    let (mut stream, status) = open(&server, "").await;
    assert!(status.contains("101"), "{status}");
    let project = ProjectId::new(7);
    server.events.publish(&progressed(project));
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
            serde_json::json!({"type":"scan_progressed","project":7,"stage":"analyze","done":3,"total":10,"elapsed_ms":250}),
            serde_json::json!({"type":"files_changed","project":7,"files":2,"rescan":true}),
            serde_json::json!({"type":"scan_stopped","project":7,"reason":"failed","code":"PRJ-201"}),
            serde_json::json!({"type":"scan_stopped","project":7,"reason":"cancelled","code":null}),
        ]
    );
}

#[tokio::test]
async fn a_stream_opened_during_a_scan_starts_with_the_last_report_of_the_scan() {
    let server = start(owner()).await;
    let project = ProjectId::new(7);
    server.events.publish(&progressed(project));
    let (mut stream, status) = open(&server, "").await;
    assert!(status.contains("101"), "{status}");
    let first: serde_json::Value = serde_json::from_str(&frame(&mut stream).await).unwrap();
    assert_eq!(first["type"], "scan_progressed");
    assert_eq!(first["project"], 7);
    assert_eq!(first["stage"], "analyze");
    assert_eq!(
        (first["done"].as_u64(), first["total"].as_u64()),
        (Some(3), Some(10))
    );
}

#[tokio::test]
async fn alerts_changed_carries_the_project_and_a_count_and_no_text() {
    let server = start(owner()).await;
    let (mut stream, status) = open(&server, "").await;
    assert!(status.contains("101"), "{status}");
    server.events.publish(&Event::AlertsChanged(AlertsChanged {
        project: ProjectId::new(7),
        open: 3,
    }));
    let seen: serde_json::Value = serde_json::from_str(&frame(&mut stream).await).unwrap();
    assert_eq!(
        seen,
        serde_json::json!({"type":"alerts_changed","project":7,"open":3})
    );
    let object = seen.as_object().unwrap();
    assert_eq!(object.len(), 3, "type, project and the count, nothing else");
    assert!(
        object
            .values()
            .all(|value| !value.is_string() || value == "alerts_changed")
    );
}

#[tokio::test]
async fn the_two_level_2_events_carry_ids_counts_and_labels_only() {
    let server = start(owner()).await;
    let (mut stream, _status) = open(&server, "").await;
    server
        .events
        .publish(&Event::Level2Progressed(Level2Progressed {
            project: ProjectId::new(7),
            language: codeskimr_domain::LanguageId::new("csharp").unwrap(),
            stage: codeskimr_domain::Level2Stage::Running,
        }));
    server
        .events
        .publish(&Event::Level2Finished(Level2Finished {
            project: ProjectId::new(7),
            snapshot: codeskimr_domain::SnapshotId::new(3),
            checked: 1,
            skipped: 2,
            failed: 0,
        }));
    let first: serde_json::Value = serde_json::from_str(&frame(&mut stream).await).unwrap();
    let second: serde_json::Value = serde_json::from_str(&frame(&mut stream).await).unwrap();
    assert_eq!(
        first,
        serde_json::json!({"type":"level2_progressed","project":7,"language":"csharp","stage":"level2_running"})
    );
    assert_eq!(
        second,
        serde_json::json!({"type":"level2_finished","project":7,"snapshot":3,"checked":1,"skipped":2,"failed":0})
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
