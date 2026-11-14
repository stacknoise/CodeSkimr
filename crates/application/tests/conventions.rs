//! Tests of the convention check with fake ports.

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use codeskimr_application::cancel::CancelToken;
    use codeskimr_application::config::Conventions;
    use codeskimr_application::conventions::{CheckRequest, ConventionChecker, Detail};
    use codeskimr_application::error::CodedError;
    use codeskimr_application::ports::{
        AnalyzeRequest, DiscoveredFile, FileAnalyzer, FileDiscovery, FileFacts, SymbolFingerprints,
    };
    use codeskimr_codes::{Coded, ErrorCode};
    use codeskimr_domain::{
        BodyCounts, ContentHash, FingerprintParams, FolderPath, LanguageId, Loc, Permission,
        PermissionSet, Position, Principal, QualifiedName, RelativePath, SourceFile, Span, Symbol,
        SymbolCounts, SymbolKind, SymbolName, TokenId, winnow,
    };

    /// What the fake analyser finds in a file: code lines and the seed of its token stream.
    struct Body {
        loc: u32,
        seed: u32,
        /// A second function with the same body inside the first one.
        inner: bool,
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
            let mut symbols = vec![function(request, &file, (4, 40))?];
            if body.inner {
                symbols.push(function(request, &file, (10, 12))?);
            }
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
                        body: Some(BodyCounts::default()),
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
        let disk = Arc::new(disk);
        ConventionChecker::new(disk.clone(), disk, limits)
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
                    },
                ),
                (
                    "src/b.rs",
                    Body {
                        loc: 90,
                        seed: 0,
                        inner: false,
                    },
                ),
                (
                    "tests/c.rs",
                    Body {
                        loc: 8,
                        seed: 500,
                        inner: false,
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

    #[test]
    fn small_functions_are_not_compared_for_duplicates() {
        let limits = Conventions {
            duplicate_min_lines: 100,
            duplicate_max_owners: 50,
            ..limits()
        };
        let report = run(
            &checker_with(sample(), limits),
            Principal::local_owner(),
            &[],
        )
        .expect("checked");
        let codes: Vec<ErrorCode> = report.findings.iter().map(|f| f.code).collect();
        assert_eq!(codes, [ErrorCode::FunctionTooLong]);
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
                ErrorCode::ComplexityTooHigh
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
                Detail::TooLong { .. } => None,
            })
            .collect();
        assert_eq!(found, [("b.rs", "a.rs"), ("c.rs", "a.rs")]);
    }

    #[test]
    fn a_function_and_one_inside_it_are_no_duplicates() {
        let nested = Body {
            loc: 5,
            seed: 0,
            inner: true,
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
                },
            ),
            (
                "src/ab.rs",
                Body {
                    loc: 5,
                    seed: 9,
                    inner: false,
                },
            ),
        ]);
        let report =
            run(&checker, Principal::local_owner(), &["src/a", "src/ab.rs"]).expect("checked");
        assert_eq!(report.unmatched, ["src/a"]);
        assert_eq!(report.files_checked, 1);
    }
}
