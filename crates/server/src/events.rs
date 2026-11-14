//! The event stream (blueprint 10.3): domain events as JSON text messages over a WebSocket.

use std::sync::Arc;
use std::sync::mpsc::RecvTimeoutError;
use std::thread;
use std::time::Duration;

use axum::body::Bytes;
use axum::extract::State;
use axum::extract::ws::{CloseFrame, Message, WebSocket, WebSocketUpgrade, close_code};
use axum::response::{IntoResponse, Response};
use codeskimr_application::access;
use codeskimr_application::events::{
    Event, EventBus, FilesChanged, ScanProgressed, ScanStopped, SnapshotCreated, StopReason,
    Subscription,
};
use codeskimr_codes::ErrorCode;
use codeskimr_domain::Permission;
use serde::Serialize;
use tokio::sync::mpsc::{Receiver, Sender, channel};
use tokio::sync::{OwnedSemaphorePermit, watch};
use tokio::time::{Instant, timeout};

use crate::consts::{
    EVENT_BUFFER, EVENT_POLL, EVENT_THREAD_NAME, MAX_SOCKET_MESSAGE_BYTES, REASON_CANCELLED,
    REASON_FAILED, SOCKET_CLOSE_REASON, SOCKET_IDLE_LIMIT, SOCKET_PING_INTERVAL,
    SOCKET_SEND_TIMEOUT, SOCKET_SLOW_BUDGET, SOCKET_SLOW_SEND,
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
        /// Milliseconds since the stage started.
        elapsed_ms: u64,
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
                elapsed_ms,
            }) => Self::ScanProgressed {
                project: project.get(),
                stage: stage.label(),
                done: *done,
                total: *total,
                elapsed_ms: *elapsed_ms,
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

/// Starts carrying every event of `bus` into a bounded channel of the async runtime. The thread
/// ends when the receiving end is dropped, or when the channel is full: a client that does not
/// keep up loses its stream instead of making the server hold more and more events.
fn bridge(bus: &EventBus) -> Option<Receiver<Event>> {
    let (sender, receiver) = channel(EVENT_BUFFER);
    let subscription = bus.subscribe();
    thread::Builder::new()
        .name(EVENT_THREAD_NAME.to_owned())
        .spawn(move || forward(&subscription, &sender))
        .ok()
        .map(|_| receiver)
}

fn forward(subscription: &Subscription, sender: &Sender<Event>) {
    loop {
        let gone = match subscription.recv_timeout(EVENT_POLL) {
            Ok(event) => sender.try_send(event).is_err(),
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
/// is read-only and needs the `read` permission only. At most `MAX_EVENT_STREAMS` streams run at
/// once; one more gets `API-502`.
pub async fn stream(State(app): State<AppState>, upgrade: WebSocketUpgrade) -> Response {
    if let Err(error) = access::require(&app.principal, Permission::Read) {
        return Failure(error).into_response();
    }
    let Ok(slot) = Arc::clone(&app.streams).try_acquire_owned() else {
        return Failure::code(ErrorCode::ServerBusy).into_response();
    };
    let stop = app.stop.clone();
    let Some(events) = bridge(&app.events) else {
        return Failure::code(ErrorCode::UnexpectedInternalError).into_response();
    };
    upgrade
        .max_message_size(MAX_SOCKET_MESSAGE_BYTES)
        .on_upgrade(move |socket| pump(socket, events, slot, stop))
}

/// When the client was last heard and last pinged.
struct Liveness {
    heard: Instant,
    pinged: Instant,
}

/// Sends events until the client goes away, falls too far behind or the bus ends. What the client
/// sends is ignored, but an idle client is pinged and must answer.
///
/// While no event comes, the socket is looked at every `EVENT_POLL`, so a client that left is
/// noticed without waiting for the next event. The slot is given back when the stream ends.
async fn pump(
    mut socket: WebSocket,
    mut events: Receiver<Event>,
    slot: OwnedSemaphorePermit,
    stop: watch::Receiver<()>,
) {
    let mut live = Liveness {
        heard: Instant::now(),
        pinged: Instant::now(),
    };
    let mut slow = SlowSends::default();
    loop {
        if stop.has_changed().unwrap_or(true) {
            close(&mut socket, close_code::AWAY).await;
            break;
        }
        match timeout(EVENT_POLL, events.recv()).await {
            Ok(Some(event)) => {
                let Ok(text) = serde_json::to_string(&WireEvent::from(&event)) else {
                    continue;
                };
                if !send_within(&mut socket, Message::Text(text.into()), &mut slow).await {
                    break;
                }
            }
            Ok(None) => {
                close(&mut socket, close_code::AGAIN).await;
                break;
            }
            Err(_) if !keep_alive(&mut socket, &mut live).await => break,
            Err(_) => {}
        }
    }
    drop(slot);
}

/// Ends the stream with a close frame of `code`.
async fn close(socket: &mut WebSocket, code: u16) {
    let frame = CloseFrame {
        code,
        reason: SOCKET_CLOSE_REASON.into(),
    };
    send(socket, Message::Close(Some(frame))).await;
}

/// Sends `message`; false if the client failed or took longer than `SOCKET_SEND_TIMEOUT`.
async fn send(socket: &mut WebSocket, message: Message) -> bool {
    matches!(
        timeout(SOCKET_SEND_TIMEOUT, socket.send(message)).await,
        Ok(Ok(()))
    )
}

/// Like [`send`], but also false once the slow sends of the stream have used up their budget.
async fn send_within(socket: &mut WebSocket, message: Message, slow: &mut SlowSends) -> bool {
    let started = Instant::now();
    let sent = send(socket, message).await;
    slow.add(started.elapsed());
    sent && !slow.exhausted()
}

/// The time one stream has spent in sends that took longer than `SOCKET_SLOW_SEND`.
#[derive(Default)]
struct SlowSends {
    spent: Duration,
}

impl SlowSends {
    /// Counts one send of `took` against the budget if it was slow.
    fn add(&mut self, took: Duration) {
        if took > SOCKET_SLOW_SEND {
            self.spent += took;
        }
    }

    fn exhausted(&self) -> bool {
        self.spent > SOCKET_SLOW_BUDGET
    }
}

/// Looks at the socket once without waiting and pings it when it has been quiet. False if the
/// client left, failed or has not been heard for `SOCKET_IDLE_LIMIT`.
async fn keep_alive(socket: &mut WebSocket, live: &mut Liveness) -> bool {
    match timeout(Duration::ZERO, socket.recv()).await {
        Ok(None | Some(Err(_) | Ok(Message::Close(_)))) => return false,
        Ok(Some(Ok(_))) => live.heard = Instant::now(),
        Err(_) => {}
    }
    if live.heard.elapsed() > SOCKET_IDLE_LIMIT {
        return false;
    }
    if live.pinged.elapsed() < SOCKET_PING_INTERVAL {
        return true;
    }
    live.pinged = Instant::now();
    send(socket, Message::Ping(Bytes::new())).await
}

#[cfg(test)]
mod tests {
    use codeskimr_domain::ProjectId;

    use super::*;

    #[test]
    fn only_slow_sends_use_up_the_budget_and_enough_of_them_end_the_stream() {
        let mut slow = SlowSends::default();
        for _ in 0..10_000 {
            slow.add(SOCKET_SLOW_SEND);
        }
        assert!(!slow.exhausted(), "fast sends cost nothing");
        slow.add(SOCKET_SEND_TIMEOUT);
        slow.add(SOCKET_SEND_TIMEOUT);
        assert!(!slow.exhausted());
        slow.add(SOCKET_SEND_TIMEOUT);
        slow.add(SOCKET_SLOW_SEND + Duration::from_millis(1));
        assert!(slow.exhausted());
    }

    fn changed(id: u64) -> Event {
        Event::FilesChanged(FilesChanged {
            project: ProjectId::new(id),
            files: 1,
            rescan: false,
        })
    }

    #[tokio::test]
    async fn a_client_that_does_not_read_loses_its_stream_after_the_buffer_is_full() {
        let bus = EventBus::new();
        let mut events = bridge(&bus).unwrap();
        for id in 0..2 * EVENT_BUFFER as u64 {
            bus.publish(&changed(id));
        }
        while events.len() < EVENT_BUFFER {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        // The forwarding thread meets the full buffer with the next event and ends.
        tokio::time::sleep(EVENT_POLL).await;
        let mut received = 0_usize;
        while timeout(Duration::from_secs(5), events.recv())
            .await
            .unwrap()
            .is_some()
        {
            received += 1;
        }
        assert_eq!(
            received, EVENT_BUFFER,
            "the buffer holds no more than its size"
        );
    }
}
