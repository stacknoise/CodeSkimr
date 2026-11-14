//! The `[indexer]` table of the language packs (ADR 0033): what level 2 runs per language.

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use codeskimr_application::ports::PLACEHOLDER_OUTPUT;
    use codeskimr_codes::{Coded, ErrorCode};
    use codeskimr_parsing::LanguagePacks;

    fn languages() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../languages")
    }

    #[test]
    fn the_shipped_packs_name_an_indexer_for_the_languages_that_have_one() {
        let packs = LanguagePacks::load(&languages()).expect("packs load");
        let with: Vec<String> = {
            let mut ids: Vec<String> = packs
                .iter()
                .filter(|pack| pack.indexer().is_some())
                .map(|pack| pack.id().to_string())
                .collect();
            ids.sort();
            ids
        };
        assert_eq!(
            with,
            [
                "csharp",
                "java",
                "javascript",
                "kotlin",
                "python",
                "rust",
                "tsx",
                "typescript"
            ]
        );
    }

    #[test]
    fn every_indexer_has_what_the_runner_needs() {
        let packs = LanguagePacks::load(&languages()).expect("packs load");
        for pack in packs.iter() {
            let Some(spec) = pack.indexer() else { continue };
            let id = pack.id();
            assert!(!spec.name.is_empty(), "{id}");
            assert!(!spec.install_hint.is_empty(), "{id}");
            assert!(!spec.indexer.version_args.is_empty(), "{id}");
            assert!(
                spec.args.iter().any(|a| a.contains(PLACEHOLDER_OUTPUT)),
                "{id}"
            );
            assert!(
                !spec.toolchain.is_empty(),
                "{id}: the toolchain is checked too"
            );
            let all_args = spec.args.join(" ");
            assert!(!all_args.contains("npx"), "{id}: never npx");
        }
    }

    fn pack_with(extra: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "codeskimr-indexer-packs-{}-{}",
            std::process::id(),
            extra.len()
        ));
        let dir = root.join("rust");
        fs::create_dir_all(&dir).expect("temp dir");
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/languages/rust");
        for file in ["language.toml", "symbols.scm"] {
            fs::copy(source.join(file), dir.join(file)).expect("copy");
        }
        let manifest = fs::read_to_string(dir.join("language.toml")).expect("read");
        fs::write(dir.join("language.toml"), format!("{manifest}\n{extra}")).expect("write");
        root
    }

    const TOOL: &str =
        "[indexer.tool]\nprogram = \"x\"\nversion_args = [\"-v\"]\nmin_version = \"1.0.0\"\n";

    #[test]
    fn a_pack_with_a_bad_indexer_is_invalid() {
        for head in ["args = [\"index\"]", "args = [\"{output}\", \"{nothing}\"]"] {
            let text = format!("[indexer]\nname = \"x\"\ninstall_hint = \"y\"\n{head}\n{TOOL}");
            let root = pack_with(&text);
            let error = LanguagePacks::load(&root).err().expect("invalid");
            assert_eq!(error.code(), ErrorCode::LanguagePackInvalid, "{head}");
            fs::remove_dir_all(root).ok();
        }
        for tool in [
            "program = \"/usr/bin/x\"\nversion_args = [\"-v\"]\nmin_version = \"1.0.0\"",
            "program = \"x\"\nversion_args = [\"-v\"]\nmin_version = \"latest\"",
        ] {
            let text = format!(
                "[indexer]\nname = \"x\"\ninstall_hint = \"y\"\nargs = [\"{{output}}\"]\n[indexer.tool]\n{tool}\n"
            );
            let root = pack_with(&text);
            let error = LanguagePacks::load(&root).err().expect("invalid");
            assert_eq!(error.code(), ErrorCode::LanguagePackInvalid, "{tool}");
            fs::remove_dir_all(root).ok();
        }
    }
}
