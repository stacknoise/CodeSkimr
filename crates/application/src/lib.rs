//! Use cases and the ports (traits) that adapters implement.
//!
//! This crate holds the application core of blueprint section 4.1: the ports, the scan pipeline
//! (pipes and filters), the event bus (observer), the command registry, the typestate of a
//! scan job and the level-1 reference resolver. It knows no database, file system, network or parser.

pub mod access;
pub mod alerts;
pub mod annotations;
pub mod cancel;
pub mod config;
mod consts;
pub mod conventions;
pub mod error;
pub mod events;
pub mod ids;
pub mod level2;
pub mod metrics;
pub mod pipeline;
pub mod ports;
pub mod registry;
pub mod rename;
pub mod rename_rows;
pub mod report_view;
pub mod resolve;
pub mod rules;
pub mod scope;
pub mod selection;
pub mod stage;
pub mod tools;
pub mod unused;
