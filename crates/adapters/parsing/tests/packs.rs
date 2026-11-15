//! Tests of loading language packs.

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use codeskimr_codes::{Coded, ErrorCode};
    use codeskimr_domain::LanguageId;
    use codeskimr_parsing::{LanguagePack, LanguagePacks, Tier};

    fn fixture_pack() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/languages/rust")
    }

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join(format!("codeskimr-packs-{name}-{}", std::process::id()))
            .join("rust");
        fs::create_dir_all(&dir).expect("temp dir");
        for file in ["language.toml", "symbols.scm"] {
            fs::copy(fixture_pack().join(file), dir.join(file)).expect("copy");
        }
        dir
    }

    fn remove(dir: &Path) {
        fs::remove_dir_all(dir.parent().expect("parent")).ok();
    }

    fn assert_invalid(dir: &Path, name: &str) {
        let error = LanguagePack::load(dir).err().expect("invalid");
        assert_eq!(error.code(), ErrorCode::LanguagePackInvalid, "{name}");
        remove(dir);
    }

    #[test]
    fn loads_the_fixture_pack() {
        let pack = LanguagePack::load(&fixture_pack()).expect("loads");
        assert_eq!(pack.id().as_str(), "rust");
        assert_eq!(pack.tier(), Tier::Full);
        assert_eq!(pack.extensions(), ["rs"]);
    }

    #[test]
    fn rejects_unknown_kind_and_visibility_labels() {
        for (name, extra) in [
            ("kind", "\n[modifier_kinds]\nconst = \"nonsense\"\n"),
            (
                "parent",
                "\n[visibility.parent_defaults]\nnonsense = \"public\"\n",
            ),
        ] {
            let dir = scratch(name);
            let manifest = fs::read_to_string(dir.join("language.toml")).expect("read");
            fs::write(dir.join("language.toml"), format!("{manifest}{extra}")).expect("write");
            assert_invalid(&dir, name);
        }
    }

    fn from_rust(file: &str) -> String {
        format!("[queries]\nfrom = \"rust\"\nfiles = [\"{file}\"]\n")
    }

    /// A pack `derived` beside the copy of the fixture pack `rust`, with `queries` in its manifest.
    fn derived(name: &str, queries: &str, own_symbols: bool) -> PathBuf {
        let rust = scratch(name);
        let dir = rust.with_file_name("derived");
        fs::create_dir_all(&dir).expect("dir");
        let manifest = fs::read_to_string(rust.join("language.toml")).expect("read");
        let manifest = manifest.replace("id = \"rust\"", "id = \"derived\"");
        fs::write(dir.join("language.toml"), format!("{manifest}\n{queries}")).expect("write");
        if own_symbols {
            fs::copy(rust.join("symbols.scm"), dir.join("symbols.scm")).expect("copy");
        }
        dir
    }

    #[test]
    fn a_pack_takes_query_files_over_from_another_pack() {
        let queries = from_rust("symbols.scm");
        let dir = derived("take-over", &queries, false);
        let pack = LanguagePack::load(&dir).expect("loads without an own symbols.scm");
        assert_eq!(pack.id().as_str(), "derived");
        remove(&dir);
    }

    #[test]
    fn the_own_file_adds_to_the_taken_over_one() {
        let queries = from_rust("symbols.scm");
        let dir = derived("add-to", &queries, true);
        LanguagePack::load(&dir).expect("both texts compile together");
        remove(&dir);
    }

    #[test]
    fn a_pack_without_its_own_symbols_file_and_without_queries_is_invalid() {
        let dir = derived("no-symbols", "", false);
        assert_invalid(&dir, "no symbols");
    }

    #[test]
    fn queries_that_break_the_rules_are_invalid() {
        for (name, queries) in [
            (
                "self",
                "[queries]\nfrom = \"derived\"\nfiles = [\"symbols.scm\"]\n",
            ),
            (
                "path",
                "[queries]\nfrom = \"../rust\"\nfiles = [\"symbols.scm\"]\n",
            ),
            (
                "missing",
                "[queries]\nfrom = \"nothing\"\nfiles = [\"symbols.scm\"]\n",
            ),
            (
                "file",
                "[queries]\nfrom = \"rust\"\nfiles = [\"highlights.scm\"]\n",
            ),
        ] {
            assert_invalid(&derived(name, queries, true), name);
        }
    }

    #[test]
    fn a_pack_cannot_take_files_from_a_pack_that_takes_files() {
        let queries = from_rust("symbols.scm");
        let dir = derived("chain", &queries, true);
        let rust = dir.with_file_name("rust");
        let manifest = fs::read_to_string(rust.join("language.toml")).expect("read");
        fs::write(
            rust.join("language.toml"),
            format!("{manifest}\n[queries]\nfrom = \"derived\"\nfiles = [\"metrics.scm\"]\n"),
        )
        .expect("write");
        assert_invalid(&dir, "chain");
    }

    #[test]
    fn rejects_unknown_manifest_keys() {
        let dir = scratch("keys");
        let manifest = fs::read_to_string(dir.join("language.toml")).expect("read");
        fs::write(
            dir.join("language.toml"),
            format!("{manifest}\nsurprise = 1\n"),
        )
        .expect("write");
        let error = LanguagePack::load(&dir).err().expect("invalid");
        assert_eq!(error.code(), ErrorCode::LanguagePackInvalid);
        remove(&dir);
    }

    #[test]
    fn rejects_queries_that_do_not_compile_or_use_unknown_captures() {
        for (name, query) in [
            ("syntax", "(unclosed"),
            ("capture", "(struct_item) @symbol.gizmo"),
        ] {
            let dir = scratch(name);
            fs::write(dir.join("symbols.scm"), query).expect("write");
            assert_invalid(&dir, name);
        }
    }

    #[test]
    fn rejects_manifests_with_an_unknown_grammar_or_a_foreign_id() {
        for (name, from, to) in [
            ("grammar", "grammar = \"rust\"", "grammar = \"klingon\""),
            ("id", "id = \"rust\"", "id = \"other\""),
        ] {
            let dir = scratch(name);
            let manifest = fs::read_to_string(dir.join("language.toml")).expect("read");
            fs::write(dir.join("language.toml"), manifest.replace(from, to)).expect("write");
            assert_invalid(&dir, name);
        }
    }

    #[test]
    fn finds_packs_by_language_and_reports_missing_ones() {
        let packs = LanguagePacks::load(fixture_pack().parent().expect("parent")).expect("loads");
        assert!(packs.get(&LanguageId::new("rust").expect("id")).is_ok());
        let missing = packs
            .get(&LanguageId::new("cobol").expect("id"))
            .err()
            .expect("missing");
        assert_eq!(missing.code(), ErrorCode::LanguagePackMissing);
    }

    #[test]
    fn requires_symbols_but_not_references() {
        let dir = scratch("files");
        assert!(
            LanguagePack::load(&dir).is_ok(),
            "references.scm is optional"
        );
        fs::remove_file(dir.join("symbols.scm")).expect("remove");
        assert_eq!(
            LanguagePack::load(&dir).err().expect("invalid").code(),
            ErrorCode::LanguagePackInvalid
        );
        remove(&dir);
    }

    #[test]
    fn accepts_manifest_files_and_test_patterns() {
        let dir = scratch("extra");
        let manifest = fs::read_to_string(dir.join("language.toml")).expect("read");
        let extra = "manifest_files = [\"Cargo.toml\"]\ntest_patterns = [\"tests/**\"]\n";
        fs::write(dir.join("language.toml"), format!("{extra}{manifest}")).expect("write");
        assert!(LanguagePack::load(&dir).is_ok());
        remove(&dir);
    }
}
