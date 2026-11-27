//! The `ComparisonSource` port: the symbols and the naming rules of a retained snapshot (ADR 0097).
//!
//! Both reads run apart from the shared connection, as one read transaction each, so a scan that
//! writes meanwhile is not held up and the comparison never sees half of a snapshot.

use codeskimr_application::error::CodedError;
use codeskimr_application::ports::ComparisonSource;
use codeskimr_domain::{
    ContentHash, MetricFact, MetricKind, MetricLevel, MetricValues, QualifiedName, RelativePath,
    SnapshotId, SymbolFact, SymbolId, SymbolKind, SymbolOrigin, Visibility, compares_kind,
};
use rusqlite::{Connection, OptionalExtension, params};

use crate::codec::{Cols, column_int, id_from, json_labels, label_from};
use crate::error::{StorageError, StorageResult};
use crate::sql;
use crate::store::ProjectDb;

/// The columns of `COMPARISON_FACTS`, still raw.
struct FactRow {
    id: i64,
    kind: String,
    name: String,
    signature: Option<String>,
    visibility: Option<String>,
    path: String,
    content_hash: Option<String>,
}

impl FactRow {
    // codeskimr: allow CHK-102 maps the columns of a fact row into its fields
    fn read(cols: &Cols<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: cols.take()?,
            kind: cols.take()?,
            name: cols.take()?,
            signature: cols.take()?,
            visibility: cols.take()?,
            path: cols.take()?,
            content_hash: cols.take()?,
        })
    }

    fn decode(self) -> StorageResult<SymbolFact> {
        Ok(SymbolFact {
            id: SymbolId::new(id_from(self.id)?),
            kind: label_from::<SymbolKind>(&self.kind)?,
            qualified_name: QualifiedName::new(self.name)?,
            signature: self.signature,
            visibility: self
                .visibility
                .as_deref()
                .map(label_from::<Visibility>)
                .transpose()?,
            path: RelativePath::new(self.path)?,
            content_hash: self.content_hash.map(ContentHash::new).transpose()?,
        })
    }
}

fn facts(conn: &Connection, snapshot: SnapshotId) -> StorageResult<Vec<SymbolFact>> {
    let kinds = json_labels(
        SymbolKind::ALL
            .iter()
            .filter(|kind| compares_kind(**kind))
            .map(|kind| kind.label()),
    );
    let mut statement = conn.prepare_cached(sql::COMPARISON_FACTS)?;
    let rows = statement.query_map(
        params![
            column_int(snapshot.get())?,
            kinds,
            SymbolOrigin::Syntax.label()
        ],
        |row| FactRow::read(&Cols::new(row)),
    )?;
    rows.map(|row| row.map_err(StorageError::from)?.decode())
        .collect()
}

/// The columns of `METRIC_FACTS`, still raw: one object and one of its metrics.
struct MetricRow {
    id: i64,
    kind: String,
    name: String,
    signature: Option<String>,
    path: String,
    metric: String,
    value: f64,
}

impl MetricRow {
    // codeskimr: allow CHK-102 maps the columns of a metric row into its fields
    fn read(cols: &Cols<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: cols.take()?,
            kind: cols.take()?,
            name: cols.take()?,
            signature: cols.take()?,
            path: cols.take()?,
            metric: cols.take()?,
            value: cols.take()?,
        })
    }

    fn fact(&self) -> StorageResult<MetricFact> {
        Ok(MetricFact {
            kind: label_from::<SymbolKind>(&self.kind)?,
            qualified_name: QualifiedName::new(self.name.clone())?,
            signature: self.signature.clone(),
            path: RelativePath::new(self.path.clone())?,
            values: MetricValues::default(),
        })
    }
}

/// Adds the metric of `row` to its object, which starts a new one when the id changes. The rows
/// come ordered by object, so the rows of one object follow each other.
fn add_metric(found: &mut Vec<(i64, MetricFact)>, row: &MetricRow) -> StorageResult<()> {
    if found.last().is_none_or(|(id, _)| *id != row.id) {
        found.push((row.id, row.fact()?));
    }
    if let (Some((_, fact)), Ok(metric)) = (found.last_mut(), label_from::<MetricKind>(&row.metric))
    {
        fact.values.set(metric, row.value);
    }
    Ok(())
}

fn metric_facts(conn: &Connection, snapshot: SnapshotId) -> StorageResult<Vec<MetricFact>> {
    let kinds = json_labels(
        SymbolKind::ALL
            .iter()
            .filter(|kind| MetricLevel::of(**kind).is_some())
            .map(|kind| kind.label()),
    );
    let mut statement = conn.prepare_cached(sql::METRIC_FACTS)?;
    let rows = statement.query_map(
        params![
            column_int(snapshot.get())?,
            kinds,
            SymbolOrigin::Syntax.label()
        ],
        |row| MetricRow::read(&Cols::new(row)),
    )?;
    let mut found = Vec::new();
    for row in rows {
        add_metric(&mut found, &row?)?;
    }
    Ok(found.into_iter().map(|(_, fact)| fact).collect())
}

fn naming_rules(conn: &Connection, snapshot: SnapshotId) -> StorageResult<Option<String>> {
    let found = conn
        .prepare_cached(sql::SNAPSHOT_NAMING_OF)?
        .query_row(params![column_int(snapshot.get())?], |row| {
            row.get::<_, Option<String>>(0)
        })
        .optional()?;
    Ok(found.flatten())
}

impl ComparisonSource for ProjectDb {
    fn comparison_facts(&self, snapshot: SnapshotId) -> Result<Vec<SymbolFact>, CodedError> {
        self.read_apart(|conn| facts(conn, snapshot))
            .map_err(CodedError::from)
    }

    fn metric_facts(&self, snapshot: SnapshotId) -> Result<Vec<MetricFact>, CodedError> {
        self.read_apart(|conn| metric_facts(conn, snapshot))
            .map_err(CodedError::from)
    }

    fn snapshot_naming_rules(&self, snapshot: SnapshotId) -> Result<Option<String>, CodedError> {
        self.read_apart(|conn| naming_rules(conn, snapshot))
            .map_err(CodedError::from)
    }
}
