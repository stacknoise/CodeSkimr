//! Domain events and the bus that delivers them (observer pattern, blueprint section 5).
//!
//! The pipeline publishes each event once; WebSocket, CLI and MCP subscribe.

mod bus;
mod event;

pub use bus::{EventBus, Subscription};
pub use event::{
    AlertsChanged, Event, FilesChanged, Level2Finished, Level2Progressed, ScanProgressed,
    ScanStopped, SnapshotCreated, StopReason,
};
