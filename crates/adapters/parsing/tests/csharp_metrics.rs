//! Golden test of `languages/csharp/metrics.scm`: one case per counting rule of ADR 0015, run
//! through the analyzer, which assigns every capture to the innermost callable.

#[cfg(test)]
mod common;

#[cfg(test)]
mod tests {
    use codeskimr_application::ids::IdSequence;
    use codeskimr_application::ports::FileFacts;
    use codeskimr_domain::{BodyCounts, FolderPath, SymbolCounts};
    use codeskimr_parsing::{LanguagePacks, TreeSitterAnalyzer};
    use tree_sitter::Query;

    use super::common::analyze_file;
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};
    use std::time::Duration;

    fn languages() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../languages")
    }

    fn facts() -> FileFacts {
        let analyzer = TreeSitterAnalyzer::new(
            LanguagePacks::load(&languages()).expect("packs load"),
            Duration::from_secs(10),
        );
        let root =
            FolderPath::new(languages().join("csharp/tests").to_string_lossy()).expect("root");
        analyze_file(
            &analyzer,
            &IdSequence::starting_at(1),
            &root,
            "metrics.cs",
            "csharp",
        )
    }

    /// Counts by the simple name of the symbol.
    fn by_name(facts: &FileFacts) -> HashMap<String, SymbolCounts> {
        let names: HashMap<_, _> = facts
            .symbols
            .iter()
            .map(|s| (s.id, s.name.to_string()))
            .collect();
        facts
            .counts
            .iter()
            .map(|c| (names[&c.symbol].clone(), *c))
            .collect()
    }

    fn body(counts: &HashMap<String, SymbolCounts>, name: &str) -> BodyCounts {
        counts[name]
            .body
            .unwrap_or_else(|| panic!("{name} has no body counts"))
    }

    #[test]
    fn decision_points_follow_the_counting_rules() {
        let found = by_name(&facts());
        let cc = |name: &str| 1 + body(&found, name).decisions;
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
        let found = by_name(&facts());
        // `return 1;` is one operator and one operand.
        let plain = body(&found, "PlainMethod").halstead;
        assert_eq!((plain.distinct_operators, plain.total_operators), (1, 1));
        assert_eq!((plain.distinct_operands, plain.total_operands), (1, 1));
        // A call counts as the operator `(`, a lambda as `=>`; the lambda belongs to Outer.
        let outer = body(&found, "Outer").halstead;
        assert!(outer.distinct_operators >= 3 && outer.total_operands > outer.distinct_operands);
    }

    #[test]
    fn methods_without_a_body_are_not_measured_and_initialisers_count_no_decisions() {
        let found = by_name(&facts());
        assert!(found["NoBody"].body.is_none());
        assert_eq!(found["NoBody"].loc.0, 1);
        assert!(found["Shape"].body.is_none());
    }

    #[test]
    fn the_loc_of_a_callable_leaves_out_its_local_function() {
        let found = by_name(&facts());
        // Outer: signature, two braces, the lambda line and the return; `Local` has its own line.
        assert_eq!(found["Local"].loc.0, 1);
        assert_eq!(found["Outer"].loc.0, 5);
    }

    #[test]
    fn highlights_compile_against_the_grammar() {
        let text =
            std::fs::read_to_string(languages().join("csharp/highlights.scm")).expect("query");
        Query::new(&tree_sitter_c_sharp::LANGUAGE.into(), &text).expect("highlights.scm compiles");
    }
}
