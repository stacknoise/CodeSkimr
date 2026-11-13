//! Parsing of the command line and the texts that come with it.

use std::path::PathBuf;

use codeskimr_cli::{
    Action, CliError, CredentialAction, Parsed, ProviderChoice, ScanArgs, ServeArgs, language_from,
    parse, serve_started,
};
use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_i18n::Messages;

#[cfg(test)]
fn run(arguments: &[&str]) -> Result<Parsed, CliError> {
    let messages = Messages::load().unwrap();
    parse(arguments.iter().copied(), &messages, "en")
}

#[cfg(test)]
fn invocation(arguments: &[&str]) -> codeskimr_cli::Invocation {
    match run(arguments).unwrap() {
        Parsed::Run(invocation) => invocation,
        Parsed::Display(text) => panic!("unexpected output: {text}"),
    }
}

#[test]
fn scan_takes_a_source_a_branch_and_the_json_switch() {
    let parsed = invocation(&["codeskimr", "scan", "repo", "--branch", "dev", "--json"]);
    assert_eq!(
        parsed.action,
        Action::Scan(ScanArgs {
            source: "repo".to_owned(),
            branch: Some("dev".to_owned()),
            json: true,
            watch: false,
            credential: None,
        })
    );
}

#[test]
fn scan_watch_is_an_opt_in_switch() {
    let parsed = invocation(&["codeskimr", "scan", "repo", "--watch"]);
    let Action::Scan(args) = parsed.action else {
        panic!("scan expected");
    };
    assert!(args.watch);
    assert!(!args.json);
}

#[test]
fn global_options_work_before_and_after_the_command() {
    let before = invocation(&["codeskimr", "--data-dir", "d", "scan", "repo"]);
    let after = invocation(&["codeskimr", "scan", "repo", "--data-dir", "d"]);
    assert_eq!(before.global.data_dir, Some(PathBuf::from("d")));
    assert_eq!(before.global, after.global);
}

#[test]
fn overrides_keep_their_order_and_split_at_the_first_equals_sign() {
    let parsed = invocation(&[
        "codeskimr",
        "scan",
        "repo",
        "--set",
        "a.b=1",
        "--set",
        "scan.excluded_folders=[\"x=y\"]",
    ]);
    assert_eq!(
        parsed.global.overrides,
        [
            ("a.b".to_owned(), "1".to_owned()),
            ("scan.excluded_folders".to_owned(), "[\"x=y\"]".to_owned())
        ]
    );
}

#[test]
fn an_override_without_a_value_is_invalid() {
    let error = run(&["codeskimr", "scan", "repo", "--set", "novalue"]).unwrap_err();
    assert_eq!(error.code(), ErrorCode::CliArgumentsInvalid);
}

#[test]
fn mcp_takes_an_optional_project_folder() {
    let with = invocation(&["codeskimr", "mcp", "--project", "p"]);
    let without = invocation(&["codeskimr", "mcp"]);
    assert!(matches!(with.action, Action::Mcp(args) if args.project == Some(PathBuf::from("p"))));
    assert!(matches!(without.action, Action::Mcp(args) if args.project.is_none()));
}

#[test]
fn serve_takes_no_argument_of_its_own() {
    let parsed = invocation(&["codeskimr", "serve", "--set", "server.port=9000"]);
    assert_eq!(parsed.action, Action::Serve(ServeArgs));
    assert_eq!(parsed.global.overrides.len(), 1);
    let error = run(&["codeskimr", "serve", "--bind", "0.0.0.0"]).unwrap_err();
    assert_eq!(error.code(), ErrorCode::CliArgumentsInvalid);
}

#[test]
fn the_listening_line_names_the_address_in_both_languages() {
    let messages = Messages::load().unwrap();
    let url = "http://127.0.0.1:7421";
    for language in ["en", "de"] {
        let line = serve_started(&messages, language, url).unwrap();
        assert!(line.contains(url), "{language}: {line}");
    }
}

#[test]
fn missing_and_unknown_arguments_are_cli_101() {
    for arguments in [
        &["codeskimr", "scan"][..],
        &["codeskimr", "scan", "repo", "--nope"],
        &["codeskimr", "nope"],
    ] {
        let error = run(arguments).unwrap_err();
        assert_eq!(
            error.code(),
            ErrorCode::CliArgumentsInvalid,
            "{arguments:?}"
        );
    }
}

#[test]
fn no_command_starts_the_app() {
    match run(&["codeskimr"]).unwrap() {
        Parsed::Run(invocation) => assert_eq!(invocation.action, Action::Start),
        Parsed::Display(_) => panic!("the bare command runs"),
    }
}

#[test]
fn help_and_version_are_output_not_errors() {
    for flag in ["--help", "--version"] {
        assert!(matches!(
            run(&["codeskimr", flag]).unwrap(),
            Parsed::Display(_)
        ));
    }
}

#[test]
fn the_language_comes_from_the_first_non_empty_tag() {
    assert_eq!(language_from(["", " ", "de_DE.UTF-8", "en"]), "de");
    assert_eq!(language_from(["DE"]), "de");
    assert_eq!(language_from(["fr_FR"]), "en");
    assert_eq!(language_from([]), "en");
}

#[test]
fn export_takes_a_project_and_the_html_file() {
    let parsed = invocation(&["codeskimr", "export", "7", "--html", "out.html"]);
    assert_eq!(
        parsed.action,
        Action::Export(codeskimr_cli::ExportArgs {
            project: "7".to_owned(),
            html: PathBuf::from("out.html"),
        })
    );
}

#[test]
fn export_without_the_html_file_is_a_usage_error() {
    let error = run(&["codeskimr", "export", "7"]).unwrap_err();
    assert_eq!(error.code(), ErrorCode::CliArgumentsInvalid);
}

#[test]
fn scan_can_name_a_credential() {
    let parsed = invocation(&[
        "codeskimr",
        "scan",
        "https://example.com/a.git",
        "--credential",
        "work",
    ]);
    let Action::Scan(args) = parsed.action else {
        panic!("scan expected");
    };
    assert_eq!(args.credential.as_deref(), Some("work"));
}

#[test]
fn credential_set_takes_a_name_and_a_user_but_never_the_secret() {
    let parsed = invocation(&["codeskimr", "credential", "set", "work", "--user", "anton"]);
    assert_eq!(
        parsed.action,
        Action::Credential(CredentialAction::Set {
            id: "work".to_owned(),
            user: "anton".to_owned(),
        })
    );
    // There is no flag for the secret, so it cannot end up in a shell history.
    for flag in ["--secret", "--password", "--token"] {
        let attempt = run(&[
            "codeskimr",
            "credential",
            "set",
            "work",
            "--user",
            "a",
            flag,
            "x",
        ]);
        assert!(attempt.is_err(), "{flag}");
    }
}

#[test]
fn credential_delete_and_login_are_parsed() {
    let deleted = invocation(&["codeskimr", "credential", "delete", "work"]);
    assert_eq!(
        deleted.action,
        Action::Credential(CredentialAction::Delete {
            id: "work".to_owned()
        })
    );
    let login = invocation(&[
        "codeskimr",
        "credential",
        "login",
        "work",
        "--provider",
        "gitlab",
        "--client-id",
        "app",
        "--host",
        "https://git.example.com",
    ]);
    assert_eq!(
        login.action,
        Action::Credential(CredentialAction::Login {
            id: "work".to_owned(),
            provider: ProviderChoice::GitLab,
            client_id: "app".to_owned(),
            host: Some("https://git.example.com".to_owned()),
        })
    );
}
