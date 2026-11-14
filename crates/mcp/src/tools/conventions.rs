//! `check_conventions`.

use codeskimr_application::access;
use codeskimr_application::conventions::{
    CheckReport, CheckRequest, Detail, Finding, Location, StoredProject, Unchecked,
};
use codeskimr_codes::ErrorCode;
use codeskimr_domain::{FolderPath, Principal, Project, ProjectSource};
use serde::Serialize;
use serde_json::Value;

use super::{Tools, to_json, truncation_notice};
use crate::args::ConventionsArgs;
use crate::consts::{LINE_OFFSET, READ_TOOL_PERMISSION};
use crate::error::{CodeReport, McpError};
use codeskimr_application::error::CodedError;

#[derive(Serialize)]
struct Place {
    path: String,
    symbol: String,
    line: u32,
    end_line: u32,
}

fn place(location: &Location) -> Place {
    Place {
        path: location.path.as_str().to_owned(),
        symbol: location.symbol.as_str().to_owned(),
        line: location.span.start().line + LINE_OFFSET,
        end_line: location.span.end().line + LINE_OFFSET,
    }
}

#[derive(Serialize)]
struct Violation {
    code: &'static str,
    description: &'static str,
    #[serde(flatten)]
    at: Place,
    /// `CHK-104`: code lines of the function.
    lines: Option<u32>,
    /// `CHK-104`: the most that is allowed.
    max_lines: Option<u32>,
    /// `CHK-102`: the earlier function this one copies.
    duplicate_of: Option<Place>,
    /// `CHK-102`: how similar the two are, in whole percent.
    similarity_percent: Option<u32>,
}

fn violation(finding: &Finding) -> Violation {
    let mut out = Violation {
        code: finding.code.id(),
        description: finding.code.description(),
        at: place(&finding.location),
        lines: None,
        max_lines: None,
        duplicate_of: None,
        similarity_percent: None,
    };
    match &finding.detail {
        Detail::TooLong { lines, limit } => {
            out.lines = Some(*lines);
            out.max_lines = Some(*limit);
        }
        Detail::Duplicate {
            of,
            similarity_percent,
        } => {
            out.duplicate_of = Some(place(of));
            out.similarity_percent = Some(*similarity_percent);
        }
    }
    out
}

#[derive(Serialize)]
struct Skipped {
    path: String,
    code: &'static str,
}

fn skipped(file: &Unchecked) -> Skipped {
    Skipped {
        path: file.path.as_str().to_owned(),
        code: file.code.id(),
    }
}

#[derive(Serialize)]
struct Answer {
    files_checked: u32,
    /// Number of findings, also those that are not listed.
    total: usize,
    findings: Vec<Violation>,
    /// Selected files that could not be analysed.
    unchecked: Vec<Skipped>,
    /// Requested paths that selected no file; the path may be a typo, so the check is not done.
    unmatched: Vec<String>,
    /// Rules this version does not check; a clean answer says nothing about them.
    not_checked: Vec<&'static str>,
    notice: Option<CodeReport>,
}

/// The root of a project the check can read: only a folder on this machine.
fn root_of(project: &Project) -> Result<&FolderPath, McpError> {
    match &project.source {
        ProjectSource::Folder(root) => Ok(root),
        ProjectSource::Git { .. } => Err(McpError::Port(CodedError::new(
            ErrorCode::ProjectSourceInvalid,
        ))),
    }
}

fn answer(report: CheckReport, limit: u32, clamped: bool) -> Answer {
    let total = report.findings.len();
    let wanted = usize::try_from(limit).unwrap_or(usize::MAX);
    Answer {
        files_checked: report.files_checked,
        total,
        findings: report.findings.iter().take(wanted).map(violation).collect(),
        unchecked: report.unchecked.iter().map(skipped).collect(),
        unmatched: report.unmatched,
        not_checked: report.not_checked.iter().map(|code| code.id()).collect(),
        notice: (clamped || total > wanted).then(|| truncation_notice(limit, Some(total))),
    }
}

impl Tools {
    /// Checks the files of a project against the code rules of blueprint section 3. The files are
    /// read afresh, so the answer shows the code as it is now and not as the last scan saw it.
    ///
    /// # Errors
    /// `AUTH-401` if the principal may not read source, `PRJ-101` if the project is not a local
    /// folder, `MCP-201` for the project, or the code of a failing file discovery.
    pub fn check_conventions(
        &self,
        principal: &Principal,
        args: &ConventionsArgs,
    ) -> Result<Value, McpError> {
        access::require(principal, READ_TOOL_PERMISSION)?;
        let project = self.project(args.project)?;
        let limit = self.apply_limit(args.limit);
        let paths = args.paths.clone().unwrap_or_default();
        // Without a snapshot the check compares the selected files with each other only.
        let snapshot = self.index().latest_snapshot(project.id)?;
        let report = self.checker().check(&CheckRequest {
            principal,
            root: root_of(&project)?,
            paths: &paths,
            stored: snapshot.map(|snapshot| StoredProject {
                index: self.similarity(),
                snapshot: snapshot.id,
            }),
        })?;
        to_json(&answer(report, limit.value, limit.clamped))
    }
}
