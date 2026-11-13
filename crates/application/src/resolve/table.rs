//! The global symbol table: every symbol of the snapshot, indexed for lookups.

use std::collections::HashMap;

use codeskimr_domain::{Symbol, SymbolId};

/// All symbols of one resolution run, by id, by simple name and by qualified name.
pub struct SymbolTable<'a> {
    ids: HashMap<SymbolId, &'a Symbol>,
    names: HashMap<&'a str, Vec<&'a Symbol>>,
    qualified_names: HashMap<&'a str, Vec<&'a Symbol>>,
}

impl<'a> SymbolTable<'a> {
    /// Indexes `symbols`.
    pub fn new(symbols: &'a [Symbol]) -> Self {
        let mut table = Self {
            ids: HashMap::with_capacity(symbols.len()),
            names: HashMap::new(),
            qualified_names: HashMap::new(),
        };
        for symbol in symbols {
            table.ids.insert(symbol.id, symbol);
            table
                .names
                .entry(symbol.name.as_str())
                .or_default()
                .push(symbol);
            table
                .qualified_names
                .entry(symbol.qualified_name.as_str())
                .or_default()
                .push(symbol);
        }
        table
    }

    /// The symbol with this id.
    pub fn get(&self, id: SymbolId) -> Option<&'a Symbol> {
        self.ids.get(&id).copied()
    }

    /// Symbols with this simple name.
    pub fn named(&self, name: &str) -> &[&'a Symbol] {
        self.names.get(name).map_or(&[], Vec::as_slice)
    }

    /// Symbols with this qualified name; several if the name is overloaded or a namespace is
    /// declared in several files.
    pub fn qualified(&self, name: &str) -> &[&'a Symbol] {
        self.qualified_names.get(name).map_or(&[], Vec::as_slice)
    }

    /// The symbol itself, then its parent, its grandparent and so on up to the top.
    pub fn ancestors(&self, id: SymbolId) -> impl Iterator<Item = &'a Symbol> + '_ {
        std::iter::successors(self.get(id), |symbol| {
            symbol.parent.and_then(|parent| self.get(parent))
        })
        .take(self.ids.len())
    }
}
