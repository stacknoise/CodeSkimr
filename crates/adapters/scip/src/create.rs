//! Symbols that level 2 creates (ADR 0055): definitions that the index states and level 1 does
//! not know, such as the types and enum variants a `macro_rules!` call generates.
//!
//! Nothing here is guessed. A symbol is created only if its definition is not matched by a
//! symbol of level 1, the pack names its SCIP kind, the display name equals the name in the
//! symbol text, and its parent is a symbol of the same file. Everything else stays unmapped.

use std::collections::{HashMap, HashSet};

use codeskimr_domain::{
    FileId, Position, QualifiedName, Span, Symbol, SymbolId, SymbolKind, SymbolName, SymbolOrigin,
};

use crate::consts::{NO_LIBRARY_ROOT, NO_SEPARATOR, SCIP_KIND_NAMES};
use crate::error::ScipError;
use crate::model::{Occurrence, ScipDocument, ScipIndex, SymbolInfo, SymbolRef};
use crate::symbol::{last_descriptor, parent_text};
use crate::table::SymbolTable;

/// What the import may create and how it reads module roots, from the language pack and the
/// configuration.
#[derive(Debug, Clone, Copy)]
pub struct Creation<'a> {
    /// The SCIP kinds (by name, `Struct`) that create a symbol, and the kind they become.
    pub kinds: &'a [(String, SymbolKind)],
    /// The separator of qualified names of the language.
    pub separator: &'a str,
    /// Most symbols one index may create; more fails the whole import.
    pub max_symbols: usize,
    /// The file name of the root that wins when a symbol is the module of several files and the
    /// folders do not decide (`[indexer] library_root` of the pack, ADR 0061); empty for none.
    pub library_root: &'a str,
}

impl Creation<'static> {
    /// Creates nothing: the state of a language whose pack has no kind table.
    #[must_use]
    pub const fn off() -> Self {
        Self {
            kinds: &[],
            separator: NO_SEPARATOR,
            max_symbols: 0,
            library_root: NO_LIBRARY_ROOT,
        }
    }
}

/// A definition that will become a symbol, if its parent is found.
struct Candidate<'a> {
    file: FileId,
    symbol: SymbolRef,
    text: &'a str,
    kind: SymbolKind,
    info: &'a SymbolInfo,
    /// The place of the name; the symbol is looked up around it.
    at: Position,
    span: Span,
}

/// Where the parent of a candidate is.
#[derive(Clone, Copy)]
enum Parent<'a> {
    /// A candidate, by its place in the list.
    Created(usize),
    /// A symbol of level 1.
    Syntax(&'a Symbol),
}

/// The kind a SCIP kind number stands for, if the pack names its SCIP name.
fn kind_for(creation: &Creation<'_>, scip_kind: u32) -> Option<SymbolKind> {
    let name = SCIP_KIND_NAMES
        .iter()
        .find_map(|(number, name)| (*number == scip_kind).then_some(*name))?;
    creation
        .kinds
        .iter()
        .find_map(|(scip, kind)| (scip == name).then_some(*kind))
}

/// The candidate that one definition is, if it is one.
fn candidate_of<'a>(
    index: &'a ScipIndex,
    infos: &HashMap<SymbolRef, &'a SymbolInfo>,
    (document, file): (&ScipDocument, FileId),
    occurrence: &Occurrence,
    (table, creation): (&SymbolTable<'_>, &Creation<'_>),
) -> Option<Candidate<'a>> {
    let info = *infos.get(&occurrence.symbol)?;
    let text = index.symbol(occurrence.symbol)?;
    let descriptor = last_descriptor(text)?;
    let kind = kind_for(creation, info.kind)?;
    if info.display_name != descriptor.name
        || table
            .definition(file, occurrence.range, &descriptor)
            .is_some()
    {
        return None;
    }
    let range = occurrence.range;
    let span = document
        .extents
        .get(&range)
        .copied()
        .filter(|extent| extent.start() <= range.start() && range.end() <= extent.end())
        .unwrap_or(range);
    Some(Candidate {
        file,
        symbol: occurrence.symbol,
        text,
        kind,
        info,
        at: range.start(),
        span,
    })
}

/// The candidates of one document: definitions of kinds the pack names that level 1 does not
/// match, once per symbol.
fn candidates_of<'a>(
    index: &'a ScipIndex,
    document: &'a ScipDocument,
    file: FileId,
    context: (&SymbolTable<'_>, &Creation<'_>),
) -> Vec<Candidate<'a>> {
    let infos: HashMap<SymbolRef, &SymbolInfo> = document
        .infos
        .iter()
        .map(|info| (info.symbol, info))
        .collect();
    let mut seen = HashSet::new();
    document
        .occurrences
        .iter()
        .filter(|occurrence| occurrence.is_definition())
        .filter_map(|occurrence| candidate_of(index, &infos, (document, file), occurrence, context))
        .filter(|candidate| seen.insert(candidate.symbol))
        .collect()
}

/// The symbols of level 1 that the descriptor paths of the candidates name: the holder of a
/// candidate (the text of its symbol without the last descriptor) that is not a candidate itself
/// but is defined in the same document and matched by a symbol of level 1.
fn named_holders<'a>(
    index: &ScipIndex,
    covered: &[(&ScipDocument, FileId)],
    candidates: &[Candidate<'_>],
    table: &SymbolTable<'a>,
    by_text: &HashMap<&str, SymbolRef>,
) -> HashMap<(FileId, SymbolRef), &'a Symbol> {
    let created: HashSet<(FileId, SymbolRef)> =
        candidates.iter().map(|c| (c.file, c.symbol)).collect();
    let wanted: HashSet<(FileId, SymbolRef)> = candidates
        .iter()
        .filter_map(|c| {
            let holder = *by_text.get(parent_text(c.text)?)?;
            (!created.contains(&(c.file, holder))).then_some((c.file, holder))
        })
        .collect();
    let mut found = HashMap::new();
    for (document, file) in covered {
        for occurrence in document.occurrences.iter().filter(|o| o.is_definition()) {
            let key = (*file, occurrence.symbol);
            let descriptor = index.symbol(occurrence.symbol).and_then(last_descriptor);
            let symbol = descriptor
                .filter(|_| wanted.contains(&key))
                .and_then(|d| table.definition(*file, occurrence.range, &d))
                .and_then(|id| table.find(*file, id));
            if let Some(symbol) = symbol {
                found.insert(key, symbol);
            }
        }
    }
    found
}

/// What `parents` looks symbols up in.
struct Holders<'a, 'b> {
    table: &'b SymbolTable<'a>,
    by_text: &'b HashMap<&'b str, SymbolRef>,
    named: HashMap<(FileId, SymbolRef), &'a Symbol>,
}

/// The parent of every candidate: the created symbol that the descriptor path names (in the same
/// file), else the symbol of level 1 that it names, else the innermost symbol of level 1 around
/// the definition. `None` where there is none, or where the descriptor path names a candidate
/// that has no parent itself.
fn parents<'a>(candidates: &[Candidate<'_>], holders: &Holders<'a, '_>) -> Vec<Option<Parent<'a>>> {
    let created: HashMap<(FileId, SymbolRef), usize> = candidates
        .iter()
        .enumerate()
        .map(|(at, c)| ((c.file, c.symbol), at))
        .collect();
    let mut found: Vec<Option<Parent<'a>>> = vec![None; candidates.len()];
    let mut order: Vec<usize> = (0..candidates.len()).collect();
    // A holder's text is the start of the text of what it holds, so the shorter one is first.
    order.sort_by_key(|at| candidates.get(*at).map_or(0, |c| c.text.len()));
    for at in order {
        let Some(candidate) = candidates.get(at) else {
            continue;
        };
        let holder = parent_text(candidate.text)
            .and_then(|text| holders.by_text.get(text))
            .map(|symbol| (candidate.file, *symbol));
        let parent = match holder {
            Some(key) if created.contains_key(&key) => created
                .get(&key)
                .filter(|holder| found.get(**holder).is_some_and(Option::is_some))
                .map(|holder| Parent::Created(*holder)),
            Some(key) if holders.named.contains_key(&key) => {
                holders.named.get(&key).copied().map(Parent::Syntax)
            }
            _ => holders
                .table
                .innermost(candidate.file, candidate.at)
                .map(Parent::Syntax),
        };
        if let Some(slot) = found.get_mut(at) {
            *slot = parent;
        }
    }
    found
}

/// The name and the qualified name of every candidate, the latter from the one of its parent.
/// A candidate whose names are not valid has none, and neither have those it holds.
fn names_of(
    candidates: &[Candidate<'_>],
    parents: &[Option<Parent<'_>>],
    separator: &str,
) -> Vec<Option<(SymbolName, QualifiedName)>> {
    let mut names: Vec<Option<(SymbolName, QualifiedName)>> = vec![None; candidates.len()];
    let mut order: Vec<usize> = (0..candidates.len()).collect();
    order.sort_by_key(|at| candidates.get(*at).map_or(0, |c| c.text.len()));
    for at in order {
        let (Some(candidate), Some(Some(parent))) = (candidates.get(at), parents.get(at)) else {
            continue;
        };
        let holder = match parent {
            Parent::Created(holder) => names
                .get(*holder)
                .and_then(Option::as_ref)
                .map(|(_, q)| q.as_str()),
            Parent::Syntax(symbol) => Some(symbol.qualified_name.as_str()),
        };
        let display = candidate.info.display_name.as_str();
        let made = holder.and_then(|holder| {
            let qualified = QualifiedName::new([holder, separator, display].concat()).ok()?;
            Some((SymbolName::new(display).ok()?, qualified))
        });
        if let Some(slot) = names.get_mut(at) {
            *slot = made;
        }
    }
    names
}

/// The symbols to create for the covered documents, with consecutive ids from `first_id`, in the
/// order of their files and places.
///
/// # Errors
/// [`ScipError::TooManyCreatedSymbols`] if the index wants more than `creation.max_symbols`.
pub(crate) fn created_symbols(
    index: &ScipIndex,
    covered: &[(&ScipDocument, FileId)],
    table: &SymbolTable<'_>,
    creation: &Creation<'_>,
    first_id: u64,
) -> Result<Vec<Symbol>, ScipError> {
    if creation.kinds.is_empty() {
        return Ok(Vec::new());
    }
    let mut candidates: Vec<Candidate<'_>> = covered
        .iter()
        .flat_map(|(document, file)| candidates_of(index, document, *file, (table, creation)))
        .collect();
    if candidates.len() > creation.max_symbols {
        return Err(ScipError::TooManyCreatedSymbols {
            wanted: candidates.len(),
            limit: creation.max_symbols,
        });
    }
    candidates.sort_by_key(|c| (c.file, c.span.start(), c.text.len()));
    let by_text: HashMap<&str, SymbolRef> = (0..)
        .map(SymbolRef)
        .zip(index.symbols.iter().map(String::as_str))
        .map(|(at, text)| (text, at))
        .collect();
    let named = named_holders(index, covered, &candidates, table, &by_text);
    let parents = parents(
        &candidates,
        &Holders {
            table,
            by_text: &by_text,
            named,
        },
    );
    let qualified = names_of(&candidates, &parents, creation.separator);
    Ok(build(&candidates, &parents, qualified, first_id))
}

/// The symbols of the candidates that have a parent and valid names, numbered from `first_id` in
/// the order of the candidates.
fn build(
    candidates: &[Candidate<'_>],
    parents: &[Option<Parent<'_>>],
    names: Vec<Option<(SymbolName, QualifiedName)>>,
    first_id: u64,
) -> Vec<Symbol> {
    let kept: Vec<usize> = (0..candidates.len())
        .filter(|at| names.get(*at).is_some_and(Option::is_some))
        .collect();
    let ids: HashMap<usize, SymbolId> = kept
        .iter()
        .zip(first_id..)
        .map(|(at, id)| (*at, SymbolId::new(id)))
        .collect();
    let mut names = names;
    kept.iter()
        .filter_map(|at| {
            let candidate = candidates.get(*at)?;
            let (name, qualified_name) = names.get_mut(*at)?.take()?;
            let parent = match (*parents.get(*at)?)? {
                Parent::Created(holder) => *ids.get(&holder)?,
                Parent::Syntax(symbol) => symbol.id,
            };
            Some(Symbol {
                id: *ids.get(at)?,
                file: candidate.file,
                kind: candidate.kind,
                name,
                qualified_name,
                parent: Some(parent),
                visibility: None,
                is_static: false,
                span: candidate.span,
                signature: candidate.info.signature.clone(),
                doc: candidate.info.documentation.clone(),
                origin: SymbolOrigin::Index,
            })
        })
        .collect()
}
