//! Use cases and the ports (traits) that adapters implement.
//!
//! This crate holds the application core of blueprint section 4.1: the ports, the scan pipeline
//! (pipes and filters), the event bus (observer), the command registry and the typestate of a
//! scan job. It knows no database, file system, network or parser.

pub mod cancel;
pub mod config;
mod consts;
pub mod error;
pub mod events;
pub mod ids;
pub mod metrics;
pub mod pipeline;
pub mod ports;
pub mod registry;
pub mod stage;
