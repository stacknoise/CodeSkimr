//! The usages that the changes of a comparison hit, read in the base scan (ADR 0097, decision 4).

use std::collections::HashSet;

use codeskimr_domain::{Comparison, Direction, ImpactSubject, SnapshotId, SymbolId, UsageQuery};

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

/// The direct usages of one subject in `base`, as adapted or not touched.
fn usages_of(
    usages: &dyn UsageIndex,
    base: SnapshotId,
    (subject, touched): (&ImpactSubject, &HashSet<SymbolId>),
    max_usages: u32,
) -> Result<Listed<ImpactUsage>, CodedError> {
    let query = UsageQuery {
        symbol: subject.symbol.id,
        direction: Direction::Incoming,
        depth: DIRECT,
        include_tests: true,
    };
    let page = usages.usages(base, &query, max_usages)?;
    Ok(Listed {
        total: page.total,
        items: page
            .entries
            .into_iter()
            .map(|entry| dress(entry, touched))
            .collect(),
    })
}

/// Whether a list of usages has a place that no change touches: the symbol is on the checklist.
fn has_open_place(listed: &Listed<ImpactUsage>) -> bool {
    listed
        .items
        .iter()
        .any(|usage| usage.status == UsageStatus::NotTouched)
}

/// The direct usages of the symbols whose usages a change can break, read in `base`. The symbols
/// are those of `kept`, the part of `whole` that the filter kept; `whole` tells which usages count
/// as adapted. With `only_open` a symbol without a place on the checklist is left out and the
/// total counts only the symbols that stay, so every symbol is read before the cut. Without it only
/// the symbols that are listed are read.
///
/// # Errors
/// A coded failure of the `STO` group.
pub(super) fn read(
    usages: &dyn UsageIndex,
    base: SnapshotId,
    (whole, kept): (&Comparison, &Comparison),
    (limits, only_open): (&CompareLimits, bool),
) -> Result<Impact, CodedError> {
    let touched = whole.touched_in_base();
    let mut total = 0;
    let mut symbols = Vec::new();
    for subject in kept.impact_subjects() {
        let listing = symbols.len() < limits.max_symbols;
        if !listing && !only_open {
            total += 1;
            continue;
        }
        let listed = usages_of(usages, base, (&subject, &touched), limits.max_usages)?;
        if only_open && !has_open_place(&listed) {
            continue;
        }
        total += 1;
        if listing {
            symbols.push(SymbolImpact {
                subject,
                usages: listed,
            });
        }
    }
    Ok(Impact {
        symbols: Listed {
            total,
            items: symbols,
        },
    })
}
