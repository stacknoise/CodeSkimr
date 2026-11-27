//! The highlighter colours code with the `highlights.scm` of the shipped packs (ADR 0064).

use std::path::Path;

use codeskimr_application::ports::{SyntaxHighlighter, TokenClass, TokenSpan};
use codeskimr_domain::RelativePath;
use codeskimr_parsing::TreeSitterHighlighter;

fn highlighter() -> TreeSitterHighlighter {
    TreeSitterHighlighter::new(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../languages"))
}

fn run(path: &str, code: &[&str]) -> Vec<TokenSpan> {
    let lines: Vec<String> = code.iter().map(|line| (*line).to_owned()).collect();
    // A path that is no path gives no runs, which the assertions of every test would notice.
    RelativePath::new(path).map_or_else(
        |_| Vec::new(),
        |path| highlighter().highlight(&path, &lines),
    )
}

/// The text of the runs that carry `class`.
fn texts(code: &[&str], spans: &[TokenSpan], class: TokenClass) -> Vec<String> {
    spans
        .iter()
        .filter(|span| span.class == class)
        .filter_map(|span| {
            let line = code.get(span.line as usize)?;
            line.get(span.start as usize..span.end as usize)
                .map(str::to_owned)
        })
        .collect()
}

#[test]
fn rust_keywords_strings_numbers_and_comments_get_their_classes() {
    let code = [
        "// note",
        "fn answer() -> u32 {",
        "    let s = \"x\"; 42",
        "}",
    ];
    let spans = run("src/lib.rs", &code);
    assert_eq!(texts(&code, &spans, TokenClass::Comment), ["// note"]);
    assert_eq!(texts(&code, &spans, TokenClass::Keyword), ["fn", "let"]);
    assert_eq!(texts(&code, &spans, TokenClass::String), ["\"x\""]);
    assert_eq!(texts(&code, &spans, TokenClass::Number), ["42"]);
    assert_eq!(texts(&code, &spans, TokenClass::Function), ["answer"]);
    assert_eq!(texts(&code, &spans, TokenClass::Type), ["u32"]);
}

#[test]
fn a_block_comment_over_two_lines_is_one_run_per_line() {
    let code = ["/* one", "two */ let x = 1;"];
    let spans = run("a.rs", &code);
    assert_eq!(
        texts(&code, &spans, TokenClass::Comment),
        ["/* one", "two */"]
    );
}

#[test]
fn the_runs_do_not_overlap_and_follow_the_text() {
    let code = ["#[derive(Debug)]", "struct A { b: \"é\" }"];
    let spans = run("a.rs", &code);
    let order: Vec<_> = spans.iter().map(|s| (s.line, s.start)).collect();
    let mut sorted = order.clone();
    sorted.sort_unstable();
    assert_eq!(order, sorted);
    for pair in spans.windows(2) {
        assert!(pair[0].line != pair[1].line || pair[0].end <= pair[1].start);
    }
}

#[test]
fn other_languages_are_coloured_by_their_own_pack() {
    let py = ["def f(x):", "    return 'a'  # c"];
    let spans = run("m.py", &py);
    assert_eq!(texts(&py, &spans, TokenClass::Keyword), ["def", "return"]);
    assert_eq!(texts(&py, &spans, TokenClass::Comment), ["# c"]);
    let ts = ["const n: number = 1;"];
    assert_eq!(texts(&ts, &run("a.ts", &ts), TokenClass::Number), ["1"]);
}

#[test]
fn an_unknown_extension_or_empty_code_gives_no_runs() {
    assert_eq!(run("notes.txt", &["fn x"]).len(), 0);
    assert_eq!(run("noextension", &["fn x"]).len(), 0);
    assert_eq!(run("a.rs", &[]).len(), 0);
}

#[test]
fn a_missing_languages_folder_gives_no_runs() {
    let none = TreeSitterHighlighter::new(Path::new("/nowhere/languages"));
    let lines = vec!["fn x() {}".to_owned()];
    assert_eq!(
        none.highlight(&RelativePath::new("a.rs").expect("a path"), &lines)
            .len(),
        0
    );
}

#[test]
fn c_is_coloured_by_its_own_pack() {
    let code = [
        "#include <stdio.h>",
        "/* note */",
        "static int add(int a) {",
        "    return a + 42; // done",
        "}",
        "const char *s = \"x\\n\";",
    ];
    let spans = run("kernel/main.c", &code);
    assert_eq!(
        texts(&code, &spans, TokenClass::Comment),
        ["/* note */", "// done"]
    );
    assert_eq!(
        texts(&code, &spans, TokenClass::Keyword),
        ["static", "return", "const"]
    );
    assert_eq!(texts(&code, &spans, TokenClass::Number), ["42"]);
    assert_eq!(texts(&code, &spans, TokenClass::Function), ["add"]);
    assert_eq!(
        texts(&code, &spans, TokenClass::Type),
        ["int", "int", "char"]
    );
    assert!(texts(&code, &spans, TokenClass::String).contains(&"<stdio.h>".to_owned()));
}

#[test]
fn cpp_is_coloured_by_its_own_pack() {
    let code = [
        "class Box { public: int size() const; };",
        "int main() { return 7; /* x */ }",
    ];
    let spans = run("a.cpp", &code);
    assert_eq!(texts(&code, &spans, TokenClass::Comment), ["/* x */"]);
    assert_eq!(texts(&code, &spans, TokenClass::Number), ["7"]);
    let keywords = texts(&code, &spans, TokenClass::Keyword);
    assert!(keywords.contains(&"class".to_owned()) && keywords.contains(&"public".to_owned()));
    assert!(texts(&code, &spans, TokenClass::Function).contains(&"main".to_owned()));
}

#[test]
fn go_is_coloured_by_its_own_pack() {
    let code = [
        "package main",
        "// note",
        "func add(a int) int { return a + 1 }",
        "var s = \"x\"",
    ];
    let spans = run("main.go", &code);
    assert_eq!(texts(&code, &spans, TokenClass::Comment), ["// note"]);
    assert_eq!(
        texts(&code, &spans, TokenClass::Keyword),
        ["package", "func", "return", "var"]
    );
    assert_eq!(texts(&code, &spans, TokenClass::Function), ["add"]);
    assert_eq!(texts(&code, &spans, TokenClass::Number), ["1"]);
    assert_eq!(texts(&code, &spans, TokenClass::String), ["\"x\""]);
}

#[test]
fn ruby_is_coloured_by_its_own_pack() {
    let code = ["# note", "def add(a)", "  a + 1 if a", "end", "x = \"s\""];
    let spans = run("a.rb", &code);
    assert_eq!(texts(&code, &spans, TokenClass::Comment), ["# note"]);
    assert_eq!(
        texts(&code, &spans, TokenClass::Keyword),
        ["def", "if", "end"]
    );
    assert_eq!(texts(&code, &spans, TokenClass::Function), ["add"]);
    assert_eq!(texts(&code, &spans, TokenClass::Number), ["1"]);
    assert_eq!(texts(&code, &spans, TokenClass::String), ["\"s\""]);
}

#[test]
fn abap_is_coloured_by_its_own_pack() {
    let code = [
        "* note",
        "CLASS zcl_a IMPLEMENTATION.",
        "  METHOD run.",
        "    IF a > 1.",
        "      x = 'q'.",
        "    ENDIF.",
        "  ENDMETHOD.",
        "ENDCLASS.",
    ];
    let spans = run("a.abap", &code);
    assert_eq!(texts(&code, &spans, TokenClass::Comment), ["* note"]);
    assert_eq!(texts(&code, &spans, TokenClass::String), ["'q'"]);
    assert_eq!(texts(&code, &spans, TokenClass::Number), ["1"]);
    let keywords = texts(&code, &spans, TokenClass::Keyword);
    assert!(keywords.contains(&"CLASS".to_owned()), "{keywords:?}");
    assert!(keywords.contains(&"IF".to_owned()), "{keywords:?}");
    assert_eq!(texts(&code, &spans, TokenClass::Function), ["run"]);
}

#[test]
fn php_is_coloured_by_its_own_pack() {
    let code = [
        "<?php",
        "// note",
        "function add($a) { return $a + 1; }",
        "$s = 'x';",
    ];
    let spans = run("a.php", &code);
    assert_eq!(texts(&code, &spans, TokenClass::Comment), ["// note"]);
    assert_eq!(
        texts(&code, &spans, TokenClass::Keyword),
        ["function", "return"]
    );
    assert_eq!(texts(&code, &spans, TokenClass::Function), ["add"]);
    assert_eq!(texts(&code, &spans, TokenClass::Number), ["1"]);
    assert_eq!(texts(&code, &spans, TokenClass::String), ["'x'"]);
}

#[test]
fn types_and_escaped_strings_are_coloured_in_go_php_and_ruby() {
    let go = ["type Box struct{}", "var s = \"a\\n\""];
    let spans = run("a.go", &go);
    assert!(texts(&go, &spans, TokenClass::Type).contains(&"Box".to_owned()));
    assert_eq!(texts(&go, &spans, TokenClass::String), ["\"a\\n\""]);
    let php = ["<?php", "class Box {}", "$s = \"a\\n\";"];
    let spans = run("a.php", &php);
    assert_eq!(texts(&php, &spans, TokenClass::Type), ["Box"]);
    assert_eq!(texts(&php, &spans, TokenClass::String), ["\"a\\n\""]);
    let rb = ["class Box", "end", "x = \"a\\n\""];
    let spans = run("a.rb", &rb);
    assert_eq!(texts(&rb, &spans, TokenClass::Type), ["Box"]);
    assert_eq!(texts(&rb, &spans, TokenClass::String), ["\"a\\n\""]);
}
