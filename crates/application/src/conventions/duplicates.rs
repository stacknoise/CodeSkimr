//! `CHK-102`: functions that copy another function (R-01).

use std::collections::HashMap;

use codeskimr_codes::ErrorCode;
use codeskimr_domain::{Fingerprints, similar_pairs};

use super::length::Callable;
use super::model::{Detail, Finding, Location};

/// A callable together with its fingerprints.
pub(super) struct Fingerprinted<'a> {
    pub callable: &'a Callable,
    pub fingerprints: &'a Fingerprints,
}

/// Reports the later one of every similar pair, in path and line order, with the earlier one it
/// copies. A function reports at most one finding: the copy with the highest similarity, the
/// earliest on a tie. Functions with fewer than `minimum_lines` code lines are not compared.
pub(super) fn duplicates(
    items: &[Fingerprinted<'_>],
    (minimum_percent, minimum_lines): (u32, u32),
    max_owners: u32,
) -> Vec<Finding> {
    let mut ordered: Vec<&Fingerprinted<'_>> = items
        .iter()
        .filter(|item| item.callable.lines >= minimum_lines)
        .collect();
    ordered.sort_by_key(|item| {
        (
            item.callable.location.path.clone(),
            item.callable.location.span.start(),
        )
    });
    let mut found: Vec<(usize, Finding)> = best_originals(&ordered, minimum_percent, max_owners)
        .into_iter()
        .filter_map(|(later, (earlier, percent))| {
            let copy = ordered.get(later)?;
            let original = ordered.get(earlier)?;
            Some((later, finding(copy, original, percent)))
        })
        .collect();
    found.sort_by_key(|(later, _)| *later);
    found.into_iter().map(|(_, finding)| finding).collect()
}

/// For every function that copies an earlier one: the index of the most similar earlier function
/// and the similarity, the earliest on a tie.
fn best_originals(
    ordered: &[&Fingerprinted<'_>],
    minimum_percent: u32,
    max_owners: u32,
) -> HashMap<usize, (usize, u32)> {
    let bodies: Vec<&Fingerprints> = ordered.iter().map(|item| item.fingerprints).collect();
    let mut best: HashMap<usize, (usize, u32)> = HashMap::new();
    let owners = usize::try_from(max_owners).unwrap_or(usize::MAX);
    for pair in similar_pairs(&bodies, minimum_percent, owners) {
        if nested(ordered.get(pair.first), ordered.get(pair.second)) {
            continue;
        }
        let better = best
            .get(&pair.second)
            .is_none_or(|(_, percent)| pair.percent > *percent);
        if better {
            best.insert(pair.second, (pair.first, pair.percent));
        }
    }
    best
}

/// The finding that says `copy` resembles `original`.
fn finding(copy: &Fingerprinted<'_>, original: &Fingerprinted<'_>, percent: u32) -> Finding {
    Finding {
        code: ErrorCode::DuplicateBlock,
        location: copy.callable.location.clone(),
        detail: Detail::Duplicate {
            of: original.callable.location.clone(),
            similarity_percent: percent,
        },
    }
}

/// Whether one function lies inside the other: an outer function always resembles its own part.
fn nested(first: Option<&&Fingerprinted<'_>>, second: Option<&&Fingerprinted<'_>>) -> bool {
    let (Some(first), Some(second)) = (first, second) else {
        return false;
    };
    let (one, two) = (&first.callable.location, &second.callable.location);
    let inside = |outer: &Location, inner: &Location| {
        outer.path == inner.path
            && outer.span.start() <= inner.span.start()
            && inner.span.end() <= outer.span.end()
    };
    inside(one, two) || inside(two, one)
}
