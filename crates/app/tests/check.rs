//! `codeskimr check` end to end, through the real binary.

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;
    use std::process::{Command, Output};

    use serde_json::Value;
    use tempfile::TempDir;

    /// Two methods with the same body, one of them in a file of its own.
    const FIRST: &str = "namespace Demo;\n\npublic class Calculator\n{\n    public int Total(int[] values)\n    {\n        var sum = 0;\n        foreach (var value in values)\n        {\n            if (value > 0) { sum += value * 2; } else { sum -= value; }\n        }\n        return sum;\n    }\n}\n";
    const SECOND: &str = "namespace Demo;\n\npublic class Ledger\n{\n    public int Balance(int[] amounts)\n    {\n        var sum = 0;\n        foreach (var value in amounts)\n        {\n            if (value > 0) { sum += value * 2; } else { sum -= value; }\n        }\n        return sum;\n    }\n}\n";
    const OTHER: &str = "namespace Demo;\n\npublic class Greeter\n{\n    public string Hello() { return \"hello\"; }\n}\n";

    struct Project {
        root: TempDir,
    }

    impl Project {
        fn new() -> Self {
            let root = tempfile::tempdir().unwrap();
            let src = root.path().join("project/src");
            fs::create_dir_all(&src).unwrap();
            fs::write(src.join("Calculator.cs"), FIRST).unwrap();
            fs::write(src.join("Ledger.cs"), SECOND).unwrap();
            fs::write(src.join("Greeter.cs"), OTHER).unwrap();
            fs::write(root.path().join("empty-config.toml"), "").unwrap();
            Self { root }
        }

        fn check(&self, args: &[&str]) -> Output {
            self.check_in("en", args)
        }

        fn check_in(&self, language: &str, args: &[&str]) -> Output {
            let languages = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../languages");
            Command::new(env!("CARGO_BIN_EXE_codeskimr"))
                .current_dir(self.root.path().join("project"))
                .env("CODESKIMR_LANG", language)
                .env("CODESKIMR_DATA_DIR", self.root.path().join("data"))
                .env("CODESKIMR_LANGUAGES_DIR", languages)
                .env(
                    "CODESKIMR_CONFIG",
                    self.root.path().join("empty-config.toml"),
                )
                .arg("check")
                .args(args)
                .output()
                .unwrap()
        }
    }

    fn report(output: &Output) -> Value {
        serde_json::from_slice(&output.stdout).unwrap_or_else(|_| {
            panic!(
                "no JSON: {}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            )
        })
    }

    #[test]
    fn a_copied_method_is_reported_and_the_exit_code_says_so() {
        let project = Project::new();
        let output = project.check(&["--json"]);
        assert_eq!(output.status.code(), Some(1));
        let json = report(&output);
        assert_eq!(json["files_checked"], 3);
        let findings = json["findings"].as_array().unwrap();
        assert_eq!(findings.len(), 1, "{json}");
        assert_eq!(findings[0]["code"], "CHK-102");
        assert_eq!(findings[0]["path"], "src/Ledger.cs");
        assert_eq!(findings[0]["duplicate_of"]["path"], "src/Calculator.cs");
        assert!(findings[0]["line"].as_u64().unwrap() >= 1);
    }

    #[test]
    fn a_path_selects_files_and_a_clean_selection_exits_with_zero() {
        let project = Project::new();
        let output = project.check(&["--json", "src/Ledger.cs", "src/Greeter.cs"]);
        assert!(output.status.success());
        let json = report(&output);
        assert_eq!(json["files_checked"], 2);
        assert!(json["findings"].as_array().unwrap().is_empty());
        assert_eq!(json["not_checked"].as_array().unwrap().len(), 3);
    }

    #[test]
    fn the_length_limit_comes_from_the_configuration() {
        let project = Project::new();
        let output = project.check(&["--json", "--set", "conventions.max_function_lines=3"]);
        let json = report(&output);
        let long: Vec<&Value> = json["findings"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|finding| finding["code"] == "CHK-104")
            .collect();
        assert_eq!(long.len(), 2, "{json}");
        assert_eq!(long[0]["limit"], 3);
    }

    #[test]
    fn the_text_report_follows_the_language() {
        let project = Project::new();
        let english = String::from_utf8_lossy(&project.check(&[]).stdout).into_owned();
        assert!(english.contains("src/Ledger.cs:5: CHK-102"), "{english}");
        assert!(
            english.contains("3 files checked, 1 findings."),
            "{english}"
        );
        let german = String::from_utf8_lossy(&project.check_in("de", &[]).stdout).into_owned();
        assert!(german.contains("3 Dateien geprüft, 1 Funde."), "{german}");
        assert!(german.contains("kopiert"), "{german}");
    }
}
