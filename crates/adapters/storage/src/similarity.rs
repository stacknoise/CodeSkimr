//! The fingerprints of a snapshot and the `SimilarityIndex` port.
//!
//! The same Jaccard measure as `Fingerprints::similarity` and the duplicate check, computed from
//! counts: the hashes of the query find the functions that share any of them, the sizes of those
//! functions complete the union.

use std::collections::HashMap;

use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{SimilarHit, SimilarQuery, SimilarityIndex, SymbolFingerprints};
use codeskimr_domain::{
    Fingerprint, Fingerprints, QualifiedName, RelativePath, SnapshotId, SymbolId,
};
use rusqlite::{Connection, params};

use crate::codec::{Cols, SpanColumns, column_int, id_from, label_from, small_from};
use crate::consts::{
    HASH_BYTES, JSON_LIST_CLOSE, JSON_LIST_OPEN, JSON_LIST_SEPARATOR, PERCENT_BASE,
};
use crate::error::StorageResult;
use crate::sql;
use crate::store::SqliteStore;

/// A 64-bit hash as the signed integer of its column, bit for bit.
fn to_column(hash: u64) -> i64 {
    i64::from_ne_bytes(hash.to_ne_bytes())
}

/// The hash of a column value.
fn from_column(value: i64) -> u64 {
    let bytes: [u8; HASH_BYTES] = value.to_ne_bytes();
    u64::from_ne_bytes(bytes)
}

/// Writes the fingerprints of the analysed callables.
pub fn insert_all(conn: &Connection, fingerprints: &[SymbolFingerprints]) -> StorageResult<()> {
    let mut statement = conn.prepare_cached(sql::FINGERPRINT_INSERT)?;
    for found in fingerprints {
        let symbol = column_int(found.symbol.get())?;
        for entry in found.fingerprints.entries() {
            statement.execute(params![
                symbol,
                i64::from(entry.position),
                to_column(entry.hash)
            ])?;
        }
    }
    Ok(())
}

/// A list of numbers as the JSON text that `json_each` reads, so one statement takes any number
/// of values.
fn json_list(values: impl Iterator<Item = i64>) -> String {
    let items: Vec<String> = values.map(|value| value.to_string()).collect();
    format!(
        "{JSON_LIST_OPEN}{}{JSON_LIST_CLOSE}",
        items.join(&JSON_LIST_SEPARATOR.to_string())
    )
}

fn stored(conn: &Connection, snapshot: i64, symbol: SymbolId) -> StorageResult<Fingerprints> {
    let mut statement = conn.prepare_cached(sql::FINGERPRINTS_OF)?;
    let rows = statement.query_map(params![snapshot, column_int(symbol.get())?], |row| {
        let cols = Cols::new(row);
        Ok((cols.take::<i64>()?, cols.take::<i64>()?))
    })?;
    let mut entries = Vec::new();
    for row in rows {
        let (position, hash) = row?;
        entries.push(Fingerprint {
            hash: from_column(hash),
            position: small_from(position)?,
        });
    }
    Ok(Fingerprints::from_entries(entries))
}

/// Counts per symbol from a statement that returns `(symbol, count)` rows.
fn counts(
    conn: &Connection,
    statement: &str,
    snapshot: i64,
    list: &str,
) -> StorageResult<HashMap<i64, u64>> {
    let mut prepared = conn.prepare_cached(statement)?;
    let rows = prepared.query_map(params![snapshot, list], |row| {
        let cols = Cols::new(row);
        Ok((cols.take::<i64>()?, cols.take::<i64>()?))
    })?;
    let mut found = HashMap::new();
    for row in rows {
        let (symbol, count) = row?;
        found.insert(symbol, id_from(count)?);
    }
    Ok(found)
}

/// A candidate: a stored function and the percent it resembles the query.
fn rank(
    shared: &HashMap<i64, u64>,
    sizes: &HashMap<i64, u64>,
    query_size: u64,
    minimum_percent: u32,
) -> Vec<(i64, u32)> {
    let mut ranked: Vec<(i64, u32)> = shared
        .iter()
        .filter_map(|(symbol, common)| {
            let size = sizes.get(symbol)?;
            let total = query_size + size - common;
            let percent = u32::try_from(common.saturating_mul(PERCENT_BASE) / total).ok()?;
            (percent >= minimum_percent).then_some((*symbol, percent))
        })
        .collect();
    ranked.sort_by_key(|(symbol, percent)| (std::cmp::Reverse(*percent), *symbol));
    ranked
}

fn labels(conn: &Connection, snapshot: i64, list: &str) -> StorageResult<HashMap<i64, Label>> {
    let mut statement = conn.prepare_cached(sql::SIMILAR_LABELS)?;
    let rows = statement.query_map(params![snapshot, list], |row| {
        let cols = Cols::new(row);
        Ok((
            cols.take::<i64>()?,
            Label {
                qualified: cols.take()?,
                kind: cols.take()?,
                path: cols.take()?,
                span: SpanColumns::read(&cols)?,
            },
        ))
    })?;
    let mut found = HashMap::new();
    for row in rows {
        let (symbol, label) = row?;
        found.insert(symbol, label);
    }
    Ok(found)
}

/// The columns of `SIMILAR_LABELS`, still raw.
struct Label {
    qualified: String,
    kind: String,
    path: String,
    span: SpanColumns,
}

fn hit(symbol: i64, percent: u32, label: &Label) -> StorageResult<SimilarHit> {
    Ok(SimilarHit {
        symbol: SymbolId::new(id_from(symbol)?),
        qualified_name: QualifiedName::new(label.qualified.clone())?,
        kind: label_from(&label.kind)?,
        path: RelativePath::new(label.path.clone())?,
        span: label.span.span()?,
        percent,
    })
}

fn similar(
    conn: &Connection,
    snapshot: SnapshotId,
    query: &SimilarQuery<'_>,
) -> StorageResult<Vec<SimilarHit>> {
    let snapshot = column_int(snapshot.get())?;
    let prints = query.fingerprints;
    let hashes = json_list(prints.entries().iter().map(|entry| to_column(entry.hash)));
    let mut shared = counts(conn, sql::FINGERPRINTS_SHARED, snapshot, &hashes)?;
    if let Some(excluded) = query.exclude {
        shared.remove(&column_int(excluded.get())?);
    }
    let sizes = counts(
        conn,
        sql::FINGERPRINTS_SIZES,
        snapshot,
        &json_list(shared.keys().copied()),
    )?;
    let distinct = u64::try_from(distinct_hashes(prints)).unwrap_or(u64::MAX);
    let mut ranked = rank(&shared, &sizes, distinct, query.minimum_percent);
    ranked.truncate(usize::try_from(query.limit).unwrap_or(usize::MAX));
    let found = labels(
        conn,
        snapshot,
        &json_list(ranked.iter().map(|(symbol, _)| *symbol)),
    )?;
    ranked
        .iter()
        .filter_map(|(symbol, percent)| Some(hit(*symbol, *percent, found.get(symbol)?)))
        .collect()
}

fn distinct_hashes(query: &Fingerprints) -> usize {
    let mut hashes: Vec<u64> = query.entries().iter().map(|entry| entry.hash).collect();
    hashes.sort_unstable();
    hashes.dedup();
    hashes.len()
}

impl SimilarityIndex for SqliteStore {
    fn fingerprints_of(
        &self,
        snapshot: SnapshotId,
        symbol: SymbolId,
    ) -> Result<Option<Fingerprints>, CodedError> {
        self.read(|conn| {
            let found = stored(conn, column_int(snapshot.get())?, symbol)?;
            Ok((!found.is_empty()).then_some(found))
        })
        .map_err(CodedError::from)
    }

    fn similar(
        &self,
        snapshot: SnapshotId,
        query: &SimilarQuery<'_>,
    ) -> Result<Vec<SimilarHit>, CodedError> {
        self.read(|conn| similar(conn, snapshot, query))
            .map_err(CodedError::from)
    }
}
