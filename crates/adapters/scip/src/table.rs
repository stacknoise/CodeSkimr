//! Looks up project symbols by file and position.

use std::cmp::Reverse;
use std::collections::HashMap;

use codeskimr_domain::{FileId, Position, Span, Symbol, SymbolId, SymbolKind};

use crate::consts::{CONSTRUCTOR_NAMES, NAME_SEGMENT_SEPARATOR};
use crate::symbol::{Descriptor, DescriptorKind};

/// The symbols of a snapshot, grouped by file.
pub(crate) struct SymbolTable<'a> {
    by_file: HashMap<FileId, Vec<&'a Symbol>>,
}

fn contains(span: Span, at: Position) -> bool {
    span.start() <= at && at < span.end()
}

/// Whether the project symbol bears the name the SCIP symbol gives. A qualified name written as
/// one escaped descriptor (explicit interface implementation) matches by its last segment, and
/// constructors match the constructor names of the indexers, as the project names them like
/// their type.
fn name_matches(symbol: &Symbol, scip_name: &str) -> bool {
    let name = symbol.name.as_str();
    name == scip_name
        || scip_name.rsplit(NAME_SEGMENT_SEPARATOR).next() == Some(name)
        || (symbol.kind == SymbolKind::Constructor && CONSTRUCTOR_NAMES.contains(&scip_name))
}

/// Whether the kind of a project symbol fits the suffix of the descriptor. Only a tie-break:
/// indexers differ (a Rust enum variant is a type descriptor, a project enum member is not).
const fn kind_fits(kind: SymbolKind, descriptor: DescriptorKind) -> bool {
    match descriptor {
        DescriptorKind::Method => kind.is_callable(),
        DescriptorKind::Type => kind.is_type() || matches!(kind, SymbolKind::TypeAlias),
        DescriptorKind::Term => !kind.is_callable() && !kind.is_type(),
        DescriptorKind::Namespace => matches!(kind, SymbolKind::Module | SymbolKind::Namespace),
        _ => false,
    }
}

/// Smaller spans win: the one that starts last, and among equal starts the one that ends first.
fn tightness(symbol: &Symbol) -> (Position, Reverse<Position>) {
    (symbol.span.start(), Reverse(symbol.span.end()))
}

impl<'a> SymbolTable<'a> {
    /// Groups `symbols` by file.
    pub(crate) fn new(symbols: &'a [Symbol]) -> Self {
        let mut by_file: HashMap<FileId, Vec<&Symbol>> = HashMap::new();
        for symbol in symbols {
            by_file.entry(symbol.file).or_default().push(symbol);
        }
        Self { by_file }
    }

    fn enclosing(&self, file: FileId, at: Position) -> impl Iterator<Item = &'a Symbol> {
        self.by_file
            .get(&file)
            .into_iter()
            .flatten()
            .copied()
            .filter(move |symbol| contains(symbol.span, at))
    }

    /// The smallest symbol of the file whose span contains `at`.
    pub(crate) fn innermost(&self, file: FileId, at: Position) -> Option<&'a Symbol> {
        self.enclosing(file, at)
            .max_by_key(|symbol| tightness(symbol))
    }

    /// The project symbol that a SCIP definition at `at` stands for: the smallest symbol of the
    /// file that contains the position and bears the name of the descriptor; among equal spans
    /// the one whose kind fits the descriptor, then the lowest id.
    pub(crate) fn definition(
        &self,
        file: FileId,
        at: Position,
        descriptor: &Descriptor,
    ) -> Option<SymbolId> {
        self.enclosing(file, at)
            .filter(|symbol| name_matches(symbol, &descriptor.name))
            .max_by_key(|symbol| {
                (
                    tightness(symbol),
                    kind_fits(symbol.kind, descriptor.kind),
                    Reverse(symbol.id),
                )
            })
            .map(|symbol| symbol.id)
    }
}
