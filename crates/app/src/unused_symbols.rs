//! The symbols without a usage under the exception list of the project file (ADR 0101), for the
//! statistics screen, the command `unused` and the tool `find_unused`.

use std::path::Path;
use std::sync::Arc;

use codeskimr_application::config::{
    Config, ConfigLayer, LayerSource, PROJECT_CONFIG_FILE, UnusedIgnore,
};
use codeskimr_application::error::CodedError;
use codeskimr_application::ports::CodeStatistics;
use codeskimr_application::unused::{Findings, UnusedSymbols, judge};
use codeskimr_domain::{Project, ProjectSource, RankingQuery, SnapshotId};

use crate::ui_commands::layers::read_project_file;

/// The exception list of a folder project; empty for a Git project or without a project file.
///
/// # Errors
/// - `CFG-102` if the project file cannot be read or is not TOML.
/// - `CFG-101` / `CFG-103` if it holds an invalid value, for example an entry without a reason.
///
/// The statistics fail with the code of the scan instead of showing every symbol again as if the
/// list were empty.
pub(crate) fn rules_of(project: &Project) -> Result<Vec<UnusedIgnore>, CodedError> {
    let ProjectSource::Folder(folder) = &project.source else {
        return Ok(Vec::new());
    };
    let path = Path::new(folder.as_str()).join(PROJECT_CONFIG_FILE);
    let text = read_project_file(&path)?;
    let layer = ConfigLayer::parse(LayerSource::Project, &text).map_err(CodedError::caused_by)?;
    let config = Config::resolve(&[layer]).map_err(CodedError::caused_by)?;
    Ok(config.unused.ignore)
}

/// Judges the symbols of a snapshot from their facts, with the exception list of the project.
pub(crate) struct JudgedUnused {
    statistics: Arc<dyn CodeStatistics>,
    /// `conventions.test_folders`.
    test_names: Vec<String>,
    /// `ui.approximate_languages`.
    approximate: Vec<String>,
}

impl JudgedUnused {
    /// Judges over `statistics` with the settings of `config`.
    pub(crate) fn new(statistics: Arc<dyn CodeStatistics>, config: &Config) -> Self {
        Self {
            statistics,
            test_names: config.conventions.test_folders.clone(),
            approximate: config.ui.approximate_languages.clone(),
        }
    }
}

impl UnusedSymbols for JudgedUnused {
    fn unused(
        &self,
        project: &Project,
        snapshot: SnapshotId,
        query: &RankingQuery,
    ) -> Result<Findings, CodedError> {
        let rules = rules_of(project)?;
        let facts = self.statistics.symbol_facts(snapshot)?;
        let judged = judge(facts, query, &rules, &self.approximate, &self.test_names);
        Ok(Findings::of(&judged, &self.approximate))
    }
}
