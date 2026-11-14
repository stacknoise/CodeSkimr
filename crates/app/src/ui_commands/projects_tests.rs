//! The project list when the stored tool report of one project cannot be read.

use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock};

use codeskimr_application::error::CodedError;
use codeskimr_application::events::EventBus;
use codeskimr_application::ports::{
    LanguageShare, ProjectCatalog, ProjectOverview, ProjectToolCheck,
};
use codeskimr_application::registry::{Command, Request};
use codeskimr_cli::GlobalOptions;
use codeskimr_codes::ErrorCode;
use codeskimr_domain::{FolderPath, Project, ProjectId, ProjectName, ProjectSource, ToolReport};

use super::Context;
use super::projects::ListProjects;
use super::running::RunningScans;
use crate::locations::Locations;
use crate::principal;

/// A catalog with two projects.
struct TwoProjects;

fn overview(id: u64, name: &str) -> ProjectOverview {
    ProjectOverview {
        project: Project {
            id: ProjectId::new(id),
            name: ProjectName::new(name).unwrap(),
            source: ProjectSource::Folder(FolderPath::new("/work/demo").unwrap()),
            layers: Vec::new(),
        },
        latest: None,
        types: 0,
        languages: Vec::new(),
    }
}

impl ProjectCatalog for TwoProjects {
    fn overviews(&self) -> Result<Vec<ProjectOverview>, CodedError> {
        Ok(vec![overview(1, "first"), overview(2, "second")])
    }

    fn languages(&self, _project: ProjectId) -> Result<Vec<LanguageShare>, CodedError> {
        Ok(Vec::new())
    }
}

/// Tool checks whose stored report of project 1 is unreadable.
struct BrokenFirst;

impl ProjectToolCheck for BrokenFirst {
    fn check(&self, _project: ProjectId) -> Result<ToolReport, CodedError> {
        Err(CodedError::new(ErrorCode::DatabaseFailed))
    }

    fn stored(&self, project: ProjectId) -> Result<Option<ToolReport>, CodedError> {
        if project == ProjectId::new(1) {
            Err(CodedError::new(ErrorCode::DatabaseFailed))
        } else {
            Ok(None)
        }
    }
}

#[test]
fn an_unreadable_tool_report_does_not_take_the_list_down() {
    let tools: Arc<dyn ProjectToolCheck> = Arc::new(BrokenFirst);
    let context = Arc::new(Context {
        global: GlobalOptions::default(),
        locations: Locations {
            data_dir: Path::new("data").to_path_buf(),
            languages_dir: Path::new("languages").to_path_buf(),
            user_config: None,
            bases: Vec::new(),
        },
        events: EventBus::new(),
        secrets: crate::memory_keystore::secrets(),
        tools: Arc::clone(&tools),
        running: RunningScans::default(),
        registration: Mutex::new(()),
        tiers: OnceLock::new(),
    });
    let list = ListProjects {
        catalog: Arc::new(TwoProjects),
        tools,
        context,
    };
    let principal = principal::resolve();
    let rows = list
        .execute(Request {
            principal: &principal,
            input: (),
        })
        .unwrap();
    let names: Vec<_> = rows.iter().map(|row| row.name.as_str()).collect();
    assert_eq!(names, ["first", "second"]);
    assert!(rows.iter().all(|row| row.tools.is_none()));
}
