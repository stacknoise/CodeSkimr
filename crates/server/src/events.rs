//! The event stream (blueprint 10.3): domain events as JSON text messages over a WebSocket.

use std::sync::mpsc::RecvTimeoutError;
use std::thread;

use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::{IntoResponse, Response};
use codeskimr_application::access;
use codeskimr_application::events::{
    Event, EventBus, FilesChanged, ScanProgressed, ScanStopped, SnapshotCreated, StopReason,
    Subscription,
};
use codeskimr_codes::ErrorCode;
use codeskimr_domain::Permission;
use serde::Serialize;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

use crate::consts::{
    EVENT_POLL, EVENT_THREAD_NAME, MAX_SOCKET_MESSAGE_BYTES, REASON_CANCELLED, REASON_FAILED,
};
use crate::error::Failure;
use crate::state::AppState;

/// One event as it goes over the wire. Only ids, counts and registered codes: no names, no paths
/// and no source text, so a stream needs no more than the `read` permission.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WireEvent {
    /// A stage of a scan advanced.
    ScanProgressed {
        /// Project being scanned.
        project: u64,
        /// Label of the stage.
        stage: &'static str,
        /// Units finished.
        done: u32,
        /// Units in total, zero if unknown.
        total: u32,
    },
    /// A snapshot was written.
    SnapshotCreated {
        /// Owning project.
        project: u64,
        /// The new snapshot.
        snapshot: u64,
    },
    /// Files changed on disk.
    FilesChanged {
        /// Project whose files changed.
        project: u64,
        /// Number of changed files.
        files: u32,
        /// Events were lost; scan the whole folder.
        rescan: bool,
    },
    /// A scan ended without a snapshot.
    ScanStopped {
        /// Project that was scanned.
        project: u64,
        /// `cancelled` or `failed`.
        reason: &'static str,
        /// The code of the failure, if it failed.
        code: Option<&'static str>,
    },
}

impl From<&Event> for WireEvent {
    fn from(event: &Event) -> Self {
        match event {
            Event::ScanProgressed(ScanProgressed {
                project,
                stage,
                done,
                total,
            }) => Self::ScanProgressed {
                project: project.get(),
                stage: stage.label(),
                done: *done,
                total: *total,
            },
            Event::SnapshotCreated(SnapshotCreated { snapshot }) => Self::SnapshotCreated {
                project: snapshot.project.get(),
                snapshot: snapshot.id.get(),
            },
            Event::FilesChanged(FilesChanged {
                project,
                files,
                rescan,
            }) => Self::FilesChanged {
                project: project.get(),
                files: *files,
                rescan: *rescan,
            },
            Event::ScanStopped(stopped) => Self::stopped(stopped),
        }
    }
}

impl WireEvent {
    fn stopped(stopped: &ScanStopped) -> Self {
        let (reason, code): (_, Option<ErrorCode>) = match stopped.reason {
            StopReason::Cancelled => (REASON_CANCELLED, None),
            StopReason::Failed(code) => (REASON_FAILED, Some(code)),
        };
        Self::ScanStopped {
            project: stopped.project.get(),
            reason,
            code: code.map(ErrorCode::id),
        }
    }
}

/// Starts carrying every event of `bus` into a channel of the async runtime. The thread ends when
/// the receiving end is dropped.
fn bridge(bus: &EventBus) -> Option<UnboundedReceiver<Event>> {
    let (sender, receiver) = unbounded_channel();
    let subscription = bus.subscribe();
    thread::Builder::new()
        .name(EVENT_THREAD_NAME.to_owned())
        .spawn(move || forward(&subscription, &sender))
        .ok()
        .map(|_| receiver)
}

fn forward(subscription: &Subscription, sender: &UnboundedSender<Event>) {
    loop {
        let gone = match subscription.recv_timeout(EVENT_POLL) {
            Ok(event) => sender.send(event).is_err(),
            Err(RecvTimeoutError::Timeout) => sender.is_closed(),
            Err(RecvTimeoutError::Disconnected) => true,
        };
        if gone {
            return;
        }
    }
}

/// `GET /api/v1/events`: upgrades to a WebSocket that streams the events from now on.
///
/// The `Origin` check of the guard has passed already. No token is read from the URL; the stream
/// is read-only and needs the `read` permission only.
pub async fn stream(State(app): State<AppState>, upgrade: WebSocketUpgrade) -> Response {
    if let Err(error) = access::require(&app.principal, Permission::Read) {
        return Failure(error).into_response();
    }
    let Some(events) = bridge(&app.events) else {
        return Failure::code(ErrorCode::UnexpectedInternalError).into_response();
    };
    upgrade
        .max_message_size(MAX_SOCKET_MESSAGE_BYTES)
        .on_upgrade(move |socket| pump(socket, events))
}

/// Sends events until the client goes away or the bus ends. What the client sends is ignored.
async fn pump(mut socket: WebSocket, mut events: UnboundedReceiver<Event>) {
    loop {
        tokio::select! {
            event = events.recv() => {
                let Some(event) = event else { return };
                let Ok(text) = serde_json::to_string(&WireEvent::from(&event)) else { continue };
                if socket.send(Message::Text(text.into())).await.is_err() {
                    return;
                }
            }
            incoming = socket.recv() => match incoming {
                None | Some(Err(_) | Ok(Message::Close(_))) => return,
                Some(Ok(_)) => {}
            },
        }
    }
}
