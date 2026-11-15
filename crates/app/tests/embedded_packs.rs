//! The real program finds its language packs without any `languages/` folder.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// A scanned Rust project, an empty working folder and an empty data folder.
struct Setup {
    root: tempfile::TempDir,
}

impl Setup {
    fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        for folder in ["project", "work"] {
            fs::create_dir_all(root.path().join(folder)).unwrap();
        }
        fs::write(root.path().join("project/main.rs"), "fn main() {}\n").unwrap();
        fs::write(root.path().join("empty-config.toml"), "").unwrap();
        Self { root }
    }

    fn work(&self) -> PathBuf {
        self.root.path().join("work")
    }

    fn data(&self) -> PathBuf {
        self.root.path().join("data")
    }

    /// Scans the project from the working folder, without any named pack folder.
    fn scan(&self) {
        let output = Command::new(env!("CARGO_BIN_EXE_codeskimr"))
            .current_dir(self.work())
            .env_remove("CODESKIMR_LANGUAGES_DIR")
            .env("CODESKIMR_LANG", "en")
            .env("CODESKIMR_DATA_DIR", self.data())
            .env(
                "CODESKIMR_CONFIG",
                self.root.path().join("empty-config.toml"),
            )
            .args(["scan", "--json"])
            .arg(self.root.path().join("project"))
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    /// The one folder the program wrote below `<data>/languages`.
    fn written(&self) -> PathBuf {
        let found: Vec<_> = fs::read_dir(self.data().join("languages"))
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect();
        assert_eq!(found.len(), 1);
        found[0].clone()
    }
}

fn is_file(folder: &Path, name: &str) -> bool {
    folder.join(name).is_file()
}

#[test]
fn a_program_alone_scans_and_writes_its_packs_to_the_data_folder() {
    let setup = Setup::new();
    setup.scan();
    let folder = setup.written();
    assert!(is_file(&folder, ".complete"));
    assert!(is_file(&folder, "rust/language.toml"));
    assert!(!folder.join("rust/tests").exists());
}

#[test]
fn a_languages_folder_in_the_current_folder_is_not_used() {
    let setup = Setup::new();
    let bogus = setup.work().join("languages/rust");
    fs::create_dir_all(&bogus).unwrap();
    fs::write(bogus.join("language.toml"), "not a pack {{{").unwrap();
    setup.scan();
    assert!(is_file(&setup.written(), "rust/language.toml"));
}
