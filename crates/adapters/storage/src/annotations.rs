//! The `AnnotationStore` port: annotations and baselines of a project (ADR 0068). The alerts are in
//! `alerts.rs`. Nothing here belongs to a snapshot, so scans and retention leave it alone.

use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{
    AlertDismissal, AlertFilter, AlertPurge, AlertRaise, AnnotationCreation, AnnotationFilter,
    AnnotationStore, AnnotationUpdate,
};
use codeskimr_domain::{
    Alert, Annotation, AnnotationContent, AnnotationId, AnnotationKind, AnnotationTarget,
    AnnotationText, AnnotationTitle, Assignee, Author, NewAnnotation, ProjectId, QualifiedName,
    SnapshotId, SymbolBaseline, Timestamp,
};
use rusqlite::{Connection, OptionalExtension, Params, Row, params};

use crate::codec::{Cols, column_int, id_from, label_from};
use crate::consts::{NAME_SEPARATOR, NO_ANNOTATION};
use crate::error::{StorageError, StorageResult};
use crate::sql;
use crate::store::ProjectDb;
use crate::{alerts, baselines};

/// One row of `annotation`, as the columns hold it.
struct AnnotationRow {
    id: i64,
    project: i64,
    target_kind: String,
    target_name: String,
    target_params: String,
    kind: String,
    title: Option<String>,
    text: Option<String>,
    status: Option<String>,
    due: Option<i64>,
    assignee: Option<String>,
    author: String,
    created: i64,
    updated: i64,
    without_target: bool,
}

impl AnnotationRow {
    fn read(row: &Row<'_>) -> rusqlite::Result<Self> {
        let cols = Cols::new(row);
        Ok(Self {
            id: cols.take()?,
            project: cols.take()?,
            target_kind: cols.take()?,
            target_name: cols.take()?,
            target_params: cols.take()?,
            kind: cols.take()?,
            title: cols.take()?,
            text: cols.take()?,
            status: cols.take()?,
            due: cols.take()?,
            assignee: cols.take()?,
            author: cols.take()?,
            created: cols.take()?,
            updated: cols.take()?,
            without_target: cols.take()?,
        })
    }

    fn into_annotation(self) -> StorageResult<Annotation> {
        let content = AnnotationContent {
            kind: label_from(&self.kind)?,
            title: self.title.map(AnnotationTitle::restore).transpose()?,
            text: self.text.map(AnnotationText::restore).transpose()?,
            status: self.status.as_deref().map(label_from).transpose()?,
            due: self.due.map(Timestamp),
            assignee: self.assignee.map(Assignee::new).transpose()?,
        };
        let target = AnnotationTarget {
            kind: label_from(&self.target_kind)?,
            name: QualifiedName::new(self.target_name)?,
            parameters: split_parameters(&self.target_params),
        };
        Ok(Annotation {
            id: AnnotationId::new(id_from(self.id)?),
            project: ProjectId::new(id_from(self.project)?),
            target,
            author: Author::new(self.author)?,
            content,
            created: Timestamp(self.created),
            updated: Timestamp(self.updated),
            without_target: self.without_target,
        })
    }
}

fn split_parameters(joined: &str) -> Vec<String> {
    if joined.is_empty() {
        Vec::new()
    } else {
        joined.split(NAME_SEPARATOR).map(str::to_owned).collect()
    }
}

/// The columns of a target.
struct TargetColumns<'a> {
    kind: &'static str,
    name: &'a str,
    parameters: String,
}

impl<'a> TargetColumns<'a> {
    fn of(target: &'a AnnotationTarget) -> Self {
        Self {
            kind: target.kind.label(),
            name: target.name.as_str(),
            parameters: target.parameters.join(&NAME_SEPARATOR.to_string()),
        }
    }
}

/// The columns of the content of an annotation.
struct ContentColumns<'a> {
    kind: &'static str,
    title: Option<&'a str>,
    text: Option<&'a str>,
    status: Option<&'static str>,
    due: Option<i64>,
    assignee: Option<&'a str>,
}

impl<'a> ContentColumns<'a> {
    fn of(content: &'a AnnotationContent) -> Self {
        Self {
            kind: content.kind.label(),
            title: content.title.as_ref().map(AnnotationTitle::as_str),
            text: content.text.as_ref().map(AnnotationText::as_str),
            status: content.status.map(codeskimr_domain::TaskStatus::label),
            due: content.due.map(|due| due.0),
            assignee: content.assignee.as_ref().map(Assignee::as_str),
        }
    }
}

fn ensure_project(conn: &Connection, project: ProjectId) -> StorageResult<()> {
    let id = column_int(project.get())?;
    let found: i64 = conn.query_row(sql::PROJECT_EXISTS, params![id], |row| row.get(0))?;
    if found == 0 {
        return Err(StorageError::ProjectNotFound);
    }
    Ok(())
}

/// The symbol must be in the newest snapshot. Its parameter types are not stored in a snapshot, so
/// kind and qualified name decide.
fn ensure_target(
    conn: &Connection,
    project: ProjectId,
    target: &AnnotationTarget,
) -> StorageResult<()> {
    let id = column_int(project.get())?;
    let found: i64 = conn.query_row(
        sql::ANNOTATION_TARGET_EXISTS,
        params![id, target.kind.label(), target.name.as_str()],
        |row| row.get(0),
    )?;
    if found == 0 {
        return Err(StorageError::TargetNotFound);
    }
    Ok(())
}

/// A symbol has at most one critical annotation, other than `except`.
fn ensure_single_critical(
    conn: &Connection,
    project: ProjectId,
    target: &AnnotationTarget,
    kind: AnnotationKind,
    except: i64,
) -> StorageResult<()> {
    if kind != AnnotationKind::Critical {
        return Ok(());
    }
    let columns = TargetColumns::of(target);
    let others: i64 = conn.query_row(
        sql::ANNOTATION_KIND_COUNT,
        params![
            column_int(project.get())?,
            columns.kind,
            columns.name,
            columns.parameters,
            kind.label(),
            except
        ],
        |row| row.get(0),
    )?;
    if others > 0 {
        return Err(StorageError::CriticalExists);
    }
    Ok(())
}

// codeskimr: allow CHK-102 looks one annotation up by id and maps its row
fn find(conn: &Connection, id: AnnotationId) -> StorageResult<Annotation> {
    conn.query_row(
        sql::ANNOTATION_BY_ID,
        params![column_int(id.get())?],
        AnnotationRow::read,
    )
    .optional()?
    .ok_or(StorageError::AnnotationNotFound)?
    .into_annotation()
}

/// Everything that makes a new annotation unfit to be stored.
fn check_new(conn: &Connection, new: &NewAnnotation) -> StorageResult<()> {
    new.content
        .check()
        .map_err(|_| StorageError::AnnotationInvalid)?;
    ensure_project(conn, new.project)?;
    ensure_target(conn, new.project, &new.target)?;
    ensure_single_critical(
        conn,
        new.project,
        &new.target,
        new.content.kind,
        NO_ANNOTATION,
    )
}

fn insert(conn: &Connection, creation: &AnnotationCreation) -> StorageResult<Annotation> {
    let new = &creation.annotation;
    check_new(conn, new)?;
    let (target, content) = (
        TargetColumns::of(&new.target),
        ContentColumns::of(&new.content),
    );
    conn.execute(
        sql::ANNOTATION_INSERT,
        params![
            column_int(new.project.get())?,
            target.kind,
            target.name,
            target.parameters,
            content.kind,
            content.title,
            content.text,
            content.status,
            content.due,
            content.assignee,
            new.author.as_str(),
            creation.now.0,
            creation.now.0
        ],
    )?;
    Ok(Annotation {
        id: AnnotationId::new(id_from(conn.last_insert_rowid())?),
        project: new.project,
        target: new.target.clone(),
        author: new.author.clone(),
        content: new.content.clone(),
        created: creation.now,
        updated: creation.now,
        without_target: false,
    })
}

/// The new content must fit its kind, and the kind of an annotation never changes.
fn ensure_same_kind(stored: &Annotation, next: &Annotation) -> StorageResult<()> {
    next.content
        .check()
        .map_err(|_| StorageError::AnnotationInvalid)?;
    if next.content.kind != stored.content.kind {
        return Err(StorageError::AnnotationInvalid);
    }
    Ok(())
}

/// Checks the new symbol of an annotation and drops the baseline of the old one.
fn retarget(conn: &Connection, stored: &Annotation, next: &Annotation) -> StorageResult<()> {
    ensure_target(conn, stored.project, &next.target)?;
    let id = column_int(next.id.get())?;
    ensure_single_critical(conn, stored.project, &next.target, next.content.kind, id)?;
    // The baseline describes the old symbol; the next scan takes one of the new symbol.
    conn.execute(sql::BASELINE_DELETE, params![id])?;
    Ok(())
}

fn update(conn: &Connection, update: &AnnotationUpdate) -> StorageResult<Annotation> {
    let next = &update.annotation;
    let stored = find(conn, next.id)?;
    ensure_same_kind(&stored, next)?;
    let id = column_int(next.id.get())?;
    // A target that was just checked against the newest snapshot is there.
    let without_target = next.target == stored.target && stored.without_target;
    if next.target != stored.target {
        retarget(conn, &stored, next)?;
    }
    let (target, content) = (
        TargetColumns::of(&next.target),
        ContentColumns::of(&next.content),
    );
    conn.execute(
        sql::ANNOTATION_UPDATE,
        params![
            id,
            target.kind,
            target.name,
            target.parameters,
            content.title,
            content.text,
            content.status,
            content.due,
            content.assignee,
            update.now.0,
            without_target
        ],
    )?;
    Ok(Annotation {
        without_target,
        project: stored.project,
        author: stored.author,
        created: stored.created,
        updated: update.now,
        ..next.clone()
    })
}

fn rows(conn: &Connection, statement: &str, values: impl Params) -> StorageResult<Vec<Annotation>> {
    let mut prepared = conn.prepare_cached(statement)?;
    let found = prepared
        .query_map(values, AnnotationRow::read)?
        .collect::<Result<Vec<_>, _>>()?;
    found
        .into_iter()
        .map(AnnotationRow::into_annotation)
        .collect()
}

fn list(conn: &Connection, filter: &AnnotationFilter) -> StorageResult<Vec<Annotation>> {
    let project = column_int(filter.project.get())?;
    let kind = filter.kind.map(AnnotationKind::label);
    match &filter.target {
        None => rows(conn, sql::ANNOTATIONS_OF_PROJECT, params![project, kind]),
        Some(target) => {
            let columns = TargetColumns::of(target);
            rows(
                conn,
                sql::ANNOTATIONS_OF_TARGET,
                params![
                    project,
                    kind,
                    columns.kind,
                    columns.name,
                    columns.parameters
                ],
            )
        }
    }
}

fn delete(conn: &Connection, id: AnnotationId) -> StorageResult<()> {
    let removed = conn.execute(sql::ANNOTATION_DELETE, params![column_int(id.get())?])?;
    if removed == 0 {
        return Err(StorageError::AnnotationNotFound);
    }
    Ok(())
}

fn any_annotation(conn: &Connection, project: ProjectId) -> StorageResult<bool> {
    Ok(conn.query_row(
        sql::ANNOTATION_ANY,
        params![column_int(project.get())?],
        |row| row.get(0),
    )?)
}

/// Marks the annotations that the snapshot has no symbol for, and unmarks those it has one for.
fn refresh(conn: &Connection, project: ProjectId, snapshot: SnapshotId) -> StorageResult<u64> {
    let values = params![column_int(project.get())?, column_int(snapshot.get())?];
    let lost = conn.execute(sql::ANNOTATION_LOSE_TARGET, values)?;
    let found = conn.execute(sql::ANNOTATION_FIND_TARGET, values)?;
    u64::try_from(lost + found).map_err(|_| StorageError::OutOfRange)
}

impl AnnotationStore for ProjectDb {
    fn create_annotation(&self, creation: &AnnotationCreation) -> Result<Annotation, CodedError> {
        self.write(|tx| insert(tx, creation))
            .map_err(CodedError::from)
    }

    fn update_annotation(&self, change: &AnnotationUpdate) -> Result<Annotation, CodedError> {
        self.write(|tx| update(tx, change))
            .map_err(CodedError::from)
    }

    fn delete_annotation(&self, id: AnnotationId) -> Result<(), CodedError> {
        self.write(|tx| delete(tx, id)).map_err(CodedError::from)
    }

    fn list_annotations(&self, filter: &AnnotationFilter) -> Result<Vec<Annotation>, CodedError> {
        self.read(|conn| list(conn, filter))
            .map_err(CodedError::from)
    }

    fn refresh_targets(&self, project: ProjectId, snapshot: SnapshotId) -> Result<u64, CodedError> {
        self.read(|conn| any_annotation(conn, project))
            .and_then(|any| {
                if any {
                    self.write(|tx| refresh(tx, project, snapshot))
                } else {
                    Ok(0)
                }
            })
            .map_err(CodedError::from)
    }

    fn baseline(&self, annotation: AnnotationId) -> Result<Option<SymbolBaseline>, CodedError> {
        self.read(|conn| baselines::read(conn, annotation))
            .map_err(CodedError::from)
    }

    fn store_baseline(
        &self,
        annotation: AnnotationId,
        baseline: &SymbolBaseline,
    ) -> Result<(), CodedError> {
        self.write(|tx| {
            baselines::ensure_critical(tx, annotation)?;
            baselines::write(tx, annotation, baseline)
        })
        .map_err(CodedError::from)
    }

    fn raise_alert(&self, raise: &AlertRaise) -> Result<Alert, CodedError> {
        self.write(|tx| alerts::raise(tx, raise))
            .map_err(CodedError::from)
    }

    fn list_alerts(&self, filter: &AlertFilter) -> Result<Vec<Alert>, CodedError> {
        self.read(|conn| alerts::list(conn, filter))
            .map_err(CodedError::from)
    }

    fn dismiss_alert(&self, dismissal: &AlertDismissal) -> Result<Alert, CodedError> {
        self.write(|tx| alerts::dismiss(tx, dismissal))
            .map_err(CodedError::from)
    }

    fn purge_dismissed(&self, purge: &AlertPurge) -> Result<u64, CodedError> {
        self.write(|tx| alerts::purge(tx, purge))
            .map_err(CodedError::from)
    }
}
