//! `codeskimr codes`: the list of the error codes and the explanation of one.

use codeskimr_cli::{Action, CliError, CodesArgs, Parsed, codes_text, parse};
use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_i18n::Messages;

fn args(code: Option<&str>) -> CodesArgs {
    CodesArgs {
        code: code.map(str::to_owned),
    }
}

#[test]
fn the_command_takes_one_optional_code() {
    let messages = Messages::load().unwrap();
    let parsed = parse(["codeskimr", "codes", "SRC-201"], &messages, "en").unwrap();
    let Parsed::Run(invocation) = parsed else {
        panic!("a request expected");
    };
    assert_eq!(invocation.action, Action::Codes(args(Some("SRC-201"))));
    let bare = parse(["codeskimr", "codes"], &messages, "en").unwrap();
    let Parsed::Run(invocation) = bare else {
        panic!("a request expected");
    };
    assert_eq!(invocation.action, Action::Codes(args(None)));
}

#[test]
fn the_list_names_every_live_code_once() {
    let messages = Messages::load().unwrap();
    let text = codes_text(&messages, "en", &args(None)).unwrap();
    let live = ErrorCode::ALL
        .iter()
        .filter(|code| !code.entry().deprecated)
        .count();
    assert_eq!(text.lines().count(), live);
    assert!(text.lines().any(|line| line.starts_with("SRC-201  ")));
}

#[test]
fn a_code_is_explained_in_both_languages_whatever_its_case() {
    let messages = Messages::load().unwrap();
    for language in ["en", "de"] {
        let text = codes_text(&messages, language, &args(Some(" src-201 "))).unwrap();
        assert!(text.starts_with("SRC-201: "), "{language}: {text}");
        assert!(text.contains(ErrorCode::SourcePathNotFound.description()));
        assert!(text.contains("0.1.0"));
    }
}

#[test]
fn an_unknown_code_is_a_usage_error_that_says_so() {
    let messages = Messages::load().unwrap();
    let error = codes_text(&messages, "en", &args(Some("XXX-1"))).unwrap_err();
    assert_eq!(error.code(), ErrorCode::CliArgumentsInvalid);
    let CliError::Arguments(text) = error else {
        panic!("a usage error expected");
    };
    assert!(text.contains("XXX-1"));
}
