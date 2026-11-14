//! Tests of reading the versions a project requires.
#![cfg(test)]

use std::fs;
use std::path::Path;

use codeskimr_application::ports::RequirementReader;
use codeskimr_domain::{FolderPath, ProjectSource, RequirementFormat, RequirementSpec};
use codeskimr_source::{GitCache, ProjectRequirements};

fn project(files: &[(&str, &str)]) -> (tempfile::TempDir, ProjectSource) {
    let dir = tempfile::tempdir().unwrap();
    for (name, text) in files {
        put(dir.path(), name, text);
    }
    let source = ProjectSource::Folder(FolderPath::new(dir.path().to_str().unwrap()).unwrap());
    (dir, source)
}

fn put(root: &Path, name: &str, text: &str) {
    let target = root.join(name);
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    fs::write(target, text).unwrap();
}

fn reader(max_bytes: u64) -> ProjectRequirements {
    ProjectRequirements::new(GitCache::new(Path::new("unused").to_path_buf()), max_bytes)
}

fn spec(file: &str, format: RequirementFormat, path: &[&str]) -> RequirementSpec {
    RequirementSpec {
        file: file.to_owned(),
        format,
        path: path.iter().map(|key| (*key).to_owned()).collect(),
    }
}

fn read(files: &[(&str, &str)], requirement: &RequirementSpec) -> Option<String> {
    let (_dir, source) = project(files);
    reader(65_536).read(&source, requirement)
}

#[test]
fn it_reads_the_sdk_version_of_global_json() {
    let json = r#"{ "sdk": { "version": "9.0.100", "rollForward": "latestFeature" } }"#;
    let requirement = spec("global.json", RequirementFormat::Json, &["sdk", "version"]);
    assert_eq!(
        read(&[("global.json", json)], &requirement).as_deref(),
        Some("9.0.100")
    );
}

#[test]
fn it_reads_the_channel_of_rust_toolchain_toml() {
    let toml = "[toolchain]\nchannel = \"1.82.0\"\ncomponents = [\"clippy\"]\n";
    let requirement = spec(
        "rust-toolchain.toml",
        RequirementFormat::Toml,
        &["toolchain", "channel"],
    );
    let found = read(&[("rust-toolchain.toml", toml)], &requirement);
    assert_eq!(found.as_deref(), Some("1.82.0"));
}

#[test]
fn it_reads_the_first_line_that_says_something_of_a_text_file() {
    let requirement = spec(".nvmrc", RequirementFormat::Text, &[]);
    let found = read(&[(".nvmrc", "# node\n\n  v20.11.1 \n18\n")], &requirement);
    assert_eq!(found.as_deref(), Some("v20.11.1"));
}

#[test]
fn a_missing_wrong_or_malformed_file_says_nothing() {
    let json = spec("global.json", RequirementFormat::Json, &["sdk", "version"]);
    assert_eq!(read(&[], &json), None);
    assert_eq!(read(&[("global.json", "{ not json")], &json), None);
    assert_eq!(
        read(&[("global.json", r#"{"sdk":{"version":9}}"#)], &json),
        None
    );
    assert_eq!(read(&[("global.json", r#"{"other":1}"#)], &json), None);
}

#[test]
fn a_file_above_the_size_limit_is_not_read() {
    let (_dir, source) = project(&[(".nvmrc", "20.1.0\n")]);
    let requirement = spec(".nvmrc", RequirementFormat::Text, &[]);
    assert_eq!(reader(3).read(&source, &requirement), None);
}

#[test]
fn a_path_that_leaves_the_project_root_is_not_read() {
    let outer = tempfile::tempdir().unwrap();
    fs::write(outer.path().join(".nvmrc"), "20.1.0\n").unwrap();
    let root = outer.path().join("project");
    fs::create_dir_all(&root).unwrap();
    let source = ProjectSource::Folder(FolderPath::new(root.to_str().unwrap()).unwrap());
    let inside = spec(".nvmrc", RequirementFormat::Text, &[]);
    assert_eq!(reader(65_536).read(&source, &inside), None);
    for escape in ["../.nvmrc", "sub/../../.nvmrc"] {
        let requirement = spec(escape, RequirementFormat::Text, &[]);
        assert_eq!(reader(65_536).read(&source, &requirement), None, "{escape}");
    }
    let absolute = spec(
        outer.path().join(".nvmrc").to_str().unwrap(),
        RequirementFormat::Text,
        &[],
    );
    assert_eq!(reader(65_536).read(&source, &absolute), None);
}

#[cfg(unix)]
#[test]
fn a_link_that_points_out_of_the_project_is_not_followed() {
    let outer = tempfile::tempdir().unwrap();
    fs::write(outer.path().join("secret.txt"), "20.1.0\n").unwrap();
    let root = outer.path().join("project");
    fs::create_dir_all(&root).unwrap();
    std::os::unix::fs::symlink(outer.path().join("secret.txt"), root.join(".nvmrc")).unwrap();
    let source = ProjectSource::Folder(FolderPath::new(root.to_str().unwrap()).unwrap());
    let requirement = spec(".nvmrc", RequirementFormat::Text, &[]);
    assert_eq!(reader(65_536).read(&source, &requirement), None);
}
