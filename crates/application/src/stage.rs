//! The named stages of the scan pipeline (blueprint section 7).

use crate::consts::{STAGE_FETCH, STAGE_LABELS};

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
    ///
    /// The labels sit in a table in execution order; the variants are declared in that order and
    /// a test pins the labels against [`Self::ALL`]. The fallback is therefore never reached.
    #[must_use]
    pub fn label(self) -> &'static str {
        STAGE_LABELS
            .get(self as usize)
            .copied()
            .unwrap_or(STAGE_FETCH)
    }
}
