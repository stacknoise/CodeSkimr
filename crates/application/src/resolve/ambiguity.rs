//! How many occurrences level 1 could not narrow down to one target (`RES-301`).

use std::collections::HashMap;

use codeskimr_domain::{Confidence, FileId, Reference, Span};

/// The number of occurrences that have several unchecked targets. One occurrence is counted once,
/// however many targets it has, so the number tells how many places are uncertain.
#[must_use]
pub fn ambiguous_occurrences<'a>(references: impl Iterator<Item = &'a Reference>) -> u64 {
    let mut targets: HashMap<(FileId, Span), u64> = HashMap::new();
    for reference in references.filter(|r| !r.verified && r.confidence == Confidence::Probable) {
        *targets.entry((reference.file, reference.span)).or_default() += 1;
    }
    let several = targets.values().filter(|count| **count > 1).count();
    u64::try_from(several).unwrap_or(u64::MAX)
}
