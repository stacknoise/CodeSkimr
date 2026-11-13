//! What the resolver knows while it runs: the symbol table, the imports of every file and the
//! type hierarchy found so far.

use std::collections::{HashMap, HashSet};

use codeskimr_domain::{FileId, ReferenceKind};

use super::hierarchy::Hierarchy;
use super::kinds::is_type;
use super::lookup::Hit;
use super::table::SymbolTable;
use crate::ports::PendingReference;

/// The state of one resolution run.
pub struct Context<'a> {
    pub(super) table: SymbolTable<'a>,
    /// Qualified names of the namespaces each file imports.
    pub(super) imports: HashMap<FileId, HashSet<String>>,
    pub(super) hierarchy: Hierarchy,
}

impl<'a> Context<'a> {
    /// Starts with an empty state over `table`.
    pub fn new(table: SymbolTable<'a>) -> Self {
        Self {
            table,
            imports: HashMap::new(),
            hierarchy: Hierarchy::default(),
        }
    }

    /// Takes in what a resolved occurrence tells: an import widens what its file sees, a base
    /// type widens what its type inherits.
    pub fn learn(&mut self, occurrence: &PendingReference, hits: &[Hit]) {
        match occurrence.kind {
            ReferenceKind::Import => self.learn_imports(occurrence, hits),
            ReferenceKind::Inherit | ReferenceKind::Implement => {
                self.learn_bases(occurrence, hits);
            }
            _ => {}
        }
    }

    fn learn_imports(&mut self, occurrence: &PendingReference, hits: &[Hit]) {
        let names = hits
            .iter()
            .filter_map(|hit| self.table.get(hit.to))
            .map(|symbol| symbol.qualified_name.as_str().to_owned());
        self.imports
            .entry(occurrence.file)
            .or_default()
            .extend(names);
    }

    fn learn_bases(&mut self, occurrence: &PendingReference, hits: &[Hit]) {
        let from_is_type = self
            .table
            .get(occurrence.from)
            .is_some_and(|symbol| is_type(symbol.kind));
        if from_is_type {
            for hit in hits {
                self.hierarchy.add(occurrence.from, hit.to);
            }
        }
    }
}
