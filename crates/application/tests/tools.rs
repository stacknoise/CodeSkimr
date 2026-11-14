//! Tests of the tool check: what it finds out about the tools of a project, and what it stores.
#![cfg(test)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};

use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{
    Clock, LanguageShare, PreviousIndex, ProbeOutcome, ProjectCatalog, ProjectOverview,
    ProjectRepository, ProjectToolCheck, RequirementReader, ToolCatalog, ToolProbe,
    ToolReportStore,
};
use codeskimr_application::tools::{ToolCheckPorts, ToolChecker};
use codeskimr_domain::{
    FolderPath, LanguageId, Project, ProjectId, ProjectName, ProjectSource, RequirementFormat,
    RequirementSpec, SourceFile, Timestamp, ToolReport, ToolRole, ToolSpec, ToolState,
};

const PROJECT: ProjectId = ProjectId::new(1);

fn language(name: &str) -> LanguageId {
    LanguageId::new(name).unwrap()
}

fn spec(language_name: &str, id: &str, role: ToolRole, requirement: Option<&str>) -> ToolSpec {
    ToolSpec {
        language: language(language_name),
        id: id.to_owned(),
        role,
        command: id.to_owned(),
        version_args: vec!["--version".to_owned()],
        min_version: None,
        purpose_key: format!("tool-{id}-purpose"),
        install_key: format!("tool-{id}-install"),
        requirement: requirement.map(|file| RequirementSpec {
            file: file.to_owned(),
            format: RequirementFormat::Text,
            path: Vec::new(),
        }),
    }
}

/// The machine, the project and the store, all in one fake.
#[derive(Default)]
struct World {
    languages: Vec<&'static str>,
    specs: Vec<ToolSpec>,
    outcomes: HashMap<String, ProbeOutcome>,
    files: HashMap<String, String>,
    probed: Mutex<Vec<String>>,
    stored: Mutex<Option<ToolReport>>,
    missing_project: bool,
}

impl ToolCatalog for World {
    fn tools_of(&self, language: &LanguageId) -> Vec<ToolSpec> {
        self.specs
            .iter()
            .filter(|spec| &spec.language == language)
            .cloned()
            .collect()
    }
}

impl ToolProbe for World {
    fn probe(&self, command: &str, _args: &[String]) -> ProbeOutcome {
        self.probed
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(command.to_owned());
        self.outcomes
            .get(command)
            .cloned()
            .unwrap_or(ProbeOutcome::NotFound)
    }
}

impl RequirementReader for World {
    fn read(&self, _: &ProjectSource, requirement: &RequirementSpec) -> Option<String> {
        self.files.get(&requirement.file).cloned()
    }
}

impl ToolReportStore for World {
    fn replace(&self, _: ProjectId, report: &ToolReport) -> Result<(), CodedError> {
        *self.stored.lock().unwrap_or_else(PoisonError::into_inner) = Some(report.clone());
        Ok(())
    }

    fn load(&self, _: ProjectId) -> Result<Option<ToolReport>, CodedError> {
        Ok(self
            .stored
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone())
    }
}

impl ProjectRepository for World {
    fn project(&self, id: ProjectId) -> Result<Project, CodedError> {
        if self.missing_project {
            return Err(CodedError::new(codeskimr_codes::ErrorCode::ProjectNotFound));
        }
        Ok(Project {
            id,
            name: ProjectName::new("app").unwrap(),
            source: ProjectSource::Folder(FolderPath::new("/work/app").unwrap()),
            layers: Vec::new(),
        })
    }

    fn latest_files(&self, _: ProjectId) -> Result<Vec<SourceFile>, CodedError> {
        Ok(Vec::new())
    }

    fn latest_index(&self, _: ProjectId) -> Result<PreviousIndex, CodedError> {
        Ok(PreviousIndex::default())
    }
}

impl ProjectCatalog for World {
    fn overviews(&self) -> Result<Vec<ProjectOverview>, CodedError> {
        Ok(Vec::new())
    }

    fn languages(&self, _: ProjectId) -> Result<Vec<LanguageShare>, CodedError> {
        Ok(self
            .languages
            .iter()
            .map(|name| LanguageShare {
                language: language(name),
                files: 1,
                loc: 1,
                symbols: 1,
            })
            .collect())
    }
}

impl Clock for World {
    fn now(&self) -> Timestamp {
        Timestamp(77)
    }
}

fn run(world: World) -> (Arc<World>, Result<ToolReport, CodedError>) {
    let world = Arc::new(world);
    let checker = ToolChecker::new(ToolCheckPorts {
        catalog: world.clone(),
        probe: world.clone(),
        requirements: world.clone(),
        store: world.clone(),
        projects: world.clone(),
        overview: world.clone(),
        clock: world.clone(),
    });
    let result = checker.check(PROJECT);
    (world, result)
}

fn csharp_world(outcomes: &[(&str, ProbeOutcome)], global_json: Option<&str>) -> World {
    World {
        languages: vec!["csharp"],
        specs: vec![
            spec("csharp", "scip-dotnet", ToolRole::Indexer, None),
            spec("csharp", "dotnet", ToolRole::Toolchain, Some("global.json")),
        ],
        outcomes: outcomes
            .iter()
            .map(|(command, outcome)| ((*command).to_owned(), outcome.clone()))
            .collect(),
        files: global_json
            .map(|text| HashMap::from([("global.json".to_owned(), text.to_owned())]))
            .unwrap_or_default(),
        ..World::default()
    }
}

fn ran(text: &str) -> ProbeOutcome {
    ProbeOutcome::Ran(text.to_owned())
}

fn states(report: &ToolReport) -> Vec<(&str, ToolState)> {
    report
        .findings
        .iter()
        .map(|finding| (finding.tool.as_str(), finding.state))
        .collect()
}

#[test]
fn a_tool_that_is_installed_is_ready() {
    let world = csharp_world(
        &[("scip-dotnet", ran("0.2.1")), ("dotnet", ran("8.0.404"))],
        None,
    );
    let (_, report) = run(world);
    let report = report.unwrap();
    assert_eq!(
        states(&report),
        [
            ("scip-dotnet", ToolState::Ready),
            ("dotnet", ToolState::Ready)
        ]
    );
    assert!(report.level2_ready(&language("csharp")));
    assert_eq!(report.findings[1].found.as_deref(), Some("8.0.404"));
}

#[test]
fn a_tool_that_is_not_installed_is_missing_and_names_what_to_show() {
    let (_, report) = run(csharp_world(&[("dotnet", ran("8.0.404"))], None));
    let report = report.unwrap();
    let problems = report.problems();
    assert_eq!(problems.len(), 1);
    assert_eq!(problems[0].tool, "scip-dotnet");
    assert_eq!(problems[0].state, ToolState::Missing);
    assert_eq!(problems[0].purpose_key, "tool-scip-dotnet-purpose");
    assert_eq!(problems[0].install_key, "tool-scip-dotnet-install");
    assert!(!report.level2_ready(&language("csharp")));
}

#[test]
fn an_sdk_older_than_the_one_in_global_json_is_too_old() {
    let json_version = "9.0.100";
    let world = csharp_world(
        &[("scip-dotnet", ran("0.2.1")), ("dotnet", ran("8.0.404"))],
        Some(json_version),
    );
    let (_, report) = run(world);
    let report = report.unwrap();
    let dotnet = &report.findings[1];
    assert_eq!(dotnet.state, ToolState::TooOld);
    assert_eq!(dotnet.found.as_deref(), Some("8.0.404"));
    assert_eq!(dotnet.required.as_deref(), Some("9.0.100"));
}

#[test]
fn a_version_the_project_requires_but_the_tool_does_not_print_is_unknown() {
    let world = csharp_world(
        &[("scip-dotnet", ran("")), ("dotnet", ran("no digits here"))],
        Some("9.0.100"),
    );
    let (_, report) = run(world);
    assert_eq!(
        states(&report.unwrap()),
        [
            ("scip-dotnet", ToolState::Ready),
            ("dotnet", ToolState::VersionUnknown)
        ]
    );
}

#[test]
fn without_a_requirement_a_tool_that_runs_is_ready_whatever_it_prints() {
    let world = csharp_world(&[("scip-dotnet", ran("")), ("dotnet", ran("?"))], None);
    let (_, report) = run(world);
    assert!(report.unwrap().level2_ready(&language("csharp")));
}

#[test]
fn a_requirement_that_is_not_a_number_is_met() {
    let world = csharp_world(
        &[("scip-dotnet", ran("1.0")), ("dotnet", ran("8.0.404"))],
        Some("stable"),
    );
    let (_, report) = run(world);
    assert!(report.unwrap().level2_ready(&language("csharp")));
}

#[test]
fn a_tool_that_does_not_answer_in_time_is_of_unknown_version_and_does_not_fail_the_check() {
    let world = csharp_world(
        &[
            ("scip-dotnet", ProbeOutcome::TimedOut),
            ("dotnet", ran("8.0.404")),
        ],
        None,
    );
    let (_, report) = run(world);
    assert_eq!(
        states(&report.unwrap()),
        [
            ("scip-dotnet", ToolState::VersionUnknown),
            ("dotnet", ToolState::Ready)
        ]
    );
}

#[test]
fn the_version_of_the_pack_is_a_minimum_too_and_the_stricter_one_counts() {
    let mut world = csharp_world(
        &[("scip-dotnet", ran("0.2.1")), ("dotnet", ran("8.0.404"))],
        None,
    );
    world.specs[1].min_version = Some("8.0.100".to_owned());
    world.files = HashMap::from([("global.json".to_owned(), "8.0.500".to_owned())]);
    let (_, report) = run(world);
    let report = report.unwrap();
    assert_eq!(report.findings[1].state, ToolState::TooOld);
    assert_eq!(report.findings[1].required.as_deref(), Some("8.0.500"));
}

#[test]
fn languages_without_tools_give_an_empty_report_that_is_still_stored() {
    let world = World {
        languages: vec!["go", "c"],
        ..World::default()
    };
    let (world, report) = run(world);
    let report = report.unwrap();
    assert_eq!(report.findings.len(), 0);
    assert_eq!(report.checked_at, Timestamp(77));
    assert_eq!(world.load(PROJECT).unwrap(), Some(report));
}

#[test]
fn a_project_that_was_not_scanned_has_no_languages_and_an_empty_report() {
    let (_, report) = run(World::default());
    assert_eq!(report.unwrap().findings.len(), 0);
}

#[test]
fn tools_that_languages_share_are_started_once() {
    let world = World {
        languages: vec!["typescript", "javascript"],
        specs: ["typescript", "javascript"]
            .into_iter()
            .map(|name| spec(name, "node", ToolRole::Toolchain, None))
            .collect(),
        outcomes: HashMap::from([("node".to_owned(), ran("v20.11.1"))]),
        ..World::default()
    };
    let (world, report) = run(world);
    assert_eq!(report.unwrap().findings.len(), 2);
    assert_eq!(world.probed.lock().unwrap().len(), 1);
}

#[test]
fn a_new_check_replaces_the_stored_report() {
    let first = csharp_world(&[("dotnet", ran("8.0.404"))], None);
    let (world, _) = run(first);
    assert_eq!(world.load(PROJECT).unwrap().unwrap().problems().len(), 1);
    let second = csharp_world(
        &[("scip-dotnet", ran("0.2.1")), ("dotnet", ran("8.0.404"))],
        None,
    );
    let (world, _) = run(second);
    assert_eq!(world.load(PROJECT).unwrap().unwrap().problems().len(), 0);
}

#[test]
fn an_unknown_project_is_the_only_error() {
    let (world, report) = run(World {
        missing_project: true,
        ..World::default()
    });
    assert!(report.is_err());
    assert_eq!(world.load(PROJECT).unwrap(), None);
}
