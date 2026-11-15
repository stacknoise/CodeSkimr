//! What the resolver can see of a set of files, without the ids of the symbols.
//!
//! The resolver reads of a symbol its name, kind, qualified name and parent, and of a type its base
//! list. If the files that changed show the same [`Shape`] as before, nothing that an unchanged
//! file could resolve against has changed, except the ids of the symbols (issue #49). The
//! incremental scan then resolves again only the names that pointed at one of those ids, and
//! otherwise all names.

use std::collections::HashMap;

use codeskimr_domain::{ReferenceKind, Symbol, SymbolId};

use crate::ports::PendingReference;

/// One symbol as the resolver sees it: qualified name, kind label, and the parent's qualified name
/// and kind label.
type SymbolKey = (String, &'static str, Option<(String, &'static str)>);

/// One base-list entry: qualified name of the type, kind label, the base name as written.
type BaseKey = (String, &'static str, String);

/// Counts of what the resolver reads in some files.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Shape {
    symbols: HashMap<SymbolKey, usize>,
    bases: HashMap<BaseKey, usize>,
}

impl Shape {
    /// The shape of `symbols` with the base lists among `names`.
    #[must_use]
    pub fn of(symbols: &[Symbol], names: &[PendingReference]) -> Self {
        let by_id: HashMap<SymbolId, &Symbol> =
            symbols.iter().map(|symbol| (symbol.id, symbol)).collect();
        let mut shape = Self::default();
        for symbol in symbols {
            let parent = symbol.parent.and_then(|id| by_id.get(&id)).map(|parent| {
                (
                    parent.qualified_name.as_str().to_owned(),
                    parent.kind.label(),
                )
            });
            *shape
                .symbols
                .entry((
                    symbol.qualified_name.as_str().to_owned(),
                    symbol.kind.label(),
                    parent,
                ))
                .or_default() += 1;
        }
        for name in names.iter().filter(|name| is_base(name.kind)) {
            if let Some(owner) = by_id.get(&name.from) {
                *shape
                    .bases
                    .entry((
                        owner.qualified_name.as_str().to_owned(),
                        name.kind.label(),
                        name.target.as_str().to_owned(),
                    ))
                    .or_default() += 1;
            }
        }
        shape
    }
}

/// Whether a name tells the base list of a type.
#[must_use]
pub const fn is_base(kind: ReferenceKind) -> bool {
    matches!(kind, ReferenceKind::Inherit | ReferenceKind::Implement)
}

/// Whether a name is an import or tells the base list of a type: what the resolver learns from.
#[must_use]
pub const fn is_structural(kind: ReferenceKind) -> bool {
    is_base(kind) || matches!(kind, ReferenceKind::Import)
}
