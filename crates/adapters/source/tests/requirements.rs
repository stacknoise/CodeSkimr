//! Tests of reading the versions a project requires.
#![cfg(test)]

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use codeskimr_application::ports::{RequirementReader, RequirementValue};
use codeskimr_domain::{
    Ceiling, FolderPath, ProjectSource, RequirementFormat, RequirementSpec, RollForwardSpec,
};
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
        roll_forward: None,
    }
}

/// The version texts of what `read_all` found.
fn versions(values: Vec<RequirementValue>) -> Vec<String> {
    values.into_iter().map(|value| value.version).collect()
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

fn sdk(file: &str) -> RequirementSpec {
    spec(file, RequirementFormat::Json, &["sdk", "version"])
}

#[test]
fn global_json_may_hold_comments_and_trailing_commas_like_the_sdk_accepts() {
    let json = r#"
    // which SDK
    {
      /* the sdk */
      "sdk": {
        "version": "9.0.100", // trailing comma follows
        "comment": "a // inside a string and /* here */ stay",
        "rollForward": "latestFeature",
      },
    }"#;
    assert_eq!(
        read(&[("global.json", json)], &sdk("global.json")).as_deref(),
        Some("9.0.100")
    );
    // A quote that is escaped does not end the string.
    let escaped = r#"{ "sdk": { "version": "8.0.1\"//", } }"#;
    assert_eq!(
        read(&[("global.json", escaped)], &sdk("global.json")).as_deref(),
        Some("8.0.1\"//")
    );
}

#[test]
fn every_manifest_folder_is_read_but_not_dependencies_or_build_output() {
    let (_dir, source) = project(&[
        ("global.json", r#"{"sdk":{"version":"8.0.100"}}"#),
        (
            "services/api/global.json",
            r#"{"sdk":{"version":"9.0.200"}}"#,
        ),
        (
            "node_modules/pkg/global.json",
            r#"{"sdk":{"version":"99.0.0"}}"#,
        ),
        ("bin/global.json", r#"{"sdk":{"version":"98.0.0"}}"#),
    ]);
    let mut found = versions(reader(65_536).read_all(&source, &sdk("global.json")));
    found.sort();
    assert_eq!(found, ["8.0.100", "9.0.200"]);
    // A name with a folder in it is the one file, as before.
    let one = versions(reader(65_536).read_all(&source, &sdk("services/api/global.json")));
    assert_eq!(one, ["9.0.200"]);
}

#[test]
fn a_git_project_has_no_requirement_before_its_first_scan_and_has_one_after_it() {
    use codeskimr_domain::{BranchName, GitUrl};

    let cache = tempfile::tempdir().unwrap();
    let git = GitCache::new(cache.path().to_path_buf());
    let url = GitUrl::new("https://example.invalid/team/repo.git").unwrap();
    let branch = BranchName::new("main").unwrap();
    let source = ProjectSource::Git {
        url: url.clone(),
        branch: branch.clone(),
        credential: None,
    };
    let reader = ProjectRequirements::new(git.clone(), 65_536);
    // Before the first scan there is no clone: nothing is required (docs/tools-check.md).
    assert_eq!(
        versions(reader.read_all(&source, &sdk("global.json"))),
        Vec::<String>::new()
    );
    // The first scan leaves the clone; the next check finds the file in it.
    let clone = git.checkout_dir(&url, &branch);
    fs::create_dir_all(&clone).unwrap();
    fs::write(
        clone.join("global.json"),
        r#"{"sdk":{"version":"9.0.100"}}"#,
    )
    .unwrap();
    assert_eq!(
        versions(reader.read_all(&source, &sdk("global.json"))),
        ["9.0.100"]
    );
}

#[test]
fn a_block_comment_that_starts_with_slash_star_slash_is_still_open() {
    // `/*/` does not close itself; the real end follows.
    let json = r#"{ /*/ "sdk": { "version": "1.0.0" } */ "sdk": { "version": "9.0.100" } }"#;
    assert_eq!(
        read(&[("global.json", json)], &sdk("global.json")).as_deref(),
        Some("9.0.100")
    );
}

#[test]
fn the_search_goes_six_levels_down_and_reads_at_most_32_files() {
    let deep = "a/b/c/d/e/f"; // six levels below the root
    let too_deep = "a/b/c/d/e/f/g";
    let (_dir, source) = project(&[
        (
            &format!("{deep}/global.json"),
            r#"{"sdk":{"version":"6.0.0"}}"#,
        ),
        (
            &format!("{too_deep}/global.json"),
            r#"{"sdk":{"version":"7.0.0"}}"#,
        ),
    ]);
    assert_eq!(
        versions(reader(65_536).read_all(&source, &sdk("global.json"))),
        ["6.0.0"]
    );
    let many: Vec<(String, &str)> = (0..40)
        .map(|n| {
            (
                format!("p{n}/global.json"),
                r#"{"sdk":{"version":"8.0.100"}}"#,
            )
        })
        .collect();
    let files: Vec<(&str, &str)> = many
        .iter()
        .map(|(name, text)| (name.as_str(), *text))
        .collect();
    let (_dir, source) = project(&files);
    assert_eq!(
        reader(65_536).read_all(&source, &sdk("global.json")).len(),
        32
    );
}

fn sdk_with_policy(file: &str) -> RequirementSpec {
    RequirementSpec {
        roll_forward: Some(RollForwardSpec {
            path: vec!["sdk".to_owned(), "rollForward".to_owned()],
            default: Ceiling::SameBand,
            values: BTreeMap::new(),
        }),
        ..sdk(file)
    }
}

#[test]
fn the_policy_comes_from_the_same_file_as_the_version() {
    let (_dir, source) = project(&[
        (
            "global.json",
            r#"{"sdk":{"version":"8.0.100","rollForward":"latestMinor"}}"#,
        ),
        ("api/global.json", r#"{"sdk":{"version":"9.0.100"}}"#),
    ]);
    let mut found = reader(65_536).read_all(&source, &sdk_with_policy("global.json"));
    found.sort_by(|a, b| a.version.cmp(&b.version));
    let pairs: Vec<(&str, Option<&str>)> = found
        .iter()
        .map(|value| (value.version.as_str(), value.policy.as_deref()))
        .collect();
    assert_eq!(pairs, [("8.0.100", Some("latestMinor")), ("9.0.100", None)]);
}

#[test]
fn a_requirement_without_a_policy_rule_reads_no_policy() {
    let (_dir, source) = project(&[(
        "global.json",
        r#"{"sdk":{"version":"8.0.100","rollForward":"latestMinor"}}"#,
    )]);
    let found = reader(65_536).read_all(&source, &sdk("global.json"));
    assert_eq!(found.first().map(|value| value.policy.clone()), Some(None));
}
