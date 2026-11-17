//! The use cases of annotations and alerts behind the commands, the REST routes, the CLI and the
//! MCP tools (ADR 0068, decisions 7 and 8).

mod page;
mod read;
mod write;

pub use page::{Page, PageRequest};
pub use read::{
    AlertEntry, AlertPage, AlertQuery, AnnotationQuery, ContentTexts, TargetSummary, alerts,
    annotations, matching_alerts, summaries,
};
pub use write::{AnnotationWrites, Change, ContentInput, Field};
