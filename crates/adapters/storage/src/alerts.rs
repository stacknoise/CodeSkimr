//! The alerts of the `AnnotationStore` port (ADR 0068): at most one open alert per annotation, kept
//! after its dismissal for the retention time.

use codeskimr_application::ports::{AlertDismissal, AlertFilter, AlertPurge, AlertRaise};
use codeskimr_domain::{
    Alert, AlertChange, AlertId, AlertSighting, AnnotationId, CommitHash, ContentHash, Loc,
    QualifiedName, SnapshotId, SymbolBaseline, Timestamp,
};
use rusqlite::{Connection, OptionalExtension, Row, params};

use crate::baselines;
use crate::codec::{Cols, column_int, id_from, label_from, small_from};
use crate::consts::{CHANGE_SEPARATOR, SECONDS_PER_DAY};
use crate::error::{StorageError, StorageResult};
use crate::sql;

/// One row of `alert`, as the columns hold it.
struct AlertRow {
    id: i64,
    annotation: i64,
    state: String,
    counter: i64,
    first_seen: i64,
    last_seen: i64,
    dismissed: Option<i64>,
    kind: String,
    snapshot: i64,
    commit: Option<String>,
    trigger: String,
    changes: String,
    references: Option<i64>,
    closest: Option<String>,
    seen: Option<(String, String, i64, i64)>,
}

impl AlertRow {
    fn read(row: &Row<'_>) -> rusqlite::Result<Self> {
        let cols = Cols::new(row);
        Ok(Self {
            id: cols.take()?,
            annotation: cols.take()?,
            state: cols.take()?,
            counter: cols.take()?,
            first_seen: cols.take()?,
            last_seen: cols.take()?,
            dismissed: cols.take()?,
            kind: cols.take()?,
            snapshot: cols.take()?,
            commit: cols.take()?,
            trigger: cols.take()?,
            changes: cols.take()?,
            references: cols.take()?,
            closest: cols.take()?,
            seen: seen_from(cols.take()?, cols.take()?, cols.take()?, cols.take()?),
        })
    }

    fn into_alert(self) -> StorageResult<Alert> {
        let sighting = AlertSighting {
            kind: label_from(&self.kind)?,
            snapshot: SnapshotId::new(id_from(self.snapshot)?),
            commit: self.commit.map(CommitHash::new).transpose()?,
            trigger: label_from(&self.trigger)?,
            changes: split_changes(&self.changes)?,
            references: self.references.map(small_from).transpose()?,
            closest: self.closest.map(QualifiedName::new).transpose()?,
            observed: self.seen.map(observed_from).transpose()?,
        };
        Ok(Alert {
            id: AlertId::new(id_from(self.id)?),
            annotation: AnnotationId::new(id_from(self.annotation)?),
            state: label_from(&self.state)?,
            counter: small_from(self.counter)?,
            first_seen: Timestamp(self.first_seen),
            last_seen: Timestamp(self.last_seen),
            dismissed: self.dismissed.map(Timestamp),
            sighting,
        })
    }
}

/// The four columns of the state a `changed` sighting saw; all four are set or none.
fn seen_from(
    content: Option<String>,
    signature: Option<String>,
    loc: Option<i64>,
    cc: Option<i64>,
) -> Option<(String, String, i64, i64)> {
    Some((content?, signature?, loc?, cc?))
}

fn observed_from(
    (content, signature, loc, cc): (String, String, i64, i64),
) -> StorageResult<SymbolBaseline> {
    Ok(SymbolBaseline {
        content_hash: ContentHash::new(content)?,
        signature_hash: ContentHash::new(signature)?,
        loc: Loc(small_from(loc)?),
        cc: small_from(cc)?,
    })
}

fn split_changes(joined: &str) -> StorageResult<Vec<AlertChange>> {
    if joined.is_empty() {
        return Ok(Vec::new());
    }
    joined.split(CHANGE_SEPARATOR).map(label_from).collect()
}

fn join_changes(changes: &[AlertChange]) -> String {
    let labels: Vec<&str> = changes.iter().copied().map(AlertChange::label).collect();
    labels.join(&CHANGE_SEPARATOR.to_string())
}

fn find(conn: &Connection, id: AlertId) -> StorageResult<Alert> {
    conn.query_row(
        sql::ALERT_BY_ID,
        params![column_int(id.get())?],
        AlertRow::read,
    )
    .optional()?
    .ok_or(StorageError::AlertNotFound)?
    .into_alert()
}

/// Counts the open alert of the annotation once more (or only updates it if the raise says it is not
/// to be counted), or opens one.
pub(crate) fn raise(conn: &Connection, raise: &AlertRaise) -> StorageResult<Alert> {
    baselines::ensure_critical(conn, raise.annotation)?;
    let sighting = &raise.sighting;
    let annotation = column_int(raise.annotation.get())?;
    let changes = join_changes(&sighting.changes);
    let snapshot = column_int(sighting.snapshot.get())?;
    let values = params![
        annotation,
        raise.now.0,
        sighting.kind.label(),
        snapshot,
        sighting.commit.as_ref().map(CommitHash::as_str),
        sighting.trigger.label(),
        changes,
        sighting.references,
        sighting.closest.as_ref().map(QualifiedName::as_str),
        sighting
            .observed
            .as_ref()
            .map(|seen| seen.content_hash.as_str()),
        sighting
            .observed
            .as_ref()
            .map(|seen| seen.signature_hash.as_str()),
        sighting.observed.as_ref().map(|seen| i64::from(seen.loc.0)),
        sighting.observed.as_ref().map(|seen| i64::from(seen.cc))
    ];
    let update = if raise.counted {
        sql::ALERT_RAISE_OPEN
    } else {
        sql::ALERT_UPDATE_OPEN
    };
    if conn.execute(update, values)? == 0 {
        conn.execute(sql::ALERT_INSERT, values)?;
    }
    conn.query_row(
        sql::ALERT_OPEN_OF_ANNOTATION,
        params![annotation],
        AlertRow::read,
    )?
    .into_alert()
}

/// The alerts of a project, newest sighting first.
pub(crate) fn list(conn: &Connection, filter: &AlertFilter) -> StorageResult<Vec<Alert>> {
    let mut statement = conn.prepare_cached(sql::ALERTS_OF_PROJECT)?;
    let state = filter.state.map(codeskimr_domain::AlertState::label);
    let found = statement
        .query_map(
            params![column_int(filter.project.get())?, state],
            AlertRow::read,
        )?
        .collect::<Result<Vec<_>, _>>()?;
    found.into_iter().map(AlertRow::into_alert).collect()
}

/// Dismisses an alert and takes the current state of the symbol as the new baseline. An alert that
/// is dismissed already stays as it is.
pub(crate) fn dismiss(conn: &Connection, dismissal: &AlertDismissal) -> StorageResult<Alert> {
    let before = find(conn, dismissal.alert)?;
    let id = column_int(dismissal.alert.get())?;
    let changed = conn.execute(sql::ALERT_DISMISS, params![id, dismissal.now.0])?;
    if let (true, Some(baseline)) = (changed > 0, &dismissal.baseline) {
        baselines::write(conn, before.annotation, baseline)?;
    }
    find(conn, dismissal.alert)
}

/// Deletes the dismissed alerts that are older than the retention.
pub(crate) fn purge(conn: &Connection, purge: &AlertPurge) -> StorageResult<u64> {
    let age = i64::from(purge.retention_days).saturating_mul(SECONDS_PER_DAY);
    let cutoff = purge.now.0.saturating_sub(age);
    let removed = conn.execute(sql::ALERT_PURGE, params![cutoff])?;
    u64::try_from(removed).map_err(|_| StorageError::OutOfRange)
}
