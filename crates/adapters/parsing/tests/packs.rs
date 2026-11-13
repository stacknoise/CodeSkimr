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
        let dir =
            std::env::temp_dir().join(format!("codeskimr-packs-{name}-{}", std::process::id()));
        fs::create_dir_all(&dir).expect("temp dir");
        for file in ["language.toml", "symbols.scm"] {
            fs::copy(fixture_pack().join(file), dir.join(file)).expect("copy");
        }
        dir
    }

    #[test]
    fn loads_the_fixture_pack() {
        let pack = LanguagePack::load(&fixture_pack()).expect("loads");
        assert_eq!(pack.id().as_str(), "rust");
        assert_eq!(pack.tier(), Tier::Full);
        assert_eq!(pack.extensions(), ["rs"]);
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
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn rejects_queries_that_do_not_compile_or_use_unknown_captures() {
        for (name, query) in [
            ("syntax", "(unclosed"),
            ("capture", "(struct_item) @symbol.gizmo"),
        ] {
            let dir = scratch(name);
            fs::write(dir.join("symbols.scm"), query).expect("write");
            let error = LanguagePack::load(&dir).err().expect("invalid");
            assert_eq!(error.code(), ErrorCode::LanguagePackInvalid, "{name}");
            fs::remove_dir_all(dir).ok();
        }
    }

    #[test]
    fn rejects_an_unknown_grammar() {
        let dir = scratch("grammar");
        let manifest = fs::read_to_string(dir.join("language.toml")).expect("read");
        fs::write(
            dir.join("language.toml"),
            manifest.replace("grammar = \"rust\"", "grammar = \"klingon\""),
        )
        .expect("write");
        assert_eq!(
            LanguagePack::load(&dir).err().expect("invalid").code(),
            ErrorCode::LanguagePackInvalid
        );
        fs::remove_dir_all(dir).ok();
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
}
