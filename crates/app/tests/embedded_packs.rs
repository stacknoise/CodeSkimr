//! The real program finds its language packs without any `languages/` folder.

use std::fs;
use std::process::Command;

#[test]
fn a_program_alone_scans_and_writes_its_packs_to_the_data_folder() {
    let root = tempfile::tempdir().unwrap();
    let project = root.path().join("project");
    let work = root.path().join("work");
    let data = root.path().join("data");
    fs::create_dir_all(&project).unwrap();
    fs::create_dir_all(&work).unwrap();
    fs::write(project.join("main.rs"), "fn main() {}\n").unwrap();
    fs::write(root.path().join("empty-config.toml"), "").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_codeskimr"))
        .current_dir(&work)
        .env_remove("CODESKIMR_LANGUAGES_DIR")
        .env("CODESKIMR_LANG", "en")
        .env("CODESKIMR_DATA_DIR", &data)
        .env("CODESKIMR_CONFIG", root.path().join("empty-config.toml"))
        .args(["scan", "--json"])
        .arg(&project)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let written: Vec<_> = fs::read_dir(data.join("languages")).unwrap().collect();
    assert_eq!(written.len(), 1);
    let folder = written[0].as_ref().unwrap().path();
    assert!(folder.join(".complete").is_file());
    assert!(folder.join("rust/language.toml").is_file());
    assert!(!folder.join("rust/tests").exists());
}

#[test]
fn a_languages_folder_in_the_current_folder_is_not_used() {
    let root = tempfile::tempdir().unwrap();
    let project = root.path().join("project");
    let work = root.path().join("work");
    let data = root.path().join("data");
    fs::create_dir_all(&project).unwrap();
    fs::create_dir_all(work.join("languages/rust")).unwrap();
    fs::write(work.join("languages/rust/language.toml"), "not a pack {{{").unwrap();
    fs::write(project.join("main.rs"), "fn main() {}\n").unwrap();
    fs::write(root.path().join("empty-config.toml"), "").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_codeskimr"))
        .current_dir(&work)
        .env_remove("CODESKIMR_LANGUAGES_DIR")
        .env("CODESKIMR_LANG", "en")
        .env("CODESKIMR_DATA_DIR", &data)
        .env("CODESKIMR_CONFIG", root.path().join("empty-config.toml"))
        .args(["scan", "--json"])
        .arg(&project)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(fs::read_dir(data.join("languages")).unwrap().count(), 1);
}
