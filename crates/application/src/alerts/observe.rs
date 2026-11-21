//! What a symbol looks like now, as a baseline (ADR 0068, decision 4). The alert step compares it
//! with the stored baseline; marking a symbol as critical and dismissing an alert store it.

use codeskimr_domain::{AnnotationTarget, ContentHash, SnapshotId, SymbolBaseline, SymbolKind};

use crate::error::CodedError;
use crate::ports::{ContentDigest, SymbolState, SymbolStates};

use super::target;

/// What the symbol looks like now. A namespace has no hash of its own: its content is the hash over
/// the sorted hashes of the types and members below it, so it changes when one of them does, is
/// added or is removed. A symbol without a hash (an old row, a kind that has none) hashes as empty,
/// so only its signature, complexity and lines can tell it apart.
///
/// # Errors
/// A coded failure of the `STO` group, or of the digest.
pub fn baseline_of(
    reads: (&dyn SymbolStates, &dyn ContentDigest),
    snapshot: SnapshotId,
    state: &SymbolState,
) -> Result<SymbolBaseline, CodedError> {
    let (states, digest) = reads;
    let content_hash = if state.kind == SymbolKind::Namespace {
        let mut hashes = states.child_hashes(snapshot, &state.name)?;
        hashes.sort();
        let parts: Vec<&str> = hashes.iter().map(ContentHash::as_str).collect();
        digest.digest(&parts)?
    } else {
        match &state.content_hash {
            Some(hash) => hash.clone(),
            None => digest.digest(&[])?,
        }
    };
    // Layout around brackets and commas is no change of the signature: hash it without whitespace.
    let compact: String = state
        .signature
        .as_deref()
        .unwrap_or_default()
        .split_whitespace()
        .collect();
    Ok(SymbolBaseline {
        content_hash,
        signature_hash: digest.digest(&[compact.as_str()])?,
        loc: state.loc,
        cc: state.cc,
    })
}

/// What each of `targets` looks like in `snapshot`, in the order given; `None` for a target the
/// snapshot does not have. All targets are looked up in one read.
///
/// # Errors
/// A coded failure of the `STO` group, or of the digest.
pub fn current_baselines(
    reads: (&dyn SymbolStates, &dyn ContentDigest),
    snapshot: SnapshotId,
    targets: &[&AnnotationTarget],
) -> Result<Vec<Option<SymbolBaseline>>, CodedError> {
    let mut names: Vec<_> = targets.iter().map(|target| target.name.clone()).collect();
    names.sort();
    names.dedup();
    let states = reads.0.states(snapshot, &names)?;
    targets
        .iter()
        .map(|wanted| {
            target::pick(&states, wanted)
                .map(|state| baseline_of(reads, snapshot, state))
                .transpose()
        })
        .collect()
}
