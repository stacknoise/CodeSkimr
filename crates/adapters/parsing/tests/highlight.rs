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
    assert!(run("notes.txt", &["fn x"]).is_empty());
    assert!(run("noextension", &["fn x"]).is_empty());
    assert!(run("a.rs", &[]).is_empty());
}

#[test]
fn a_missing_languages_folder_gives_no_runs() {
    let none = TreeSitterHighlighter::new(Path::new("/nowhere/languages"));
    let lines = vec!["fn x() {}".to_owned()];
    assert!(
        none.highlight(&RelativePath::new("a.rs").expect("a path"), &lines)
            .is_empty()
    );
}
