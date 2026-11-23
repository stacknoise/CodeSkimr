//! `SQLite` storage, read models and the search index (blueprint 6.2).
//!
//! [`SqliteStore`] implements the ports `ProjectRepository`, `ProjectCatalog`, `SnapshotStore`, `CodeIndex`,
//! `UsageIndex`, `SimilarityIndex`, `ToolReportStore`, `AnnotationStore` and `SymbolStates` of `codeskimr_application`. One snapshot is written in one transaction
//! together with its read models, or not at all. The queries on the read models are inherent
//! methods of the store.

mod alerts;
mod annotations;
mod answers;
mod baselines;
mod catalog;
mod codec;
mod consent;
mod consts;
mod cycles;
mod detail;
mod error;
mod headers;
mod integrity;
mod migrate;
mod outline;
mod pending;
mod previous;
mod projects;
mod queries;
mod ranking_rows;
mod read_models;
mod references;
mod removal;
mod replace;
mod reports;
mod similarity;
mod sites;
mod snapshots;
mod sql;
mod states;
mod statistics;
mod store;
mod symbol_statistics;
mod tools;
mod usages;

pub use codeskimr_application::ports::SearchHit;
pub use queries::TypeSummary;
pub use store::{SqliteStore, StoreOptions};
