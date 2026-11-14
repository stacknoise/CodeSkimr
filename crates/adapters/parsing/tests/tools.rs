//! Tests of the tool specifications in the language packs.
#![cfg(test)]

use std::fs;
use std::path::{Path, PathBuf};

use codeskimr_application::ports::ToolCatalog;
use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_domain::{LanguageId, RequirementFormat};
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

const INDEXER: &str = "\n[indexer]\nname = \"x\"\ninstall_hint = \"y\"\nargs = [\"{output}\"]\n[indexer.tool]\nprogram = \"x\"\nversion_args = []\nmin_version = \"1.0.0\"\n";

#[test]
fn csharp_needs_an_indexer_and_the_sdk_the_project_names() {
    let tools = PackTools::load(&shipped()).unwrap();
    let plan = tools.plan_of(&language("csharp")).unwrap();
    assert_eq!(plan.indexer.indexer.program, "scip-dotnet");
    let toolchain: Vec<_> = plan
        .indexer
        .toolchain
        .iter()
        .map(|t| t.program.as_str())
        .collect();
    assert_eq!(toolchain, ["dotnet"]);
    assert_eq!(plan.requirements.len(), 1);
    let requirement = &plan.requirements[0];
    assert_eq!(requirement.program, "dotnet");
    assert_eq!(requirement.file.file, "global.json");
    assert_eq!(requirement.file.format, RequirementFormat::Json);
    assert_eq!(requirement.file.path, ["sdk", "version"]);
}

#[test]
fn the_languages_of_the_adr_have_a_plan_and_the_basic_tier_has_none() {
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
        let plan = tools.plan_of(&language(name)).expect(name);
        assert_eq!(plan.indexer.toolchain.len(), 1, "{name}");
    }
    for name in ["go", "c", "cpp", "ruby", "php", "unknown"] {
        assert!(tools.plan_of(&language(name)).is_none(), "{name}");
    }
}

#[test]
fn every_tool_of_a_pack_has_its_purpose_and_install_hint_in_german_and_english() {
    let tools = PackTools::load(&shipped()).unwrap();
    for name in ["csharp", "typescript", "python", "rust", "java", "kotlin"] {
        let plan = tools.plan_of(&language(name)).unwrap();
        let programs = std::iter::once(&plan.indexer.indexer).chain(&plan.indexer.toolchain);
        for tool in programs {
            for tongue in ["de", "en"] {
                let file = Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../../locales")
                    .join(tongue)
                    .join("tools.ftl");
                let text = fs::read_to_string(file).unwrap();
                for suffix in ["purpose", "install"] {
                    let key = format!("tool-{}-{suffix} =", tool.program);
                    assert!(text.contains(&key), "{tongue}: {key}");
                }
            }
        }
    }
}

#[test]
fn a_wrong_requirement_makes_the_pack_invalid() {
    let toolchain =
        "\n[[indexer.toolchain]]\nprogram = \"y\"\nversion_args = []\nmin_version = \"1.0.0\"\n";
    for (name, requirement) in [
        (
            "format",
            "requirement = { file = \"a\", format = \"xml\" }\n",
        ),
        (
            "empty",
            "requirement = { file = \"\", format = \"text\" }\n",
        ),
        (
            "typo",
            "requirement = { file = \"a\", format = \"text\", pth = [] }\n",
        ),
    ] {
        let manifest = format!("id = \"rust\"\n{BASE}{INDEXER}{toolchain}{requirement}");
        let root = pack_with("rust", &manifest);
        let error = PackTools::load(root.path()).expect_err(name);
        assert_eq!(error.code(), ErrorCode::LanguagePackInvalid, "{name}");
    }
}
