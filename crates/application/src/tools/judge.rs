//! Turns what the indexer runner found into findings.

use codeskimr_domain::{Ceiling, LanguageId, ToolFinding, ToolRole, ToolState};

use crate::consts::{TOOL_INSTALL_SUFFIX, TOOL_KEY_PREFIX, TOOL_PURPOSE_SUFFIX};
use crate::ports::{IndexerSpec, IndexerStatus, ToolIssue, ToolProblem, ToolSpec, ToolVersion};

/// A file of the project that names a version of a tool and limits how new the tool may be
/// (`rollForward` of `global.json`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolLimit {
    /// The program the file speaks about.
    pub program: String,
    /// The version the file names.
    pub asked: ToolVersion,
    /// How far above it a tool may go.
    pub ceiling: Ceiling,
}

/// What the check found out about one tool.
struct Outcome {
    state: ToolState,
    found: Option<String>,
    required: ToolVersion,
}

/// The first limit that the version found goes beyond.
fn exceeded<'a>(
    tool: &ToolSpec,
    version: ToolVersion,
    limits: &'a [ToolLimit],
) -> Option<&'a ToolLimit> {
    limits
        .iter()
        .find(|limit| limit.program == tool.program && !limit.asked.allows(limit.ceiling, version))
}

/// The state and the version found of `tool`, given what the runner reported. A version is known
/// of a tool that is too old and of one that the runner found ready; nothing is judged about a
/// ready tool whose version the runner did not report.
fn outcome(tool: &ToolSpec, status: &IndexerStatus, limits: &[ToolLimit]) -> Outcome {
    let of = |state, found: Option<String>| Outcome {
        state,
        found,
        required: tool.min_version,
    };
    if let Some(problem) = status.problems().iter().find(|p| p.program == tool.program) {
        return outcome_of_problem(tool, problem);
    }
    let seen = status
        .found()
        .iter()
        .find(|seen| seen.program == tool.program);
    let Some(seen) = seen else {
        return of(ToolState::Ready, None);
    };
    match exceeded(tool, seen.version, limits) {
        Some(limit) => Outcome {
            state: ToolState::TooNew,
            found: Some(seen.version.to_string()),
            required: limit.asked,
        },
        None => of(ToolState::Ready, Some(seen.version.to_string())),
    }
}

fn outcome_of_problem(tool: &ToolSpec, problem: &ToolProblem) -> Outcome {
    let (state, found) = match &problem.issue {
        ToolIssue::Missing => (ToolState::Missing, None),
        ToolIssue::NotRunnable { output } => (
            ToolState::NotRunnable,
            Some(output.clone()).filter(|text| !text.is_empty()),
        ),
        ToolIssue::TooOld { found, .. } => match ToolVersion::find_in(found) {
            Some(version) => (ToolState::TooOld, Some(version.to_string())),
            None => (ToolState::VersionUnknown, None),
        },
    };
    Outcome {
        state,
        found,
        required: tool.min_version,
    }
}

fn finding(
    language: &LanguageId,
    role: ToolRole,
    tool: &ToolSpec,
    status: &IndexerStatus,
    limits: &[ToolLimit],
) -> ToolFinding {
    let outcome = outcome(tool, status, limits);
    let key = |suffix: &str| [TOOL_KEY_PREFIX, tool.program.as_str(), suffix].concat();
    ToolFinding {
        language: language.clone(),
        tool: tool.program.clone(),
        role,
        state: outcome.state,
        found: outcome.found,
        required: Some(outcome.required.to_string()),
        purpose_key: key(TOOL_PURPOSE_SUFFIX),
        install_key: key(TOOL_INSTALL_SUFFIX),
    }
}

/// One finding for the indexer and one for each tool of its toolchain, in that order. `spec`
/// carries the versions that were asked for (the pack's and the project's, whichever is newer),
/// `status` is what `IndexRunner::check` said about it, and `limits` are the ceilings that the
/// files of the project set: a tool that is ready but newer than one of them is `too_new`.
#[must_use]
pub fn findings_of(
    language: &LanguageId,
    spec: &IndexerSpec,
    status: &IndexerStatus,
    limits: &[ToolLimit],
) -> Vec<ToolFinding> {
    std::iter::once((ToolRole::Indexer, &spec.indexer))
        .chain(
            spec.toolchain
                .iter()
                .map(|tool| (ToolRole::Toolchain, tool)),
        )
        .map(|(role, tool)| finding(language, role, tool, status, limits))
        .collect()
}
