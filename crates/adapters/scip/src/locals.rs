//! Local functions: `const has = (id) => ...` inside another function. An indexer gives such a
//! name a local symbol (`local 12`), which has no name to look up, so the definition is matched
//! to the project symbol by its place and its name. Only one shape is certain enough: the
//! function starts behind its name on the same line, the name is the last definition on that line
//! in front of it, and the text of the definition reads exactly as the name of the function. The
//! text is known only where the columns are bytes (see `convert_columns`); without it nothing
//! is mapped.

use std::collections::HashMap;

use codeskimr_domain::{Symbol, SymbolId};

use crate::model::{Occurrence, SymbolRef};

/// The one-line definitions of a document by the line they are on, in the order of their ends.
fn by_line<'a>(defs: &[&'a Occurrence]) -> HashMap<u32, Vec<&'a Occurrence>> {
    let mut lines: HashMap<u32, Vec<&Occurrence>> = HashMap::new();
    for def in defs {
        if def.range.start().line == def.range.end().line {
            lines.entry(def.range.end().line).or_default().push(def);
        }
    }
    for list in lines.values_mut() {
        list.sort_by_key(|def| def.range.end());
    }
    lines
}

/// The local symbols of a document that are the local functions among `symbols` (those of the
/// file), with their targets. `names` reads what each local symbol is called. A definition that
/// two functions claim is left out.
pub(crate) fn match_functions(
    symbols: &[&Symbol],
    defs: &[&Occurrence],
    names: &HashMap<SymbolRef, String>,
) -> HashMap<SymbolRef, Vec<SymbolId>> {
    let lines = by_line(defs);
    let mut found: HashMap<SymbolRef, Vec<SymbolId>> = HashMap::new();
    for symbol in symbols.iter().filter(|symbol| symbol.kind.is_callable()) {
        let start = symbol.span.start();
        let Some(list) = lines.get(&start.line) else {
            continue;
        };
        let before = list.partition_point(|def| def.range.end() <= start);
        let nearest = before.checked_sub(1).and_then(|at| list.get(at));
        if let Some(def) = nearest
            .filter(|def| names.get(&def.symbol).map(String::as_str) == Some(symbol.name.as_str()))
        {
            found.entry(def.symbol).or_default().push(symbol.id);
        }
    }
    found.retain(|_, targets| targets.len() == 1);
    found
}
