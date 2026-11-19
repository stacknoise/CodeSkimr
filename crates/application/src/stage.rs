//! The named stages of the scan pipeline (blueprint section 7).

use crate::consts::{
    STAGE_ANALYZE, STAGE_DISCOVER, STAGE_FETCH, STAGE_MEASURE, STAGE_PERSIST, STAGE_RESOLVE,
};

/// Identifies a pipeline stage in events and errors.
///
/// Parsing and symbol extraction form one stage: Tree-sitter trees never cross a port, so the
/// application layer sees only the extracted facts of a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum StageKind {
    /// Make the source available locally.
    Fetch,
    /// Walk the files, detect languages, hash contents.
    Discover,
    /// Parse each file and extract its symbols.
    Analyze,
    /// Map reference names to symbols.
    Resolve,
    /// Compute metrics.
    Measure,
    /// Write the snapshot and update read models.
    Persist,
}

impl StageKind {
    /// Every stage in execution order.
    pub const ALL: &'static [Self] = &[
        Self::Fetch,
        Self::Discover,
        Self::Analyze,
        Self::Resolve,
        Self::Measure,
        Self::Persist,
    ];

    /// The label used on the wire.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Fetch => STAGE_FETCH,
            Self::Discover => STAGE_DISCOVER,
            Self::Analyze => STAGE_ANALYZE,
            Self::Resolve => STAGE_RESOLVE,
            Self::Measure => STAGE_MEASURE,
            Self::Persist => STAGE_PERSIST,
        }
    }
}
