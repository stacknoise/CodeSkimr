//! Moving the ids of what an import created to ids that are free (ADR 0055).

use codeskimr_domain::{Reference, ReferenceId, Symbol, SymbolId};

/// The created `symbols` and the `references` with new ids: the symbols are numbered from
/// `first_symbol`, the references from `first_reference`. The importer numbers the symbols
/// consecutively from the first free id of the snapshot; every id from there on, as a symbol,
/// parent or target, moves by the same distance.
#[must_use]
pub fn renumbered<'a>(
    symbols: &[Symbol],
    references: impl Iterator<Item = &'a Reference>,
    first_symbol: u64,
    first_reference: u64,
) -> (Vec<Symbol>, Vec<Reference>) {
    let old_first = symbols.iter().map(|s| s.id.get()).min();
    let moved = |id: SymbolId| match old_first {
        Some(old) if id.get() >= old => SymbolId::new(first_symbol.saturating_add(id.get() - old)),
        _ => id,
    };
    let symbols = symbols
        .iter()
        .map(|symbol| Symbol {
            id: moved(symbol.id),
            parent: symbol.parent.map(moved),
            ..symbol.clone()
        })
        .collect();
    let references = references
        .zip(first_reference..)
        .map(|(reference, id)| Reference {
            id: ReferenceId::new(id),
            from: moved(reference.from),
            to: moved(reference.to),
            ..*reference
        })
        .collect();
    (symbols, references)
}
