//! Maps SCIP occurrences onto project symbols and replaces the level 1 references of the files
//! the index covers (ADR 0033, decisions 2 and 3). A pure function over the decoded index and
//! the snapshot: no input or output, no state.

use std::collections::{HashMap, HashSet};

use codeskimr_domain::{
    Confidence, FileId, Reference, ReferenceId, ReferenceKind, SourceFile, Span, Symbol, SymbolId,
    SymbolKind,
};

use crate::consts::{NO_OWNER, SUFFIX_TYPE};
use crate::error::ScipError;
use crate::fitting::Fitting;
use crate::impl_symbol::{ImplSymbol, TypeName, parse as parse_impl};
use crate::model::{Occurrence, ScipDocument, ScipIndex, SymbolRef};
use crate::symbol::{Descriptor, DescriptorKind, is_local, last_descriptor, last_two, namespaces};
use crate::table::SymbolTable;

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
    /// Occurrences of local symbols, parameters, type parameters, macros and namespaces without
    /// a definition; they are counted neither as mapped nor as dropped.
    pub ignored: u64,
    /// Files whose level 1 references were replaced.
    pub files_replaced: u64,
    /// Documents of the index that match no file of the snapshot.
    pub files_unmatched: u64,
}

impl ImportStats {
    /// All dropped occurrences: libraries and unmapped project symbols.
    #[must_use]
    pub const fn dropped(&self) -> u64 {
        self.library.saturating_add(self.unmapped)
    }
}

/// The result of an import.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportOutcome {
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
    Unmapped,
    Map(Vec<SymbolId>),
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
    by_line: HashMap<(FileId, u32), Vec<&'a Reference>>,
}

impl<'a> LevelOne<'a> {
    fn new(references: &'a [Reference], covered: &HashSet<FileId>) -> Self {
        let mut by_line: HashMap<(FileId, u32), Vec<&Reference>> = HashMap::new();
        for reference in references.iter().filter(|r| covered.contains(&r.file)) {
            by_line
                .entry((reference.file, reference.span.start().line))
                .or_default()
                .push(reference);
        }
        Self { by_line }
    }

    /// The kind level 1 gave to the same place: a reference at the same start that points at the
    /// same target, else one at the same start, else one whose span holds the occurrence.
    fn kind_at(&self, file: FileId, span: Span, targets: &[SymbolId]) -> Option<ReferenceKind> {
        let line = self.by_line.get(&(file, span.start().line))?;
        let same_start = |r: &&&Reference| r.span.start() == span.start();
        let holds = |r: &&&Reference| r.span.start() <= span.start() && span.end() <= r.span.end();
        line.iter()
            .find(|r| same_start(r) && targets.contains(&r.to))
            .or_else(|| line.iter().find(same_start))
            .or_else(|| line.iter().find(holds))
            .map(|r| r.kind)
    }
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
    targets: HashMap<SymbolRef, Vec<SymbolId>>,
    defined: HashSet<SymbolRef>,
    impls: Vec<(SymbolRef, ImplSymbol)>,
    overrides: HashMap<SymbolRef, Vec<SymbolId>>,
    trait_members: HashSet<SymbolId>,
    stats: ImportStats,
    next_id: u64,
}

impl<'a> Importer<'a> {
    fn new(index: &'a ScipIndex, snapshot: SnapshotView<'a>) -> Self {
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
            trait_members: members_of_traits(snapshot.symbols),
            kinds_of: snapshot.symbols.iter().map(|s| (s.id, s.kind)).collect(),
            targets: HashMap::new(),
            defined: HashSet::new(),
            stats: ImportStats::default(),
            next_id: highest.map_or(0, |id| id.saturating_add(1)),
        }
    }

    fn shape(&self, symbol: SymbolRef) -> &Shape {
        usize::try_from(symbol.0)
            .ok()
            .and_then(|position| self.shapes.get(position))
            .unwrap_or(&Shape::Malformed)
    }

    /// Documents that match a file of the snapshot, with that file.
    fn covered(&mut self) -> Vec<(&'a ScipDocument, FileId)> {
        let by_path: HashMap<&str, FileId> = self
            .snapshot
            .files
            .iter()
            .map(|file| (file.path.as_str(), file.id))
            .collect();
        let mut covered = Vec::new();
        for document in &self.index.documents {
            match by_path.get(document.path.as_str()) {
                Some(file) => covered.push((document, *file)),
                None => self.stats.files_unmatched += 1,
            }
        }
        covered
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
                let Shape::Named(descriptor) = self.shape(occurrence.symbol) else {
                    continue;
                };
                if is_ignorable(descriptor.kind) {
                    continue;
                }
                let found = self
                    .table
                    .definition(*file, occurrence.range.start(), descriptor);
                if let Some(target) = found {
                    let targets = self.targets.entry(occurrence.symbol).or_default();
                    if !targets.contains(&target) {
                        targets.push(target);
                    }
                }
            }
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
                if symbols.len() < 2 && !symbols.contains(&occurrence.symbol) {
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

    fn fate(&self, symbol: SymbolRef) -> Fate {
        let descriptor = match self.shape(symbol) {
            Shape::SelfType => {
                return match self.targets.get(&symbol) {
                    Some(targets) if !targets.is_empty() => Fate::Map(targets.clone()),
                    _ => Fate::Ignore,
                };
            }
            Shape::Named(descriptor) => descriptor,
            Shape::Local => return Fate::Ignore,
            Shape::Malformed => return Fate::Unmapped,
        };
        if is_ignorable(descriptor.kind) {
            return Fate::Ignore;
        }
        match self.targets.get(&symbol) {
            Some(targets) if !targets.is_empty() => Fate::Map(targets.clone()),
            _ if self.defined.contains(&symbol) => Fate::Unmapped,
            _ if descriptor.kind == DescriptorKind::Namespace => Fate::Ignore,
            _ => Fate::Library,
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

    /// Step (c): the references of one occurrence, one per target; empty if it is dropped.
    fn place(
        &mut self,
        file: FileId,
        occurrence: &Occurrence,
        level_one: &LevelOne<'_>,
    ) -> Vec<Reference> {
        let targets = match self.fate(occurrence.symbol) {
            Fate::Ignore => return self.count(Count::Ignored),
            Fate::Library => return self.count(Count::Library),
            Fate::Unmapped => return self.count(Count::Unmapped),
            Fate::Map(targets) => targets,
        };
        let Some(from) = self.table.innermost(file, occurrence.range.start()) else {
            return self.count(Count::Unmapped);
        };
        self.stats.mapped += 1;
        let level_one_kind = level_one.kind_at(file, occurrence.range, &targets);
        let mut placed = Vec::with_capacity(targets.len());
        for to in targets {
            placed.push(Reference {
                id: self.fresh_id(),
                from: from.id,
                to,
                file,
                span: occurrence.range,
                kind: level_one_kind.unwrap_or_else(|| self.kind_of_target(to)),
                confidence: Confidence::Certain,
                verified: true,
            });
        }
        placed
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
            Count::Unmapped => self.stats.unmapped += 1,
        }
        Vec::new()
    }

    /// The level 2 references of all covered documents, each place once.
    fn place_all(
        &mut self,
        covered: &[(&ScipDocument, FileId)],
        level_one: &LevelOne<'_>,
    ) -> Vec<Reference> {
        let mut seen: HashSet<(FileId, SymbolId, SymbolId, Span)> = HashSet::new();
        let mut placed = Vec::new();
        for (document, file) in covered {
            for occurrence in &document.occurrences {
                let references = if occurrence.is_definition() {
                    self.place_override(*file, occurrence)
                } else {
                    self.place(*file, occurrence, level_one)
                };
                for reference in references {
                    let key = (reference.file, reference.from, reference.to, reference.span);
                    if seen.insert(key) {
                        placed.push(reference);
                    }
                }
            }
        }
        placed
    }
}

#[derive(Clone, Copy)]
enum Count {
    Ignored,
    Library,
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

/// Replaces the level 1 references of every file the index covers by level 2 references.
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
    let mut importer = Importer::new(index, *snapshot);
    let covered = importer.covered();
    let covered_files: HashSet<FileId> = covered.iter().map(|(_, file)| *file).collect();
    importer.stats.files_replaced = u64::try_from(covered_files.len()).unwrap_or(u64::MAX);
    importer.collect_definitions(&covered);
    importer.link_impls();
    let level_one = LevelOne::new(snapshot.references, &covered_files);
    let fresh = importer.place_all(&covered, &level_one);
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
        references,
        covered,
        stats: importer.stats,
    })
}
