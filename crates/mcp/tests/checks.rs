//! `find_similar_code`, `find_constants` and `check_conventions` against fake ports.

#[cfg(test)]
mod common;

use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_domain::{Permission, PermissionSet, Principal};
use codeskimr_mcp::{ConstantsArgs, ConventionsArgs, SimilarArgs};
use common::tools;

#[cfg(test)]
fn owner() -> Principal {
    Principal::local_owner()
}

#[cfg(test)]
fn similar() -> SimilarArgs {
    SimilarArgs {
        symbol: None,
        snippet: None,
        language: None,
        min_similarity_percent: None,
        project: None,
        limit: None,
    }
}

#[cfg(test)]
fn snippet(code: &str, language: &str) -> SimilarArgs {
    SimilarArgs {
        snippet: Some(code.to_owned()),
        language: Some(language.to_owned()),
        ..similar()
    }
}

#[cfg(test)]
fn symbol(name: &str) -> SimilarArgs {
    SimilarArgs {
        symbol: Some(name.to_owned()),
        ..similar()
    }
}

#[cfg(test)]
fn conventions(paths: &[&str]) -> ConventionsArgs {
    ConventionsArgs {
        paths: Some(paths.iter().map(|path| (*path).to_owned()).collect()),
        project: None,
        limit: None,
    }
}

#[cfg(test)]
fn code_of<T>(result: Result<T, codeskimr_mcp::McpError>) -> ErrorCode {
    result.err().expect("a failure").code()
}

#[test]
fn a_snippet_finds_similar_functions_with_one_based_lines() {
    let answer = tools()
        .find_similar_code(&owner(), &snippet("class A {}", "csharp"))
        .unwrap();
    let matches = answer["matches"].as_array().unwrap();
    assert_eq!(answer["min_similarity_percent"], 50);
    assert_eq!(matches[0]["qualified_name"], "App.Copy30");
    assert_eq!(matches[0]["similarity_percent"], 92);
    assert_eq!(matches[0]["line"], 5);
    assert_eq!(matches[0]["end_line"], 10);
    assert_eq!(answer["snapshot"]["id"], 10);
}

#[test]
fn a_stored_function_does_not_find_itself() {
    let answer = tools().find_similar_code(&owner(), &symbol("1")).unwrap();
    assert_eq!(answer["matches"].as_array().unwrap().len(), 2);
}

#[test]
fn the_minimum_similarity_cuts_the_list_and_the_limit_adds_a_notice() {
    let found = |args: SimilarArgs| {
        let answer = tools().find_similar_code(&owner(), &args).unwrap();
        (
            answer["matches"].as_array().unwrap().len(),
            answer["notice"]["code"].clone(),
        )
    };
    let strict = SimilarArgs {
        min_similarity_percent: Some(80),
        ..snippet("class A {}", "csharp")
    };
    assert_eq!(found(strict), (1, serde_json::Value::Null));
    let short = SimilarArgs {
        limit: Some(1),
        ..snippet("class A {}", "csharp")
    };
    assert_eq!(found(short), (1, serde_json::json!("MCP-501")));
}

#[test]
fn the_arguments_must_name_exactly_one_subject() {
    for args in [
        similar(),
        SimilarArgs {
            symbol: Some("1".to_owned()),
            ..snippet("x", "csharp")
        },
    ] {
        let error = tools().find_similar_code(&owner(), &args).unwrap_err();
        assert_eq!(error.code(), ErrorCode::ToolArgumentsInvalid);
        assert_eq!(error.report().params["detail"], "symbol or snippet");
    }
}

#[test]
fn bad_snippets_and_percentages_are_rejected_by_name() {
    let detail = |args: SimilarArgs| {
        let error = tools().find_similar_code(&owner(), &args).unwrap_err();
        assert_eq!(error.code(), ErrorCode::ToolArgumentsInvalid);
        error.report().params["detail"].clone()
    };
    let no_language = SimilarArgs {
        language: None,
        ..snippet("x", "csharp")
    };
    assert_eq!(detail(no_language), "language");
    assert_eq!(detail(snippet(&"x".repeat(65), "csharp")), "snippet");
    assert_eq!(detail(snippet("short", "csharp")), "snippet");
    assert_eq!(detail(snippet("x", "  ")), "language");
    let zero = SimilarArgs {
        min_similarity_percent: Some(0),
        ..symbol("1")
    };
    assert_eq!(detail(zero), "min_similarity_percent");
    let over = SimilarArgs {
        min_similarity_percent: Some(101),
        ..symbol("1")
    };
    assert_eq!(detail(over), "min_similarity_percent");
}

#[test]
fn a_language_without_a_pack_keeps_its_own_code() {
    let result = tools().find_similar_code(&owner(), &snippet("x", "cobol"));
    assert_eq!(code_of(result), ErrorCode::LanguagePackMissing);
}

#[test]
fn an_unknown_symbol_and_one_without_fingerprints_are_told_apart() {
    assert_eq!(
        code_of(tools().find_similar_code(&owner(), &symbol("App.Nothing"))),
        ErrorCode::SymbolNotFound
    );
    let args = symbol("1");
    let error = common::tools_without_prints()
        .find_similar_code(&owner(), &args)
        .unwrap_err();
    assert_eq!(error.code(), ErrorCode::ToolArgumentsInvalid);
    assert_eq!(error.report().params["detail"], "symbol");
}

#[test]
fn constants_come_with_their_declaration() {
    let args = ConstantsArgs {
        query: "limit".to_owned(),
        project: None,
        limit: Some(3),
    };
    let answer = tools().find_constants(&owner(), &args).unwrap();
    let found = answer["constants"].as_array().unwrap();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0]["qualified_name"], "App.OrderLimit");
    assert_eq!(found[0]["path"], "src/order.cs");
}

#[test]
fn config_and_translation_keys_are_found_by_name_with_one_based_lines() {
    let args = ConstantsArgs {
        query: "LIMIT".to_owned(),
        project: None,
        limit: Some(3),
    };
    let answer = tools().find_constants(&owner(), &args).unwrap();
    assert_eq!(answer["keys_searched"], true);
    let keys = answer["keys"].as_array().unwrap();
    assert_eq!(keys.len(), 3);
    assert_eq!(keys[0]["name"], "mcp.default_limit");
    assert_eq!(keys[0]["kind"], "config");
    assert_eq!(keys[0]["line"], 5);
    assert_eq!(keys[2]["kind"], "i18n");
    assert_eq!(keys[2]["path"], "locales/en/errors.ftl");
    assert_eq!(answer["notice"], serde_json::Value::Null);
}

#[test]
fn keys_are_cut_to_the_limit_with_a_notice() {
    let args = ConstantsArgs {
        query: "limit".to_owned(),
        project: None,
        limit: Some(2),
    };
    let answer = tools().find_constants(&owner(), &args).unwrap();
    assert_eq!(answer["keys"].as_array().unwrap().len(), 2);
    assert_eq!(answer["notice"]["code"], "MCP-501");
}

#[test]
fn without_the_right_to_read_source_the_key_files_are_not_looked_at() {
    let reader = Principal::limited(PermissionSet::none().with(Permission::Read));
    let args = ConstantsArgs {
        query: "limit".to_owned(),
        project: None,
        limit: Some(3),
    };
    let answer = tools().find_constants(&reader, &args).unwrap();
    assert_eq!(answer["keys_searched"], false);
    assert_eq!(answer["keys"].as_array().unwrap().len(), 0);
    assert_eq!(answer["constants"].as_array().unwrap().len(), 1);
}

#[test]
fn an_empty_constant_query_is_rejected() {
    let args = ConstantsArgs {
        query: "  ".to_owned(),
        project: None,
        limit: None,
    };
    assert_eq!(
        code_of(tools().find_constants(&owner(), &args)),
        ErrorCode::ToolArgumentsInvalid
    );
}

#[test]
fn the_check_reports_a_long_function_with_its_numbers() {
    let answer = tools()
        .check_conventions(&owner(), &conventions(&["src"]))
        .unwrap();
    let finding = &answer["findings"][0];
    assert_eq!(answer["files_checked"], 1);
    assert_eq!(finding["code"], "CHK-104");
    assert_eq!(finding["path"], "src/big.cs");
    assert_eq!(finding["symbol"], "App.Big.Run");
    assert_eq!(finding["lines"], 50);
    assert_eq!(finding["max_lines"], 40);
    assert_eq!(
        answer["not_checked"],
        serde_json::json!(["CHK-101", "CHK-103", "CHK-105"])
    );
}

#[test]
fn a_path_outside_the_project_selects_nothing_and_is_listed() {
    let answer = tools()
        .check_conventions(&owner(), &conventions(&["../secret", "/etc"]))
        .unwrap();
    assert_eq!(answer["files_checked"], 0);
    assert_eq!(answer["unmatched"].as_array().unwrap().len(), 2);
}

#[test]
fn no_paths_check_the_whole_project() {
    let args = ConventionsArgs {
        paths: None,
        project: None,
        limit: None,
    };
    let answer = tools().check_conventions(&owner(), &args).unwrap();
    assert_eq!(answer["files_checked"], 1);
    assert_eq!(answer["total"], 1);
}

#[test]
fn the_check_needs_the_right_to_read_source() {
    let reader = Principal::limited(PermissionSet::none().with(Permission::Read));
    let result = tools().check_conventions(&reader, &conventions(&[]));
    assert_eq!(code_of(result), ErrorCode::PermissionDenied);
}

#[test]
fn a_principal_without_read_permission_gets_no_answer_from_the_new_tools() {
    let nobody = Principal::limited(PermissionSet::none().with(Permission::ReadSource));
    let tools = tools();
    let constants = ConstantsArgs {
        query: "limit".to_owned(),
        project: None,
        limit: None,
    };
    let errors = [
        tools.find_similar_code(&nobody, &symbol("1")),
        tools.find_constants(&nobody, &constants),
        tools.check_conventions(&nobody, &conventions(&[])),
    ];
    for result in errors {
        let error = result.unwrap_err();
        assert_eq!(error.code(), ErrorCode::PermissionDenied);
        assert!(error.report().params.is_empty());
    }
}
