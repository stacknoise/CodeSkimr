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

    fn function(request: &AnalyzeRequest<'_>, file: &SourceFile) -> Result<Symbol, CodedError> {
        let at = Position { line: 4, column: 0 };
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
            span: model(Span::new(at, at))?,
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
            let symbol = function(request, &file)?;
            Ok(FileFacts {
                fingerprints: vec![prints(body, &symbol)?],
                counts: vec![SymbolCounts {
                    symbol: symbol.id,
                    loc: Loc(body.loc),
                    body: Some(BodyCounts::default()),
                    counted: true,
                }],
                symbols: vec![symbol],
                pending: Vec::new(),
                file,
            })
        }
    }

    fn checker(disk: Disk) -> ConventionChecker {
        let disk = Arc::new(disk);
        ConventionChecker::new(
            disk.clone(),
            disk,
            Conventions {
                max_function_lines: 40,
                duplicate_similarity_percent: 85,
            },
        )
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
                ("src/a.rs", Body { loc: 12, seed: 0 }),
                ("src/b.rs", Body { loc: 90, seed: 0 }),
                ("tests/c.rs", Body { loc: 8, seed: 500 }),
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
}
