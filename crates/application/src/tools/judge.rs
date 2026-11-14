//! Turns what the indexer runner found into findings.

use codeskimr_domain::{LanguageId, ToolFinding, ToolRole, ToolState};

use crate::consts::{TOOL_INSTALL_SUFFIX, TOOL_KEY_PREFIX, TOOL_PURPOSE_SUFFIX};
use crate::ports::{IndexerSpec, IndexerStatus, ToolIssue, ToolProblem, ToolSpec, ToolVersion};

/// The state and the version found of `tool`, given the problems the runner reported.
fn outcome(tool: &ToolSpec, problems: &[ToolProblem]) -> (ToolState, Option<String>) {
    let Some(problem) = problems.iter().find(|p| p.program == tool.program) else {
        return (ToolState::Ready, None);
    };
    match &problem.issue {
        ToolIssue::Missing => (ToolState::Missing, None),
        ToolIssue::TooOld { found, .. } => match ToolVersion::find_in(found) {
            Some(version) => (ToolState::TooOld, Some(version.to_string())),
            None => (ToolState::VersionUnknown, None),
        },
    }
}

fn finding(
    language: &LanguageId,
    role: ToolRole,
    tool: &ToolSpec,
    status: &IndexerStatus,
) -> ToolFinding {
    let problems: &[ToolProblem] = match status {
        IndexerStatus::Ready => &[],
        IndexerStatus::Unavailable(problems) => problems,
    };
    let (state, found) = outcome(tool, problems);
    let key = |suffix: &str| [TOOL_KEY_PREFIX, tool.program.as_str(), suffix].concat();
    ToolFinding {
        language: language.clone(),
        tool: tool.program.clone(),
        role,
        state,
        found,
        required: Some(tool.min_version.to_string()),
        purpose_key: key(TOOL_PURPOSE_SUFFIX),
        install_key: key(TOOL_INSTALL_SUFFIX),
    }
}

/// One finding for the indexer and one for each tool of its toolchain, in that order. `spec`
/// carries the versions that were asked for (the pack's and the project's, whichever is newer),
/// `status` is what `IndexRunner::check` said about it.
#[must_use]
pub fn findings_of(
    language: &LanguageId,
    spec: &IndexerSpec,
    status: &IndexerStatus,
) -> Vec<ToolFinding> {
    std::iter::once((ToolRole::Indexer, &spec.indexer))
        .chain(
            spec.toolchain
                .iter()
                .map(|tool| (ToolRole::Toolchain, tool)),
        )
        .map(|(role, tool)| finding(language, role, tool, status))
        .collect()
}
