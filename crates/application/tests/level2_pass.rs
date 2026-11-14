//! The second pass of a scan: who runs, what is replaced, and what stays level 1 (ADR 0033).

#[cfg(test)]
mod cases {
    use std::sync::{Arc, Mutex, PoisonError};

    use codeskimr_application::cancel::CancelToken;
    use codeskimr_application::config::Config;
    use codeskimr_application::error::CodedError;
    use codeskimr_application::ids::IdSequence;
    use codeskimr_application::level2::{
        InMemoryConsent, LanguageReport, Level2Outcome, Level2Pass, Level2Result, Skipped, confirm,
    };
    use codeskimr_application::ports::{
        ConsentStore, DiscoveredFile, FileDiscovery, ImportRequest, Imported, IndexArtifact,
        IndexJob, IndexRunner, IndexerSpec, IndexerStatus, PreviousIndex, ProjectRepository,
        ReferenceImporter, ReferenceReplacement, SnapshotData, SnapshotStore, ToolSpec,
        ToolVersion,
    };
    use codeskimr_codes::ErrorCode;
    use codeskimr_domain::{
        ContentHash, FileId, FolderPath, LanguageId, Loc, Permission, PermissionSet, Principal,
        Project, ProjectId, ProjectName, ProjectSource, Reference, ReferenceId, ReferenceKind,
        RelativePath, ScanMode, Snapshot, SnapshotId, SnapshotStats, SourceFile, SymbolId,
        Timestamp,
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
            toolchain: Vec::new(),
            args: Vec::new(),
            env_allow: Vec::new(),
            env: Vec::new(),
            install_hint: String::new(),
        }
    }

    /// The repository, the disk and the store in one.
    struct World {
        files: Vec<SourceFile>,
        changed: Vec<&'static str>,
        replaced: Mutex<Vec<ReferenceReplacement>>,
        files_fail: bool,
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
    }

    impl SnapshotStore for World {
        fn persist(&self, _: SnapshotData) -> Result<Snapshot, CodedError> {
            Err(CodedError::new(ErrorCode::UnexpectedInternalError))
        }

        fn replace_references(
            &self,
            replacement: ReferenceReplacement,
        ) -> Result<Snapshot, CodedError> {
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
    }

    impl IndexRunner for Runner {
        fn check(&self, _: &IndexerSpec) -> IndexerStatus {
            self.status.clone()
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
        fails: Option<ErrorCode>,
        cancels: Option<CancelToken>,
    }

    impl ReferenceImporter for Importer {
        fn import(&self, request: &ImportRequest<'_>) -> Result<Imported, CodedError> {
            self.seen
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .push(
                    request
                        .files
                        .iter()
                        .map(|f| f.path.as_str().to_owned())
                        .collect(),
                );
            if let Some(token) = &self.cancels {
                token.cancel();
            }
            if let Some(code) = self.fails {
                return Err(CodedError::new(code));
            }
            let first = request.files.first().map(|file| file.id);
            Ok(Imported {
                files: request.files.iter().map(|file| file.id).collect(),
                references: first.map_or_else(Vec::new, |file| vec![level_two(file)]),
                mapped: 4,
                dropped: 1,
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

    /// Everything a case may change.
    struct Setup {
        files: Vec<SourceFile>,
        changed: Vec<&'static str>,
        indexers: Vec<(&'static str, &'static str)>,
        status: IndexerStatus,
        run_fails: Option<ErrorCode>,
        import_fails: Option<ErrorCode>,
        cancel_in_import: bool,
        files_fail: bool,
        confirmed: bool,
    }

    impl Default for Setup {
        fn default() -> Self {
            Self {
                files: vec![file(1, "a.cs", "csharp")],
                changed: Vec::new(),
                indexers: vec![("csharp", "scip-dotnet")],
                status: IndexerStatus::Ready,
                run_fails: None,
                import_fails: None,
                cancel_in_import: false,
                files_fail: false,
                confirmed: true,
            }
        }
    }

    struct Outcome {
        result: Level2Result,
        runs: u32,
        imports: Vec<Vec<String>>,
        replaced: Vec<ReferenceReplacement>,
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
                files_fail: setup.files_fail,
            }),
            runner: Arc::new(Runner {
                runs: Mutex::default(),
                status: setup.status.clone(),
                fails: setup.run_fails,
            }),
            importer: Arc::new(Importer {
                seen: Mutex::default(),
                fails: setup.import_fails,
                cancels: setup.cancel_in_import.then(|| cancel.clone()),
            }),
        }
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
            ids: Arc::new(IdSequence::starting_at(1000)),
            max_file_bytes: 1 << 20,
            settings: Config::resolve(&[]).unwrap().level2,
            indexers,
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
        Outcome {
            result,
            runs,
            imports,
            replaced,
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
                    dropped: 1
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
    fn a_missing_tool_keeps_level_one() {
        let outcome = run(Setup {
            status: IndexerStatus::Unavailable(Vec::new()),
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
    fn a_language_without_files_is_not_run() {
        let outcome = run(Setup {
            indexers: vec![("csharp", "scip-dotnet"), ("java", "scip-java")],
            ..Setup::default()
        });
        assert_eq!(outcome.runs, 1);
        assert_eq!(outcome.result.reports.len(), 1);
    }

    #[test]
    fn a_file_that_changed_since_the_scan_keeps_level_one() {
        let outcome = run(Setup {
            files: vec![file(1, "a.cs", "csharp"), file(2, "b.cs", "csharp")],
            changed: vec!["b.cs"],
            ..Setup::default()
        });
        assert_eq!(outcome.imports, vec![vec!["a.cs".to_owned()]]);
        assert!(matches!(
            outcome.result.reports.as_slice(),
            [LanguageReport {
                outcome: Level2Outcome::Checked {
                    files: 1,
                    stale: 1,
                    ..
                },
                ..
            }]
        ));
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
