//! Looks up project symbols by file and position.

use std::cmp::Reverse;
use std::collections::HashMap;

use codeskimr_domain::{FileId, Position, Span, Symbol, SymbolId, SymbolKind};

use crate::consts::{
    ACCESSOR_PREFIXES, CONSTRUCTOR_NAMES, MAX_LINE_CANDIDATES, NAME_SEGMENT_SEPARATOR,
};
use crate::symbol::{Descriptor, DescriptorKind};

/// The symbols of a snapshot, grouped by file.
pub(crate) struct SymbolTable<'a> {
    by_file: HashMap<FileId, Vec<&'a Symbol>>,
    /// By file, line of the start and name; each list in the order of the starts.
    by_line_name: HashMap<(FileId, u32, &'a str), Vec<&'a Symbol>>,
}

/// The names under which a project symbol can bear the SCIP name (see `name_matches`), without
/// the constructor case, which has no declaration behind a name.
fn name_keys(scip_name: &str) -> Vec<&str> {
    let stripped = ACCESSOR_PREFIXES
        .iter()
        .find_map(|prefix| scip_name.strip_prefix(prefix))
        .unwrap_or(scip_name);
    let last = stripped
        .rsplit(NAME_SEGMENT_SEPARATOR)
        .next()
        .unwrap_or(stripped);
    let mut keys = vec![scip_name];
    for key in [stripped, last] {
        if !keys.contains(&key) {
            keys.push(key);
        }
    }
    keys
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
    let scip_name = ACCESSOR_PREFIXES
        .iter()
        .find_map(|prefix| scip_name.strip_prefix(prefix))
        .unwrap_or(scip_name);
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
        let mut by_line_name: HashMap<(FileId, u32, &str), Vec<&Symbol>> = HashMap::new();
        for symbol in symbols {
            let key = (symbol.file, symbol.span.start().line, symbol.name.as_str());
            by_line_name.entry(key).or_default().push(symbol);
        }
        for list in by_line_name.values_mut() {
            list.sort_by_key(|symbol| tightness(symbol));
        }
        Self {
            by_file,
            by_line_name,
        }
    }

    /// The symbols of the file that bear one of the names and start on `line`.
    fn named_on_line<'t>(
        &'t self,
        file: FileId,
        line: u32,
        descriptor: &'t Descriptor,
    ) -> impl Iterator<Item = &'t [&'a Symbol]> {
        name_keys(&descriptor.name)
            .into_iter()
            .filter_map(move |name| self.by_line_name.get(&(file, line, name)))
            .map(Vec::as_slice)
    }

    fn enclosing(&self, file: FileId, at: Position) -> impl Iterator<Item = &'a Symbol> {
        self.by_file
            .get(&file)
            .into_iter()
            .flatten()
            .copied()
            .filter(move |symbol| contains(symbol.span, at))
    }

    /// The symbols of one file.
    pub(crate) fn symbols_of(&self, file: FileId) -> &[&'a Symbol] {
        self.by_file.get(&file).map_or(&[], Vec::as_slice)
    }

    /// The smallest symbol of the file whose span contains `at`.
    pub(crate) fn innermost(&self, file: FileId, at: Position) -> Option<&'a Symbol> {
        self.enclosing(file, at)
            .max_by_key(|symbol| tightness(symbol))
    }

    /// The project symbol that a SCIP definition at `range` stands for: the smallest symbol of the
    /// file that contains the position and bears the name of the descriptor; among equal spans
    /// the one whose kind fits the descriptor, then the lowest id. If none contains it, a symbol
    /// whose span starts behind the name on the same line (`const f = () => ...`, whose span
    /// starts at the function and not at the name; the indexer calls it a variable) counts.
    pub(crate) fn definition(
        &self,
        file: FileId,
        range: Span,
        descriptor: &Descriptor,
    ) -> Option<SymbolId> {
        let at = range.start();
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
            .or_else(|| self.after_name(file, range, descriptor))
    }

    /// The symbol of that name that starts on the line of `range`, behind it. Several of them
    /// are not told apart: nothing is returned.
    fn after_name(&self, file: FileId, range: Span, descriptor: &Descriptor) -> Option<SymbolId> {
        let end = range.end();
        let mut behind = self
            .named_on_line(file, range.start().line, descriptor)
            .flat_map(|list| {
                list.iter()
                    .skip(list.partition_point(|s| s.span.start() < end))
            });
        let first = behind.next()?;
        behind.next().is_none().then_some(first.id)
    }

    /// The symbol of the file that is declared at `at`: its span starts on that line at or before
    /// the position, it is callable and bears the name. Only if exactly one does, and if no
    /// more than `MAX_LINE_CANDIDATES` symbols of that name start in front of the place.
    pub(crate) fn declared_at(
        &self,
        file: FileId,
        at: Position,
        descriptor: &Descriptor,
    ) -> Option<&'a Symbol> {
        let mut found = None;
        let mut weighed = 0_usize;
        for list in self.named_on_line(file, at.line, descriptor) {
            let count = list.partition_point(|s| s.span.start() <= at);
            for symbol in list.iter().take(count).rev() {
                weighed += 1;
                if weighed > MAX_LINE_CANDIDATES {
                    return None;
                }
                if contains(symbol.span, at) && symbol.kind.is_callable() {
                    if found.is_some() {
                        return None;
                    }
                    found = Some(*symbol);
                }
            }
        }
        found
    }
}
