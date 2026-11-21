//! The project list when the stored tool report of one project cannot be read.

use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock};

use codeskimr_application::error::CodedError;
use codeskimr_application::events::EventBus;
use codeskimr_application::ports::{
    LanguageShare, LastScan, LeftOut, ProjectCatalog, ProjectOverview, ProjectToolCheck,
    ReportWrite, ScanReport, ScanReportStore, ScanWarning,
};
use codeskimr_application::registry::CommandExt;
use codeskimr_cli::GlobalOptions;
use codeskimr_codes::ErrorCode;
use codeskimr_domain::{
    FolderPath, Project, ProjectId, ProjectName, ProjectSource, RelativePath, ScanLevel,
    ScanOrigin, Timestamp, ToolReport,
};

use super::Context;
use super::projects::{LeftOutRow, ListProjects, WarningRow};
use crate::locations::Locations;
use crate::principal;
use crate::running::RunningScans;

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
        // The first project carries two warnings of its newest scan.
        warnings: if id == 1 {
            vec![
                ScanWarning {
                    code: ErrorCode::AmbiguousReference,
                    count: 4,
                },
                ScanWarning {
                    code: ErrorCode::FileEncodingUnsupported,
                    count: 3,
                },
            ]
        } else {
            Vec::new()
        },
        // The first project left out three files of which it names one.
        left_out: (id == 1)
            .then(|| LeftOut {
                path: RelativePath::new("src/legacy.txt").unwrap(),
                code: ErrorCode::FileEncodingUnsupported,
                kept: false,
            })
            .into_iter()
            .collect(),
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
pub(super) struct BrokenFirst;

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

/// Reports whose stored report of project 1 is unreadable and project 2 has a level 1 scan.
struct ReportsOfTwo;

impl ScanReportStore for ReportsOfTwo {
    fn store_report(&self, _write: &ReportWrite) -> Result<(), CodedError> {
        Err(CodedError::new(ErrorCode::DatabaseFailed))
    }

    fn report(&self, project: ProjectId) -> Result<Option<ScanReport>, CodedError> {
        if project == ProjectId::new(1) {
            return Err(CodedError::new(ErrorCode::DatabaseFailed));
        }
        Ok(Some(ScanReport {
            last: LastScan {
                at: Timestamp(7),
                level: ScanLevel::One,
                origin: ScanOrigin::Browser,
            },
            level2: None,
        }))
    }
}

/// A context over empty folders, with the given tool checks and scan lock.
pub(super) fn context(tools: Arc<dyn ProjectToolCheck>, running: RunningScans) -> Arc<Context> {
    Arc::new(Context {
        global: GlobalOptions::default(),
        locations: Locations {
            data_dir: Path::new("data").to_path_buf(),
            languages_dir: Path::new("languages").to_path_buf(),
            user_config: None,
            bases: Vec::new(),
            program_dir: None,
        },
        events: EventBus::new(),
        secrets: crate::memory_keystore::secrets(),
        tools,
        running,
        level2_runs: crate::level2_runs::Level2Runs::default(),
        registration: Mutex::new(()),
        tiers: OnceLock::new(),
    })
}

/// The list over the two projects, whose stored tool report and report of project 1 are
/// unreadable.
fn list_of_two() -> ListProjects {
    let tools: Arc<dyn ProjectToolCheck> = Arc::new(BrokenFirst);
    let context = context(Arc::clone(&tools), RunningScans::default());
    ListProjects {
        catalog: Arc::new(TwoProjects),
        tools,
        reports: Arc::new(ReportsOfTwo),
        context,
    }
}

#[test]
fn an_unreadable_tool_report_does_not_take_the_list_down() {
    let list = list_of_two();
    let principal = principal::resolve();
    let rows = list.run(&principal, ()).unwrap();
    let names: Vec<_> = rows.iter().map(|row| row.name.as_str()).collect();
    assert_eq!(names, ["first", "second"]);
    assert!(rows.iter().all(|row| row.tools.is_none()));
    assert_eq!(
        rows[0].warnings,
        [
            WarningRow {
                code: "RES-301",
                count: 4
            },
            WarningRow {
                code: "PRS-101",
                count: 3
            }
        ]
    );
    assert_eq!(rows[1].warnings, Vec::<WarningRow>::new());
}

#[test]
fn the_list_names_the_files_a_scan_left_out_and_counts_the_rest() {
    let rows = list_of_two().run(&principal::resolve(), ()).unwrap();
    assert_eq!(
        rows[0].left_out,
        [LeftOutRow {
            path: "src/legacy.txt".to_owned(),
            code: "PRS-101",
            kept: false
        }]
    );
    // The warning counts three files, the list names one of them.
    assert_eq!(rows[0].left_out_more, 2);
    assert_eq!((rows[1].left_out.len(), rows[1].left_out_more), (0, 0));
}

#[test]
fn the_list_carries_the_stored_report_and_an_unreadable_one_is_left_out() {
    let list = list_of_two();
    let rows = list.run(&principal::resolve(), ()).unwrap();
    assert_eq!(rows[0].report, None);
    let report = rows[1].report.as_ref().unwrap();
    assert_eq!(
        (
            report.last_scan.at,
            report.last_scan.level,
            report.last_scan.origin
        ),
        (7, "1", "browser")
    );
    assert_eq!(report.level2, None);
}
