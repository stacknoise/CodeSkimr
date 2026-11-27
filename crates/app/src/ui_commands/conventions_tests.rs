//! The tab "Conventions" of the screen Rules through the path the server takes (ADR 0116).

use std::fmt::Write as _;
use std::fs;

use serde_json::{Value, json};

use super::tests::{Fixture, run_as};
use codeskimr_codes::{Coded, ErrorCode};

use crate::principal;

/// A C# class whose method has `lines` statements, so that it is over the limit of 40 lines.
fn long_method(lines: u32) -> String {
    let mut body = String::new();
    for number in 0..lines {
        writeln!(body, "        total += values[{number}];").unwrap();
    }
    format!(
        "namespace Demo;\n\npublic class Long\n{{\n    public int Sum(int[] values)\n    {{\n        var total = 0;\n{body}        return total;\n    }}\n}}\n"
    )
}

fn check(fixture: &Fixture, project: u64) -> Value {
    fixture
        .call("conventions.check", json!({ "project": project }))
        .unwrap()
}

#[test]
fn a_function_over_the_limit_is_found_in_a_file_no_scan_has_seen() {
    let fixture = Fixture::new();
    let project = fixture.scanned();
    let app = fixture.root.path().join("app");
    fs::write(app.join("Long.cs"), long_method(60)).unwrap();
    let answer = check(&fixture, project);
    let found = answer["findings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|finding| finding["code"] == "CHK-104")
        .unwrap_or_else(|| panic!("a finding for the long method: {answer}"));
    assert_eq!(found["path"], "Long.cs");
    assert_eq!(found["symbol"], "Demo.Long.Sum");
    assert!(found["lines"].as_u64().unwrap() > found["max_lines"].as_u64().unwrap());
    assert!(answer["files_checked"].as_u64().unwrap() >= 1);
    assert_eq!(
        answer["total"],
        answer["findings"].as_array().unwrap().len()
    );
}

#[test]
fn a_clean_folder_has_no_finding_and_still_lists_what_was_not_checked() {
    let fixture = Fixture::new();
    let project = fixture.scanned();
    let answer = check(&fixture, project);
    assert_eq!(answer["total"], 0, "{answer}");
    assert!(answer["not_checked"].is_array());
}

#[test]
fn reading_the_code_needs_the_right_to_read_source() {
    let fixture = Fixture::new();
    let project = fixture.scanned();
    let input = json!({ "project": project });
    for who in [principal::reader(), principal::nobody()] {
        assert_eq!(
            run_as(&fixture, who, "conventions.check", input.clone()),
            Err("AUTH-401")
        );
    }
}

#[test]
fn an_unknown_project_is_refused_with_its_code() {
    let fixture = Fixture::new();
    assert_eq!(
        fixture.call("conventions.check", json!({ "project": 999 })),
        Err("PRJ-201")
    );
}

#[test]
fn a_project_is_checked_by_one_run_at_a_time() {
    let runs = super::conventions::Runs::default();
    let first = runs.begin(7).unwrap();
    assert_eq!(
        runs.begin(7).unwrap_err().code(),
        ErrorCode::SnapshotInProgress
    );
    assert!(runs.begin(8).is_ok(), "another project is not held up");
    drop(first);
    assert!(runs.begin(7).is_ok(), "the project is free again");
}
