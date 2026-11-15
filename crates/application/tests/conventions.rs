//! Tests of the convention check with fake ports.

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use codeskimr_application::cancel::CancelToken;
    use codeskimr_application::config::Conventions;
    use codeskimr_application::conventions::{
        CheckRequest, ConventionChecker, Detail, StoredProject,
    };
    use codeskimr_application::error::CodedError;
    use codeskimr_application::ports::{
        AnalyzeRequest, DiscoveredFile, Excerpt, FileAnalyzer, FileDiscovery, FileFacts,
        SimilarHit, SimilarQuery, SimilarityIndex, SourceText, SymbolFingerprints,
    };
    use codeskimr_codes::{Coded, ErrorCode};
    use codeskimr_domain::{
        BodyCounts, ContentHash, FingerprintParams, FolderPath, LanguageId, Loc, Permission,
        PermissionSet, Position, Principal, QualifiedName, RelativePath, SnapshotId, SourceFile,
        Span, Symbol, SymbolCounts, SymbolId, SymbolKind, SymbolName, TokenId, winnow,
    };

    /// What the fake analyser finds in a file: code lines and the seed of its token stream.
    struct Body {
        loc: u32,
        seed: u32,
        /// A second function with the same body inside the first one.
        inner: bool,
        /// Cognitive complexity and parameters of its functions; `None` is a pack that does not count.
        counted: Option<(u32, u32)>,
    }

    /// Files by path; `broken` fails to analyse.
    struct Disk {
        files: Vec<(&'static str, Body)>,
        broken: Option<&'static str>,
    }

    fn model<T>(value: Result<T, codeskimr_domain::ModelError>) -> Result<T, CodedError> {
        value.map_err(|_| CodedError::new(ErrorCode::UnexpectedInternalError))
    }

    impl FileDiscovery for Disk {
        fn discover(
            &self,
            _: &FolderPath,
            _: &[SourceFile],
            _: &CancelToken,
        ) -> Result<Vec<DiscoveredFile>, CodedError> {
            self.files
                .iter()
                .map(|(path, _)| *path)
                .chain(self.broken)
                .map(|path| {
                    Ok(DiscoveredFile {
                        path: model(RelativePath::new(path))?,
                        language: model(LanguageId::new("rust"))?,
                        content_hash: model(ContentHash::new("h"))?,
                        changed: true,
                    })
                })
                .collect()
        }
    }

    fn function(
        request: &AnalyzeRequest<'_>,
        file: &SourceFile,
        lines: (u32, u32),
    ) -> Result<Symbol, CodedError> {
        let start = Position {
            line: lines.0,
            column: 0,
        };
        let end = Position {
            line: lines.1,
            column: 0,
        };
        let path = request.file.path.as_str();
        Ok(Symbol {
            id: request.ids.symbol(),
            file: file.id,
            kind: SymbolKind::Function,
            name: model(SymbolName::new("run"))?,
            qualified_name: model(QualifiedName::new(format!("{path}::run")))?,
            parent: None,
            visibility: None,
            is_static: false,
            span: model(Span::new(start, end))?,
            signature: None,
            doc: None,
        })
    }

    fn prints(body: &Body, symbol: &Symbol) -> Result<SymbolFingerprints, CodedError> {
        let tokens: Vec<TokenId> = (body.seed..body.seed + 40)
            .map(|n| TokenId::node_kind(&format!("kind{n}")))
            .collect();
        let params = model(FingerprintParams::new(3, 2))?;
        Ok(SymbolFingerprints {
            symbol: symbol.id,
            fingerprints: winnow(&tokens, params),
        })
    }

    /// A body without code lines is an empty file: no function at all.
    fn symbols_of(
        body: &Body,
        request: &AnalyzeRequest<'_>,
        file: &SourceFile,
    ) -> Result<Vec<Symbol>, CodedError> {
        let mut symbols = Vec::new();
        if body.loc > 0 {
            symbols.push(function(request, file, (4, 40))?);
        }
        if body.inner {
            symbols.push(function(request, file, (10, 12))?);
        }
        Ok(symbols)
    }

    impl FileAnalyzer for Disk {
        fn analyze(&self, request: &AnalyzeRequest<'_>) -> Result<FileFacts, CodedError> {
            let path = request.file.path.as_str();
            if self.broken == Some(path) {
                return Err(CodedError::new(ErrorCode::ParseTimeout));
            }
            let (_, body) = self
                .files
                .iter()
                .find(|(name, _)| *name == path)
                .ok_or_else(|| CodedError::new(ErrorCode::UnexpectedInternalError))?;
            let file = SourceFile {
                loc: Loc(body.loc),
                path: request.file.path.clone(),
                id: request.ids.file(),
                content_hash: request.file.content_hash.clone(),
                snapshot: request.snapshot,
                language: request.file.language.clone(),
            };
            let symbols = symbols_of(body, request, &file)?;
            Ok(FileFacts {
                fingerprints: symbols
                    .iter()
                    .map(|symbol| prints(body, symbol))
                    .collect::<Result<_, _>>()?,
                counts: symbols
                    .iter()
                    .map(|symbol| SymbolCounts {
                        symbol: symbol.id,
                        loc: Loc(body.loc),
                        body: Some(BodyCounts {
                            cognitive: body.counted.map(|(cognitive, _)| cognitive),
                            parameters: body.counted.map(|(_, parameters)| parameters),
                            ..BodyCounts::default()
                        }),
                        counted: true,
                    })
                    .collect(),
                symbols,
                pending: Vec::new(),
                file,
            })
        }
    }

    fn limits() -> Conventions {
        Conventions {
            max_function_lines: 40,
            max_cognitive_complexity: 15,
            max_parameters: 5,
            duplicate_similarity_percent: 85,
            duplicate_min_lines: 5,
            duplicate_max_owners: 50,
            test_folders: vec!["spec".to_owned()],
        }
    }

    fn checker(disk: Disk) -> ConventionChecker {
        checker_with(disk, limits())
    }

    fn checker_with(disk: Disk, limits: Conventions) -> ConventionChecker {
        checker_noted(disk, limits, &Notes::default())
    }

    fn checker_noted(disk: Disk, limits: Conventions, notes: &Notes) -> ConventionChecker {
        let disk = Arc::new(disk);
        ConventionChecker::new(disk.clone(), disk, Arc::new(notes.clone()), limits)
    }

    /// The text above every declaration: a comment line `note`, optionally after a blank line.
    #[derive(Clone, Default)]
    struct Notes {
        note: Option<&'static str>,
        blank_between: bool,
    }

    impl SourceText for Notes {
        fn excerpt(
            &self,
            _: &FolderPath,
            _: &RelativePath,
            (first, last): (u32, u32),
            _: (u32, u64),
        ) -> Result<Excerpt, CodedError> {
            let lines = (first..=last)
                .map(|line| match (last - line, self.note) {
                    (1, Some(note)) if !self.blank_between => note.to_owned(),
                    (1, Some(_)) => String::new(),
                    (2, Some(note)) if self.blank_between => note.to_owned(),
                    _ => "let x = 1;".to_owned(),
                })
                .collect();
            Ok(Excerpt {
                lines,
                truncated: false,
            })
        }
    }

    const REASON: &str = "// codeskimr: allow CHK-102 maps the storage row into the model";

    /// The files `a.rs` and `b.rs` with the same body; its cognitive complexity is `cognitive`.
    fn twins(cognitive: u32) -> Disk {
        let body = || Body {
            loc: 9,
            seed: 0,
            inner: false,
            counted: Some((cognitive, 1)),
        };
        Disk {
            files: vec![("a.rs", body()), ("b.rs", body())],
            broken: None,
        }
    }

    fn copies_with(cognitive: u32, notes: &Notes) -> usize {
        let checker = checker_noted(twins(cognitive), limits(), notes);
        let report = run(&checker, Principal::local_owner(), &[]).expect("checked");
        report.findings.len()
    }

    #[test]
    fn a_pure_mapping_function_with_a_reasoned_marker_is_not_a_copy() {
        let notes = Notes {
            note: Some(REASON),
            blank_between: false,
        };
        assert_eq!(copies_with(0, &notes), 0);
    }

    #[test]
    fn without_a_marker_or_a_reason_the_copy_is_reported() {
        let bare = "// codeskimr: allow CHK-102";
        let notes = |note| Notes {
            note,
            blank_between: false,
        };
        assert_eq!(copies_with(0, &notes(None)), 1);
        assert_eq!(copies_with(0, &notes(Some(bare))), 1);
    }

    #[test]
    fn a_marker_cut_off_by_a_blank_line_does_not_count() {
        let notes = Notes {
            note: Some(REASON),
            blank_between: true,
        };
        assert_eq!(copies_with(0, &notes), 1);
    }

    #[test]
    fn a_function_with_branches_cannot_be_excused() {
        let notes = Notes {
            note: Some(REASON),
            blank_between: false,
        };
        assert_eq!(copies_with(1, &notes), 1);
    }

    fn root() -> FolderPath {
        FolderPath::new("/work").expect("valid folder")
    }

    fn run(
        checker: &ConventionChecker,
        principal: Principal,
        paths: &[&str],
    ) -> Result<codeskimr_application::conventions::CheckReport, CodedError> {
        let paths: Vec<String> = paths.iter().map(|path| (*path).to_owned()).collect();
        checker.check(&CheckRequest {
            principal: &principal,
            root: &root(),
            paths: &paths,
            stored: None,
        })
    }

    fn sample() -> Disk {
        Disk {
            files: vec![
                (
                    "src/a.rs",
                    Body {
                        loc: 12,
                        seed: 0,
                        inner: false,
                        counted: None,
                    },
                ),
                (
                    "src/b.rs",
                    Body {
                        loc: 90,
                        seed: 0,
                        inner: false,
                        counted: None,
                    },
                ),
                (
                    "tests/c.rs",
                    Body {
                        loc: 8,
                        seed: 500,
                        inner: false,
                        counted: None,
                    },
                ),
            ],
            broken: None,
        }
    }

    #[test]
    fn long_functions_and_copies_are_reported_in_path_order() {
        let report = run(&checker(sample()), Principal::local_owner(), &[]).expect("checked");
        assert_eq!(report.files_checked, 3);
        let codes: Vec<(&str, ErrorCode)> = report
            .findings
            .iter()
            .map(|finding| (finding.location.path.as_str(), finding.code))
            .collect();
        assert_eq!(
            codes,
            [
                ("src/b.rs", ErrorCode::DuplicateBlock),
                ("src/b.rs", ErrorCode::FunctionTooLong)
            ]
        );
        assert_eq!(
            report.findings[1].detail,
            Detail::TooLong {
                lines: 90,
                limit: 40
            }
        );
        let Detail::Duplicate {
            of,
            similarity_percent,
        } = &report.findings[0].detail
        else {
            panic!("a duplicate was expected");
        };
        assert_eq!(of.path.as_str(), "src/a.rs");
        assert_eq!(*similarity_percent, 100);
    }

    /// The codes of the findings for the sample files under `limits`.
    fn codes_with(limits: Conventions) -> Vec<ErrorCode> {
        let report = run(
            &checker_with(sample(), limits),
            Principal::local_owner(),
            &[],
        )
        .expect("checked");
        report.findings.iter().map(|f| f.code).collect()
    }

    #[test]
    fn small_functions_are_not_compared_for_duplicates() {
        let limits = Conventions {
            duplicate_min_lines: 100,
            ..limits()
        };
        assert_eq!(codes_with(limits), [ErrorCode::FunctionTooLong]);
    }

    #[test]
    fn a_fingerprint_shared_by_more_functions_than_the_limit_finds_no_copies() {
        let limits = Conventions {
            duplicate_max_owners: 1,
            ..limits()
        };
        assert_eq!(codes_with(limits), [ErrorCode::FunctionTooLong]);
    }

    fn counted(cognitive: u32, parameters: u32) -> Disk {
        Disk {
            files: vec![(
                "src/a.rs",
                Body {
                    loc: 12,
                    seed: 0,
                    inner: false,
                    counted: Some((cognitive, parameters)),
                },
            )],
            broken: None,
        }
    }

    #[test]
    fn a_function_above_the_cognitive_limit_or_the_parameter_limit_is_reported() {
        let report = run(&checker(counted(16, 6)), Principal::local_owner(), &[]).expect("checked");
        let codes: Vec<ErrorCode> = report.findings.iter().map(|finding| finding.code).collect();
        assert_eq!(
            codes,
            [ErrorCode::ComplexityTooHigh, ErrorCode::TooManyParameters]
        );
        assert_eq!(
            report.findings[0].detail,
            Detail::Over {
                value: 16,
                limit: 15
            }
        );
        assert_eq!(
            report.findings[1].detail,
            Detail::Over { value: 6, limit: 5 }
        );
        assert_eq!(
            report.not_checked,
            [ErrorCode::LiteralFound, ErrorCode::LayerViolation],
            "a language that counts both is checked for both"
        );
    }

    #[test]
    fn a_function_at_the_limits_is_clean() {
        let report = run(&checker(counted(15, 5)), Principal::local_owner(), &[]).expect("checked");
        assert_eq!(report.findings.len(), 0);
    }

    #[test]
    fn files_in_test_folders_are_not_checked() {
        let limits = Conventions {
            test_folders: vec!["src".to_owned()],
            ..limits()
        };
        let report = run(
            &checker_with(sample(), limits),
            Principal::local_owner(),
            &[],
        )
        .expect("checked");
        assert_eq!(report.files_checked, 1);
        assert_eq!(report.findings.len(), 0);
    }

    #[test]
    fn only_the_named_paths_are_checked() {
        let checker = checker(sample());
        let owner = Principal::local_owner();
        for (paths, files) in [
            (&["src"][..], 2),
            (&["./src/a.rs"][..], 1),
            (&["."][..], 3),
            (&["./"][..], 3),
            (&["src\\b.rs", "tests/"][..], 2),
            (&["src/a"][..], 0),
            (&["../etc/passwd"][..], 0),
        ] {
            let report = run(&checker, owner, paths).expect("checked");
            assert_eq!(report.files_checked, files, "{paths:?}");
        }
    }

    #[test]
    fn a_file_that_cannot_be_analysed_is_listed_not_fatal() {
        let disk = Disk {
            broken: Some("src/z.rs"),
            ..sample()
        };
        let report = run(&checker(disk), Principal::local_owner(), &[]).expect("checked");
        assert_eq!(report.files_checked, 3);
        assert_eq!(report.unchecked.len(), 1);
        assert_eq!(report.unchecked[0].code, ErrorCode::ParseTimeout);
    }

    #[test]
    fn the_report_names_the_rules_it_does_not_check() {
        let report = run(&checker(sample()), Principal::local_owner(), &[]).expect("checked");
        assert_eq!(
            report.not_checked,
            [
                ErrorCode::LiteralFound,
                ErrorCode::LayerViolation,
                ErrorCode::ComplexityTooHigh,
                ErrorCode::TooManyParameters
            ]
        );
    }

    #[test]
    fn reading_source_needs_the_permission() {
        let reader = Principal::limited(PermissionSet::none().with(Permission::Read));
        let error = run(&checker(sample()), reader, &[]).expect_err("denied");
        assert_eq!(error.code(), ErrorCode::PermissionDenied);
    }

    fn only(files: Vec<(&'static str, Body)>) -> ConventionChecker {
        checker(Disk {
            files,
            broken: None,
        })
    }

    #[test]
    fn every_copy_names_the_first_original_and_reports_once() {
        let copy = || Body {
            loc: 5,
            seed: 0,
            inner: false,
            counted: None,
        };
        let checker = only(vec![("a.rs", copy()), ("b.rs", copy()), ("c.rs", copy())]);
        let report = run(&checker, Principal::local_owner(), &[]).expect("checked");
        let found: Vec<(&str, &str)> = report
            .findings
            .iter()
            .filter_map(|finding| match &finding.detail {
                Detail::Duplicate { of, .. } => {
                    Some((finding.location.path.as_str(), of.path.as_str()))
                }
                Detail::TooLong { .. } | Detail::Over { .. } => None,
            })
            .collect();
        assert_eq!(found, [("b.rs", "a.rs"), ("c.rs", "a.rs")]);
    }

    /// A stored project that answers every query with the same hits.
    struct Stored(Vec<SimilarHit>);

    impl SimilarityIndex for Stored {
        fn fingerprints_of(
            &self,
            _: SnapshotId,
            _: SymbolId,
        ) -> Result<Option<codeskimr_domain::Fingerprints>, CodedError> {
            Ok(None)
        }

        fn similar(
            &self,
            _: SnapshotId,
            _: &SimilarQuery<'_>,
        ) -> Result<Vec<SimilarHit>, CodedError> {
            Ok(self.0.clone())
        }
    }

    fn hit(path: &str, percent: u32) -> SimilarHit {
        SimilarHit {
            symbol: SymbolId::new(9),
            qualified_name: QualifiedName::new(format!("{path}::run")).expect("name"),
            kind: SymbolKind::Function,
            path: RelativePath::new(path).expect("path"),
            span: Span::new(
                Position { line: 0, column: 0 },
                Position { line: 1, column: 0 },
            )
            .expect("span"),
            percent,
        }
    }

    fn duplicates_against(
        stored: &Stored,
        files: Vec<(&'static str, Body)>,
    ) -> Vec<(String, String)> {
        let checker = only(files);
        let request = CheckRequest {
            principal: &Principal::local_owner(),
            root: &root(),
            paths: &[],
            stored: Some(StoredProject {
                index: stored,
                snapshot: SnapshotId::new(1),
            }),
        };
        let report = checker.check(&request).expect("checked");
        report
            .findings
            .iter()
            .filter_map(|finding| match &finding.detail {
                Detail::Duplicate { of, .. } => Some((
                    finding.location.path.as_str().to_owned(),
                    of.path.as_str().to_owned(),
                )),
                Detail::TooLong { .. } | Detail::Over { .. } => None,
            })
            .collect()
    }

    fn short(seed: u32) -> Body {
        Body {
            loc: 5,
            seed,
            inner: false,
            counted: None,
        }
    }

    #[test]
    fn a_copy_of_stored_code_outside_the_checked_files_is_found() {
        let stored = Stored(vec![hit("old/x.rs", 91)]);
        let found = duplicates_against(&stored, vec![("a.rs", short(0))]);
        assert_eq!(found, [("a.rs".to_owned(), "old/x.rs".to_owned())]);
    }

    /// How many copies of stored code are reported for one mapping function under `notes`.
    fn stored_copies_with(notes: &Notes) -> usize {
        let disk = Disk {
            files: vec![(
                "a.rs",
                Body {
                    loc: 9,
                    seed: 0,
                    inner: false,
                    counted: Some((0, 1)),
                },
            )],
            broken: None,
        };
        let stored = Stored(vec![hit("old/x.rs", 91)]);
        let request = CheckRequest {
            principal: &Principal::local_owner(),
            root: &root(),
            paths: &[],
            stored: Some(StoredProject {
                index: &stored,
                snapshot: SnapshotId::new(1),
            }),
        };
        let report = checker_noted(disk, limits(), notes)
            .check(&request)
            .expect("checked");
        report.findings.len()
    }

    #[test]
    fn a_marked_mapping_function_is_no_copy_of_stored_code_either() {
        let marked = Notes {
            note: Some(REASON),
            blank_between: false,
        };
        assert_eq!(stored_copies_with(&Notes::default()), 1);
        assert_eq!(stored_copies_with(&marked), 0);
    }

    #[test]
    fn stored_versions_of_checked_files_and_tests_are_no_originals() {
        // The first hit is the older version of the file itself, the second one lies in a test
        // folder; the third one counts.
        let stored = Stored(vec![
            hit("a.rs", 100),
            hit("spec/t.rs", 99),
            hit("lib/y.rs", 90),
        ]);
        let found = duplicates_against(&stored, vec![("a.rs", short(0))]);
        assert_eq!(found, [("a.rs".to_owned(), "lib/y.rs".to_owned())]);
        let none = duplicates_against(&Stored(vec![hit("a.rs", 100)]), vec![("a.rs", short(0))]);
        assert_eq!(none, []);
    }

    #[test]
    fn the_stored_version_of_a_checked_file_without_functions_is_no_original() {
        // A function moved from empty-now.rs to a.rs: the stored copy lies in a checked file.
        let stored = Stored(vec![hit("gone.rs", 100)]);
        let found = duplicates_against(
            &stored,
            vec![
                ("a.rs", short(0)),
                (
                    "gone.rs",
                    Body {
                        loc: 0,
                        seed: 0,
                        inner: false,
                        counted: None,
                    },
                ),
            ],
        );
        assert_eq!(found, []);
    }

    #[test]
    fn a_copy_inside_the_checked_files_is_not_reported_twice() {
        let stored = Stored(vec![hit("old/x.rs", 91)]);
        let found = duplicates_against(&stored, vec![("a.rs", short(0)), ("b.rs", short(0))]);
        // b.rs copies a.rs (checked files first); a.rs copies the stored function.
        assert_eq!(
            found,
            [
                ("a.rs".to_owned(), "old/x.rs".to_owned()),
                ("b.rs".to_owned(), "a.rs".to_owned())
            ]
        );
    }

    #[test]
    fn a_function_and_one_inside_it_are_no_duplicates() {
        let nested = Body {
            loc: 5,
            seed: 0,
            inner: true,
            counted: None,
        };
        let report =
            run(&only(vec![("a.rs", nested)]), Principal::local_owner(), &[]).expect("checked");
        assert!(report.findings.is_empty(), "{:?}", report.findings);
    }

    #[test]
    fn a_path_that_selects_nothing_is_reported_and_a_name_prefix_is_no_match() {
        let checker = only(vec![
            (
                "src/a.rs",
                Body {
                    loc: 5,
                    seed: 0,
                    inner: false,
                    counted: None,
                },
            ),
            (
                "src/ab.rs",
                Body {
                    loc: 5,
                    seed: 9,
                    inner: false,
                    counted: None,
                },
            ),
        ]);
        let report =
            run(&checker, Principal::local_owner(), &["src/a", "src/ab.rs"]).expect("checked");
        assert_eq!(report.unmatched, ["src/a"]);
        assert_eq!(report.files_checked, 1);
    }
}
