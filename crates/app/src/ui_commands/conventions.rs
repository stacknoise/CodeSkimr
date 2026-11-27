//! The command behind the screen "Conventions": the convention check of the
//! console (`codeskimr check`) for a project that is a folder on this machine (ADR 0116).
//!
//! The check reads the files as they are now, so it needs the right to read source, like the
//! MCP tool `check_conventions` (ADR 0035). It stores nothing and takes no path from the caller:
//! the folder is the one the project was registered with.

use std::collections::BTreeSet;
use std::sync::{Arc, Mutex, PoisonError};

use codeskimr_application::{
    access,
    error::CodedError,
    ports::{CodeIndex, ProjectRepository},
    registry::{Command, CommandDescriptor, CommandId, Request},
};
use codeskimr_cli::GlobalOptions;
use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_domain::{Permission, ProjectId, ProjectSource, SnapshotId};
use codeskimr_mcp::conventions_answer;
use serde::Deserialize;
use serde_json::{Value, json};

use super::consts::{
    CONVENTIONS_CHECK, CONVENTIONS_CHECK_DESCRIPTION, CONVENTIONS_CHECK_TITLE,
    CONVENTIONS_FINDINGS, CONVENTIONS_SYMBOL, CONVENTIONS_SYMBOL_REF,
};
use super::rules_symbols::resolve;
use crate::check::check_folder;
use crate::locations::Locations;

/// The input of `conventions.check`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckInput {
    /// Id of the project.
    pub project: u64,
}

/// What the check needs besides the project: the settings of the program.
pub struct ConventionsPorts {
    /// The registered projects.
    pub(super) projects: Arc<dyn ProjectRepository>,
    /// The index, which names the symbol of the newest scan that a finding stands in.
    pub(super) index: Arc<dyn CodeIndex>,
    /// The options of the command line, which the configuration is resolved with.
    pub(super) global: GlobalOptions,
    /// Where the language packs and the settings are.
    pub(super) locations: Locations,
    /// The projects being checked right now: a project is checked by one run at a time.
    pub(super) running: Runs,
}

/// The projects with a running check; a second check of the same project is refused (`PRJ-304`),
/// so a double click or a second tab cannot start several full analyses of one folder.
#[derive(Debug, Default)]
pub struct Runs {
    projects: Arc<Mutex<BTreeSet<u64>>>,
}

/// Marks a project as being checked until it is dropped.
#[derive(Debug)]
pub struct RunGuard {
    projects: Arc<Mutex<BTreeSet<u64>>>,
    project: u64,
}

impl Runs {
    /// Marks `project` as being checked.
    ///
    /// # Errors
    /// `PRJ-304` if a check of the project is running already.
    pub fn begin(&self, project: u64) -> Result<RunGuard, CodedError> {
        let fresh = self
            .projects
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(project);
        if !fresh {
            return Err(CodedError::new(ErrorCode::CheckInProgress));
        }
        Ok(RunGuard {
            projects: Arc::clone(&self.projects),
            project,
        })
    }
}

impl Drop for RunGuard {
    fn drop(&mut self) {
        self.projects
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .remove(&self.project);
    }
}

/// Checks the files of a folder project against the code rules.
pub struct ConventionsCheck {
    pub(super) ports: Arc<ConventionsPorts>,
}

impl Command for ConventionsCheck {
    type Input = CheckInput;
    type Output = Value;

    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new(CONVENTIONS_CHECK),
        title_key: CONVENTIONS_CHECK_TITLE,
        description_key: CONVENTIONS_CHECK_DESCRIPTION,
        shortcut: None,
        read_only: true,
        permission: Permission::ReadSource,
    };

    /// # Errors
    /// - `AUTH-401` without the right to read source.
    /// - `PRJ-304` if a check of the project is running already.
    /// - `PRJ-201` for an unknown project, `PRJ-101` for a Git project (its files live in the
    ///   cache of the scan, not in a folder of the person).
    /// - The coded failure of the configuration, the language packs or the file discovery.
    fn execute(&self, request: Request<'_, Self>) -> Result<Value, CodedError> {
        access::require(request.principal, Permission::ReadSource)?;
        let project = self
            .ports
            .projects
            .project(ProjectId::new(request.input.project))?;
        let ProjectSource::Folder(root) = &project.source else {
            return Err(CodedError::new(ErrorCode::ProjectSourceInvalid));
        };
        let _run = self.ports.running.begin(request.input.project)?;
        let checked = check_folder(
            root,
            &[],
            *request.principal,
            (&self.ports.global, &self.ports.locations),
        )?;
        let mut answer =
            conventions_answer(checked.report, checked.max_limit).map_err(CodedError::caused_by)?;
        let snapshot = self
            .ports
            .index
            .latest_snapshot(project.id)
            .inspect_err(|error| tracing::warn!(code = error.code().id()))
            .ok()
            .flatten()
            .map(|snapshot| snapshot.id);
        attach_symbols(self.ports.index.as_ref(), snapshot, &mut answer);
        Ok(answer)
    }
}

/// Sets `symbol_ref` of every finding to the symbol of the newest scan with the same qualified
/// name, so the screen can show the function in the explorer. It is `null` if there is no scan, if
/// the index could not be read (the check itself is not lost for that) or if the scan has no such
/// symbol, for instance for a file written after it.
fn attach_symbols(index: &dyn CodeIndex, snapshot: Option<SnapshotId>, answer: &mut Value) {
    let Some(findings) = answer
        .get_mut(CONVENTIONS_FINDINGS)
        .and_then(Value::as_array_mut)
    else {
        return;
    };
    for finding in findings {
        let found = finding
            .get(CONVENTIONS_SYMBOL)
            .and_then(Value::as_str)
            .zip(snapshot)
            .and_then(|(name, snapshot)| resolve(index, snapshot, name));
        finding[CONVENTIONS_SYMBOL_REF] = found.map_or(Value::Null, |found| json!(found));
    }
}
