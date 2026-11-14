//! The check of the tools that level 2 needs (ADR 0033, decision 8).
//!
//! The ports here are the only way the check touches the machine: a probe that starts a tool with
//! its fixed version arguments, a reader for the files in which a project names the versions it
//! needs, and the store of the result. None of them runs project code.

use codeskimr_domain::{
    LanguageId, ProjectId, ProjectSource, RequirementSpec, ToolReport, ToolSpec,
};

use crate::error::CodedError;

/// How starting a tool for its version ended. The probe never fails: every way it can go wrong is
/// one of these.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProbeOutcome {
    /// The tool ran and printed this (standard output and standard error, cut at a limit).
    Ran(String),
    /// The tool could not be started: it is not installed or not on the search path.
    NotFound,
    /// The tool did not finish in time and was killed.
    TimedOut,
}

/// Knows which tools the language packs ask for (adapter: `codeskimr_parsing`).
pub trait ToolCatalog: Send + Sync {
    /// The tools level 2 of `language` needs; empty for a language without level 2.
    fn tools_of(&self, language: &LanguageId) -> Vec<ToolSpec>;
}

/// Starts a tool to learn its version (adapter: `codeskimr_source`).
///
/// An adapter starts `command` with exactly `args` and no shell, in a folder that is not a
/// project, with an allow-listed environment and a time limit after which the process is killed.
/// It reads no project file and runs no project code.
pub trait ToolProbe: Send + Sync {
    /// Runs the tool once.
    fn probe(&self, command: &str, args: &[String]) -> ProbeOutcome;
}

/// Reads the version a project requires from one of its files (adapter: `codeskimr_source`).
pub trait RequirementReader: Send + Sync {
    /// The version text that `requirement` finds in the project `source`, or `None` if the file
    /// is missing, too large, unreadable, not as described, or not inside the project (a path
    /// that leaves the project root, also through a link, is never read).
    fn read(&self, source: &ProjectSource, requirement: &RequirementSpec) -> Option<String>;
}

/// Keeps the newest check of each project (adapter: `codeskimr_storage`).
pub trait ToolReportStore: Send + Sync {
    /// Replaces the report of `project`.
    ///
    /// # Errors
    /// A coded failure of the `STO` group.
    fn replace(&self, project: ProjectId, report: &ToolReport) -> Result<(), CodedError>;

    /// The newest report of `project`, `None` if it was never checked.
    ///
    /// # Errors
    /// A coded failure of the `STO` group.
    fn load(&self, project: ProjectId) -> Result<Option<ToolReport>, CodedError>;
}

/// Checks the tools of a project whenever it is loaded: registration, scan and reopening. It is the
/// one interface that the command line, the UI commands and the MCP server use.
pub trait ProjectToolCheck: Send + Sync {
    /// Checks the tools of the languages in the newest snapshot of `project`, stores the result
    /// and returns it. A project that was not scanned yet has no languages and gets an empty
    /// report. A missing tool is a finding, never an error.
    ///
    /// # Errors
    /// `PRJ-201` if the project does not exist, or a coded failure of the `STO` group.
    fn check(&self, project: ProjectId) -> Result<ToolReport, CodedError>;

    /// The report of the last check without checking again, `None` if there was none.
    ///
    /// # Errors
    /// A coded failure of the `STO` group.
    fn stored(&self, project: ProjectId) -> Result<Option<ToolReport>, CodedError>;
}
