//! `CHK-102`: functions that copy another function (R-01).

use std::collections::HashMap;

use codeskimr_codes::ErrorCode;
use codeskimr_domain::{Fingerprints, similar_pairs};

use super::length::Callable;
use super::model::{Detail, Finding};

/// A callable together with its fingerprints.
pub(super) struct Fingerprinted<'a> {
    pub callable: &'a Callable,
    pub fingerprints: &'a Fingerprints,
}

/// Reports the later one of every similar pair, in path and line order, with the earlier one it
/// copies. A function reports at most one finding: the copy with the highest similarity, the
/// earliest on a tie.
pub(super) fn duplicates(items: &[Fingerprinted<'_>], minimum_percent: u32) -> Vec<Finding> {
    let mut ordered: Vec<&Fingerprinted<'_>> = items.iter().collect();
    ordered.sort_by_key(|item| {
        (
            item.callable.location.path.clone(),
            item.callable.location.span.start(),
        )
    });
    let bodies: Vec<&Fingerprints> = ordered.iter().map(|item| item.fingerprints).collect();
    let mut best: HashMap<usize, (usize, u32)> = HashMap::new();
    for pair in similar_pairs(&bodies, minimum_percent) {
        let better = best
            .get(&pair.second)
            .is_none_or(|(_, percent)| pair.percent > *percent);
        if better {
            best.insert(pair.second, (pair.first, pair.percent));
        }
    }
    let mut found: Vec<(usize, Finding)> = best
        .into_iter()
        .filter_map(|(later, (earlier, percent))| {
            let copy = ordered.get(later)?;
            let original = ordered.get(earlier)?;
            Some((
                later,
                Finding {
                    code: ErrorCode::DuplicateBlock,
                    location: copy.callable.location.clone(),
                    detail: Detail::Duplicate {
                        of: original.callable.location.clone(),
                        similarity_percent: percent,
                    },
                },
            ))
        })
        .collect();
    found.sort_by_key(|(later, _)| *later);
    found.into_iter().map(|(_, finding)| finding).collect()
}
