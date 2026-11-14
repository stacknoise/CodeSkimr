//! Turns what a probe returned into a finding.

use codeskimr_domain::{ToolFinding, ToolSpec, ToolState, ToolVersion};

use crate::ports::ProbeOutcome;

/// The strictest of the versions that apply: the one the pack names and the one the project names.
/// A version that is not a number (`stable`) asks for nothing.
fn strictest(spec: &ToolSpec, project: Option<&str>) -> Option<ToolVersion> {
    [spec.min_version.as_deref(), project]
        .into_iter()
        .flatten()
        .filter_map(ToolVersion::parse_requirement)
        .max()
}

fn state_of(
    outcome: &ProbeOutcome,
    found: Option<&ToolVersion>,
    required: Option<&ToolVersion>,
) -> ToolState {
    match (outcome, required, found) {
        (ProbeOutcome::NotFound, _, _) => ToolState::Missing,
        (ProbeOutcome::TimedOut, _, _) | (ProbeOutcome::Ran(_), Some(_), None) => {
            ToolState::VersionUnknown
        }
        (ProbeOutcome::Ran(_), Some(required), Some(found)) if found < required => {
            ToolState::TooOld
        }
        (ProbeOutcome::Ran(_), _, _) => ToolState::Ready,
    }
}

/// The finding for `spec`, given what the probe returned and the version text the project names
/// (from `global.json`, `.nvmrc` and the like).
///
/// A tool that ran is ready unless a version is required: then it is ready if its version is at
/// least the required one, too old if it is lower, and of unknown version if it printed none.
#[must_use]
pub fn judge(spec: &ToolSpec, outcome: &ProbeOutcome, project: Option<&str>) -> ToolFinding {
    let found = match outcome {
        ProbeOutcome::Ran(output) => ToolVersion::find_in(output),
        ProbeOutcome::NotFound | ProbeOutcome::TimedOut => None,
    };
    let required = strictest(spec, project);
    ToolFinding {
        language: spec.language.clone(),
        tool: spec.id.clone(),
        role: spec.role,
        state: state_of(outcome, found.as_ref(), required.as_ref()),
        found: found.map(|version| version.to_string()),
        required: required.map(|version| version.to_string()),
        purpose_key: spec.purpose_key.clone(),
        install_key: spec.install_key.clone(),
    }
}
