//! The architecture rules of a project, judged for the doors that are not the screen: the tool
//! `check_rules` and `codeskimr rules` (ADR 0104).

use std::path::Path;
use std::sync::Arc;

use codeskimr_application::config::{Config, ConfigLayer, LayerSource, PROJECT_CONFIG_FILE, Rules};
use codeskimr_application::error::CodedError;
use codeskimr_application::ports::ReferencePlaces;
use codeskimr_application::rules::{BASELINE_FILE, Baseline, RulesChecks, RulesReport, report};
use codeskimr_domain::{Layer, Project, ProjectSource, SnapshotId};

use crate::project_file::{read_project_file, version_of};

/// The configuration that the text of a project file makes.
fn config_of(text: &str) -> Result<Config, CodedError> {
    let layer = ConfigLayer::parse(LayerSource::Project, text).map_err(CodedError::caused_by)?;
    Config::resolve(&[layer]).map_err(CodedError::caused_by)
}

/// The rules of the project file and the held state; both empty for a Git project.
pub(crate) fn files_of(source: &ProjectSource) -> Result<(Rules, Option<Baseline>), CodedError> {
    let files = project_files(source)?;
    Ok((files.rules, files.baseline))
}

/// What the files of a folder project say, all taken from one read of the project file.
pub(crate) struct ProjectFiles {
    /// The rules of the project file.
    pub(crate) rules: Rules,
    /// The layers of the project file; `None` for a Git project, which has none.
    pub(crate) layers: Option<Vec<Layer>>,
    /// The held state, if there is one.
    pub(crate) baseline: Option<Baseline>,
    /// The version of the project file (empty for a Git project).
    pub(crate) version: String,
}

/// The rules, the layers and the version of the project file of `source`, without the held
/// state (a state that is about to be replaced must not be read). The version and the layers come
/// from the very text the rules were read from, so a client that sends the version back with a
/// change has seen exactly the state the version stands for.
pub(crate) fn project_file_state(source: &ProjectSource) -> Result<ProjectFiles, CodedError> {
    let ProjectSource::Folder(folder) = source else {
        return Ok(ProjectFiles {
            rules: Rules::default(),
            layers: None,
            baseline: None,
            version: String::new(),
        });
    };
    let text = read_project_file(&Path::new(folder.as_str()).join(PROJECT_CONFIG_FILE))?;
    let config = config_of(&text)?;
    Ok(ProjectFiles {
        layers: Some(config.project_layers()),
        rules: config.rules,
        baseline: None,
        version: version_of(&text),
    })
}

/// [`project_file_state`] and the held state; both empty for a Git project.
pub(crate) fn project_files(source: &ProjectSource) -> Result<ProjectFiles, CodedError> {
    let files = project_file_state(source)?;
    let ProjectSource::Folder(folder) = source else {
        return Ok(files);
    };
    let held = read_project_file(&Path::new(folder.as_str()).join(BASELINE_FILE))?;
    let baseline = if held.trim().is_empty() {
        None
    } else {
        Some(Baseline::parse(&held)?)
    };
    Ok(ProjectFiles { baseline, ..files })
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
            &project.layers,
            baseline.as_ref(),
            self.references.as_ref(),
            snapshot,
        )
    }
}
