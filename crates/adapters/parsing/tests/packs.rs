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
