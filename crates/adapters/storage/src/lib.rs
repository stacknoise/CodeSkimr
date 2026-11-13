//! `SQLite` storage, read models and the search index (blueprint 6.2).
//!
//! [`SqliteStore`] implements the ports `ProjectRepository`, `ProjectCatalog`, `SnapshotStore`, `CodeIndex`,
//! `UsageIndex` and `SimilarityIndex` of `codeskimr_application`. One snapshot is written in one transaction
//! together with its read models, or not at all. The queries on the read models are inherent
//! methods of the store.

mod catalog;
mod codec;
mod consts;
mod cycles;
mod detail;
mod error;
mod integrity;
mod migrate;
mod outline;
mod previous;
mod projects;
mod queries;
mod read_models;
mod similarity;
mod snapshots;
mod sql;
mod store;
mod usages;

pub use codeskimr_application::ports::SearchHit;
pub use queries::TypeSummary;
pub use store::SqliteStore;
