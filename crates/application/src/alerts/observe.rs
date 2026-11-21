//! What a symbol looks like now, as a baseline (ADR 0068, decision 4). The alert step compares it
//! with the stored baseline; marking a symbol as critical and dismissing an alert store it.

use codeskimr_domain::{AnnotationTarget, ContentHash, SnapshotId, SymbolBaseline, SymbolKind};

use crate::error::CodedError;
use crate::ports::{ContentDigest, SymbolState, SymbolStates};

use super::target;

/// Kinds that had no signature before functions and methods got one.
const CALLABLES: &[SymbolKind] = &[
    SymbolKind::Function,
    SymbolKind::Method,
    SymbolKind::Constructor,
];

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
    Ok(SymbolBaseline {
        content_hash,
        signature_hash: signature_hash(digest, state.signature.as_deref())?,
        loc: state.loc,
        cc: state.cc,
    })
}

/// The hash of a signature. Layout around brackets and commas is no change of the signature, so
/// it is hashed without whitespace; no signature hashes as empty.
fn signature_hash(
    digest: &dyn ContentDigest,
    signature: Option<&str>,
) -> Result<ContentHash, CodedError> {
    let compact: String = signature.unwrap_or_default().split_whitespace().collect();
    if compact.is_empty() {
        digest.digest(&[])
    } else {
        digest.digest(&[compact.as_str()])
    }
}

/// The signature hashes that older versions stored for a symbol that has not changed: the hash
/// over the words of its signature, and for a function, method or constructor, which had no
/// signature then, the empty hash. A baseline that holds one of them is taken over without an
/// alert (see `AlertStep`).
///
/// # Errors
/// A coded failure of the digest.
pub fn legacy_signature_hashes(
    digest: &dyn ContentDigest,
    state: &SymbolState,
) -> Result<Vec<ContentHash>, CodedError> {
    let words: Vec<&str> = state
        .signature
        .as_deref()
        .unwrap_or_default()
        .split_whitespace()
        .collect();
    let mut hashes = vec![digest.digest(&words)?];
    if CALLABLES.contains(&state.kind) {
        hashes.push(digest.digest(&[])?);
    }
    Ok(hashes)
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
