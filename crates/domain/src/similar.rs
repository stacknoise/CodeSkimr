//! Finding the similar pairs in a set of fingerprinted bodies.
//!
//! The duplicate check of `check_conventions` and `find_similar_code` share this measure with
//! [`Fingerprints::similarity`] (blueprint 10.4: one implementation). The pairs come from an
//! inverted index of the hashes, so bodies that share no hash are never compared. A hash that
//! very many bodies share still costs the square of that number.

use std::collections::{HashMap, HashSet};

use crate::consts::PERCENT_BASE;
use crate::fingerprint::{Fingerprints, ratio};

/// Two bodies whose similarity reached the minimum.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SimilarPair {
    /// Index of the earlier body in the input.
    pub first: usize,
    /// Index of the later body in the input.
    pub second: usize,
    /// Jaccard similarity, the same value as [`Fingerprints::similarity`] gives.
    pub similarity: f64,
    /// The similarity in whole percent, rounded down.
    pub percent: u32,
}

/// All pairs of `bodies` with a similarity of at least `minimum_percent`, ordered by
/// `first`, then `second`. Bodies without fingerprints are never part of a pair.
///
/// A hash that more than `max_sharing` bodies share (getters, constructors, boilerplate) says
/// little about any pair and would cost the square of that number, so it does not nominate pairs.
/// A pair is still measured over all its hashes; it is only missed if the hashes it shares
/// beyond the skipped ones are too few to nominate it at all.
#[must_use]
pub fn similar_pairs(
    bodies: &[&Fingerprints],
    minimum_percent: u32,
    max_sharing: usize,
) -> Vec<SimilarPair> {
    let sets: Vec<_> = bodies.iter().map(|body| body.hashes()).collect();
    let mut pairs: Vec<SimilarPair> = candidates(&sets, max_sharing)
        .into_iter()
        .filter_map(|(first, second)| measure(&sets, (first, second)))
        .filter(|pair| pair.percent >= minimum_percent)
        .collect();
    pairs.sort_by_key(|pair| (pair.first, pair.second));
    pairs
}

/// The pairs that share at least one hash of at most `max_sharing` owners.
fn candidates(sets: &[HashSet<u64>], max_sharing: usize) -> HashSet<(usize, usize)> {
    let mut owners: HashMap<u64, Vec<usize>> = HashMap::new();
    for (index, set) in sets.iter().enumerate() {
        for hash in set {
            owners.entry(*hash).or_default().push(index);
        }
    }
    let mut found = HashSet::new();
    for indexes in owners.values().filter(|owners| owners.len() <= max_sharing) {
        for (at, first) in indexes.iter().enumerate() {
            for second in indexes.iter().skip(at + 1) {
                found.insert((*first.min(second), *first.max(second)));
            }
        }
    }
    found
}

/// The similarity of two bodies over all their hashes.
fn measure(sets: &[HashSet<u64>], (first, second): (usize, usize)) -> Option<SimilarPair> {
    let (one, two) = (sets.get(first)?, sets.get(second)?);
    let common = one.intersection(two).count();
    let total = one.len() + two.len() - common;
    let percent = common.saturating_mul(PERCENT_BASE) / total;
    Some(SimilarPair {
        first,
        second,
        similarity: ratio(common, total),
        percent: u32::try_from(percent).ok()?,
    })
}
