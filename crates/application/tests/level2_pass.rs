//! The second pass of a scan: who runs, what is replaced, and what stays level 1 (ADR 0033).

#[cfg(test)]
mod common;

#[cfg(test)]
mod cases {
    use super::common::InMemoryConsent;
    use std::sync::{Arc, Mutex, PoisonError};

    use codeskimr_application::cancel::CancelToken;
    use codeskimr_application::config::{Config, ConfigLayer, LayerSource};
    use codeskimr_application::error::CodedError;
    use codeskimr_application::ids::ProjectIds;
    use codeskimr_application::level2::{
        LanguageReport, Level2Outcome, Level2Pass, Level2Result, Skipped, confirm,
    };
    use codeskimr_application::ports::{
        ConsentStore, DiscoveredFile, FileDiscovery, ImportRequest, Imported, IndexArtifact,
        IndexJob, IndexRunner, IndexerSpec, IndexerStatus, Level2Replacement, PendingReference,
        PreviousIndex, ProjectRepository, ReferenceImporter, SnapshotData, SnapshotStore,
        StoredNames, ToolSpec, ToolVersion,
    };
    use codeskimr_codes::ErrorCode;
    use codeskimr_domain::{
        ContentHash, FileId, FolderPath, LanguageId, Loc, Permission, PermissionSet, Position,
        Principal, Project, ProjectId, ProjectName, ProjectSource, QualifiedName, Reference,
        ReferenceId, ReferenceKind, RelativePath, ScanMode, Snapshot, SnapshotId, SnapshotStats,
        SourceFile, Span, SymbolId, Timestamp,
    };

    const PROJECT: ProjectId = ProjectId::new(7);
    const SNAPSHOT: SnapshotId = SnapshotId::new(5);

    fn language(name: &str) -> LanguageId {
        LanguageId::new(name).unwrap()
    }

    fn file(id: u64, path: &str, lang: &str) -> SourceFile {
        SourceFile {
            id: FileId::new(id),
            snapshot: SNAPSHOT,
            path: RelativePath::new(path).unwrap(),
            language: language(lang),
            content_hash: ContentHash::new("h").unwrap(),
            loc: Loc(3),
        }
    }

    fn snapshot() -> Snapshot {
        Snapshot {
            id: SNAPSHOT,
            project: PROJECT,
            commit: None,
            created: Timestamp(1),
            mode: ScanMode::Full,
            stats: SnapshotStats {
                files: 2,
                symbols: 2,
                references: 1,
            },
        }
    }

    fn spec(name: &str) -> IndexerSpec {
        IndexerSpec {
            name: name.to_owned(),
            indexer: ToolSpec {
                program: name.to_owned(),
                version_args: Vec::new(),
                min_version: ToolVersion::parse("1.0").unwrap(),
            },
            ..IndexerSpec::default()
        }
    }

    /// The repository, the disk and the store in one.
    struct World {
        files: Vec<SourceFile>,
        changed: Vec<&'static str>,
        replaced: Mutex<Vec<Level2Replacement>>,
        /// The paths the pass asked to read again, one list per question.
        asked: Mutex<Vec<Vec<String>>>,
        files_fail: bool,
        /// The names of imports the store holds, and the failure it reports instead.
        import_names: Vec<PendingReference>,
        names_fail: Option<ErrorCode>,
    }

    impl ProjectRepository for World {
        fn project(&self, _: ProjectId) -> Result<Project, CodedError> {
            Ok(Project {
                id: PROJECT,
                name: ProjectName::new("demo").unwrap(),
                source: ProjectSource::Folder(FolderPath::new("/work").unwrap()),
                layers: Vec::new(),
            })
        }

        fn latest_files(&self, _: ProjectId) -> Result<Vec<SourceFile>, CodedError> {
            if self.files_fail {
                return Err(CodedError::new(ErrorCode::ProjectNotFound));
            }
            Ok(self.files.clone())
        }

        fn latest_index(&self, _: ProjectId) -> Result<PreviousIndex, CodedError> {
            Ok(PreviousIndex::default())
        }

        fn stored_names(
            &self,
            _: SnapshotId,
            _: StoredNames<'_>,
        ) -> Result<Vec<PendingReference>, CodedError> {
            self.names_fail.map_or_else(
                || Ok(self.import_names.clone()),
                |code| Err(CodedError::new(code)),
            )
        }
    }

    impl FileDiscovery for World {
        fn discover(
            &self,
            _: &FolderPath,
            previous: &[SourceFile],
            _: &CancelToken,
        ) -> Result<Vec<DiscoveredFile>, CodedError> {
            Ok(previous
                .iter()
                .map(|file| DiscoveredFile {
                    path: file.path.clone(),
                    language: file.language.clone(),
                    content_hash: file.content_hash.clone(),
                    changed: self.changed.contains(&file.path.as_str()),
                })
                .collect())
        }

        fn unchanged_files(
            &self,
            _: &FolderPath,
            files: &[SourceFile],
            _: &CancelToken,
        ) -> Result<Vec<RelativePath>, CodedError> {
            self.asked
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .push(files.iter().map(|file| file.path.to_string()).collect());
            Ok(files
                .iter()
                .filter(|file| !self.changed.contains(&file.path.as_str()))
                .map(|file| file.path.clone())
                .collect())
        }
    }

    impl SnapshotStore for World {
        fn persist(&self, _: SnapshotData) -> Result<Snapshot, CodedError> {
            Err(CodedError::new(ErrorCode::UnexpectedInternalError))
        }

        fn next_free_ids(&self, _project: ProjectId) -> Result<(u64, u64), CodedError> {
            Ok((1000, 0))
        }

        fn confirm(
            &self,
            _: &codeskimr_application::ports::Confirmation,
        ) -> Result<Option<codeskimr_application::ports::Confirmed>, CodedError> {
            Ok(None)
        }

        fn replace_level2(&self, replacement: Level2Replacement) -> Result<Snapshot, CodedError> {
            self.replaced
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .push(replacement);
            Ok(Snapshot {
                stats: SnapshotStats {
                    references: 99,
                    ..snapshot().stats
                },
                ..snapshot()
            })
        }
    }

    /// An indexer that writes nothing and counts its runs; `fails` is what `run` reports.
    struct Runner {
        runs: Mutex<u32>,
        status: IndexerStatus,
        fails: Option<ErrorCode>,
        has_project_file: bool,
        /// The workspaces the runner names; empty: the project root alone, if it has the file.
        workspaces: Vec<&'static str>,
    }

    impl IndexRunner for Runner {
        fn check(&self, _: &IndexerSpec) -> IndexerStatus {
            self.status.clone()
        }

        fn has_project_file(&self, _: &IndexerSpec, _: &std::path::Path) -> bool {
            self.has_project_file
        }

        fn workspaces(
            &self,
            _: &IndexerSpec,
            _: &std::path::Path,
            _: &[&RelativePath],
        ) -> Vec<Option<RelativePath>> {
            if !self.workspaces.is_empty() {
                return self
                    .workspaces
                    .iter()
                    .map(|folder| RelativePath::new(*folder).ok())
                    .collect();
            }
            if self.has_project_file {
                vec![None]
            } else {
                Vec::new()
            }
        }

        fn run(&self, _: &IndexJob<'_>, _: &CancelToken) -> Result<IndexArtifact, CodedError> {
            *self.runs.lock().unwrap_or_else(PoisonError::into_inner) += 1;
            self.fails.map_or_else(
                || Ok(IndexArtifact::new("index.scip".into(), Box::new(()))),
                |code| Err(CodedError::new(code)),
            )
        }
    }

    /// An importer that remembers which files it was given and covers all of them.
    struct Importer {
        seen: Mutex<Vec<Vec<String>>>,
        /// The import sites of the requests.
        sites: Mutex<Vec<(FileId, Span)>>,
        fails: Option<ErrorCode>,
        /// The folder of each request, in order.
        places: Mutex<Vec<Option<String>>>,
        cancels: Option<CancelToken>,
        /// The ids (of the importer's own numbering) of the symbols the import creates, each the
        /// child of the one before, and a use of the last.
        creates: Vec<u64>,
    }

    /// The symbol an import creates, numbered by the importer from 500.
    fn created(id: u64, file: FileId, parent: Option<u64>) -> codeskimr_domain::Symbol {
        codeskimr_domain::Symbol {
            id: SymbolId::new(id),
            file,
            kind: codeskimr_domain::SymbolKind::Enum,
            name: codeskimr_domain::SymbolName::new("Gen").unwrap(),
            qualified_name: codeskimr_domain::QualifiedName::new("app::Gen").unwrap(),
            parent: parent.map(SymbolId::new),
            visibility: None,
            is_static: false,
            span: codeskimr_domain::Span::new(
                codeskimr_domain::Position { line: 1, column: 0 },
                codeskimr_domain::Position { line: 2, column: 0 },
            )
            .unwrap(),
            signature: None,
            doc: None,
            origin: codeskimr_domain::SymbolOrigin::Index,
        }
    }

    impl Importer {
        /// Remembers the files and the import sites of a request.
        fn remember(&self, request: &ImportRequest<'_>) {
            self.places
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .push(request.workspace.map(|folder| folder.as_str().to_owned()));
            let paths = request
                .files
                .iter()
                .map(|f| f.path.as_str().to_owned())
                .collect();
            self.seen
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .push(paths);
            self.sites
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .extend_from_slice(request.import_sites);
        }
    }

    impl ReferenceImporter for Importer {
        fn import(&self, request: &ImportRequest<'_>) -> Result<Imported, CodedError> {
            self.remember(request);
            if let Some(token) = &self.cancels {
                token.cancel();
            }
            if let Some(code) = self.fails {
                return Err(CodedError::new(code));
            }
            let first = request.files.first().map(|file| file.id);
            let symbols = first.map_or_else(Vec::new, |file| {
                let parents = std::iter::once(1).chain(self.creates.iter().copied());
                let pairs = self.creates.iter().copied().zip(parents);
                pairs
                    .map(|(id, parent)| created(id, file, Some(parent)))
                    .collect()
            });
            let target = self.creates.last().copied().unwrap_or(2);
            Ok(Imported {
                files: request.files.iter().map(|file| file.id).collect(),
                symbols,
                references: first.map_or_else(Vec::new, |file| {
                    vec![Reference {
                        to: SymbolId::new(target),
                        ..level_two(file)
                    }]
                }),
                mapped: 4,
                dropped: 3,
                outside_scan: 2,
            })
        }
    }

    fn level_two(file: FileId) -> Reference {
        Reference {
            id: ReferenceId::new(0),
            from: SymbolId::new(1),
            to: SymbolId::new(2),
            file,
            span: codeskimr_domain::Span::new(
                codeskimr_domain::Position { line: 1, column: 1 },
                codeskimr_domain::Position { line: 1, column: 2 },
            )
            .unwrap(),
            kind: ReferenceKind::Call,
            confidence: codeskimr_domain::Confidence::Certain,
            verified: true,
        }
    }

    /// Whether the folder holds the file the indexer needs.
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum ProjectFile {
        Present,
        Missing,
    }

    /// Everything a case may change.
    struct Setup {
        files: Vec<SourceFile>,
        changed: Vec<&'static str>,
        indexers: Vec<(&'static str, &'static str)>,
        status: IndexerStatus,
        run_fails: Option<ErrorCode>,
        project_file: ProjectFile,
        import_fails: Option<ErrorCode>,
        cancel_in_import: bool,
        files_fail: bool,
        confirmed: bool,
        creates: Vec<u64>,
        import_names: Vec<PendingReference>,
        names_fail: Option<ErrorCode>,
        workspaces: Vec<&'static str>,
    }

    impl Default for Setup {
        fn default() -> Self {
            Self {
                files: vec![file(1, "a.cs", "csharp")],
                changed: Vec::new(),
                indexers: vec![("csharp", "scip-dotnet")],
                status: IndexerStatus::ready(),
                run_fails: None,
                project_file: ProjectFile::Present,
                import_fails: None,
                cancel_in_import: false,
                files_fail: false,
                confirmed: true,
                creates: Vec::new(),
                import_names: Vec::new(),
                names_fail: None,
                workspaces: Vec::new(),
            }
        }
    }

    struct Outcome {
        result: Level2Result,
        runs: u32,
        imports: Vec<Vec<String>>,
        replaced: Vec<Level2Replacement>,
        asked: Vec<Vec<String>>,
        sites: Vec<(FileId, Span)>,
    }

    /// The ports of a case, kept to read what the pass did to them.
    struct Fakes {
        world: Arc<World>,
        runner: Arc<Runner>,
        importer: Arc<Importer>,
    }

    fn consent(confirmed: bool) -> Arc<dyn ConsentStore> {
        let consent: Arc<dyn ConsentStore> = Arc::new(InMemoryConsent::default());
        if confirmed {
            confirm(&Principal::local_owner(), consent.as_ref(), PROJECT).unwrap();
        }
        consent
    }

    fn fakes_for(setup: &mut Setup, cancel: &CancelToken) -> Fakes {
        Fakes {
            world: Arc::new(World {
                files: std::mem::take(&mut setup.files),
                changed: std::mem::take(&mut setup.changed),
                replaced: Mutex::default(),
                asked: Mutex::default(),
                files_fail: setup.files_fail,
                import_names: std::mem::take(&mut setup.import_names),
                names_fail: setup.names_fail,
            }),
            runner: Arc::new(Runner {
                runs: Mutex::default(),
                status: setup.status.clone(),
                fails: setup.run_fails,
                has_project_file: setup.project_file == ProjectFile::Present,
                workspaces: setup.workspaces.clone(),
            }),
            importer: Arc::new(Importer {
                seen: Mutex::default(),
                sites: Mutex::default(),
                places: Mutex::default(),
                fails: setup.import_fails,
                cancels: setup.cancel_in_import.then(|| cancel.clone()),
                creates: setup.creates.clone(),
            }),
        }
    }

    /// The settings of an owner who switched level 2 on in the user configuration (ADR 0076).
    fn switched_on() -> codeskimr_application::config::Level2 {
        let layer = ConfigLayer::parse(LayerSource::User, "[level2]\nenabled = true\n").unwrap();
        Config::resolve(&[layer]).unwrap().level2
    }

    fn pass_for(mut setup: Setup, cancel: &CancelToken) -> (Level2Pass, Fakes) {
        let fakes = fakes_for(&mut setup, cancel);
        let (world, runner, importer) = (&fakes.world, &fakes.runner, &fakes.importer);
        let consent = consent(setup.confirmed);
        let indexers = setup
            .indexers
            .iter()
            .map(|(lang, name)| (language(lang), spec(name)))
            .collect();
        let pass = Level2Pass {
            runner: runner.clone(),
            importer: importer.clone(),
            consent,
            discovery: world.clone(),
            projects: world.clone(),
            store: world.clone(),
            ids: Arc::new(ProjectIds::default()),
            max_file_bytes: 1 << 20,
            settings: switched_on(),
            indexers,
            events: codeskimr_application::events::EventBus::new(),
        };
        (pass, fakes)
    }

    fn run_with(setup: Setup, principal: Principal, cancel: &CancelToken) -> Outcome {
        let (pass, fakes) = pass_for(setup, cancel);
        let root = FolderPath::new("/work").unwrap();
        let result = pass.run(&principal, &snapshot(), &root, cancel);
        let runs = *fakes.runner.runs.lock().unwrap();
        let imports = fakes.importer.seen.lock().unwrap().clone();
        let replaced = fakes.world.replaced.lock().unwrap().clone();
        let asked = fakes.world.asked.lock().unwrap().clone();
        let sites = fakes.importer.sites.lock().unwrap().clone();
        Outcome {
            result,
            runs,
            imports,
            replaced,
            asked,
            sites,
        }
    }

    fn run(setup: Setup) -> Outcome {
        run_with(setup, Principal::local_owner(), &CancelToken::new())
    }

    fn skipped(outcome: &Outcome) -> Vec<&Skipped> {
        outcome
            .result
            .reports
            .iter()
            .filter_map(|report| match &report.outcome {
                Level2Outcome::Skipped(reason) => Some(reason),
                Level2Outcome::Checked { .. } => None,
            })
            .collect()
    }

    fn assert_level_one_stays(outcome: &Outcome, reason: &Skipped) {
        assert_eq!(skipped(outcome), vec![reason]);
        assert_eq!(outcome.replaced, Vec::new());
        assert_eq!(outcome.result.snapshot, snapshot());
    }

    #[test]
    fn a_confirmed_run_replaces_the_references_with_fresh_ids() {
        let outcome = run(Setup::default());
        assert_eq!(outcome.runs, 1);
        assert_eq!(
            outcome.result.reports,
            vec![LanguageReport {
                language: language("csharp"),
                outcome: Level2Outcome::Checked {
                    files: 1,
                    stale: 0,
                    mapped: 4,
                    dropped: 3,
                    outside_scan: 2,
                },
            }]
        );
        let [replacement] = outcome.replaced.as_slice() else {
            panic!("expected one replacement");
        };
        assert_eq!(replacement.snapshot, SNAPSHOT);
        assert_eq!(replacement.files, vec![FileId::new(1)]);
        assert_eq!(
            replacement
                .references
                .iter()
                .map(|r| r.id)
                .collect::<Vec<_>>(),
            vec![ReferenceId::new(1000)]
        );
        assert_eq!(outcome.result.snapshot.stats.references, 99);
    }

    #[test]
    fn created_symbols_get_ids_from_the_shared_sequence_and_everything_that_points_at_them_follows()
    {
        let outcome = run(Setup {
            creates: vec![500, 501],
            ..Setup::default()
        });
        let [replacement] = outcome.replaced.as_slice() else {
            panic!("expected one replacement");
        };
        // The references are numbered first (1000), then the two symbols (1001 and 1002).
        let ids: Vec<(u64, Option<u64>)> = replacement
            .symbols
            .iter()
            .map(|s| (s.id.get(), s.parent.map(SymbolId::get)))
            .collect();
        // The first symbol keeps its parent 1, a symbol of level 1, and the second follows the first.
        assert_eq!(ids, [(1001, Some(1)), (1002, Some(1001))]);
        let reference = replacement.references.first().expect("one reference");
        assert_eq!(
            (reference.id.get(), reference.from.get(), reference.to.get()),
            (1000, 1, 1002)
        );
    }

    #[test]
    fn without_a_confirmation_nothing_runs() {
        let outcome = run(Setup {
            confirmed: false,
            ..Setup::default()
        });
        assert_eq!(outcome.runs, 0);
        assert_level_one_stays(&outcome, &Skipped::NotConfirmed);
    }

    #[test]
    fn a_principal_without_the_right_does_not_start_anything() {
        let reader = Principal::limited(PermissionSet::from_iter([Permission::Scan]));
        let outcome = run_with(Setup::default(), reader, &CancelToken::new());
        assert_eq!(outcome.runs, 0);
        assert_level_one_stays(&outcome, &Skipped::NotPermitted);
    }

    #[test]
    fn a_folder_without_the_project_file_starts_nothing() {
        let outcome = run(Setup {
            project_file: ProjectFile::Missing,
            ..Setup::default()
        });
        assert_eq!(outcome.runs, 0);
        assert_level_one_stays(&outcome, &Skipped::NoProjectFile);
    }

    #[test]
    fn every_workspace_gets_its_own_run_and_import() {
        let (pass, fakes) = pass_for(
            Setup {
                files: vec![
                    file(1, "cli/a.rs", "csharp"),
                    file(2, "ext/x/b.rs", "csharp"),
                    file(3, "loose.rs", "csharp"),
                ],
                workspaces: vec!["cli", "ext/x"],
                ..Setup::default()
            },
            &CancelToken::new(),
        );
        let root = FolderPath::new("/work").unwrap();
        let result = pass.run(
            &Principal::local_owner(),
            &snapshot(),
            &root,
            &CancelToken::new(),
        );
        assert_eq!(*fakes.runner.runs.lock().unwrap(), 2);
        assert_eq!(
            *fakes.importer.places.lock().unwrap(),
            vec![Some("cli".to_owned()), Some("ext/x".to_owned())]
        );
        assert_eq!(
            *fakes.importer.seen.lock().unwrap(),
            vec![vec!["cli/a.rs".to_owned()], vec!["ext/x/b.rs".to_owned()]]
        );
        assert_eq!(counted(&result), Some((2, 0, 8, 6, 4)));
    }

    /// The numbers of the one checked language of a result: files, stale, mapped, dropped, outside.
    fn counted(result: &Level2Result) -> Option<(u64, u64, u64, u64, u64)> {
        match result.reports.as_slice() {
            [
                LanguageReport {
                    outcome:
                        Level2Outcome::Checked {
                            files,
                            stale,
                            mapped,
                            dropped,
                            outside_scan,
                        },
                    ..
                },
            ] => Some((*files, *stale, *mapped, *dropped, *outside_scan)),
            _ => None,
        }
    }

    #[test]
    fn a_workspace_that_fails_does_not_take_the_others_down() {
        let outcome = run(Setup {
            files: vec![
                file(1, "cli/a.rs", "csharp"),
                file(2, "ext/x/b.rs", "csharp"),
            ],
            changed: vec!["ext/x/b.rs"],
            workspaces: vec!["cli", "ext/x"],
            ..Setup::default()
        });
        assert_eq!(outcome.runs, 2);
        assert_eq!(outcome.imports, vec![vec!["cli/a.rs".to_owned()]]);
        assert_eq!(counted(&outcome.result), Some((1, 1, 4, 3, 2)));
    }

    #[test]
    fn when_every_workspace_fails_the_first_reason_is_reported() {
        let outcome = run(Setup {
            files: vec![
                file(1, "cli/a.rs", "csharp"),
                file(2, "ext/x/b.rs", "csharp"),
            ],
            run_fails: Some(ErrorCode::IndexerFailed),
            workspaces: vec!["cli", "ext/x"],
            ..Setup::default()
        });
        assert_eq!(outcome.runs, 2);
        assert_level_one_stays(&outcome, &Skipped::Failed(ErrorCode::IndexerFailed));
    }

    #[test]
    fn a_missing_tool_keeps_level_one() {
        let outcome = run(Setup {
            status: IndexerStatus::unavailable(Vec::new()),
            ..Setup::default()
        });
        assert_eq!(outcome.runs, 0);
        assert_level_one_stays(&outcome, &Skipped::Unavailable(Vec::new()));
    }

    #[test]
    fn languages_with_the_same_indexer_share_one_run() {
        let outcome = run(Setup {
            files: vec![file(1, "a.ts", "typescript"), file(2, "b.js", "javascript")],
            indexers: vec![
                ("typescript", "scip-typescript"),
                ("javascript", "scip-typescript"),
            ],
            ..Setup::default()
        });
        assert_eq!(outcome.runs, 1);
        assert_eq!(outcome.result.reports.len(), 2);
        assert_eq!(
            outcome.imports,
            vec![vec!["a.ts".to_owned(), "b.js".to_owned()]]
        );
    }

    #[test]
    fn after_a_run_only_the_files_of_its_group_are_read_again() {
        let outcome = run(Setup {
            files: vec![file(1, "a.ts", "typescript"), file(2, "b.py", "python")],
            indexers: vec![("typescript", "scip-typescript"), ("python", "scip-python")],
            ..Setup::default()
        });
        assert_eq!(outcome.runs, 2);
        let mut asked = outcome.asked.clone();
        asked.sort();
        assert_eq!(
            asked,
            vec![vec!["a.ts".to_owned()], vec!["b.py".to_owned()]]
        );
    }

    #[test]
    fn a_language_without_files_is_not_run() {
        let outcome = run(Setup {
            indexers: vec![("csharp", "scip-dotnet"), ("java", "scip-java")],
            ..Setup::default()
        });
        assert_eq!(outcome.runs, 1);
        assert_eq!(outcome.result.reports.len(), 1);
    }

    /// A pass over `files`, with these (language, indexer) pairs.
    fn pass_over(
        files: Vec<SourceFile>,
        indexers: Vec<(&'static str, &'static str)>,
    ) -> (Level2Pass, Fakes) {
        let setup = Setup {
            files,
            indexers,
            ..Setup::default()
        };
        pass_for(setup, &CancelToken::new())
    }

    #[test]
    fn the_languages_to_report_are_those_with_files() {
        let (pass, _) = pass_over(
            vec![file(1, "a.c", "c")],
            vec![("c", "scip-clang"), ("java", "scip-java")],
        );
        let found: Vec<String> = pass
            .languages_in(snapshot().project, None)
            .into_iter()
            .map(|language| language.as_str().to_owned())
            .collect();
        assert_eq!(found, vec!["c".to_owned()]);
    }

    #[test]
    fn a_run_for_one_language_leaves_the_others_alone() {
        let (pass, fakes) = pass_over(
            vec![file(1, "a.ts", "typescript"), file(2, "b.py", "python")],
            vec![("typescript", "scip-typescript"), ("python", "scip-python")],
        );
        let root = FolderPath::new("/work").unwrap();
        let only = LanguageId::new("python").unwrap();
        let result = pass.run_only(
            &Principal::local_owner(),
            (&snapshot(), &root),
            &CancelToken::new(),
            Some(&only),
        );
        assert_eq!(*fakes.runner.runs.lock().unwrap(), 1);
        assert_eq!(result.reports.len(), 1);
        assert_eq!(result.reports[0].language, only);
    }

    #[test]
    fn a_file_that_changed_since_the_scan_keeps_level_one() {
        let outcome = run(Setup {
            files: vec![file(1, "a.cs", "csharp"), file(2, "b.cs", "csharp")],
            changed: vec!["b.cs"],
            ..Setup::default()
        });
        assert_eq!(outcome.imports, vec![vec!["a.cs".to_owned()]]);
        assert_eq!(counted(&outcome.result), Some((1, 1, 4, 3, 2)));
    }

    #[test]
    fn when_every_file_changed_nothing_is_imported() {
        let outcome = run(Setup {
            changed: vec!["a.cs"],
            ..Setup::default()
        });
        assert_eq!(outcome.runs, 1);
        assert_eq!(outcome.imports, Vec::<Vec<String>>::new());
        assert_level_one_stays(&outcome, &Skipped::Stale);
    }

    #[test]
    fn a_cancel_during_the_run_keeps_level_one() {
        let outcome = run(Setup {
            run_fails: Some(ErrorCode::ScanCancelled),
            ..Setup::default()
        });
        assert_level_one_stays(&outcome, &Skipped::Cancelled);
    }

    #[test]
    fn a_cancel_before_the_start_runs_nothing() {
        let cancel = CancelToken::new();
        cancel.cancel();
        let outcome = run_with(Setup::default(), Principal::local_owner(), &cancel);
        assert_eq!(outcome.runs, 0);
        assert_level_one_stays(&outcome, &Skipped::Cancelled);
    }

    #[test]
    fn a_failed_run_or_import_keeps_level_one_and_names_the_code() {
        let run_failed = run(Setup {
            run_fails: Some(ErrorCode::UnexpectedInternalError),
            ..Setup::default()
        });
        assert_level_one_stays(
            &run_failed,
            &Skipped::Failed(ErrorCode::UnexpectedInternalError),
        );
        let import_failed = run(Setup {
            import_fails: Some(ErrorCode::ScipMappingTooLow),
            ..Setup::default()
        });
        assert_level_one_stays(
            &import_failed,
            &Skipped::Failed(ErrorCode::ScipMappingTooLow),
        );
    }

    fn import_name(file: u64) -> PendingReference {
        let span = Span::new(
            Position { line: 0, column: 7 },
            Position {
                line: 0,
                column: 12,
            },
        )
        .unwrap();
        PendingReference {
            from: SymbolId::new(1),
            file: FileId::new(file),
            span,
            kind: ReferenceKind::Import,
            target: QualifiedName::new("a.b").unwrap(),
            receiver: None,
        }
    }

    #[test]
    fn the_import_gets_the_places_of_the_import_names_of_level_one() {
        let outcome = run(Setup {
            import_names: vec![import_name(1)],
            ..Setup::default()
        });
        assert_eq!(outcome.sites, [(FileId::new(1), import_name(1).span)]);
        assert_eq!(outcome.replaced.len(), 1);
    }

    #[test]
    fn a_store_that_cannot_give_the_import_names_keeps_level_one_and_names_the_code() {
        let outcome = run(Setup {
            names_fail: Some(ErrorCode::ProjectNotFound),
            ..Setup::default()
        });
        assert_level_one_stays(&outcome, &Skipped::Failed(ErrorCode::ProjectNotFound));
        assert_eq!(outcome.sites, Vec::<(FileId, Span)>::new());
    }

    #[test]
    fn a_cancel_after_the_import_replaces_nothing() {
        let outcome = run(Setup {
            cancel_in_import: true,
            ..Setup::default()
        });
        assert_eq!(outcome.imports.len(), 1);
        assert_level_one_stays(&outcome, &Skipped::Cancelled);
    }

    #[test]
    fn when_the_snapshot_cannot_be_read_every_indexer_language_is_reported_failed() {
        let outcome = run(Setup {
            files_fail: true,
            indexers: vec![("csharp", "scip-dotnet"), ("java", "scip-java")],
            ..Setup::default()
        });
        assert_eq!(outcome.runs, 0);
        let failed = Skipped::Failed(ErrorCode::ProjectNotFound);
        assert_eq!(skipped(&outcome), vec![&failed, &failed]);
    }
}
