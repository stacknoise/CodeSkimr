//! The `SymbolStates` port: what the alert step asks of the newest snapshot (ADR 0068).

use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{SameName, SymbolState, SymbolStates};
use codeskimr_domain::{
    ContentHash, Loc, MetricKind, QualifiedName, SnapshotId, SymbolKind, SymbolName, SymbolOrigin,
};
use rusqlite::{Connection, params};

use crate::codec::{Cols, column_int, label_from, small_from};
use crate::error::StorageResult;
use crate::sql;
use crate::store::SqliteStore;

/// The columns of `STATE_SYMBOLS`, still raw.
struct StateRow {
    kind: String,
    name: String,
    signature: Option<String>,
    content_hash: Option<String>,
    loc: i64,
    cc: i64,
}

impl StateRow {
    fn read(cols: &Cols<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            kind: cols.take()?,
            name: cols.take()?,
            signature: cols.take()?,
            content_hash: cols.take()?,
            loc: cols.take()?,
            cc: cols.take()?,
        })
    }

    fn decode(self) -> StorageResult<SymbolState> {
        Ok(SymbolState {
            kind: label_from(&self.kind)?,
            name: QualifiedName::new(self.name)?,
            signature: self.signature,
            content_hash: self.content_hash.map(ContentHash::new).transpose()?,
            loc: Loc(small_from(self.loc)?),
            cc: small_from(self.cc)?,
        })
    }
}

fn states(
    conn: &Connection,
    snapshot: SnapshotId,
    names: &[QualifiedName],
) -> StorageResult<Vec<SymbolState>> {
    let mut statement = conn.prepare_cached(sql::STATE_SYMBOLS)?;
    let mut found = Vec::new();
    for name in names {
        let rows = statement.query_map(
            params![
                column_int(snapshot.get())?,
                name.as_str(),
                SymbolOrigin::Syntax.label(),
                MetricKind::Loc.label(),
                MetricKind::CyclomaticComplexity.label()
            ],
            |row| StateRow::read(&Cols::new(row)),
        )?;
        for row in rows {
            found.push(row?.decode()?);
        }
    }
    Ok(found)
}

fn child_hashes(
    conn: &Connection,
    snapshot: SnapshotId,
    name: &QualifiedName,
) -> StorageResult<Vec<ContentHash>> {
    let mut statement = conn.prepare_cached(sql::STATE_CHILD_HASHES)?;
    let rows = statement.query_map(
        params![
            column_int(snapshot.get())?,
            SymbolKind::Namespace.label(),
            name.as_str()
        ],
        |row| row.get::<_, String>(0),
    )?;
    rows.map(|row| Ok(ContentHash::new(row?)?)).collect()
}

fn remaining(
    conn: &Connection,
    snapshot: SnapshotId,
    (name, simple): (&QualifiedName, &SymbolName),
) -> StorageResult<u32> {
    let count: i64 = conn.prepare_cached(sql::STATE_REMAINING_NAMES)?.query_row(
        params![column_int(snapshot.get())?, name.as_str(), simple.as_str()],
        |row| row.get(0),
    )?;
    small_from(count)
}

fn same_name(
    conn: &Connection,
    snapshot: SnapshotId,
    simple: &SymbolName,
) -> StorageResult<Vec<SameName>> {
    let mut statement = conn.prepare_cached(sql::STATE_SAME_NAME)?;
    let rows = statement.query_map(
        params![
            column_int(snapshot.get())?,
            simple.as_str(),
            SymbolOrigin::Syntax.label()
        ],
        |row| {
            let cols = Cols::new(row);
            Ok((cols.take::<String>()?, cols.take::<String>()?))
        },
    )?;
    rows.map(|row| {
        let (kind, name) = row?;
        Ok(SameName {
            kind: label_from(&kind)?,
            name: QualifiedName::new(name)?,
        })
    })
    .collect()
}

impl SqliteStore {
    /// Makes the indexes of the alert step if the database lacks one (once per database: a scan
    /// that writes symbols keeps them up to date from then on).
    fn ensure_state_indexes(&self) -> StorageResult<()> {
        let present: i64 =
            self.read(|conn| Ok(conn.query_row(sql::STATE_INDEX_PROBE, [], |row| row.get(0))?))?;
        if present >= crate::consts::STATE_INDEX_COUNT {
            return Ok(());
        }
        self.write(|tx| Ok(tx.execute_batch(sql::ENSURE_STATE_INDEXES)?))
    }
}

impl SymbolStates for SqliteStore {
    fn states(
        &self,
        snapshot: SnapshotId,
        names: &[QualifiedName],
    ) -> Result<Vec<SymbolState>, CodedError> {
        if names.is_empty() {
            return Ok(Vec::new());
        }
        self.ensure_state_indexes().map_err(CodedError::from)?;
        self.read(|conn| states(conn, snapshot, names))
            .map_err(CodedError::from)
    }

    fn child_hashes(
        &self,
        snapshot: SnapshotId,
        name: &QualifiedName,
    ) -> Result<Vec<ContentHash>, CodedError> {
        self.ensure_state_indexes().map_err(CodedError::from)?;
        self.read(|conn| child_hashes(conn, snapshot, name))
            .map_err(CodedError::from)
    }

    fn remaining_references(
        &self,
        snapshot: SnapshotId,
        name: &QualifiedName,
        simple: &SymbolName,
    ) -> Result<u32, CodedError> {
        self.ensure_state_indexes().map_err(CodedError::from)?;
        self.read(|conn| remaining(conn, snapshot, (name, simple)))
            .map_err(CodedError::from)
    }

    fn same_simple_name(
        &self,
        snapshot: SnapshotId,
        simple: &SymbolName,
    ) -> Result<Vec<SameName>, CodedError> {
        self.ensure_state_indexes().map_err(CodedError::from)?;
        self.read(|conn| same_name(conn, snapshot, simple))
            .map_err(CodedError::from)
    }
}
