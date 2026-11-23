//! The answers of the usages, impact, namespace and type queries, read from the database (ADR 0089).
//!
//! Nothing is built ahead of a query. A usages walk reads the references of the symbols it
//! reaches through the indexes on the end of a reference, one statement per step, and walks the
//! few that came back with the `ReverseIndex`. The labels and paths that name the entries of an
//! answer are read for the entries that are shown, not for the whole snapshot.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use codeskimr_application::ports::{
    ImpactedSymbol, SymbolLabel, TypeFocus, TypeNeighbour, UsageEntry,
};
use codeskimr_domain::{
    Direction, NamespaceGraph, QualifiedName, Reached, Reference, ReferenceKind, RelativePath,
    ReverseIndex, SymbolId, SymbolKind, TypeLink, TypeOwners, Usage, UsageQuery, Visibility,
    namespace_graph,
};
use rusqlite::{Connection, params};

use crate::codec::{Cols, column_int, id_from, id_list, label_from};
use crate::error::{StorageError, StorageResult};
use crate::references::{certain_ends, load_around};
use crate::sql;

/// A symbol as it is stored: its label, visibility and file.
#[derive(Debug, Clone)]
struct Placed {
    label: SymbolLabel,
    visibility: Option<Visibility>,
    path: RelativePath,
}

/// The columns of `SYMBOL_LABELS_OF`, still raw.
struct LabelRow {
    id: i64,
    qualified: String,
    kind: String,
    visibility: Option<String>,
    path: String,
}

// codeskimr: allow CHK-102 maps a row of the label query into its columns
fn read_label(cols: &Cols<'_>) -> rusqlite::Result<LabelRow> {
    let (id, qualified, kind) = (cols.take()?, cols.take()?, cols.take()?);
    let _parent: Option<i64> = cols.take()?;
    Ok(LabelRow {
        id,
        qualified,
        kind,
        visibility: cols.take()?,
        path: cols.take()?,
    })
}

fn decode_place(row: LabelRow) -> StorageResult<(u64, Placed)> {
    let id = id_from(row.id)?;
    let placed = Placed {
        label: SymbolLabel {
            id: SymbolId::new(id),
            qualified_name: QualifiedName::new(row.qualified)?,
            kind: label_from(&row.kind)?,
        },
        visibility: row.visibility.as_deref().map(label_from).transpose()?,
        path: RelativePath::new(row.path)?,
    };
    Ok((id, placed))
}

/// The symbols of a snapshot named in `ids`, by id; one that the snapshot does not hold is missing.
fn places_of(
    conn: &Connection,
    snapshot: i64,
    ids: &HashSet<u64>,
) -> StorageResult<HashMap<u64, Placed>> {
    if ids.is_empty() {
        return Ok(HashMap::new());
    }
    let list = id_list(ids.iter().copied())?;
    let mut prepared = conn.prepare_cached(sql::SYMBOL_LABELS_OF)?;
    let rows = prepared.query_map(params![snapshot, list], |row| read_label(&Cols::new(row)))?;
    rows.map(|row| decode_place(row?)).collect()
}

/// The paths of the files of a snapshot named in `ids`, by id.
fn paths_of(
    conn: &Connection,
    snapshot: i64,
    ids: &HashSet<u64>,
) -> StorageResult<HashMap<u64, RelativePath>> {
    if ids.is_empty() {
        return Ok(HashMap::new());
    }
    let list = id_list(ids.iter().copied())?;
    let mut prepared = conn.prepare_cached(sql::FILE_PATHS_OF)?;
    let rows = prepared.query_map(params![snapshot, list], |row| {
        let cols = Cols::new(row);
        Ok((cols.take::<i64>()?, cols.take::<String>()?))
    })?;
    rows.map(|row| {
        let (id, path) = row?;
        Ok((id_from(id)?, RelativePath::new(path)?))
    })
    .collect()
}

/// The references a walk of `query` can touch, read step by step: the references of the start
/// symbol, then those of the symbols the step reached for the first time, up to the depth. The
/// walk of the `ReverseIndex` over them finds what a walk over the whole snapshot would.
fn walked_references(
    conn: &Connection,
    snapshot: i64,
    query: &UsageQuery,
) -> StorageResult<ReverseIndex> {
    if query.direction == Direction::Incoming && !holds_symbol(conn, snapshot, query.symbol)? {
        return Ok(ReverseIndex::default());
    }
    let mut seen = HashSet::from([query.symbol]);
    let mut frontier = vec![query.symbol];
    let mut all = Vec::new();
    for _ in 0..query.depth {
        if frontier.is_empty() {
            break;
        }
        let found = load_around(conn, snapshot, query.direction, &frontier)?;
        frontier = found
            .iter()
            .filter(|reference| query.include_tests || reference.kind != ReferenceKind::Test)
            .map(|reference| far_end(reference, query.direction))
            .filter(|symbol| seen.insert(*symbol))
            .collect();
        all.extend(found);
    }
    Ok(ReverseIndex::new(all))
}

/// Whether the snapshot holds `symbol`; the references into a symbol of another snapshot do not
/// count.
fn holds_symbol(conn: &Connection, snapshot: i64, symbol: SymbolId) -> StorageResult<bool> {
    let raw = column_int(symbol.get())?;
    let found: i64 = conn
        .prepare_cached(sql::SYMBOL_IN_SNAPSHOT)?
        .query_row(params![snapshot, raw], |row| row.get(0))?;
    Ok(found.is_positive())
}

const fn far_end(reference: &Reference, direction: Direction) -> SymbolId {
    match direction {
        Direction::Incoming => reference.from,
        Direction::Outgoing => reference.to,
    }
}

fn dangling<T>(found: &HashMap<u64, T>, id: u64) -> StorageResult<&T> {
    found.get(&id).ok_or(StorageError::DanglingReference)
}

/// The entries of an answer: both ends and the file of every usage in `shown`.
fn entries(conn: &Connection, snapshot: i64, shown: &[Usage]) -> StorageResult<Vec<UsageEntry>> {
    let symbols: HashSet<u64> = shown
        .iter()
        .flat_map(|usage| [usage.reference.from.get(), usage.reference.to.get()])
        .collect();
    let files: HashSet<u64> = shown
        .iter()
        .map(|usage| usage.reference.file.get())
        .collect();
    let places = places_of(conn, snapshot, &symbols)?;
    let paths = paths_of(conn, snapshot, &files)?;
    shown
        .iter()
        .map(|usage| {
            let reference = usage.reference;
            Ok(UsageEntry {
                depth: usage.depth,
                reference,
                path: dangling(&paths, reference.file.get())?.clone(),
                from: dangling(&places, reference.from.get())?.label.clone(),
                to: dangling(&places, reference.to.get())?.label.clone(),
            })
        })
        .collect()
}

/// The usages of a query: how many there are, and the first `limit` of them as entries.
pub(crate) fn usages(
    conn: &Connection,
    snapshot: i64,
    query: &UsageQuery,
    limit: usize,
) -> StorageResult<(usize, Vec<UsageEntry>)> {
    let found = walked_references(conn, snapshot, query)?.usages(query);
    let shown = found.get(..limit.min(found.len())).unwrap_or_default();
    Ok((found.len(), entries(conn, snapshot, shown)?))
}

fn impacted(found: &Placed, reached: &Reached) -> ImpactedSymbol {
    ImpactedSymbol {
        label: found.label.clone(),
        depth: reached.depth,
        verified: reached.verified,
        visibility: found.visibility,
        path: found.path.clone(),
    }
}

/// The symbols a change reaches: how many there are, and the first `limit` of them.
pub(crate) fn impact(
    conn: &Connection,
    snapshot: i64,
    query: &UsageQuery,
    limit: usize,
) -> StorageResult<(usize, Vec<ImpactedSymbol>)> {
    let reached = walked_references(conn, snapshot, query)?.reached(query);
    let shown = reached.get(..limit.min(reached.len())).unwrap_or_default();
    let ids: HashSet<u64> = shown.iter().map(|found| found.symbol.get()).collect();
    let places = places_of(conn, snapshot, &ids)?;
    let symbols = shown
        .iter()
        .map(|found| Ok(impacted(dangling(&places, found.symbol.get())?, found)))
        .collect::<StorageResult<Vec<_>>>()?;
    Ok((reached.len(), symbols))
}

/// A symbol with what the namespace graph and the type focus read of it.
struct Named {
    id: SymbolId,
    kind: SymbolKind,
    name: QualifiedName,
    parent: Option<SymbolId>,
}

/// The symbols of a snapshot by name; a statement reads them in one pass.
fn named_symbols(conn: &Connection, snapshot: i64) -> StorageResult<Vec<Named>> {
    let mut prepared = conn.prepare_cached(sql::SYMBOL_PARENTS)?;
    let rows = prepared.query_map(params![snapshot], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
    })?;
    rows.map(|row| {
        let (id, name, kind, parent): (i64, String, String, Option<i64>) = row?;
        Ok(Named {
            id: SymbolId::new(id_from(id)?),
            kind: label_from(&kind)?,
            name: QualifiedName::new(name)?,
            parent: parent.map(id_from).transpose()?.map(SymbolId::new),
        })
    })
    .collect()
}

/// The ends (start, target) of the rows of a statement that returns two ids per row.
fn read_ends(
    conn: &Connection,
    statement: &str,
    arguments: impl rusqlite::Params,
) -> StorageResult<Vec<(SymbolId, SymbolId)>> {
    let mut prepared = conn.prepare_cached(statement)?;
    let rows = prepared.query_map(arguments, |row| {
        let cols = Cols::new(row);
        Ok((cols.take::<i64>()?, cols.take::<i64>()?))
    })?;
    rows.map(|row| {
        let (from, to) = row?;
        Ok((SymbolId::new(id_from(from)?), SymbolId::new(id_from(to)?)))
    })
    .collect()
}

/// The namespaces of a snapshot and the references between them. Only certain references count
/// as a dependency of namespaces (issue #464), and references of kind `test` do not, as in the
/// layer graph.
pub(crate) fn namespaces(conn: &Connection, snapshot: i64) -> StorageResult<NamespaceGraph> {
    let symbols = named_symbols(conn, snapshot)?;
    let ends = certain_ends(conn, snapshot)?;
    Ok(namespace_graph(
        symbols
            .iter()
            .map(|found| (found.id, found.kind, &found.name)),
        ends,
    ))
}

fn neighbours(
    places: &HashMap<u64, Placed>,
    links: &[TypeLink],
) -> StorageResult<Vec<TypeNeighbour>> {
    links
        .iter()
        .map(|link| {
            Ok(TypeNeighbour {
                label: dangling(places, link.symbol.get())?.label.clone(),
                count: link.count,
            })
        })
        .collect()
}

/// The owning type of every symbol of a snapshot, for the type focus.
pub(crate) fn type_owners(conn: &Connection, snapshot: i64) -> StorageResult<Arc<TypeOwners>> {
    let symbols = named_symbols(conn, snapshot)?;
    Ok(Arc::new(TypeOwners::new(symbols.iter().map(|found| {
        (found.id, found.kind, &found.name, found.parent)
    }))))
}

/// The types around `symbol`, or `None` if it is no type of the snapshot. The references read
/// are those that touch a member of the type.
pub(crate) fn type_focus(
    conn: &Connection,
    snapshot: i64,
    owners: &TypeOwners,
    symbol: SymbolId,
) -> StorageResult<Option<TypeFocus>> {
    let Some(members) = owners.members(symbol) else {
        return Ok(None);
    };
    let list = id_list(members.iter().map(|member| member.get()))?;
    let ends = read_ends(conn, sql::REFERENCE_ENDS_TOUCHING, params![snapshot, list])?;
    let Some(found) = owners.neighbours(symbol, ends) else {
        return Ok(None);
    };
    let ids: HashSet<u64> = found
        .users
        .iter()
        .chain(&found.used)
        .map(|link| link.symbol.get())
        .chain([symbol.get()])
        .collect();
    let places = places_of(conn, snapshot, &ids)?;
    Ok(Some(TypeFocus {
        focus: dangling(&places, symbol.get())?.label.clone(),
        users: neighbours(&places, &found.users)?,
        used: neighbours(&places, &found.used)?,
    }))
}
