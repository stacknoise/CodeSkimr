//! The usages that the changes of a comparison hit, read in the base scan (ADR 0097, decision 4).

use std::collections::HashSet;

use codeskimr_domain::{Comparison, Direction, SnapshotId, SymbolId, UsageQuery};

use super::CompareLimits;
use super::report::{Impact, ImpactUsage, Listed, SymbolImpact, UsageStatus};
use crate::error::CodedError;
use crate::ports::{UsageEntry, UsageIndex};

/// Only the references that point at the symbol itself are known and listed.
const DIRECT: u32 = 1;

/// Adapted if the symbol around the usage is touched itself, else on the checklist.
fn dress(entry: UsageEntry, touched: &HashSet<SymbolId>) -> ImpactUsage {
    let status = if touched.contains(&entry.from.id) {
        UsageStatus::Adapted
    } else {
        UsageStatus::NotTouched
    };
    ImpactUsage {
        from: entry.from,
        path: entry.path,
        reference: entry.reference,
        status,
    }
}

/// The direct usages of the symbols whose usages a change can break, read in `base`.
///
/// # Errors
/// A coded failure of the `STO` group.
pub(super) fn read(
    usages: &dyn UsageIndex,
    base: SnapshotId,
    comparison: &Comparison,
    limits: &CompareLimits,
) -> Result<Impact, CodedError> {
    let touched = comparison.touched_in_base();
    let subjects = comparison.impact_subjects();
    let total = subjects.len();
    let mut symbols = Vec::new();
    for subject in subjects.into_iter().take(limits.max_symbols) {
        let query = UsageQuery {
            symbol: subject.symbol.id,
            direction: Direction::Incoming,
            depth: DIRECT,
            include_tests: true,
        };
        let page = usages.usages(base, &query, limits.max_usages)?;
        let items = page
            .entries
            .into_iter()
            .map(|entry| dress(entry, &touched))
            .collect();
        symbols.push(SymbolImpact {
            subject,
            usages: Listed {
                total: page.total,
                items,
            },
        });
    }
    Ok(Impact {
        symbols: Listed {
            total,
            items: symbols,
        },
    })
}
