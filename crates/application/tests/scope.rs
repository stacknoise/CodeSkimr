//! The scope a user asks for: what is accepted, what the overview counts and what the page shows.

#[cfg(test)]
mod tests {

    use codeskimr_application::ports::ListedFile;
    use codeskimr_application::scope::{OverviewLimits, checked_scope, overview, scope_view};
    use codeskimr_codes::{Coded, ErrorCode};
    use codeskimr_domain::{LanguageId, RelativePath, Scope};

    fn strings(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| (*item).to_owned()).collect()
    }

    fn language(name: &str) -> LanguageId {
        LanguageId::new(name).unwrap()
    }

    fn path(text: &str) -> RelativePath {
        RelativePath::new(text).unwrap()
    }

    fn listed(path_text: &str, language_name: Option<&str>) -> ListedFile {
        ListedFile {
            path: path(path_text),
            language: language_name.map(language),
        }
    }

    const LIMITS: OverviewLimits = OverviewLimits {
        depth: 2,
        max_folders: 50,
    };

    #[test]
    fn folders_are_tidied_into_the_form_the_project_uses() {
        let scope = checked_scope(
            &strings(&["sql", " markdown "]),
            &strings(&["./docs/", "web\\old", "docs/deep"]),
            20,
        )
        .unwrap();
        assert_eq!(scope.languages().len(), 2);
        let folders: Vec<&str> = scope.folders().iter().map(RelativePath::as_str).collect();
        assert_eq!(folders, ["docs", "web/old"]);
    }

    #[test]
    fn a_folder_outside_the_project_or_a_bad_language_is_refused() {
        for folder in [
            "", ".", "..", "../x", "a/../b", "/abs", "C:/x", "a//b", "a\u{7}b",
        ] {
            let refused = checked_scope(&[], &strings(&[folder]), 20).unwrap_err();
            assert_eq!(refused.code(), ErrorCode::ProjectScopeInvalid, "{folder:?}");
        }
        for name in ["", "two words", "a/b", "x".repeat(65).as_str()] {
            let refused = checked_scope(&strings(&[name]), &[], 20).unwrap_err();
            assert_eq!(refused.code(), ErrorCode::ProjectScopeInvalid, "{name:?}");
        }
    }

    #[test]
    fn more_entries_than_the_limit_are_refused() {
        let many = strings(&["a", "b", "c"]);
        assert!(checked_scope(&many, &[], 3).is_ok());
        let refused = checked_scope(&many, &strings(&["d"]), 3).unwrap_err();
        assert_eq!(refused.code(), ErrorCode::ProjectScopeInvalid);
    }

    #[test]
    fn the_overview_counts_files_per_language_and_folder_down_to_the_depth() {
        let files = [
            listed("Cargo.toml", None),
            listed("cli/Cargo.toml", None),
            listed("cli/src/main.rs", Some("rust")),
            listed("cli/src/deep/more.rs", Some("rust")),
            listed("web/app.ts", Some("typescript")),
            listed("web/package.json", None),
            listed("web/util.py", Some("python")),
        ];
        let wanted = strings(&["Cargo.toml", "package.json", "*.sln"]);
        let counted = overview(&files, &wanted, LIMITS);
        let languages: Vec<(&str, u64)> = counted
            .languages
            .iter()
            .map(|(language, files)| (language.as_str(), *files))
            .collect();
        assert_eq!(languages, [("rust", 2), ("python", 1), ("typescript", 1)]);
        assert_eq!(counted.root_project_files, ["Cargo.toml"]);
        let folders: Vec<(&str, u64, &[String])> = counted
            .folders
            .iter()
            .map(|row| (row.path.as_str(), row.files, row.project_files.as_slice()))
            .collect();
        assert_eq!(
            folders,
            [
                ("cli", 2, strings(&["Cargo.toml"]).as_slice()),
                ("cli/src", 2, [].as_slice()),
                ("web", 2, strings(&["package.json"]).as_slice()),
            ]
        );
        assert_eq!(counted.folders_more, 0);
    }

    #[test]
    fn a_project_file_deeper_than_the_depth_is_not_noted_for_a_higher_folder() {
        let files = [
            listed("a/b/c/Cargo.toml", None),
            listed("a/b/c/x.rs", Some("rust")),
        ];
        let counted = overview(&files, &strings(&["Cargo.toml"]), LIMITS);
        assert!(
            counted
                .folders
                .iter()
                .all(|row| row.project_files.is_empty())
        );
    }

    #[test]
    fn folders_beyond_the_limit_are_counted_not_listed() {
        let files = [
            listed("a/x.rs", Some("rust")),
            listed("b/x.rs", Some("rust")),
            listed("c/x.rs", Some("rust")),
        ];
        let limits = OverviewLimits {
            depth: 1,
            max_folders: 2,
        };
        let counted = overview(&files, &[], limits);
        assert_eq!(counted.folders.len(), 2);
        assert_eq!(counted.folders_more, 1);
    }

    #[test]
    fn the_page_lists_what_is_left_out_even_when_nothing_is_counted_for_it() {
        let files = [listed("web/app.ts", Some("typescript"))];
        let counted = overview(&files, &[], LIMITS);
        let scope = Scope::new([language("sql")], [path("docs"), path("web")]);
        let view = scope_view(&scope, counted);
        let languages: Vec<(&str, Option<u64>, bool)> = view
            .languages
            .iter()
            .map(|entry| (entry.language.as_str(), entry.files, entry.covered))
            .collect();
        assert_eq!(
            languages,
            [("typescript", Some(1), true), ("sql", None, false)]
        );
        let folders: Vec<(&str, Option<u64>, bool, bool)> = view
            .folders
            .iter()
            .map(|entry| {
                (
                    entry.path.as_str(),
                    entry.files,
                    entry.covered,
                    entry.left_out,
                )
            })
            .collect();
        assert_eq!(
            folders,
            [("docs", None, false, true), ("web", Some(1), false, true)]
        );
    }
}
