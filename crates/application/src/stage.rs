//! The named stages of the scan pipeline (blueprint section 7).

use crate::consts::{
    STAGE_FETCH, STAGE_LABELS, STEP_COMMIT, STEP_DETAILS, STEP_FACTS, STEP_FINGERPRINTS,
    STEP_HEADER, STEP_INTEGRITY, STEP_PRUNE, STEP_PRUNE_BEFORE, STEP_READ_MODELS,
    STEP_REFERENCE_INDEXES, STEP_REFERENCES,
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

/// One step of writing a snapshot, in the order the store takes them. A scan of a large project
/// spends minutes in a single one of them, so the store tells which one it is in (the stage
/// `Persist` reports it with its progress). A step that a write does not need is left out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PersistStep {
    /// Insert the snapshot header.
    Header,
    /// Drop the reference indexes and delete expired snapshots before a large full scan loads.
    PruneBeforeLoad,
    /// Insert files, symbols and metrics.
    Facts,
    /// Insert the references.
    References,
    /// Build the reference indexes that a bulk load left out.
    ReferenceIndexes,
    /// Insert the fingerprints for similar code.
    Fingerprints,
    /// Insert naming rules, parse times, warnings and left-out files; carry rows over.
    Details,
    /// Check the integrity of what was written.
    Integrity,
    /// Build the read models and set the totals.
    ReadModels,
    /// Delete the snapshots beyond the retention.
    Prune,
    /// Commit the transaction.
    Commit,
}

impl PersistStep {
    /// Every step in the order of a write.
    pub const ALL: &'static [Self] = &[
        Self::Header,
        Self::PruneBeforeLoad,
        Self::Facts,
        Self::References,
        Self::ReferenceIndexes,
        Self::Fingerprints,
        Self::Details,
        Self::Integrity,
        Self::ReadModels,
        Self::Prune,
        Self::Commit,
    ];

    /// The label used on the wire and in the log.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Header => STEP_HEADER,
            Self::PruneBeforeLoad => STEP_PRUNE_BEFORE,
            Self::Facts => STEP_FACTS,
            Self::References => STEP_REFERENCES,
            Self::ReferenceIndexes => STEP_REFERENCE_INDEXES,
            Self::Fingerprints => STEP_FINGERPRINTS,
            Self::Details => STEP_DETAILS,
            Self::Integrity => STEP_INTEGRITY,
            Self::ReadModels => STEP_READ_MODELS,
            Self::Prune => STEP_PRUNE,
            Self::Commit => STEP_COMMIT,
        }
    }

    /// The place of the step in the write, counted from one.
    #[must_use]
    pub fn number(self) -> u32 {
        let before = Self::ALL.iter().take_while(|step| **step != self).count();
        u32::try_from(before).map_or(u32::MAX, |before| before.saturating_add(1))
    }

    /// How many steps a write has at most.
    #[must_use]
    pub fn count() -> u32 {
        u32::try_from(Self::ALL.len()).unwrap_or(u32::MAX)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_steps_are_numbered_from_one_in_the_order_of_the_write() {
        let numbers: Vec<u32> = PersistStep::ALL.iter().map(|step| step.number()).collect();
        let expected: Vec<u32> = (1..=PersistStep::count()).collect();
        assert_eq!(numbers, expected);
        assert!(PersistStep::ALL.windows(2).all(|pair| pair[0] < pair[1]));
    }

    #[test]
    fn every_step_has_its_own_label() {
        let mut labels: Vec<&str> = PersistStep::ALL.iter().map(|step| step.label()).collect();
        labels.sort_unstable();
        labels.dedup();
        assert_eq!(labels.len(), PersistStep::ALL.len());
    }
}
