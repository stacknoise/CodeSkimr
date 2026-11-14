//! Local functions: `const has = (id) => ...` inside another function. An indexer gives such a
//! name a local symbol (`local 12`), which has no name to look up, so the definition is matched
//! to the project symbol by its place. Only one shape is certain enough: the function starts
//! behind its name on the same line, and the name is the last definition on that line in front
//! of it and as long as the name of the symbol.

use std::collections::HashMap;

use codeskimr_domain::{Position, Symbol, SymbolId};

use crate::model::{Occurrence, SymbolRef};

/// The definition of a one-line name that ends in front of `start`, if it is the last such
/// definition on that line and has the length of `name`.
fn name_before<'a>(defs: &[&'a Occurrence], start: Position, name: &str) -> Option<&'a Occurrence> {
    let last = defs
        .iter()
        .filter(|def| def.range.end().line == start.line && def.range.end() <= start)
        .max_by_key(|def| def.range.end())?;
    let one_line = last.range.start().line == last.range.end().line;
    let width = last
        .range
        .end()
        .column
        .checked_sub(last.range.start().column)?;
    let length = u32::try_from(name.chars().count()).ok()?;
    (one_line && width == length).then_some(*last)
}

/// The local symbols of a document that are the local functions among `symbols` (those of the
/// file), with their targets. A definition that two functions claim is left out.
pub(crate) fn match_functions(
    symbols: &[&Symbol],
    defs: &[&Occurrence],
) -> HashMap<SymbolRef, Vec<SymbolId>> {
    let mut found: HashMap<SymbolRef, Vec<SymbolId>> = HashMap::new();
    for symbol in symbols.iter().filter(|symbol| symbol.kind.is_callable()) {
        if let Some(def) = name_before(defs, symbol.span.start(), symbol.name.as_str()) {
            found.entry(def.symbol).or_default().push(symbol.id);
        }
    }
    found.retain(|_, targets| targets.len() == 1);
    found
}
