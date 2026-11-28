//! The filter of a comparison: which symbols of the difference a surface wants to see (#826).
//!
//! The filter is applied to the difference before any list is cut to its limit, so a search finds
//! every hit and not only those in the first entries of a long list.

use crate::compare::{Comparison, Matched, SymbolFact};
use crate::consts::{CAUSE_ADDED, CAUSE_CHANGED, CAUSE_MOVED, CAUSE_REMOVED};
use crate::symbol::{SymbolKind, Visibility};
use crate::vocabulary::labeled_enum;

labeled_enum!(
    /// What happened to a symbol between the two scans.
    ChangeCause {
        /// Only the target has it.
        Added => CAUSE_ADDED,
        /// Its signature, visibility or content changed.
        Changed => CAUSE_CHANGED,
        /// Only the base has it.
        Removed => CAUSE_REMOVED,
        /// It has the same key in another file.
        Moved => CAUSE_MOVED,
    }
);

/// Which symbols of a difference to keep. A field left empty lets everything pass.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ChangeFilter {
    /// Text that the qualified name contains, ignoring case; empty lets every name pass.
    pub name: String,
    /// Only symbols to which this happened.
    pub cause: Option<ChangeCause>,
    /// Only symbols of this kind.
    pub kind: Option<SymbolKind>,
    /// Only symbols that are visible everywhere.
    pub public_only: bool,
    /// Only the symbols with places that no change touches (the checklist); counts in the list of
    /// the usages, not in the lists of the symbols.
    pub only_open: bool,
}

impl ChangeFilter {
    fn keeps(&self, fact: &SymbolFact) -> bool {
        let name = self.name.trim().to_lowercase();
        (name.is_empty() || fact.qualified_name.as_str().to_lowercase().contains(&name))
            && self.kind.is_none_or(|kind| fact.kind == kind)
            && (!self.public_only || fact.visibility == Some(Visibility::Public))
    }

    fn wants(&self, cause: ChangeCause) -> bool {
        self.cause.is_none_or(|wanted| wanted == cause)
    }

    fn keeps_pair(&self, matched: &Matched) -> bool {
        self.keeps(&matched.target)
    }
}

impl Comparison {
    /// The part of the difference the filter keeps, in the same order. The usages that count as
    /// adapted are still found with the whole difference ([`Comparison::touched_in_base`]).
    #[must_use]
    pub fn filtered(&self, filter: &ChangeFilter) -> Self {
        let facts = |list: &[SymbolFact], cause| {
            if filter.wants(cause) {
                list.iter()
                    .filter(|fact| filter.keeps(fact))
                    .cloned()
                    .collect()
            } else {
                Vec::new()
            }
        };
        let pairs = |list: &[Matched], cause| {
            if filter.wants(cause) {
                list.iter()
                    .filter(|matched| filter.keeps_pair(matched))
                    .cloned()
                    .collect()
            } else {
                Vec::new()
            }
        };
        Self {
            added: facts(&self.added, ChangeCause::Added),
            removed: facts(&self.removed, ChangeCause::Removed),
            changed: pairs(&self.changed, ChangeCause::Changed),
            moved: pairs(&self.moved, ChangeCause::Moved),
        }
    }
}
