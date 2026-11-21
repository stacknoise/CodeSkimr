//! The baselines of critical annotations (ADR 0068): what a symbol looked like when the user last
//! accepted it. Shared by the annotations and the alerts.

use codeskimr_domain::{AnnotationId, AnnotationKind, ContentHash, Loc, SymbolBaseline};
use rusqlite::{Connection, OptionalExtension, params};

use crate::codec::{Cols, column_int, label_from, small_from};
use crate::error::{StorageError, StorageResult};
use crate::sql;

/// The kind of an annotation.
fn kind_of(conn: &Connection, id: AnnotationId) -> StorageResult<AnnotationKind> {
    let kind: Option<String> = conn
        .query_row(
            sql::ANNOTATION_KIND,
            params![column_int(id.get())?],
            |row| row.get(0),
        )
        .optional()?;
    kind.as_deref()
        .map(label_from)
        .transpose()?
        .ok_or(StorageError::AnnotationNotFound)
}

/// The annotation must exist and be critical: only those have baselines and alerts.
pub(crate) fn ensure_critical(conn: &Connection, id: AnnotationId) -> StorageResult<()> {
    match kind_of(conn, id)? {
        AnnotationKind::Critical => Ok(()),
        AnnotationKind::Note | AnnotationKind::Task => Err(StorageError::AnnotationInvalid),
    }
}

/// The baseline of an annotation, `None` if none was taken yet.
pub(crate) fn read(conn: &Connection, id: AnnotationId) -> StorageResult<Option<SymbolBaseline>> {
    kind_of(conn, id)?;
    let row: Option<(String, String, i64, i64)> = conn
        .query_row(
            sql::BASELINE_BY_ANNOTATION,
            params![column_int(id.get())?],
            |row| {
                let cols = Cols::new(row);
                Ok((cols.take()?, cols.take()?, cols.take()?, cols.take()?))
            },
        )
        .optional()?;
    row.map(|(content, signature, loc, cc)| {
        Ok(SymbolBaseline {
            content_hash: ContentHash::new(content)?,
            signature_hash: ContentHash::new(signature)?,
            loc: Loc(small_from(loc)?),
            cc: small_from(cc)?,
        })
    })
    .transpose()
}

/// Writes the baseline of an annotation, replacing the old one.
pub(crate) fn write(
    conn: &Connection,
    id: AnnotationId,
    baseline: &SymbolBaseline,
) -> StorageResult<()> {
    conn.execute(
        sql::BASELINE_UPSERT,
        params![
            column_int(id.get())?,
            baseline.content_hash.as_str(),
            baseline.signature_hash.as_str(),
            i64::from(baseline.loc.0),
            i64::from(baseline.cc)
        ],
    )?;
    Ok(())
}
