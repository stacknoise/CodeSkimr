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

    /// The labels of [`Self::ALL`], in the same order.
    const LABELS: [&'static str; 6] = [
        STAGE_FETCH,
        STAGE_DISCOVER,
        STAGE_ANALYZE,
        STAGE_RESOLVE,
        STAGE_MEASURE,
        STAGE_PERSIST,
    ];

    /// The label used on the wire.
    #[must_use]
    pub fn label(self) -> &'static str {
        Self::ALL
            .iter()
            .zip(Self::LABELS)
            .find(|(kind, _)| **kind == self)
            .map_or(STAGE_FETCH, |(_, label)| label)
    }
}
