//! The check of the tools that level 2 needs (ADR 0033, decision 8).
//!
//! The check asks the indexer runner (`IndexRunner::check`) whether the tools exist and are new
//! enough. The ports here add what the pack says about the project's own version files, a reader
//! for those files, and the store of the result. None of them runs project code.

use codeskimr_domain::{LanguageId, ProjectId, ProjectSource, RequirementSpec, ToolReport};

use super::IndexerSpec;
use crate::error::CodedError;

/// Where a project names the version of one program of the toolchain, such as `global.json` for
/// `dotnet`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolRequirement {
    /// The program the file speaks about.
    pub program: String,
    /// Where and how the version is written.
    pub file: RequirementSpec,
}

/// What the language pack of one language asks for: how to run its indexer (`[indexer]`) and where
/// the project names the versions of the toolchain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolPlan {
    /// The indexer with its toolchain.
    pub indexer: IndexerSpec,
    /// The files in which a project names versions of the toolchain.
    pub requirements: Vec<ToolRequirement>,
}

/// Knows which tools the language packs ask for (adapter: `codeskimr_parsing`).
pub trait ToolCatalog: Send + Sync {
    /// The plan of level 2 for `language`; `None` for a language without an indexer.
    fn plan_of(&self, language: &LanguageId) -> Option<ToolPlan>;
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
