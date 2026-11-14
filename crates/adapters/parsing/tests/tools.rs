//! Tests of the tool specifications in the language packs.
#![cfg(test)]

use std::fs;
use std::path::{Path, PathBuf};

use codeskimr_application::ports::ToolCatalog;
use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_domain::{LanguageId, RequirementFormat, ToolRole};
use codeskimr_parsing::PackTools;

fn shipped() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../languages")
}

fn language(name: &str) -> LanguageId {
    LanguageId::new(name).unwrap()
}

/// A folder with one pack whose manifest is `manifest`.
fn pack_with(name: &str, manifest: &str) -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    let dir = root.path().join(name);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("language.toml"), manifest).unwrap();
    root
}

const BASE: &str = "grammar = \"rust\"\ntier = \"full\"\nextensions = [\"rs\"]\nseparator = \".\"\ncomment_nodes = []\n";

#[test]
fn csharp_needs_an_indexer_and_the_sdk_the_project_names() {
    let tools = PackTools::load(&shipped()).unwrap();
    let specs = tools.tools_of(&language("csharp"));
    let ids: Vec<_> = specs
        .iter()
        .map(|spec| (spec.id.as_str(), spec.role))
        .collect();
    assert_eq!(
        ids,
        [
            ("scip-dotnet", ToolRole::Indexer),
            ("dotnet", ToolRole::Toolchain)
        ]
    );
    let requirement = specs[1].requirement.as_ref().unwrap();
    assert_eq!(requirement.file, "global.json");
    assert_eq!(requirement.format, RequirementFormat::Json);
    assert_eq!(requirement.path, ["sdk", "version"]);
}

#[test]
fn the_languages_of_the_adr_have_tools_and_the_basic_tier_has_none() {
    let tools = PackTools::load(&shipped()).unwrap();
    for name in [
        "csharp",
        "typescript",
        "javascript",
        "tsx",
        "python",
        "rust",
        "java",
        "kotlin",
    ] {
        assert_eq!(tools.tools_of(&language(name)).len(), 2, "{name}");
    }
    for name in ["go", "c", "cpp", "ruby", "php", "unknown"] {
        assert!(tools.tools_of(&language(name)).is_empty(), "{name}");
    }
}

#[test]
fn a_tool_with_a_wrong_role_or_format_makes_the_pack_invalid() {
    let tool = "\n[[tools]]\nid = \"x\"\ncommand = \"x\"\nversion_args = []\npurpose_key = \"p\"\ninstall_key = \"i\"\n";
    for (name, extra) in [
        ("role", format!("{tool}role = \"compiler\"\n")),
        (
            "format",
            format!(
                "{tool}role = \"indexer\"\n[tools.requirement]\nfile = \"a\"\nformat = \"xml\"\n"
            ),
        ),
        (
            "empty",
            format!("{tool}role = \"indexer\"\n").replace("id = \"x\"", "id = \"\""),
        ),
        (
            "typo",
            format!("{tool}role = \"indexer\"\nmin_versoin = \"1\"\n"),
        ),
    ] {
        let root = pack_with("rust", &format!("id = \"rust\"\n{BASE}{extra}"));
        let error = PackTools::load(root.path()).expect_err(name);
        assert_eq!(error.code(), ErrorCode::LanguagePackInvalid, "{name}");
    }
}
