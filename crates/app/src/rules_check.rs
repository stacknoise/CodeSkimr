//! The architecture rules of a project, judged for the doors that are not the screen: the tool
//! `check_rules` and `codeskimr rules` (ADR 0104).

use std::path::Path;
use std::sync::Arc;

use codeskimr_application::config::{Config, ConfigLayer, LayerSource, PROJECT_CONFIG_FILE, Rules};
use codeskimr_application::error::CodedError;
use codeskimr_application::ports::ReferencePlaces;
use codeskimr_application::rules::{BASELINE_FILE, Baseline, RulesChecks, RulesReport, report};
use codeskimr_domain::{Project, ProjectSource, SnapshotId};

use crate::project_file::read_project_file;

/// The rules of the project file and the held state; both empty for a Git project.
pub(crate) fn files_of(source: &ProjectSource) -> Result<(Rules, Option<Baseline>), CodedError> {
    let ProjectSource::Folder(folder) = source else {
        return Ok((Rules::default(), None));
    };
    let root = Path::new(folder.as_str());
    let text = read_project_file(&root.join(PROJECT_CONFIG_FILE))?;
    let layer = ConfigLayer::parse(LayerSource::Project, &text).map_err(CodedError::caused_by)?;
    let config = Config::resolve(&[layer]).map_err(CodedError::caused_by)?;
    let held = read_project_file(&root.join(BASELINE_FILE))?;
    let baseline = if held.trim().is_empty() {
        None
    } else {
        Some(Baseline::parse(&held)?)
    };
    Ok((config.rules, baseline))
}

/// Judges over the references of the store, with the project file and the held state next to the
/// code.
pub(crate) struct StoredRules {
    references: Arc<dyn ReferencePlaces>,
}

impl StoredRules {
    /// Judges over `references`.
    pub(crate) fn new(references: Arc<dyn ReferencePlaces>) -> Self {
        Self { references }
    }
}

impl RulesChecks for StoredRules {
    fn check(&self, project: &Project, snapshot: SnapshotId) -> Result<RulesReport, CodedError> {
        let (rules, baseline) = files_of(&project.source)?;
        report(
            &rules,
            baseline.as_ref(),
            self.references.as_ref(),
            snapshot,
        )
    }
}
