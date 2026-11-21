//! Tests of the basic tier: packs are listed with their tier without loading, and a basic pack is
//! loaded when its language is first needed (blueprint 8.2, ADR 0020).

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use codeskimr_codes::{Coded, ErrorCode};
    use codeskimr_domain::LanguageId;
    use codeskimr_parsing::{LanguagePack, LanguagePacks, PackSummary, Tier};

    fn languages() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../languages")
    }

    fn fixture_pack() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/languages/rust")
    }

    /// A folder with one pack `rust` of the given tier whose `symbols.scm` is not a valid query.
    fn broken_pack(tier: &str, case: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "codeskimr-basic-{tier}-{case}-{}",
            std::process::id()
        ));
        let dir = root.join("rust");
        fs::create_dir_all(&dir).expect("temp dir");
        let manifest = fs::read_to_string(fixture_pack().join("language.toml")).expect("read");
        fs::write(
            dir.join("language.toml"),
            manifest.replace("tier = \"full\"", &format!("tier = \"{tier}\"")),
        )
        .expect("write manifest");
        fs::write(dir.join("symbols.scm"), "(nonsense_node) @name").expect("write query");
        root
    }

    fn id(name: &str) -> LanguageId {
        LanguageId::new(name).expect("id")
    }

    /// Folder names of all packs in `languages/` whose manifest says `tier = "basic"`.
    fn basic_languages() -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(languages())
            .expect("languages folder")
            .filter_map(|entry| {
                let dir = entry.expect("entry").path();
                let manifest = fs::read_to_string(dir.join("language.toml")).ok()?;
                let basic = manifest
                    .lines()
                    .any(|line| line.trim() == "tier = \"basic\"");
                basic.then(|| {
                    dir.file_name()
                        .expect("name")
                        .to_string_lossy()
                        .into_owned()
                })
            })
            .collect();
        names.sort();
        names
    }

    #[test]
    fn lists_every_pack_with_its_tier() {
        let packs = LanguagePacks::load(&languages()).expect("packs load");
        let tier_of = |name: &str| {
            packs
                .iter()
                .find(|pack| pack.id().as_str() == name)
                .map(PackSummary::tier)
        };
        assert_eq!(tier_of("csharp"), Some(Tier::Full));
        let basic = basic_languages();
        assert!(basic.len() >= 5, "{basic:?}");
        for name in &basic {
            assert_eq!(tier_of(name), Some(Tier::Basic), "{name}");
        }
    }

    #[test]
    fn every_basic_pack_loads_and_claims_extensions() {
        for name in basic_languages() {
            let pack = LanguagePack::load(&languages().join(&name)).expect(&name);
            assert_eq!(pack.tier(), Tier::Basic, "{name}");
            assert!(!pack.extensions().is_empty(), "{name}");
        }
    }

    #[test]
    fn a_basic_pack_is_checked_when_its_language_is_first_needed() {
        let root = broken_pack("basic", "first");
        let packs = LanguagePacks::load(&root).expect("listing does not load the pack");
        assert_eq!(packs.iter().count(), 1);
        let error = packs.get(&id("rust")).err().expect("invalid on first use");
        assert_eq!(error.code(), ErrorCode::LanguagePackInvalid);
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn a_pack_that_failed_to_load_is_not_built_again() {
        let root = broken_pack("basic", "again");
        let packs = LanguagePacks::load(&root).expect("listing does not load the pack");
        assert!(packs.get(&id("rust")).is_err());
        // Repairing the files does not matter: the first failure is remembered for the run.
        let fixed = fixture_pack().join("symbols.scm");
        fs::copy(fixed, root.join("rust/symbols.scm")).expect("repair");
        let error = packs.get(&id("rust")).err().expect("still failed");
        assert_eq!(error.code(), ErrorCode::LanguagePackInvalid);
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn the_naming_rules_carry_the_revision_of_the_engine() {
        let packs = LanguagePacks::load(&languages()).expect("packs load");
        let rules = packs.naming_rules();
        assert!(rules.contains("revision=5"), "{rules}");
        assert!(rules.contains("rust=file"), "{rules}");
        // The test patterns are part of the rules: other patterns, other kinds of references.
        assert!(
            rules.contains("rust=file+tests:**/tests/**/*.rs"),
            "{rules}"
        );
    }

    #[test]
    fn other_test_patterns_give_other_naming_rules() {
        let rules = |case: &str, patterns: &str| {
            let root = broken_pack("basic", case);
            let manifest_path = root.join("rust/language.toml");
            let manifest = fs::read_to_string(&manifest_path).expect("read");
            fs::write(
                &manifest_path,
                format!("test_patterns = [{patterns}]\n{manifest}"),
            )
            .expect("write");
            let text = LanguagePacks::load(&root)
                .expect("packs load")
                .naming_rules();
            fs::remove_dir_all(root).ok();
            text
        };
        let one = rules("patterns-one", "\"**/tests/**/*.rs\"");
        let two = rules("patterns-two", "\"**/tests/**/*.rs\", \"**/*_test.rs\"");
        assert_ne!(one, two);
        assert!(
            two.contains("+tests:**/tests/**/*.rs,**/*_test.rs"),
            "{two}"
        );
    }

    #[test]
    fn a_full_pack_is_loaded_when_its_language_is_first_needed_and_verify_loads_all() {
        let root = broken_pack("full", "load");
        let packs = LanguagePacks::load(&root).expect("listing does not load the pack");
        let error = packs.verify().expect_err("verify finds the broken pack");
        assert_eq!(error.code(), ErrorCode::LanguagePackInvalid);
        let error = packs.get(&id("rust")).err().expect("invalid when needed");
        assert_eq!(error.code(), ErrorCode::LanguagePackInvalid);
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn loading_a_pack_takes_time_that_is_counted_once() {
        let packs = LanguagePacks::load(&languages()).expect("packs list");
        assert_eq!(packs.load_time(), std::time::Duration::ZERO);
        packs.get(&id("rust")).expect("rust loads");
        let first = packs.load_time();
        assert!(first > std::time::Duration::ZERO);
        packs.get(&id("rust")).expect("rust is loaded already");
        assert_eq!(packs.load_time(), first);
    }

    #[test]
    fn every_installed_pack_loads() {
        let packs = LanguagePacks::load(&languages()).expect("packs list");
        packs.verify().expect("every pack compiles");
    }

    /// The queries of a pack compile on threads of their own; the error must still be the one of
    /// the first broken file in the order symbols, references, metrics, whichever thread ends first.
    #[test]
    fn the_first_broken_query_in_file_order_is_reported() {
        let root = broken_pack("full", "order");
        let dir = root.join("rust");
        fs::copy(fixture_pack().join("symbols.scm"), dir.join("symbols.scm")).expect("copy");
        fs::write(dir.join("metrics.scm"), "(broken_metrics) @x").expect("write");
        for _ in 0..20 {
            let error = LanguagePack::load(&dir).err().expect("metrics are broken");
            assert!(format!("{error:?}").contains("broken_metrics"), "{error:?}");
        }
        fs::write(dir.join("references.scm"), "(broken_references) @x").expect("write");
        for _ in 0..20 {
            let error = LanguagePack::load(&dir).err().expect("both are broken");
            assert_eq!(error.code(), ErrorCode::LanguagePackInvalid);
            assert!(
                format!("{error:?}").contains("broken_references"),
                "{error:?}"
            );
        }
        fs::write(dir.join("symbols.scm"), "(broken_symbols) @x").expect("write");
        let error = LanguagePack::load(&dir).err().expect("all are broken");
        assert!(format!("{error:?}").contains("broken_symbols"), "{error:?}");
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn the_tiers_are_read_from_the_manifests_alone() {
        let tiers = codeskimr_parsing::tiers(&languages()).expect("tiers");
        let tier_of = |id: &str| tiers.iter().find(|(found, _)| found == id).map(|(_, t)| *t);
        assert_eq!(tier_of("csharp"), Some(Tier::Full));
        assert_eq!(tier_of("go"), Some(Tier::Basic));
        assert_eq!(Tier::Full.label(), "full");
        assert_eq!(Tier::Basic.label(), "basic");
    }

    #[test]
    fn a_missing_folder_is_an_error_and_a_broken_manifest_only_loses_its_own_pack() {
        let gone = std::env::temp_dir().join(format!("codeskimr-gone-{}", std::process::id()));
        assert!(codeskimr_parsing::tiers(&gone).is_err());
        let root = std::env::temp_dir().join(format!("codeskimr-tiers-{}", std::process::id()));
        fs::create_dir_all(root.join("rust")).expect("good dir");
        fs::create_dir_all(root.join("bad")).expect("bad dir");
        fs::copy(
            fixture_pack().join("language.toml"),
            root.join("rust/language.toml"),
        )
        .expect("copy");
        fs::write(root.join("bad/language.toml"), "this is not a manifest").expect("write");
        let tiers = codeskimr_parsing::tiers(&root).expect("tiers");
        fs::remove_dir_all(&root).ok();
        assert_eq!(tiers, vec![("rust".to_string(), Tier::Full)]);
    }
}
