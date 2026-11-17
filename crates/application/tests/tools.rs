//! Tests of the tool check: what it finds out about the tools of a project, and what it stores.
#![cfg(test)]

use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex, PoisonError};

use codeskimr_application::cancel::CancelToken;
use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{
    Clock, IndexArtifact, IndexJob, IndexRunner, IndexerSpec, IndexerStatus, LanguageShare,
    PreviousIndex, ProjectCatalog, ProjectOverview, ProjectRepository, ProjectToolCheck,
    RequirementReader, RequirementValue, ToolCatalog, ToolIssue, ToolPlan, ToolProblem,
    ToolReportStore, ToolRequirement, ToolSeen, ToolSpec, ToolVersion,
};
use codeskimr_application::tools::{IndexerCheck, IndexerOverview, ToolCheckPorts, ToolChecker};
use codeskimr_domain::{
    Ceiling, FolderPath, LanguageId, Project, ProjectId, ProjectName, ProjectSource,
    RequirementFormat, RequirementSpec, RollForwardSpec, SourceFile, Timestamp, ToolReport,
    ToolState,
};

const PROJECT: ProjectId = ProjectId::new(1);

fn language(name: &str) -> LanguageId {
    LanguageId::new(name).unwrap()
}

fn version(text: &str) -> ToolVersion {
    ToolVersion::find_in(text).unwrap()
}

fn tool(program: &str, min: &str) -> ToolSpec {
    ToolSpec {
        program: program.to_owned(),
        version_args: vec!["--version".to_owned()],
        min_version: version(min),
    }
}

/// The rule of the C# pack: no policy means the same feature band, `latestMinor` the same major.
fn roll_forward() -> RollForwardSpec {
    RollForwardSpec {
        path: vec!["policy".to_owned()],
        default: Ceiling::SameBand,
        values: BTreeMap::from([
            ("disable".to_owned(), Ceiling::Exact),
            ("latestFeature".to_owned(), Ceiling::SameMinor),
            ("latestMinor".to_owned(), Ceiling::SameMajor),
            ("latestMajor".to_owned(), Ceiling::Any),
        ]),
    }
}

fn plan(indexer: &str, toolchain: &str, requirement: Option<&str>) -> ToolPlan {
    ToolPlan {
        indexer: IndexerSpec {
            name: indexer.to_owned(),
            indexer: tool(indexer, "0.2.0"),
            toolchain: vec![tool(toolchain, "8.0.0")],
            args: vec!["{output}".to_owned()],
            env_allow: Vec::new(),
            home_dirs: Vec::new(),
            env: Vec::new(),
            install_hint: "install it".to_owned(),
            symbol_kinds: Vec::new(),
            separator: String::new(),
            library_root: String::new(),
        },
        requirements: requirement
            .map(|file| ToolRequirement {
                program: toolchain.to_owned(),
                file: RequirementSpec {
                    file: file.to_owned(),
                    format: RequirementFormat::Text,
                    path: Vec::new(),
                    roll_forward: Some(roll_forward()),
                },
            })
            .into_iter()
            .collect(),
    }
}

/// The machine, the project and the store, all in one fake.
#[derive(Default)]
struct World {
    languages: Vec<&'static str>,
    plans: HashMap<String, ToolPlan>,
    /// What each installed program prints for its version.
    installed: HashMap<String, String>,
    files: HashMap<String, String>,
    /// The policy that the file of that path holds besides its version.
    policies: HashMap<String, String>,
    asked: Mutex<Vec<IndexerSpec>>,
    stored: Mutex<Option<ToolReport>>,
    missing_project: bool,
}

impl ToolCatalog for World {
    fn plan_of(&self, language: &LanguageId) -> Option<ToolPlan> {
        self.plans.get(language.as_str()).cloned()
    }

    fn plans(&self) -> Vec<(LanguageId, ToolPlan)> {
        let mut all: Vec<_> = self
            .plans
            .iter()
            .map(|(language, plan)| (LanguageId::new(language).unwrap(), plan.clone()))
            .collect();
        all.sort_by(|left: &(LanguageId, ToolPlan), right| left.0.as_str().cmp(right.0.as_str()));
        all
    }
}

impl IndexRunner for World {
    fn check(&self, spec: &IndexerSpec) -> IndexerStatus {
        self.asked
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(spec.clone());
        let mut problems = Vec::new();
        let mut found = Vec::new();
        for tool in std::iter::once(&spec.indexer).chain(&spec.toolchain) {
            let issue = match self.installed.get(&tool.program) {
                None => ToolIssue::Missing,
                // `crash: text` stands for a program that is found but whose version command fails.
                Some(output) if output.starts_with(CRASH_PREFIX) => ToolIssue::NotRunnable {
                    output: output[CRASH_PREFIX.len()..].to_owned(),
                },
                Some(output) => match ToolVersion::find_in(output) {
                    Some(version) if version >= tool.min_version => {
                        found.push(ToolSeen {
                            program: tool.program.clone(),
                            version,
                        });
                        continue;
                    }
                    _ => ToolIssue::TooOld {
                        found: output.clone(),
                        required: tool.min_version,
                    },
                },
            };
            problems.push(ToolProblem {
                program: tool.program.clone(),
                issue,
            });
        }
        if problems.is_empty() {
            IndexerStatus::Ready { found }
        } else {
            IndexerStatus::Unavailable { problems, found }
        }
    }

    fn run(&self, _: &IndexJob<'_>, _: &CancelToken) -> Result<IndexArtifact, CodedError> {
        unreachable!("the check never runs an indexer")
    }
}

impl RequirementReader for World {
    fn read(&self, _: &ProjectSource, requirement: &RequirementSpec) -> Option<String> {
        self.files.get(&requirement.file).cloned()
    }

    /// The file in the root and every one of that name in a folder below (`api/global.json`).
    fn read_all(&self, _: &ProjectSource, requirement: &RequirementSpec) -> Vec<RequirementValue> {
        let below = format!("/{}", requirement.file);
        self.files
            .iter()
            .filter(|(path, _)| **path == requirement.file || path.ends_with(&below))
            .map(|(path, text)| RequirementValue {
                version: text.clone(),
                policy: self.policies.get(path).cloned(),
            })
            .collect()
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
                parse_micros: None,
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
        runner: world.clone(),
        requirements: world.clone(),
        store: world.clone(),
        projects: world.clone(),
        overview: world.clone(),
        clock: world.clone(),
    });
    let result = checker.check(PROJECT);
    (world, result)
}

fn csharp_world(installed: &[(&str, &str)], global_json: Option<&str>) -> World {
    World {
        languages: vec!["csharp"],
        plans: HashMap::from([(
            "csharp".to_owned(),
            plan("scip-dotnet", "dotnet", Some("global.json")),
        )]),
        installed: installed
            .iter()
            .map(|(program, output)| ((*program).to_owned(), (*output).to_owned()))
            .collect(),
        files: global_json
            .map(|text| HashMap::from([("global.json".to_owned(), text.to_owned())]))
            .unwrap_or_default(),
        ..World::default()
    }
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
    let world = csharp_world(&[("scip-dotnet", "0.2.1"), ("dotnet", "8.0.404")], None);
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
    assert_eq!(
        report.findings[1].found.as_deref(),
        Some("8.0.404"),
        "a ready tool shows the version the runner found"
    );
    assert_eq!(report.findings[1].required.as_deref(), Some("8.0.0"));
}

#[test]
fn a_tool_that_is_not_installed_is_missing_and_names_what_to_show() {
    let (_, report) = run(csharp_world(&[("dotnet", "8.0.404")], None));
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
        &[("scip-dotnet", "0.2.1"), ("dotnet", "8.0.404")],
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
fn the_newest_version_of_all_manifest_folders_counts() {
    let mut world = csharp_world(
        &[("scip-dotnet", "0.2.1"), ("dotnet", "8.0.404")],
        Some("8.0.100"),
    );
    // A nested solution asks for more than the root does.
    world
        .files
        .insert("services/api/global.json".to_owned(), "9.0.200".to_owned());
    let (_, report) = run(world);
    let dotnet = &report.unwrap().findings[1];
    assert_eq!(dotnet.state, ToolState::TooOld);
    assert_eq!(dotnet.required.as_deref(), Some("9.0.200"));
}

const CRASH_PREFIX: &str = "crash:";

#[test]
fn a_tool_that_does_not_start_is_not_runnable_with_what_it_said_and_blocks_level_2() {
    for (said, found) in [
        ("crash:SyntaxError: boom", Some("SyntaxError: boom")),
        ("crash:", None),
    ] {
        let world = csharp_world(&[("scip-dotnet", said), ("dotnet", "8.0.404")], None);
        let report = run(world).1.unwrap();
        let indexer = &report.findings[0];
        assert_eq!(indexer.state, ToolState::NotRunnable);
        assert_eq!(indexer.found.as_deref(), found);
        assert!(!report.level2_ready(&language("csharp")));
    }
}

#[test]
fn a_tool_that_prints_no_version_is_of_unknown_version() {
    let world = csharp_world(
        &[("scip-dotnet", "0.2.1"), ("dotnet", "no digits here")],
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
fn a_requirement_that_is_not_a_number_is_met() {
    let world = csharp_world(
        &[("scip-dotnet", "1.0"), ("dotnet", "8.0.404")],
        Some("stable"),
    );
    let (_, report) = run(world);
    assert!(report.unwrap().level2_ready(&language("csharp")));
}

#[test]
fn the_version_of_the_pack_is_a_minimum_too_and_the_stricter_one_counts() {
    // The pack asks for 8.0.0, the project for 8.0.500: the machine has 8.0.404.
    let mut world = csharp_world(&[("scip-dotnet", "0.2.1"), ("dotnet", "8.0.404")], None);
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
fn tools_that_languages_share_are_asked_for_once() {
    let shared = plan("scip-typescript", "node", None);
    let world = World {
        languages: vec!["typescript", "javascript"],
        plans: HashMap::from([
            ("typescript".to_owned(), shared.clone()),
            ("javascript".to_owned(), shared),
        ]),
        installed: HashMap::from([
            ("scip-typescript".to_owned(), "0.3.0".to_owned()),
            ("node".to_owned(), "v20.11.1".to_owned()),
        ]),
        ..World::default()
    };
    let (world, report) = run(world);
    assert_eq!(report.unwrap().findings.len(), 4);
    assert_eq!(world.asked.lock().unwrap().len(), 1);
}

#[test]
fn the_requirement_is_read_as_a_number_a_tool_prints_or_a_project_writes() {
    for (output, expected) in [
        ("8.0.404\n", "8.0.404"),
        ("v20.11.1", "20.11.1"),
        ("Python 3.12.1", "3.12.1"),
        ("cargo 1.82.0 (8f40fc59f 2024-08-21)", "1.82.0"),
    ] {
        assert_eq!(version(output).to_string(), expected, "{output}");
    }
    assert_eq!(
        ToolVersion::parse_requirement("20").unwrap().to_string(),
        "20.0.0"
    );
    assert_eq!(
        ToolVersion::parse_requirement(" v18.19.0\n")
            .unwrap()
            .to_string(),
        "18.19.0"
    );
    assert_eq!(
        ToolVersion::parse_requirement("3.12.1-rc1")
            .unwrap()
            .to_string(),
        "3.12.1"
    );
    for text in [
        "stable",
        "nightly-2024-01-01",
        "lts/*",
        "pypy3.10",
        "",
        ">=8",
    ] {
        assert!(ToolVersion::parse_requirement(text).is_none(), "{text}");
    }
}

#[test]
fn a_new_check_replaces_the_stored_report() {
    let first = csharp_world(&[("dotnet", "8.0.404")], None);
    let (world, _) = run(first);
    assert_eq!(world.load(PROJECT).unwrap().unwrap().problems().len(), 1);
    let second = csharp_world(&[("scip-dotnet", "0.2.1"), ("dotnet", "8.0.404")], None);
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

/// The machine has SDK `installed`; `global.json` names 8.0.100 and holds `policy` (or none).
fn too_new_world(installed: &str, policy: Option<&str>) -> World {
    let mut world = csharp_world(
        &[("scip-dotnet", "0.2.1"), ("dotnet", installed)],
        Some("8.0.100"),
    );
    if let Some(policy) = policy {
        world
            .policies
            .insert("global.json".to_owned(), policy.to_owned());
    }
    world
}

fn dotnet_of(world: World) -> (ToolState, Option<String>, Option<String>) {
    let report = run(world).1.unwrap();
    let dotnet = &report.findings[1];
    (dotnet.state, dotnet.found.clone(), dotnet.required.clone())
}

#[test]
fn an_sdk_beyond_the_feature_band_of_global_json_is_too_new() {
    // No rollForward means latestPatch: 8.0.1xx only; 8.0.404 is another band.
    let (state, found, required) = dotnet_of(too_new_world("8.0.404", None));
    assert_eq!(state, ToolState::TooNew);
    assert_eq!(found.as_deref(), Some("8.0.404"));
    assert_eq!(
        required.as_deref(),
        Some("8.0.100"),
        "what the project names"
    );
}

#[test]
fn a_newer_patch_in_the_same_band_is_ready() {
    assert_eq!(
        dotnet_of(too_new_world("8.0.150", None)).0,
        ToolState::Ready
    );
}

#[test]
fn the_policy_of_global_json_moves_the_ceiling() {
    let cases = [
        ("8.0.100", "disable", ToolState::Ready),
        ("8.0.101", "disable", ToolState::TooNew),
        ("8.0.404", "latestFeature", ToolState::Ready),
        ("8.1.100", "latestFeature", ToolState::TooNew),
        ("8.9.100", "latestMinor", ToolState::Ready),
        ("9.0.100", "latestMinor", ToolState::TooNew),
        ("9.0.100", "latestMajor", ToolState::Ready),
    ];
    for (installed, policy, expected) in cases {
        let (state, ..) = dotnet_of(too_new_world(installed, Some(policy)));
        assert_eq!(state, expected, "{installed} with {policy}");
    }
}

#[test]
fn a_policy_that_the_pack_does_not_know_limits_nothing() {
    let (state, ..) = dotnet_of(too_new_world("9.0.100", Some("someFutureValue")));
    assert_eq!(state, ToolState::Ready);
}

#[test]
fn a_tool_whose_version_is_unknown_is_never_too_new() {
    let mut world = too_new_world("8.0.404", None);
    world
        .installed
        .insert("dotnet".to_owned(), "no digits here".to_owned());
    assert_eq!(dotnet_of(world).0, ToolState::VersionUnknown);
}

#[test]
fn any_manifest_folder_can_make_the_tool_too_new_and_names_its_version() {
    let mut world = too_new_world("8.0.150", Some("latestMajor"));
    // A nested solution is strict: the SDK may only be 8.0.100 itself.
    world
        .files
        .insert("services/api/global.json".to_owned(), "8.0.100".to_owned());
    world
        .policies
        .insert("services/api/global.json".to_owned(), "disable".to_owned());
    let (state, found, required) = dotnet_of(world);
    assert_eq!(state, ToolState::TooNew);
    assert_eq!(found.as_deref(), Some("8.0.150"));
    assert_eq!(required.as_deref(), Some("8.0.100"));
}

#[test]
fn a_too_new_tool_does_not_make_level_2_ready() {
    let report = run(too_new_world("9.0.100", None)).1.unwrap();
    assert!(!report.level2_ready(&language("csharp")));
    assert_eq!(report.problems().len(), 1);
}

#[test]
fn a_ceiling_of_the_toolchain_tool_does_not_judge_the_indexer() {
    // The limit belongs to `dotnet`; scip-dotnet 0.9.0 is not judged by it.
    let world = csharp_world(
        &[("scip-dotnet", "0.9.0"), ("dotnet", "8.0.150")],
        Some("8.0.100"),
    );
    let report = run(world).1.unwrap();
    assert_eq!(report.findings[0].state, ToolState::Ready);
}

#[test]
fn the_listing_of_all_indexers_needs_no_project_and_reports_every_tool_once() {
    let world = Arc::new(csharp_world(
        &[
            ("scip-dotnet", "crash:SyntaxError: boom"),
            ("dotnet", "8.0.404"),
        ],
        None,
    ));
    let overview = IndexerOverview::new(world.clone(), world.clone(), world.clone());
    let view = overview.check();
    let states: Vec<_> = view
        .tools
        .iter()
        .map(|tool| (tool.tool.as_str(), tool.state, tool.found.as_deref()))
        .collect();
    assert_eq!(
        states,
        [
            ("dotnet", "ready", Some("8.0.404")),
            ("scip-dotnet", "not_runnable", Some("SyntaxError: boom")),
        ]
    );
    assert!(view.tools.iter().all(|tool| tool.path.is_none()));
    assert!(view.checked_at.is_some());
}
