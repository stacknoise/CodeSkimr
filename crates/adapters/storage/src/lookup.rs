//! What a usages query needs from one snapshot, loaded once and kept in memory.

use std::collections::HashMap;

use codeskimr_application::ports::{ImpactedSymbol, SymbolLabel, UsageEntry};
use codeskimr_domain::{
    NamespaceGraph, QualifiedName, RelativePath, ReverseIndex, SymbolId, Visibility,
    namespace_graph,
};
use rusqlite::{Connection, params};

use crate::codec::{Cols, id_from, label_from};
use crate::error::{StorageError, StorageResult};
use crate::references::load_references;
use crate::sql;

/// A symbol as it is stored: its label, visibility and file.
#[derive(Debug, Clone)]
struct Placed {
    label: SymbolLabel,
    visibility: Option<Visibility>,
    path: RelativePath,
}

/// What a query needs from one snapshot: the reverse index, and the places of all symbols and
/// the paths of all files, so that the entries of an answer are looked up in memory and not one
/// by one in the database. Built once per snapshot, which never changes.
#[derive(Debug, Default)]
pub(crate) struct SnapshotLookup {
    pub(crate) index: ReverseIndex,
    places: HashMap<u64, Placed>,
    files: HashMap<u64, RelativePath>,
    pub(crate) namespaces: NamespaceGraph,
}

/// The columns of `SYMBOL_LABELS`, still raw.
struct LabelRow {
    id: i64,
    qualified: String,
    kind: String,
    visibility: Option<String>,
    path: String,
}

fn read_label(cols: &Cols<'_>) -> rusqlite::Result<LabelRow> {
    Ok(LabelRow {
        id: cols.take()?,
        qualified: cols.take()?,
        kind: cols.take()?,
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

/// Runs `statement` for `snapshot` and collects the decoded rows into a map by id.
fn load_map<R, V>(
    conn: &Connection,
    (statement, snapshot): (&str, i64),
    read: impl Fn(&Cols<'_>) -> rusqlite::Result<R>,
    decode: impl Fn(R) -> StorageResult<(u64, V)>,
) -> StorageResult<HashMap<u64, V>> {
    let mut prepared = conn.prepare_cached(statement)?;
    let rows = prepared.query_map(params![snapshot], |row| read(&Cols::new(row)))?;
    rows.map(|row| decode(row?)).collect()
}

fn decode_file(row: (i64, String)) -> StorageResult<(u64, RelativePath)> {
    Ok((id_from(row.0)?, RelativePath::new(row.1)?))
}

impl SnapshotLookup {
    pub(crate) fn load(conn: &Connection, snapshot: i64) -> StorageResult<Self> {
        let references = load_references(conn, snapshot)?;
        let places = load_map(
            conn,
            (sql::SYMBOL_LABELS, snapshot),
            read_label,
            decode_place,
        )?;
        let namespaces = namespace_graph(
            places.values().map(|found| {
                (
                    found.label.id,
                    found.label.kind,
                    &found.label.qualified_name,
                )
            }),
            references.iter().map(|found| (found.from, found.to)),
        );
        Ok(Self {
            index: ReverseIndex::new(references),
            places,
            files: load_map(
                conn,
                (sql::FILE_PATHS, snapshot),
                |cols| Ok((cols.take()?, cols.take()?)),
                decode_file,
            )?,
            namespaces,
        })
    }

    fn placed(&self, symbol: SymbolId) -> StorageResult<&Placed> {
        self.places
            .get(&symbol.get())
            .ok_or(StorageError::DanglingReference)
    }

    pub(crate) fn entry(&self, usage: &codeskimr_domain::Usage) -> StorageResult<UsageEntry> {
        let reference = usage.reference;
        Ok(UsageEntry {
            depth: usage.depth,
            reference,
            path: self
                .files
                .get(&reference.file.get())
                .cloned()
                .ok_or(StorageError::DanglingReference)?,
            from: self.placed(reference.from)?.label.clone(),
            to: self.placed(reference.to)?.label.clone(),
        })
    }

    pub(crate) fn impacted(
        &self,
        reached: &codeskimr_domain::Reached,
    ) -> StorageResult<ImpactedSymbol> {
        let found = self.placed(reached.symbol)?;
        Ok(ImpactedSymbol {
            label: found.label.clone(),
            depth: reached.depth,
            visibility: found.visibility,
            path: found.path.clone(),
        })
    }
}
