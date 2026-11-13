//! Finding the similar pairs in a set of fingerprinted bodies.
//!
//! The duplicate check of `check_conventions` and `find_similar_code` share this measure with
//! [`Fingerprints::similarity`] (blueprint 10.4: one implementation). Comparing every pair would
//! cost the square of the number of bodies, so the pairs come from an inverted index of the
//! hashes: only bodies that share at least one hash are ever counted together.

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
#[must_use]
pub fn similar_pairs(bodies: &[&Fingerprints], minimum_percent: u32) -> Vec<SimilarPair> {
    let sets: Vec<_> = bodies.iter().map(|body| body.hashes()).collect();
    let mut owners: HashMap<u64, Vec<usize>> = HashMap::new();
    for (index, set) in sets.iter().enumerate() {
        for hash in set {
            owners.entry(*hash).or_default().push(index);
        }
    }
    let mut shared: HashMap<(usize, usize), usize> = HashMap::new();
    for indexes in owners.values() {
        for (at, first) in indexes.iter().enumerate() {
            for second in indexes.iter().skip(at + 1) {
                *shared.entry((*first, *second)).or_default() += 1;
            }
        }
    }
    let size = |index: usize| sets.get(index).map_or(0, HashSet::len);
    let mut pairs: Vec<SimilarPair> = shared
        .into_iter()
        .filter_map(|((first, second), common)| {
            let total = size(first) + size(second) - common;
            let percent = common.saturating_mul(PERCENT_BASE) / total;
            Some(SimilarPair {
                first,
                second,
                similarity: ratio(common, total),
                percent: u32::try_from(percent).ok()?,
            })
        })
        .filter(|pair| pair.percent >= minimum_percent)
        .collect();
    pairs.sort_by_key(|pair| (pair.first, pair.second));
    pairs
}
