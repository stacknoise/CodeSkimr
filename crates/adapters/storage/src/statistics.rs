//! The symbols of a snapshot with their usage counts and measured values, counted from the
//! references in the database in one pass (ADR 0081, ADR 0089). The scan counts them once and
//! stores the numbers (`symbol_statistics`, ADR 0091); this is the calculation they are made by.

use std::collections::HashMap;

use codeskimr_domain::{
    FileId, LanguageId, Measures, MetricKind, QualifiedName, RelativePath, SymbolFacts, SymbolId,
    SymbolName, UsageTally,
};
use rusqlite::{Connection, params};

use crate::codec::{Cols, id_from, label_from};
use crate::error::StorageResult;
use crate::references::reference_facts;
use crate::sql;

/// The columns of `STATISTICS_SYMBOLS`, still raw.
pub(crate) struct FactRow {
    pub(crate) id: i64,
    pub(crate) name: String,
    pub(crate) qualified: String,
    pub(crate) kind: String,
    pub(crate) parent: Option<i64>,
    pub(crate) visibility: Option<String>,
    pub(crate) file: i64,
    pub(crate) path: String,
    pub(crate) language: String,
}

// codeskimr: allow CHK-102 maps a row of the statistics query into its columns
pub(crate) fn read_fact(cols: &Cols<'_>) -> rusqlite::Result<FactRow> {
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
pub(crate) fn decode_fact(row: FactRow) -> StorageResult<(SymbolFacts, FileId)> {
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
        overrides: false,
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

pub(crate) fn load_facts(conn: &Connection, snapshot: i64) -> StorageResult<Vec<SymbolFacts>> {
    let mut measures = load_measures(conn, snapshot)?;
    let counted = reference_facts(conn, snapshot)?;
    let mut statement = conn.prepare_cached(sql::STATISTICS_SYMBOLS)?;
    let rows = statement.query_map(params![snapshot], |row| read_fact(&Cols::new(row)))?;
    rows.map(|row| {
        let (mut facts, file) = decode_fact(row?)?;
        facts.test = counted.test_files.contains(&file);
        facts.overrides = counted.overriding.contains(&facts.id);
        facts.usage = counted.tallies.get(&facts.id).copied().unwrap_or_default();
        facts.measures = measures.remove(&facts.id.get()).unwrap_or_default();
        Ok(facts)
    })
    .collect()
}
