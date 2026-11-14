//! `CHK-102` against the stored project: a function that copies code outside the checked files.

use std::collections::HashSet;

use codeskimr_codes::ErrorCode;
use codeskimr_domain::RelativePath;

use super::duplicates::Fingerprinted;
use super::model::{Detail, Finding, Location, StoredProject};
use crate::consts::STORED_DUPLICATE_HITS;
use crate::error::CodedError;
use crate::ports::{SimilarHit, SimilarQuery};

/// What decides which functions are compared and which stored hits count.
pub(super) struct Rules<'a> {
    pub minimum_percent: u32,
    pub minimum_lines: u32,
    /// Files that are checked now: their stored versions are older and must not match.
    pub checked: &'a HashSet<RelativePath>,
    /// Whether a stored file is a test; tests are no original to copy from.
    pub is_test: &'a dyn Fn(&RelativePath) -> bool,
}

/// For every function of `items` that is long enough and not reported yet (`reported` holds the
/// locations that are), the most similar stored function outside the checked files.
///
/// # Errors
/// A coded failure of the `STO` group.
pub(super) fn against_stored(
    items: &[Fingerprinted<'_>],
    stored: &StoredProject<'_>,
    rules: &Rules<'_>,
    reported: &[Finding],
) -> Result<Vec<Finding>, CodedError> {
    let mut found = Vec::new();
    for item in items {
        let place = &item.callable.location;
        let compared = item.callable.lines >= rules.minimum_lines;
        if !compared || reported.iter().any(|finding| finding.location == *place) {
            continue;
        }
        if let Some(hit) = original(item, stored, rules)? {
            found.push(finding(place, hit));
        }
    }
    Ok(found)
}

/// The most similar stored function that counts as an original of `item`.
fn original(
    item: &Fingerprinted<'_>,
    stored: &StoredProject<'_>,
    rules: &Rules<'_>,
) -> Result<Option<SimilarHit>, CodedError> {
    let hits = stored.index.similar(
        stored.snapshot,
        &SimilarQuery {
            fingerprints: item.fingerprints,
            minimum_percent: rules.minimum_percent,
            exclude: None,
            limit: STORED_DUPLICATE_HITS,
        },
    )?;
    Ok(hits
        .into_iter()
        .find(|hit| !rules.checked.contains(&hit.path) && !(rules.is_test)(&hit.path)))
}

fn finding(place: &Location, hit: SimilarHit) -> Finding {
    Finding {
        code: ErrorCode::DuplicateBlock,
        location: place.clone(),
        detail: Detail::Duplicate {
            of: Location {
                path: hit.path,
                span: hit.span,
                symbol: hit.qualified_name,
            },
            similarity_percent: hit.percent,
        },
    }
}
