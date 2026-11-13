//! Tests of the basic tier: packs are listed with their tier without loading, and a basic pack is
//! loaded when its language is first needed (blueprint 8.2, ADR 0020).

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use codeskimr_codes::{Coded, ErrorCode};
    use codeskimr_domain::LanguageId;
    use codeskimr_parsing::{LanguagePack, LanguagePacks, PackSummary, Tier};

    const BASIC_LANGUAGES: [&str; 5] = ["go", "c", "cpp", "ruby", "php"];

    fn languages() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../languages")
    }

    fn fixture_pack() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/languages/rust")
    }

    /// A folder with one pack `rust` of the given tier whose `symbols.scm` is not a valid query.
    fn broken_pack(tier: &str) -> PathBuf {
        let root =
            std::env::temp_dir().join(format!("codeskimr-basic-{tier}-{}", std::process::id()));
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
        for name in BASIC_LANGUAGES {
            assert_eq!(tier_of(name), Some(Tier::Basic), "{name}");
        }
    }

    #[test]
    fn every_basic_pack_loads_and_claims_extensions() {
        for name in BASIC_LANGUAGES {
            let pack = LanguagePack::load(&languages().join(name)).expect(name);
            assert_eq!(pack.tier(), Tier::Basic, "{name}");
            assert!(!pack.extensions().is_empty(), "{name}");
        }
    }

    #[test]
    fn a_basic_pack_is_checked_when_its_language_is_first_needed() {
        let root = broken_pack("basic");
        let packs = LanguagePacks::load(&root).expect("listing does not load the pack");
        assert_eq!(packs.iter().count(), 1);
        let error = packs.get(&id("rust")).err().expect("invalid on first use");
        assert_eq!(error.code(), ErrorCode::LanguagePackInvalid);
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn a_full_pack_is_checked_when_the_packs_are_loaded() {
        let root = broken_pack("full");
        let error = LanguagePacks::load(&root).err().expect("invalid at once");
        assert_eq!(error.code(), ErrorCode::LanguagePackInvalid);
        fs::remove_dir_all(root).ok();
    }
}
