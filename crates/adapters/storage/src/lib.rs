//! `SQLite` storage, read models and the search index (blueprint 6.2).
//!
//! [`SqliteStore`] implements the ports `ProjectRepository` and `SnapshotStore` of
//! `codeskimr_application`. One snapshot is written in one transaction together with its read
//! models, or not at all. The queries on the read models are inherent methods of the store.

mod codec;
mod consts;
mod cycles;
mod error;
mod migrate;
mod projects;
mod queries;
mod read_models;
mod snapshots;
mod sql;
mod store;

pub use queries::{SearchHit, TypeSummary};
pub use store::SqliteStore;
