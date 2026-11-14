//! The composition root hands the real `redact` to the runner: what a tool prints on `--version`
//! must not reach a problem report with a token in it (ADR 0033, blueprint S-01).

#[cfg(all(test, unix))]
mod cases {
    use std::ffi::OsString;

    use codeskimr_application::ports::{
        IndexRunner, IndexerSpec, IndexerStatus, ToolIssue, ToolSpec, ToolVersion,
    };
    use codeskimr_indexer::ProcessIndexRunner;

    use crate::redact::redact;

    const TOKEN: &str = "ghp_0123456789abcdefghijklmnopqrstuvwxyz";
    const SEARCH_PATH: &str = "/usr/bin:/bin";

    /// A tool that is `echo`: it "reports its version" by printing the token.
    fn spec() -> IndexerSpec {
        IndexerSpec {
            name: "echo".to_owned(),
            indexer: ToolSpec {
                program: "echo".to_owned(),
                version_args: vec![TOKEN.to_owned()],
                min_version: ToolVersion::default(),
            },
            install_hint: "no hint".to_owned(),
            env: vec![("A".to_owned(), "b".to_owned())],
            toolchain: Vec::new(),
            args: vec!["--out".to_owned()],
            env_allow: Vec::new(),
            home_dirs: Vec::new(),
        }
    }

    #[test]
    fn a_token_in_the_output_of_a_tool_is_masked_by_the_real_redaction() {
        let data = tempfile::tempdir().unwrap();
        let runner = ProcessIndexRunner::new(data.path().to_path_buf(), redact)
            .with_environment(vec![(OsString::from("PATH"), OsString::from(SEARCH_PATH))]);
        let IndexerStatus::Unavailable(problems) = runner.check(&spec()) else {
            panic!("the tool prints no version, so it cannot be ready");
        };
        let ToolIssue::TooOld { found, .. } = &problems[0].issue else {
            panic!("expected a version problem");
        };
        assert!(!found.contains(TOKEN), "{found}");
        assert!(found.contains(crate::consts::REDACTED), "{found}");
    }
}
