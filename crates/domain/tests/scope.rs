//! What a project leaves out of its scans (ADR 0101).

#[cfg(test)]
mod tests {

    use codeskimr_domain::{LanguageId, RelativePath, Scope};

    fn language(name: &str) -> LanguageId {
        LanguageId::new(name).unwrap()
    }

    fn path(text: &str) -> RelativePath {
        RelativePath::new(text).unwrap()
    }

    fn scope(languages: &[&str], folders: &[&str]) -> Scope {
        Scope::new(
            languages.iter().map(|name| language(name)),
            folders.iter().map(|folder| path(folder)),
        )
    }

    #[test]
    fn an_empty_scope_covers_everything() {
        let everything = Scope::default();
        assert!(everything.is_everything());
        assert!(everything.covers(&path("src/main.rs"), &language("rust")));
        assert!(everything.covers_folder(&path("vendor")));
    }

    #[test]
    fn a_language_that_is_left_out_covers_none_of_its_files() {
        let scope = scope(&["sql"], &[]);
        assert!(!scope.covers(&path("db/schema.sql"), &language("sql")));
        assert!(scope.covers(&path("db/run.py"), &language("python")));
        assert!(!scope.covers_language(&language("sql")));
    }

    #[test]
    fn a_folder_that_is_left_out_covers_everything_below_it() {
        let scope = scope(&[], &["docs", "web/old"]);
        assert!(!scope.covers(&path("docs/a.md"), &language("markdown")));
        assert!(!scope.covers(&path("docs/deep/er/a.md"), &language("markdown")));
        assert!(!scope.covers(&path("web/old/app.ts"), &language("typescript")));
        assert!(scope.covers(&path("web/new/app.ts"), &language("typescript")));
        assert!(!scope.covers_folder(&path("docs")));
        assert!(!scope.covers_folder(&path("docs/deep")));
        assert!(scope.covers_folder(&path("web")));
    }

    #[test]
    fn a_folder_matches_whole_names_only() {
        let scope = scope(&[], &["doc"]);
        assert!(scope.covers(&path("docs/a.md"), &language("markdown")));
        assert!(scope.covers(&path("doc.md"), &language("markdown")));
        assert!(scope.covers_folder(&path("documents")));
    }

    #[test]
    fn a_folder_below_another_left_out_folder_is_dropped() {
        let scope = scope(&[], &["web/old/deep", "web", "web/old", "docs"]);
        let kept: Vec<&str> = scope.folders().iter().map(RelativePath::as_str).collect();
        assert_eq!(kept, ["docs", "web"]);
    }

    #[test]
    fn two_scopes_with_the_same_entries_are_equal_whatever_the_order() {
        assert_eq!(
            scope(&["sql", "markdown"], &["b", "a"]),
            scope(&["markdown", "sql"], &["a", "b"])
        );
    }
}
