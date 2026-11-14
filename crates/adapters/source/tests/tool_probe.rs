//! Tests of the tool probe: it starts only the program it is told to, in the right place, with
//! the right environment and within the time limit. Unix only: the tools are standard programs.
#![cfg(test)]
#![cfg(unix)]

use std::time::{Duration, Instant};

use codeskimr_application::ports::{ProbeOutcome, ToolProbe};
use codeskimr_source::{ProbeSettings, ProcessProbe};

fn settings(work_dir: &std::path::Path) -> ProbeSettings {
    ProbeSettings {
        timeout: Duration::from_secs(5),
        max_output: 4096,
        env_allowlist: vec!["PATH".to_owned()],
        work_dir: work_dir.to_path_buf(),
    }
}

fn shell(probe: &ProcessProbe, script: &str) -> ProbeOutcome {
    probe.probe("sh", &["-c".to_owned(), script.to_owned()])
}

fn output(outcome: ProbeOutcome) -> String {
    match outcome {
        ProbeOutcome::Ran(text) => text,
        other => panic!("expected output, got {other:?}"),
    }
}

#[test]
fn it_returns_what_the_tool_prints_on_both_streams() {
    let dir = tempfile::tempdir().unwrap();
    let probe = ProcessProbe::new(settings(dir.path()));
    let text = output(shell(&probe, "echo 8.0.404; echo openjdk 17.0.9 >&2"));
    assert!(
        text.contains("8.0.404") && text.contains("17.0.9"),
        "{text}"
    );
}

#[test]
fn a_program_that_does_not_exist_is_not_found() {
    let dir = tempfile::tempdir().unwrap();
    let probe = ProcessProbe::new(settings(dir.path()));
    let outcome = probe.probe("codeskimr-no-such-tool", &[]);
    assert_eq!(outcome, ProbeOutcome::NotFound);
}

#[test]
fn it_runs_in_the_given_folder_and_not_in_the_current_one() {
    let dir = tempfile::tempdir().unwrap();
    let probe = ProcessProbe::new(settings(dir.path()));
    let text = output(shell(&probe, "pwd -P"));
    let expected = std::fs::canonicalize(dir.path()).unwrap();
    assert_eq!(text.trim(), expected.to_str().unwrap());
}

#[test]
fn the_process_sees_only_the_allow_listed_variables() {
    let dir = tempfile::tempdir().unwrap();
    let probe = ProcessProbe::new(settings(dir.path()));
    // Cargo sets this one for every test process; it is not on the list.
    assert!(std::env::var_os("CARGO_MANIFEST_DIR").is_some());
    let text = output(shell(&probe, "env"));
    assert!(text.contains("PATH="), "{text}");
    assert!(!text.contains("CARGO_MANIFEST_DIR"), "{text}");
}

#[test]
fn a_tool_that_does_not_finish_is_killed_at_the_limit() {
    let dir = tempfile::tempdir().unwrap();
    let probe = ProcessProbe::new(ProbeSettings {
        timeout: Duration::from_millis(200),
        ..settings(dir.path())
    });
    let started = Instant::now();
    let outcome = shell(&probe, "exec sleep 30");
    assert_eq!(outcome, ProbeOutcome::TimedOut);
    assert!(started.elapsed() < Duration::from_secs(5));
}

#[test]
fn output_is_cut_at_the_limit_and_a_flood_does_not_block_the_tool() {
    let dir = tempfile::tempdir().unwrap();
    let probe = ProcessProbe::new(ProbeSettings {
        max_output: 16,
        ..settings(dir.path())
    });
    let text = output(shell(&probe, "yes 1.2.3 | head -c 300000"));
    assert!(text.len() <= 16 + 1, "{}", text.len());
}

#[test]
fn the_tool_gets_no_input() {
    let dir = tempfile::tempdir().unwrap();
    let probe = ProcessProbe::new(settings(dir.path()));
    assert_eq!(output(probe.probe("cat", &[])).trim(), "");
}
