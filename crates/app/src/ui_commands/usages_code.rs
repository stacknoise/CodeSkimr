//! The command behind the code lines of the hit list (mock 6): for each hit of a usages query, the
//! line of code it stands on, read from the project folder as it is now. It hands out source
//! text and therefore needs `read_source` and the installation token (ADR 0035). The paths come
//! from the index, never from the caller, so the command cannot be asked for an arbitrary file.

use std::collections::BTreeSet;
use std::sync::Arc;

use codeskimr_application::access;
use codeskimr_application::error::CodedError;
use codeskimr_application::registry::{Command, CommandDescriptor, CommandId, Request};
use codeskimr_codes::ErrorCode;
use codeskimr_domain::{Permission, ProjectId, ProjectSource, RelativePath};
use serde::Serialize;

use super::consts::{
    SOURCE_MAX_BYTES, USAGES_CODE, USAGES_CODE_DESCRIPTION, USAGES_CODE_MAX_CHARS,
    USAGES_CODE_MAX_HITS, USAGES_CODE_TITLE,
};
use super::usages::{Hit, UsagesInput, UsagesPorts};

/// The code line of one hit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CodeLine {
    /// Path of the file, as in the hit.
    pub path: String,
    /// Line of the hit, counted from 1.
    pub line: u32,
    /// The text of the line, trimmed at both ends and cut at a fixed length.
    pub text: String,
}

/// The answer of `usages.code`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CodeOut {
    /// Whether code could be read: only projects that are a folder on this machine have any.
    pub available: bool,
    /// One entry per distinct place of the first hits; a place whose file or line is gone is left out.
    pub lines: Vec<CodeLine>,
}

/// The code lines of the hits of a usages query.
pub struct UsagesCode {
    pub(super) ports: Arc<UsagesPorts>,
}

fn trimmed(line: &str) -> String {
    line.trim().chars().take(USAGES_CODE_MAX_CHARS).collect()
}

impl UsagesPorts {
    /// The distinct places (file, line) of the first hits of both walks, in walk order.
    fn places(&self, input: &UsagesInput) -> Result<Vec<(String, u32)>, CodedError> {
        let id = ProjectId::new(input.project);
        let project = self.projects.project(id)?;
        let snapshot = self
            .index
            .latest_snapshot(id)?
            .ok_or_else(|| CodedError::new(ErrorCode::ProjectNotScanned))?
            .id;
        let depth = if input.depth == 0 {
            self.max_depth
        } else {
            input.depth.min(self.max_depth)
        };
        let (incoming, outgoing, _) = self.walks(&project, snapshot, input, depth)?;
        let hits = incoming
            .into_iter()
            .chain(outgoing)
            .flat_map(|walk| walk.hits);
        let mut seen = BTreeSet::new();
        Ok(hits
            .map(|hit: Hit| (hit.path, hit.line))
            .filter(|place| seen.insert(place.clone()))
            .take(USAGES_CODE_MAX_HITS)
            .collect())
    }

    fn code_of(&self, input: &UsagesInput) -> Result<CodeOut, CodedError> {
        let project = self.projects.project(ProjectId::new(input.project))?;
        let ProjectSource::Folder(root) = &project.source else {
            return Ok(CodeOut {
                available: false,
                lines: Vec::new(),
            });
        };
        let mut lines = Vec::new();
        for (path, line) in self.places(input)? {
            let Ok(relative) = RelativePath::new(&path) else {
                continue;
            };
            let read = self
                .text
                .excerpt(root, &relative, (line, line), (1, SOURCE_MAX_BYTES));
            if let Some(text) = read.ok().and_then(|found| found.lines.into_iter().next()) {
                lines.push(CodeLine {
                    path,
                    line,
                    text: trimmed(&text),
                });
            }
        }
        Ok(CodeOut {
            available: true,
            lines,
        })
    }
}

impl Command for UsagesCode {
    type Input = UsagesInput;
    type Output = CodeOut;

    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new(USAGES_CODE),
        title_key: USAGES_CODE_TITLE,
        description_key: USAGES_CODE_DESCRIPTION,
        shortcut: None,
        read_only: true,
        permission: Permission::ReadSource,
    };

    fn execute(&self, request: Request<'_, UsagesInput>) -> Result<CodeOut, CodedError> {
        access::require(request.principal, Permission::ReadSource)?;
        self.ports.code_of(&request.input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_long_line_is_trimmed_and_cut() {
        let long = format!("   {}   ", "x".repeat(USAGES_CODE_MAX_CHARS + 10));
        let cut = trimmed(&long);
        assert_eq!(cut.chars().count(), USAGES_CODE_MAX_CHARS);
        assert!(!cut.starts_with(' '));
    }
}
