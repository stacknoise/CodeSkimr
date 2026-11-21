//! The `CodeStatistics` port: the symbols of a snapshot with their usage counts and measured
//! values (ADR 0081). The usage counts come from the reverse index that is kept in memory.

use std::collections::HashMap;

use codeskimr_application::error::CodedError;
use codeskimr_application::ports::CodeStatistics;
use codeskimr_domain::{
    FileId, LanguageId, Measures, MetricKind, QualifiedName, RelativePath, SnapshotId, SymbolFacts,
    SymbolId, SymbolName, UsageTally,
};
use rusqlite::{Connection, params};

use crate::codec::{Cols, column_int, id_from, label_from};
use crate::error::StorageResult;
use crate::lookup::SnapshotLookup;
use crate::sql;
use crate::store::SqliteStore;

/// The columns of `STATISTICS_SYMBOLS`, still raw.
struct FactRow {
    id: i64,
    name: String,
    qualified: String,
    kind: String,
    parent: Option<i64>,
    visibility: Option<String>,
    file: i64,
    path: String,
    language: String,
}

// codeskimr: allow CHK-102 maps a row of the statistics query into its columns
fn read_fact(row: &rusqlite::Row<'_>) -> rusqlite::Result<FactRow> {
    let cols = Cols::new(row);
    Ok(FactRow {
        id: cols.take()?,
        name: cols.take()?,
        qualified: cols.take()?,
        kind: cols.take()?,
        parent: cols.take()?,
        visibility: cols.take()?,
        file: cols.take()?,
        path: cols.take()?,
        language: cols.take()?,
    })
}

/// A symbol without usage counts and measured values, and the file that declares it.
fn decode_fact(row: FactRow) -> StorageResult<(SymbolFacts, FileId)> {
    let facts = SymbolFacts {
        id: SymbolId::new(id_from(row.id)?),
        name: SymbolName::new(row.name)?,
        qualified_name: QualifiedName::new(row.qualified)?,
        kind: label_from(&row.kind)?,
        parent: row.parent.map(id_from).transpose()?.map(SymbolId::new),
        visibility: row.visibility.as_deref().map(label_from).transpose()?,
        path: RelativePath::new(row.path)?,
        language: LanguageId::new(row.language)?,
        test: false,
        usage: UsageTally::default(),
        measures: Measures::default(),
    };
    Ok((facts, FileId::new(id_from(row.file)?)))
}

fn load_measures(conn: &Connection, snapshot: i64) -> StorageResult<HashMap<u64, Measures>> {
    let mut statement = conn.prepare_cached(sql::STATISTICS_METRICS)?;
    let mut rows = statement.query(params![snapshot])?;
    let mut measures: HashMap<u64, Measures> = HashMap::new();
    while let Some(row) = rows.next()? {
        let cols = Cols::new(row);
        let symbol = id_from(cols.take::<i64>()?)?;
        let kind: MetricKind = label_from(&cols.take::<String>()?)?;
        let value: f64 = cols.take()?;
        set_measure(measures.entry(symbol).or_default(), kind, value);
    }
    Ok(measures)
}

fn set_measure(measures: &mut Measures, kind: MetricKind, value: f64) {
    let slot = match kind {
        MetricKind::CyclomaticComplexity => &mut measures.complexity,
        MetricKind::Loc => &mut measures.loc,
        MetricKind::MaintainabilityIndex => &mut measures.maintainability,
        MetricKind::EfferentCoupling => &mut measures.efferent,
        MetricKind::AfferentCoupling => &mut measures.afferent,
        MetricKind::CyclomaticComplexityMax | MetricKind::Instability => return,
    };
    *slot = Some(value);
}

fn load_facts(
    conn: &Connection,
    snapshot: i64,
    lookup: &SnapshotLookup,
) -> StorageResult<Vec<SymbolFacts>> {
    let mut measures = load_measures(conn, snapshot)?;
    let tallies = lookup.index.tallies();
    let test_files = lookup.index.test_files();
    let mut statement = conn.prepare_cached(sql::STATISTICS_SYMBOLS)?;
    let rows = statement.query_map(params![snapshot], read_fact)?;
    rows.map(|row| {
        let (mut facts, file) = decode_fact(row?)?;
        facts.test = test_files.contains(&file);
        facts.usage = tallies.get(&facts.id).copied().unwrap_or_default();
        facts.measures = measures.remove(&facts.id.get()).unwrap_or_default();
        Ok(facts)
    })
    .collect()
}

impl CodeStatistics for SqliteStore {
    fn symbol_facts(&self, snapshot: SnapshotId) -> Result<Vec<SymbolFacts>, CodedError> {
        let lookup = self.lookup(snapshot)?;
        let raw = column_int(snapshot.get())?;
        Ok(self.read(|conn| load_facts(conn, raw, &lookup))?)
    }
}
