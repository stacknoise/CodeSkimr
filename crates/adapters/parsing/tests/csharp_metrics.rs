//! Golden test of `languages/csharp/metrics.scm`: one case per counting rule of ADR 0015.
//!
//! The analyzer does not read `metrics.scm` yet; this test runs the query itself and assigns every
//! capture to the innermost callable body, as the adapter will.

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};
    use std::fs;
    use std::path::Path;

    use tree_sitter::{Node, Parser, Query, QueryCursor, StreamingIterator};

    #[derive(Default, Debug, PartialEq, Eq)]
    struct Counts {
        decisions: usize,
        operators: Vec<String>,
        operands: Vec<String>,
    }

    fn root_dir() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../languages/csharp")
    }

    const CALLABLES: [&str; 3] = [
        "method_declaration",
        "local_function_statement",
        "constructor_declaration",
    ];

    fn callable_name(node: Node<'_>, source: &str) -> Option<String> {
        CALLABLES.contains(&node.kind()).then(|| {
            let name = node.child_by_field_name("name").expect("name");
            source[name.byte_range()].to_owned()
        })
    }

    /// Innermost callable whose body contains the node.
    fn owner(node: Node<'_>, source: &str) -> Option<String> {
        let mut current = node.parent();
        while let Some(parent) = current {
            if let Some(name) = callable_name(parent, source) {
                let body = parent.child_by_field_name("body")?;
                return (body.start_byte() <= node.start_byte()
                    && node.end_byte() <= body.end_byte())
                .then_some(name);
            }
            current = parent.parent();
        }
        None
    }

    fn counts() -> BTreeMap<String, Counts> {
        let source = fs::read_to_string(root_dir().join("tests/metrics.cs")).expect("sample");
        let mut parser = Parser::new();
        let language = tree_sitter_c_sharp::LANGUAGE.into();
        parser.set_language(&language).expect("language");
        let tree = parser.parse(&source, None).expect("parse");
        let query_text = fs::read_to_string(root_dir().join("metrics.scm")).expect("query");
        let query = Query::new(&language, &query_text).expect("metrics.scm compiles");
        let mut result: BTreeMap<String, Counts> = BTreeMap::new();
        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());
        while let Some(found) = matches.next() {
            for capture in found.captures() {
                let Some(name) = owner(capture.node, &source) else {
                    continue;
                };
                let entry = result.entry(name).or_default();
                let text = source[capture.node.byte_range()].to_owned();
                match query.capture_names()[capture.index as usize] {
                    "decision" => entry.decisions += 1,
                    "operator" => entry.operators.push(text),
                    "operand" => entry.operands.push(text),
                    _ => {}
                }
            }
        }
        result
    }

    #[test]
    fn decision_points_follow_the_counting_rules() {
        let found = counts();
        let cc = |name: &str| 1 + found.get(name).map_or(0, |c| c.decisions);
        // if, &&, else if, ||, ??, ??=, ?:
        assert_eq!(cc("Branches"), 8);
        assert_eq!(cc("PlainMethod"), 1);
        assert_eq!(cc("Loops"), 5); // for, foreach, while, do
        assert_eq!(cc("Cases"), 2); // case 1 and case 2 share one arm; default does not count
        assert_eq!(cc("Arms"), 3); // two arms, the discard arm does not count
        assert_eq!(cc("Catches"), 3); // two catches, the when filter adds nothing
        assert_eq!(cc("Outer"), 2); // the lambda's ?: counts for Outer
        assert_eq!(cc("Local"), 2); // the local function counts for itself only
    }

    #[test]
    fn halstead_counts_of_a_small_body() {
        let found = counts();
        let plain = &found["PlainMethod"];
        assert_eq!(plain.operators, ["return"]);
        assert_eq!(plain.operands, ["1"]);
        let operators: BTreeSet<_> = found["Outer"].operators.iter().collect();
        assert!(
            operators.contains(&"(".to_owned()),
            "a call is the operator ("
        );
        assert!(operators.contains(&"=>".to_owned()));
    }

    #[test]
    fn highlights_compile_against_the_grammar() {
        let text = fs::read_to_string(root_dir().join("highlights.scm")).expect("query");
        Query::new(&tree_sitter_c_sharp::LANGUAGE.into(), &text).expect("highlights.scm compiles");
    }
}
