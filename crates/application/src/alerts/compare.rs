//! Comparing what a critical symbol looks like now with its baseline (ADR 0068, decision 4).

use codeskimr_domain::{AlertChange, SymbolBaseline};

/// What differs between the baseline and the symbol as it is now, in the order content,
/// signature, complexity, lines.
///
/// A difference in complexity or lines alone is no change worth an alert: both are measured from
/// the same tokens as the content hash, so they differ without it only if the code was laid out
/// differently, which is the cosmetic edit that must raise nothing. They are named when the content
/// or the signature differs too.
#[must_use]
pub fn differences(baseline: &SymbolBaseline, now: &SymbolBaseline) -> Vec<AlertChange> {
    let mut changes = Vec::new();
    if baseline.content_hash != now.content_hash {
        changes.push(AlertChange::Content);
    }
    if baseline.signature_hash != now.signature_hash {
        changes.push(AlertChange::Signature);
    }
    if changes.is_empty() {
        return changes;
    }
    if baseline.cc != now.cc {
        changes.push(AlertChange::Cc);
    }
    if baseline.loc != now.loc {
        changes.push(AlertChange::Loc);
    }
    changes
}
