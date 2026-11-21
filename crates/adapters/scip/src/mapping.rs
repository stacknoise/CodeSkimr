//! Maps SCIP occurrences onto project symbols and replaces the level 1 references of the files
//! the index covers (ADR 0033, decisions 2 and 3). A pure function over the decoded index and
//! the snapshot: no input or output, no state.

use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use codeskimr_domain::{
    Confidence, FileId, Position, Reference, ReferenceId, ReferenceKind, SourceFile, Span, Symbol,
    SymbolId, SymbolKind, SymbolOrigin,
};

use crate::consts::{
    MAX_OVERLOAD_TARGETS, MAX_SYMBOLS_PER_KEY, MAX_TARGETS_PER_SYMBOL, NO_OWNER, SUFFIX_TYPE,
};
use crate::create::{Creation, created_symbols};
use crate::error::ScipError;
use crate::fitting::Fitting;
use crate::impl_symbol::{ImplSymbol, TypeName, parse as parse_impl};
use crate::interval::IntervalIndex;
use crate::locals::match_functions;
use crate::model::{Occurrence, ScipDocument, ScipIndex, SymbolRef};
use crate::roots::{Pick, Roots};
use crate::symbol::{Descriptor, DescriptorKind, is_local, last_descriptor, last_two, namespaces};
use crate::table::SymbolTable;

mod implicit;

/// The parts of a snapshot the import reads.
#[derive(Debug, Clone, Copy)]
pub struct SnapshotView<'a> {
    /// The files of the language the index was made for.
    pub files: &'a [SourceFile],
    /// All symbols of the snapshot.
    pub symbols: &'a [Symbol],
    /// All references of the snapshot, level 1 so far.
    pub references: &'a [Reference],
}

/// What the import counted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ImportStats {
    /// Occurrences that became references.
    pub mapped: u64,
    /// Occurrences of symbols that the index does not define, which are the symbols of libraries.
    /// They are dropped.
    pub library: u64,
    /// Occurrences of symbols that the index defines but that match no symbol of the snapshot (a
    /// definition in generated code, a name the language pack does not report), or that lie in
    /// no project symbol, or whose symbol text is malformed. They are dropped, and the share of
    /// them decides whether the index is usable.
    pub unmapped: u64,
    /// Occurrences of symbols that the index defines only in documents that match no file of the
    /// snapshot: folders the scan skips, generated code that is not scanned (ADR 0061). The
    /// snapshot cannot hold their symbols, so they are dropped like those of a library and stay
    /// out of the share that `level2.min_mapped` asks for.
    pub outside_scan: u64,
    /// Occurrences of local symbols, parameters, type parameters, macros and namespaces without
    /// a definition; they are counted neither as mapped nor as dropped.
    pub ignored: u64,
    /// Files whose level 1 references were replaced.
    pub files_replaced: u64,
    /// Documents of the index that match no file of the snapshot.
    pub files_unmatched: u64,
}

impl ImportStats {
    /// All dropped occurrences: libraries, symbols outside the scan and unmapped project symbols.
    #[must_use]
    pub const fn dropped(&self) -> u64 {
        self.library
            .saturating_add(self.outside_scan)
            .saturating_add(self.unmapped)
    }
}

/// The result of an import.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportOutcome {
    /// The symbols of origin `index` that the import creates (ADR 0055), in the covered files,
    /// numbered consecutively from the first id after the highest id of the snapshot's symbols.
    pub symbols: Vec<Symbol>,
    /// The new list of references: the references of files without an index as they were, and
    /// the level 2 references of the covered files. New ids follow the highest id of the input.
    pub references: Vec<Reference>,
    /// The files whose level 1 references were replaced, in id order.
    pub covered: Vec<FileId>,
    /// The counters.
    pub stats: ImportStats,
}

/// What one occurrence leads to.
enum Fate {
    Ignore,
    Library,
    OutsideScan,
    Unmapped,
    /// The targets, shared with the table they come from instead of copied for every occurrence.
    Map(Rc<[SymbolId]>),
}

/// How a SCIP symbol is used by the import, decided once per distinct symbol.
enum Shape {
    Local,
    Malformed,
    Named(Descriptor),
    /// `impl#[Type]`: what `Self` names; it points at the type, if the index defines it.
    SelfType,
}

/// Parameters, type parameters, meta and macros have no project symbol to point at.
const fn is_ignorable(kind: DescriptorKind) -> bool {
    matches!(
        kind,
        DescriptorKind::Parameter
            | DescriptorKind::TypeParameter
            | DescriptorKind::Meta
            | DescriptorKind::Macro
    )
}

fn shape_of(symbol: &str, parsed: Option<&ImplSymbol>) -> Shape {
    if is_local(symbol) {
        return Shape::Local;
    }
    if let Some(ImplSymbol::SelfType { .. }) = parsed {
        return Shape::SelfType;
    }
    last_descriptor(symbol).map_or(Shape::Malformed, Shape::Named)
}

/// The level 1 references of the covered files, by file and line of their start.
struct LevelOne<'a> {
    by_line: HashMap<(FileId, u32), IntervalIndex<&'a Reference>>,
    by_start: HashMap<(FileId, u32, u32), Vec<&'a Reference>>,
}

impl<'a> LevelOne<'a> {
    fn new(references: &'a [Reference], covered: &HashSet<FileId>) -> Self {
        let mut by_line: HashMap<(FileId, u32), Vec<&Reference>> = HashMap::new();
        let mut by_start: HashMap<(FileId, u32, u32), Vec<&Reference>> = HashMap::new();
        for reference in references.iter().filter(|r| covered.contains(&r.file)) {
            let start = reference.span.start();
            by_line
                .entry((reference.file, start.line))
                .or_default()
                .push(reference);
            by_start
                .entry((reference.file, start.line, start.column))
                .or_default()
                .push(reference);
        }
        let by_line = by_line
            .into_iter()
            .map(|(key, list)| {
                let items = list
                    .into_iter()
                    .map(|reference| (reference.span.start(), reference.span.end(), reference));
                (key, IntervalIndex::new(items))
            })
            .collect();
        Self { by_line, by_start }
    }

    /// The references that start exactly where `span` starts.
    fn starting_at(&self, file: FileId, span: Span) -> &[&'a Reference] {
        let start = span.start();
        self.by_start
            .get(&(file, start.line, start.column))
            .map_or(&[], Vec::as_slice)
    }

    /// Whether level 1 has a reference at the same start that points at `target`.
    fn reads_at(&self, file: FileId, span: Span, target: SymbolId) -> Option<ReferenceKind> {
        self.starting_at(file, span)
            .iter()
            .find(|r| r.to == target)
            .map(|r| r.kind)
    }

    /// The kind level 1 gave to the same place: a reference at the same start that points at the
    /// same target, else one at the same start, else one whose span holds the occurrence.
    fn kind_at(&self, file: FileId, span: Span, targets: &[SymbolId]) -> Option<ReferenceKind> {
        let same_start = self.starting_at(file, span);
        same_start
            .iter()
            .find(|r| targets.contains(&r.to))
            .or_else(|| same_start.first())
            .copied()
            .or_else(|| {
                // The first (in the order of the snapshot) reference of the line that holds the
                // occurrence, found without looking at every reference of the line (issue #220).
                self.by_line
                    .get(&(file, span.start().line))?
                    .containing(span.start(), |end| span.end() <= end)
                    .into_iter()
                    .next()
                    .copied()
            })
            .map(|r| r.kind)
    }
}

/// The type of every constructor of the project that has one.
fn constructor_owners(symbols: &[Symbol]) -> HashMap<SymbolId, SymbolId> {
    symbols
        .iter()
        .filter(|symbol| symbol.kind == SymbolKind::Constructor)
        .filter_map(|symbol| Some((symbol.id, symbol.parent?)))
        .collect()
}

/// The kind of a level 2 reference when level 1 has none at the place: by the target's kind.
const fn kind_by_target(kind: SymbolKind) -> ReferenceKind {
    if kind.is_callable() {
        ReferenceKind::Call
    } else if kind.is_type() {
        ReferenceKind::TypeUse
    } else {
        ReferenceKind::Read
    }
}

/// The impl symbols of the index with their handles.
fn impl_symbols(index: &ScipIndex) -> Vec<(SymbolRef, ImplSymbol)> {
    (0..)
        .map(SymbolRef)
        .zip(&index.symbols)
        .filter(|(_, text)| !is_local(text))
        .filter_map(|(at, text)| parse_impl(text).map(|parsed| (at, parsed)))
        .collect()
}

struct Importer<'a> {
    index: &'a ScipIndex,
    snapshot: SnapshotView<'a>,
    table: SymbolTable<'a>,
    shapes: Vec<Shape>,
    kinds_of: HashMap<SymbolId, SymbolKind>,
    owners: HashMap<SymbolId, SymbolId>,
    targets: HashMap<SymbolRef, Vec<SymbolId>>,
    defined: HashSet<SymbolRef>,
    /// The symbols that a document of the snapshot defines; the rest of `defined` lies outside.
    scanned: HashSet<SymbolRef>,
    /// The paths of the files of the snapshot.
    paths: HashMap<FileId, &'a str>,
    /// The files that the index defines as modules (ADR 0061).
    roots: Roots<'a>,
    heads: HashSet<(FileId, Position, SymbolRef)>,
    locals: HashMap<SymbolRef, Rc<[SymbolId]>>,
    /// The targets of every symbol once the steps before the placing are done.
    frozen: HashMap<SymbolRef, Rc<[SymbolId]>>,
    impls: Vec<(SymbolRef, ImplSymbol)>,
    overrides: HashMap<SymbolRef, Vec<SymbolId>>,
    /// The constructors of every type that the text of its file shows (Java).
    constructors: implicit::Constructors,
    /// The superclass of every class that has one in its `extends` (Java).
    superclass: HashMap<SymbolId, Vec<SymbolRef>>,
    trait_members: HashSet<SymbolId>,
    stats: ImportStats,
    next_id: u64,
    /// Where level 1 read the name of an import, by file and line: a use inside such a range is no
    /// usage (issue #250).
    import_sites: HashMap<(FileId, u32), Vec<Span>>,
}

impl<'a> Importer<'a> {
    fn new(index: &'a ScipIndex, snapshot: SnapshotView<'a>, library_root: &'a str) -> Self {
        let highest = snapshot.references.iter().map(|r| r.id.get()).max();
        let impls = impl_symbols(index);
        let by_ref: HashMap<SymbolRef, &ImplSymbol> =
            impls.iter().map(|(at, parsed)| (*at, parsed)).collect();
        let shapes = (0..)
            .map(SymbolRef)
            .zip(&index.symbols)
            .map(|(at, text)| shape_of(text, by_ref.get(&at).copied()))
            .collect();
        Self {
            index,
            snapshot,
            table: SymbolTable::new(snapshot.symbols),
            shapes,
            impls,
            overrides: HashMap::new(),
            constructors: implicit::constructors_by_owner(index),
            superclass: HashMap::new(),
            trait_members: members_of_traits(snapshot.symbols),
            kinds_of: snapshot.symbols.iter().map(|s| (s.id, s.kind)).collect(),
            owners: constructor_owners(snapshot.symbols),
            targets: HashMap::new(),
            defined: HashSet::new(),
            scanned: HashSet::new(),
            paths: snapshot
                .files
                .iter()
                .map(|file| (file.id, file.path.as_str()))
                .collect(),
            roots: Roots::new(library_root),
            heads: HashSet::new(),
            locals: HashMap::new(),
            frozen: HashMap::new(),
            stats: ImportStats::default(),
            next_id: highest.map_or(0, |id| id.saturating_add(1)),
            import_sites: HashMap::new(),
        }
    }

    fn shape(&self, symbol: SymbolRef) -> &Shape {
        usize::try_from(symbol.0)
            .ok()
            .and_then(|position| self.shapes.get(position))
            .unwrap_or(&Shape::Malformed)
    }

    /// Step (a): every definition occurrence names the project symbol of its SCIP symbol. A
    /// symbol with several definitions (a partial class, one per file) names all of them.
    fn collect_definitions(&mut self, covered: &[(&ScipDocument, FileId)]) {
        for document in &self.index.documents {
            let definitions = document.occurrences.iter().filter(|o| o.is_definition());
            self.defined.extend(definitions.map(|o| o.symbol));
        }
        for (document, file) in covered {
            for occurrence in document.occurrences.iter().filter(|o| o.is_definition()) {
                self.scanned.insert(occurrence.symbol);
                self.define(*file, occurrence);
            }
        }
    }

    /// The project symbol of one definition: by name and position, and for a namespace that the
    /// name does not find, the module of the file if the definition is the whole file (ADR 0061).
    fn define(&mut self, file: FileId, occurrence: &Occurrence) {
        let Shape::Named(descriptor) = self.shape(occurrence.symbol) else {
            return;
        };
        if is_ignorable(descriptor.kind) {
            return;
        }
        let namespace = descriptor.kind == DescriptorKind::Namespace;
        match self.table.definition(file, occurrence.range, descriptor) {
            Some(target) => {
                let targets = self.targets.entry(occurrence.symbol).or_default();
                if targets.len() < MAX_TARGETS_PER_SYMBOL && !targets.contains(&target) {
                    targets.push(target);
                }
            }
            None if namespace => self.add_root(file, occurrence),
            None => {}
        }
    }

    /// Records the file as a root of the symbol if the definition spans the whole file and the
    /// file has a module symbol.
    fn add_root(&mut self, file: FileId, occurrence: &Occurrence) {
        let (Some(module), Some(path)) = (
            self.table.file_root(file, occurrence.range),
            self.paths.get(&file).copied(),
        ) else {
            return;
        };
        self.roots.add(occurrence.symbol, file, path, module);
    }

    /// The local functions of one document (see `locals.rs`), by their local symbol.
    fn locals_of(
        &self,
        document: &ScipDocument,
        file: FileId,
    ) -> HashMap<SymbolRef, Vec<SymbolId>> {
        let defs: Vec<&Occurrence> = document
            .occurrences
            .iter()
            .filter(|o| o.is_definition() && matches!(self.shape(o.symbol), Shape::Local))
            .collect();
        match_functions(self.table.symbols_of(file), &defs, &document.local_names)
    }

    /// Step (a2): an indexer that has one symbol for all overloads (scip-python) defines it at the
    /// first one and lists the heads of the others, and of the implementation, as plain
    /// occurrences. An occurrence that stands on the declaration of a function of the same name,
    /// in the same file and in the same parent as a definition of its symbol, is that function's
    /// own head: the function becomes a target of the symbol, and the occurrence is no use. A
    /// function that is the target of a symbol already has its own symbol, so it is no head: a
    /// call on the line of a one-line method of the same name (C#, `IEnumerator IEnumerable.
    /// GetEnumerator() => GetEnumerator();`) stays a use.
    fn collect_overload_heads(&mut self, covered: &[(&ScipDocument, FileId)]) {
        let by_id: HashMap<SymbolId, &Symbol> =
            self.snapshot.symbols.iter().map(|s| (s.id, s)).collect();
        let mut owned: HashSet<SymbolId> = self.targets.values().flatten().copied().collect();
        for (document, file) in covered {
            let mut ordered: Vec<&Occurrence> = document
                .occurrences
                .iter()
                .filter(|o| !o.is_definition())
                .collect();
            ordered.sort_by_key(|o| o.range.start());
            let mut looked_at = HashSet::new();
            for occurrence in ordered {
                let at = occurrence.range.start();
                if looked_at.insert((at.line, occurrence.symbol))
                    && self.adopt_head(&by_id, &mut owned, *file, occurrence)
                {
                    self.heads.insert((*file, at, occurrence.symbol));
                }
            }
        }
    }

    /// Makes the function that is declared at `occurrence` a target of its symbol, if it is a
    /// sibling of a target; reports whether it did.
    fn adopt_head(
        &mut self,
        by_id: &HashMap<SymbolId, &Symbol>,
        owned: &mut HashSet<SymbolId>,
        file: FileId,
        occurrence: &Occurrence,
    ) -> bool {
        let Shape::Named(descriptor) = self.shape(occurrence.symbol) else {
            return false;
        };
        if descriptor.kind != DescriptorKind::Method {
            return false;
        }
        // Nothing is looked up for a symbol that has no target yet or has all it may have: most
        // occurrences are plain calls.
        let open = self
            .targets
            .get(&occurrence.symbol)
            .is_some_and(|targets| !targets.is_empty() && targets.len() < MAX_OVERLOAD_TARGETS);
        if !open {
            return false;
        }
        let Some(head) = self
            .table
            .declared_at(file, occurrence.range.start(), descriptor)
        else {
            return false;
        };
        let sibling = |id: &SymbolId| {
            by_id
                .get(id)
                .is_some_and(|s| s.file == head.file && s.parent == head.parent)
        };
        match self.targets.get_mut(&occurrence.symbol) {
            Some(targets) if !owned.contains(&head.id) && targets.iter().any(sibling) => {
                targets.push(head.id);
                owned.insert(head.id);
                true
            }
            _ => false,
        }
    }

    /// Step (b): `impl#[Type]` points at the type, and a method of `impl#[Type][Trait]`
    /// overrides the method of the trait. Both are found by the symbol of the type (the same
    /// module as the impl block) or else by a type or trait name that only one defined symbol has.
    fn link_impls(&mut self) {
        if self.impls.is_empty() {
            return;
        }
        let impls = std::mem::take(&mut self.impls);
        self.link_self_types(&impls);
        self.link_header_types(&impls);
        self.link_trait_members(&impls);
        self.impls = impls;
    }

    fn link_self_types(&mut self, impls: &[(SymbolRef, ImplSymbol)]) {
        let by_text: HashMap<&str, SymbolRef> = self
            .index
            .symbols
            .iter()
            .map(String::as_str)
            .zip((0..).map(SymbolRef))
            .collect();
        let types = self.candidates(DescriptorKind::Type, None, |_| true);
        for (at, parsed) in impls {
            let ImplSymbol::SelfType { prefix, ty } = parsed else {
                continue;
            };
            let found = if is_bare(ty) {
                // A bare name is the type of the module of the block; nothing else is guessed.
                by_text
                    .get(format!("{prefix}{}{SUFFIX_TYPE}", ty.name).as_str())
                    .copied()
            } else {
                types.unique(NO_OWNER, ty)
            };
            if let Some(found) = found.and_then(|ty| self.targets.get(&ty).cloned()) {
                self.targets.insert(*at, found);
            }
        }
    }

    /// A bare name that is not the type of the module of the block: the header of the block
    /// (`impl Trait for Type`) names the type in the same document, so the type is the one
    /// symbol of that name that occurs there. If a library type of that name occurs as well, or
    /// the project type does not occur, nothing is mapped.
    fn link_header_types(&mut self, impls: &[(SymbolRef, ImplSymbol)]) {
        let mut open: HashMap<SymbolRef, String> = impls
            .iter()
            .filter_map(|(at, parsed)| match parsed {
                ImplSymbol::SelfType { ty, .. }
                    if is_bare(ty) && !self.targets.contains_key(at) =>
                {
                    Some((*at, ty.name.clone()))
                }
                _ => None,
            })
            .collect();
        let mut resolved = Vec::new();
        for document in &self.index.documents {
            let here: Vec<SymbolRef> = document
                .occurrences
                .iter()
                .map(|o| o.symbol)
                .filter(|symbol| open.contains_key(symbol))
                .collect();
            if here.is_empty() {
                continue;
            }
            let named = self.types_in(document);
            for at in here {
                let candidates = open.get(&at).and_then(|name| named.get(name));
                let found = candidates.and_then(|symbols| match symbols.as_slice() {
                    [only] => self.targets.get(only),
                    _ => None,
                });
                if let Some(found) = found {
                    resolved.push((at, found.clone()));
                    open.remove(&at);
                }
            }
        }
        self.targets.extend(resolved);
    }

    /// The distinct type symbols that occur in `document`, by name.
    fn types_in(&self, document: &ScipDocument) -> HashMap<String, Vec<SymbolRef>> {
        let mut named: HashMap<String, Vec<SymbolRef>> = HashMap::new();
        for occurrence in &document.occurrences {
            if let Shape::Named(descriptor) = self.shape(occurrence.symbol) {
                if descriptor.kind != DescriptorKind::Type {
                    continue;
                }
                let symbols = named.entry(descriptor.name.clone()).or_default();
                // Two different symbols of a name are as ambiguous as two hundred.
                if symbols.len() < MAX_SYMBOLS_PER_KEY && !symbols.contains(&occurrence.symbol) {
                    symbols.push(occurrence.symbol);
                }
            }
        }
        named
    }

    /// A method of a trait implementation overrides the method of the trait if exactly one trait
    /// of the project has that name (and fits the path the impl symbol gives) and defines it. A
    /// type that is no trait, or a trait of a library, gives nothing.
    fn link_trait_members(&mut self, impls: &[(SymbolRef, ImplSymbol)]) {
        let of_trait = |symbol: SymbolRef| {
            self.targets
                .get(&symbol)
                .is_some_and(|ids| ids.iter().all(|id| self.trait_members.contains(id)))
        };
        let methods = self.candidates(DescriptorKind::Method, Some(DescriptorKind::Type), of_trait);
        for (at, parsed) in impls {
            let ImplSymbol::TraitMember { trait_ty, member } = parsed else {
                continue;
            };
            let found = methods
                .unique(
                    &trait_ty.name,
                    &TypeName {
                        name: member.clone(),
                        ..trait_ty.clone()
                    },
                )
                .and_then(|m| self.targets.get(&m).cloned());
            if let Some(found) = found {
                self.overrides.insert(*at, found);
            }
        }
    }

    /// The symbols of the index that map onto the project, by the name of the last descriptor of
    /// `kind` and, if `owner` is given, the name of the descriptor before it, which then has that
    /// kind. Each comes with the modules it lies in.
    fn candidates(
        &self,
        kind: DescriptorKind,
        owner: Option<DescriptorKind>,
        accept: impl Fn(SymbolRef) -> bool,
    ) -> Fitting {
        let mut found = Fitting::default();
        for at in self.targets.keys().filter(|at| accept(**at)) {
            let Some(text) = self.index.symbol(*at) else {
                continue;
            };
            let Some((before, last)) = last_two(text) else {
                continue;
            };
            let parent = match (owner, before) {
                (None, _) => String::new(),
                (Some(want), Some(before)) if before.kind == want => before.name,
                _ => continue,
            };
            if last.kind == kind {
                found.add(&parent, &last.name, &namespaces(text), *at);
            }
        }
        found
    }

    fn fate(&self, file: FileId, symbol: SymbolRef) -> Fate {
        let descriptor = match self.shape(symbol) {
            Shape::SelfType => {
                return match self.frozen.get(&symbol) {
                    Some(targets) => Fate::Map(Rc::clone(targets)),
                    None => Fate::Ignore,
                };
            }
            Shape::Named(descriptor) => descriptor,
            Shape::Local => {
                return self
                    .locals
                    .get(&symbol)
                    .map_or(Fate::Ignore, |targets| Fate::Map(Rc::clone(targets)));
            }
            Shape::Malformed => return Fate::Unmapped,
        };
        if is_ignorable(descriptor.kind) {
            return Fate::Ignore;
        }
        match self.frozen.get(&symbol) {
            Some(targets) => Fate::Map(Rc::clone(targets)),
            None if self.defined.contains(&symbol) => self.fate_defined(file, symbol),
            None if descriptor.kind == DescriptorKind::Namespace => Fate::Ignore,
            None => Fate::Library,
        }
    }

    /// The fate of a symbol that the index defines but that no definition mapped by name: a
    /// module that is a root of its file (ADR 0061), the default constructor of a type, else it
    /// is dropped: outside the scan if no document of the snapshot defines it, else unmapped.
    fn fate_defined(&self, file: FileId, symbol: SymbolRef) -> Fate {
        let path = self.paths.get(&file).copied().unwrap_or_default();
        match self.roots.pick(symbol, file, path) {
            Pick::Module(module) => return Fate::Map(module),
            Pick::Nothing => return Fate::Unmapped,
            Pick::NoRoots => {}
        }
        if let Some(targets) = self
            .constructors
            .defaults
            .get(&symbol)
            .and_then(|ty| self.frozen.get(ty))
        {
            Fate::Map(Rc::clone(targets))
        } else if self.scanned.contains(&symbol) {
            Fate::Unmapped
        } else {
            Fate::OutsideScan
        }
    }

    fn fresh_id(&mut self) -> ReferenceId {
        let id = ReferenceId::new(self.next_id);
        self.next_id = self.next_id.saturating_add(1);
        id
    }

    fn kind_of_target(&self, target: SymbolId) -> ReferenceKind {
        self.kinds_of
            .get(&target)
            .map_or(ReferenceKind::Read, |kind| kind_by_target(*kind))
    }

    /// The targets of one occurrence with the kind each is used as: the kind level 1 gave at that
    /// place, else the kind the target stands for, and the types of constructors.
    fn aims(
        &self,
        file: FileId,
        occurrence: &Occurrence,
        targets: &[SymbolId],
        level_one: &LevelOne<'_>,
    ) -> Vec<(SymbolId, ReferenceKind)> {
        let level_one_kind = level_one.kind_at(file, occurrence.range, targets);
        let types = self.types_of_constructors(file, occurrence.range, targets, level_one);
        let mut aims: Vec<(SymbolId, ReferenceKind)> = targets
            .iter()
            .copied()
            .map(|to| {
                let kind = level_one_kind.unwrap_or_else(|| self.kind_of_target(to));
                (to, kind)
            })
            .collect();
        aims.extend(types);
        aims
    }

    /// Whether `range` lies inside the name of an import statement as level 1 read it. The range
    /// of a dotted path (`import a.b.c`) holds the occurrences of all its parts.
    fn in_import(&self, file: FileId, range: Span) -> bool {
        self.import_sites
            .get(&(file, range.start().line))
            .is_some_and(|sites| {
                sites
                    .iter()
                    .any(|site| site.start() <= range.start() && range.end() <= site.end())
            })
    }

    /// Step (c): the references of one occurrence, one per target; empty if it is dropped.
    fn place(
        &mut self,
        file: FileId,
        occurrence: &Occurrence,
        level_one: &LevelOne<'_>,
    ) -> Vec<Reference> {
        if self.in_import(file, occurrence.range) {
            // The name in an import statement is not a use (issue #250), as at level 1.
            return self.count(Count::Ignored);
        }
        let targets = match self.fate(file, occurrence.symbol) {
            Fate::Ignore => return self.count(Count::Ignored),
            Fate::Library => return self.count(Count::Library),
            Fate::OutsideScan => return self.count(Count::OutsideScan),
            Fate::Unmapped => return self.count(Count::Unmapped),
            Fate::Map(targets) => targets,
        };
        let Some(from) = self.table.innermost(file, occurrence.range.start()) else {
            return self.count(Count::Unmapped);
        };
        self.stats.mapped += 1;
        let aims = self.aims(file, occurrence, &targets, level_one);
        let mut placed = Vec::with_capacity(aims.len());
        for (to, kind) in aims {
            placed.push(Reference {
                id: self.fresh_id(),
                from: from.id,
                to,
                file,
                span: occurrence.range,
                kind,
                confidence: Confidence::Certain,
                verified: true,
            });
        }
        placed
    }

    /// A constructor reference (`new Gson(...)`) is a use of its type too, where level 1 reads the
    /// type name at the same place: the types, with the kind level 1 gave. `this(...)` and
    /// `super(...)` stand on a keyword, level 1 has no reference to the type there, and so none is
    /// added.
    fn types_of_constructors(
        &self,
        file: FileId,
        span: Span,
        targets: &[SymbolId],
        level_one: &LevelOne<'_>,
    ) -> Vec<(SymbolId, ReferenceKind)> {
        targets
            .iter()
            .take(MAX_OVERLOAD_TARGETS)
            .filter_map(|to| self.owners.get(to).copied())
            .filter_map(|owner| Some((owner, level_one.reads_at(file, span, owner)?)))
            .collect()
    }

    /// The `override` reference that the definition of a method of a trait implementation stands
    /// for: from the method to the method of the trait, `certain` and `verified`.
    fn place_override(&mut self, file: FileId, occurrence: &Occurrence) -> Vec<Reference> {
        let Some(targets) = self.overrides.get(&occurrence.symbol).cloned() else {
            return Vec::new();
        };
        let Some(from) = self.table.innermost(file, occurrence.range.start()) else {
            return Vec::new();
        };
        let from = from.id;
        targets
            .into_iter()
            .map(|to| Reference {
                id: self.fresh_id(),
                from,
                to,
                file,
                span: occurrence.range,
                kind: ReferenceKind::Override,
                confidence: Confidence::Certain,
                verified: true,
            })
            .collect()
    }

    fn count(&mut self, what: Count) -> Vec<Reference> {
        match what {
            Count::Ignored => self.stats.ignored += 1,
            Count::Library => self.stats.library += 1,
            Count::OutsideScan => self.stats.outside_scan += 1,
            Count::Unmapped => self.stats.unmapped += 1,
        }
        Vec::new()
    }

    /// The references of one document: those of its occurrences and its implicit calls.
    fn place_document(
        &mut self,
        document: &ScipDocument,
        file: FileId,
        level_one: &LevelOne<'_>,
        limit: usize,
    ) -> Result<Vec<Reference>, ScipError> {
        self.locals = self
            .locals_of(document, file)
            .into_iter()
            .map(|(at, targets)| (at, Rc::from(targets)))
            .collect();
        let mut found = Vec::new();
        for occurrence in &document.occurrences {
            if self
                .heads
                .contains(&(file, occurrence.range.start(), occurrence.symbol))
            {
                self.count(Count::Ignored);
                continue;
            }
            found.extend(if occurrence.is_definition() {
                self.place_override(file, occurrence)
            } else {
                self.place(file, occurrence, level_one)
            });
            if found.len() > limit {
                return Err(ScipError::TooManyReferences { limit });
            }
        }
        found.extend(self.implicit_calls(file, document));
        Ok(found)
    }

    /// The level 2 references of all covered documents, each place once.
    fn place_all(
        &mut self,
        covered: &[(&ScipDocument, FileId)],
        level_one: &LevelOne<'_>,
        limit: usize,
    ) -> Result<Vec<Reference>, ScipError> {
        let mut seen: HashSet<(FileId, SymbolId, SymbolId, Span)> = HashSet::new();
        let mut placed = Vec::new();
        self.frozen = self
            .targets
            .iter()
            .filter(|(_, targets)| !targets.is_empty())
            .map(|(at, targets)| (*at, Rc::from(targets.as_slice())))
            .collect();
        self.collect_superclasses(covered, level_one);
        for (document, file) in covered {
            let found = self.place_document(document, *file, level_one, limit)?;
            for reference in found {
                let key = (reference.file, reference.from, reference.to, reference.span);
                if seen.insert(key) {
                    placed.push(reference);
                }
            }
            if placed.len() > limit {
                return Err(ScipError::TooManyReferences { limit });
            }
        }
        Ok(placed)
    }
}

#[derive(Clone, Copy)]
enum Count {
    Ignored,
    Library,
    OutsideScan,
    Unmapped,
}

/// A name without a path.
const fn is_bare(ty: &TypeName) -> bool {
    ty.path.is_empty() && !ty.rooted
}

/// The symbols that are declared inside a trait of the project.
fn members_of_traits(symbols: &[Symbol]) -> HashSet<SymbolId> {
    let traits: HashSet<SymbolId> = symbols
        .iter()
        .filter(|symbol| symbol.kind == SymbolKind::Trait)
        .map(|symbol| symbol.id)
        .collect();
    symbols
        .iter()
        .filter(|symbol| symbol.parent.is_some_and(|parent| traits.contains(&parent)))
        .map(|symbol| symbol.id)
        .collect()
}

fn to_f64(count: u64) -> f64 {
    f64::from(u32::try_from(count).unwrap_or(u32::MAX))
}

/// Step (e): of the occurrences of project symbols (those the index defines), the share that
/// mapped must reach `min_mapped`, and the index must cover at least one file. Library
/// occurrences do not count: most occurrences of a real project point into libraries. An index
/// whose covered documents hold uses but not one of them mapped is unusable too (an indexer
/// that never marks definitions leaves nothing to map onto), whatever the share says.
fn check_threshold(stats: &ImportStats, min_mapped: f64) -> Result<(), ScipError> {
    let total = stats.mapped.saturating_add(stats.unmapped);
    let uses = total
        .saturating_add(stats.library)
        .saturating_add(stats.outside_scan)
        .saturating_add(stats.ignored);
    let uncovered = stats.files_replaced == 0;
    let nothing_mapped = stats.mapped == 0 && uses > 0;
    let too_low = total > 0 && to_f64(stats.mapped) < to_f64(total) * min_mapped;
    if uncovered || nothing_mapped || too_low {
        Err(ScipError::MappingTooLow {
            mapped: stats.mapped,
            total,
        })
    } else {
        Ok(())
    }
}

/// Adds level 2 references for every file the index covers; they replace the level 1 references
/// where they answer.
///
/// * A definition occurrence (role bit 1) is mapped onto the smallest project symbol of its file
///   that contains its position and bears the name of the last descriptor of its SCIP symbol.
///   Overloads, which SCIP tells apart and the project does too (one symbol per declaration),
///   follow from the position; a partial class maps to the symbol of each of its files.
/// * Every other occurrence of a mapped symbol becomes a reference from the smallest symbol that
///   contains the place to the mapped symbol, `certain` and `verified`. Its kind is the kind of
///   the level 1 reference at the same place, else by the target: callable is a call, type is a
///   type use, anything else a read.
/// * Occurrences of local symbols, parameters, type parameters and macros are ignored. Those of
///   symbols that the index does not define (libraries) and those of defined symbols that match
///   no project symbol are dropped and counted apart.
/// * An occurrence that lies in no symbol of its file (code at file level that the language
///   pack reports no module for) cannot get an origin and counts as `unmapped`.
/// * The references of files the index does not cover stay as they are.
///
/// # Errors
/// [`ScipError::MappingTooLow`] if no document matches a file of the snapshot, the covered
/// documents hold uses but none of them mapped, or the share of mapped occurrences among those
/// of project symbols is below `min_mapped` (config key `level2.min_mapped`); the caller keeps
/// level 1 then.
pub fn import_references(
    index: &ScipIndex,
    snapshot: &SnapshotView<'_>,
    min_mapped: f64,
) -> Result<ImportOutcome, ScipError> {
    import_with_creation(index, snapshot, min_mapped, &Creation::off())
}

/// Documents that match a file of the snapshot, with that file, and the number of those that do
/// not.
fn covered_documents<'a>(
    index: &'a ScipIndex,
    files: &[SourceFile],
) -> (Vec<(&'a ScipDocument, FileId)>, u64) {
    let by_path: HashMap<&str, FileId> = files
        .iter()
        .map(|file| (file.path.as_str(), file.id))
        .collect();
    let mut covered = Vec::new();
    let mut unmatched = u64::default();
    for document in &index.documents {
        match by_path.get(document.path.as_str()) {
            Some(file) => covered.push((document, *file)),
            None => unmatched += 1,
        }
    }
    (covered, unmatched)
}

/// The symbols of level 1: those that a former level 2 import created (origin `index`) are left
/// out, so the result does not depend on whether level 2 ran before (ADR 0055, decision 5).
fn syntax_symbols(symbols: &[Symbol]) -> Cow<'_, [Symbol]> {
    if symbols.iter().all(|s| s.origin == SymbolOrigin::Syntax) {
        Cow::Borrowed(symbols)
    } else {
        Cow::Owned(
            symbols
                .iter()
                .filter(|s| s.origin == SymbolOrigin::Syntax)
                .cloned()
                .collect(),
        )
    }
}

/// [`import_references`], and level 2 creates the symbols that the index defines and level 1 does
/// not know, as far as `creation` allows (ADR 0055). Level 1 symbols are never changed, and the
/// symbols of origin `index` that the snapshot holds are not looked at: the import creates them
/// again, and the caller replaces them.
///
/// # Errors
/// As [`import_references`], and [`ScipError::TooManyCreatedSymbols`] if the index wants more
/// symbols than `creation` allows; nothing of the import is kept then.
pub fn import_with_creation(
    index: &ScipIndex,
    snapshot: &SnapshotView<'_>,
    min_mapped: f64,
    creation: &Creation<'_>,
) -> Result<ImportOutcome, ScipError> {
    import_with_sites(index, snapshot, (min_mapped, creation), &[])
}

/// The first symbol id that no symbol of the snapshot uses.
fn first_free_id(symbols: &[Symbol]) -> u64 {
    symbols
        .iter()
        .map(|symbol| symbol.id.get())
        .max()
        .map_or(0, |id| id.saturating_add(1))
}

/// The symbols of the syntax and the created ones, copied only if something was created.
fn with_created<'a>(syntax: Cow<'a, [Symbol]>, created: &[Symbol]) -> Cow<'a, [Symbol]> {
    if created.is_empty() {
        syntax
    } else {
        Cow::Owned(syntax.iter().chain(created).cloned().collect())
    }
}

/// As [`import_with_creation`], and the occurrences inside one of `import_sites` (file and range
/// of the name in an import statement, as level 1 read it) are not uses: the import of a name is
/// no usage of it (issue #250). Level 1 keeps no reference for such a place either.
///
/// # Errors
/// As [`import_with_creation`].
pub fn import_with_sites(
    index: &ScipIndex,
    snapshot: &SnapshotView<'_>,
    (min_mapped, creation): (f64, &Creation<'_>),
    import_sites: &[(FileId, Span)],
) -> Result<ImportOutcome, ScipError> {
    let (covered, unmatched) = covered_documents(index, snapshot.files);
    let syntax = syntax_symbols(snapshot.symbols);
    let first_id = first_free_id(snapshot.symbols);
    let table = SymbolTable::new(&syntax);
    let created = created_symbols(index, &covered, &table, creation, first_id)?;
    let all = with_created(syntax, &created);
    let view = SnapshotView {
        symbols: &all,
        ..*snapshot
    };
    let mut outcome = place_references(
        index,
        &view,
        &covered,
        (
            min_mapped,
            creation.library_root,
            creation.max_references,
            import_sites,
        ),
    )?;
    outcome.stats.files_unmatched = unmatched;
    outcome.symbols = created;
    Ok(outcome)
}

/// Steps (a) to (e) over the covered documents.
fn place_references(
    index: &ScipIndex,
    snapshot: &SnapshotView<'_>,
    covered: &[(&ScipDocument, FileId)],
    (min_mapped, library_root, max_references, import_sites): (f64, &str, usize, &[(FileId, Span)]),
) -> Result<ImportOutcome, ScipError> {
    let mut importer = Importer::new(index, *snapshot, library_root);
    for (file, span) in import_sites {
        importer
            .import_sites
            .entry((*file, span.start().line))
            .or_default()
            .push(*span);
    }
    let covered_files: HashSet<FileId> = covered.iter().map(|(_, file)| *file).collect();
    importer.stats.files_replaced = u64::try_from(covered_files.len()).unwrap_or(u64::MAX);
    importer.collect_definitions(covered);
    importer.collect_overload_heads(covered);
    importer.link_impls();
    let level_one = LevelOne::new(snapshot.references, &covered_files);
    let fresh = importer.place_all(covered, &level_one, max_references)?;
    check_threshold(&importer.stats, min_mapped)?;
    let mut references: Vec<Reference> = snapshot
        .references
        .iter()
        .filter(|reference| !covered_files.contains(&reference.file))
        .copied()
        .collect();
    references.extend(fresh);
    let mut covered: Vec<FileId> = covered_files.into_iter().collect();
    covered.sort_unstable();
    Ok(ImportOutcome {
        symbols: Vec::new(),
        references,
        covered,
        stats: importer.stats,
    })
}
