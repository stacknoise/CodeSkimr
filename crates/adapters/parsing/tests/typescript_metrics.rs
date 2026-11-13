//! Golden test of `languages/typescript/metrics.scm` (the file is the same for the TSX and
//! JavaScript packs): one case per counting rule of ADR 0015, and the highlights of the three packs.

#[cfg(test)]
mod common;
#[cfg(test)]
#[path = "common/metrics.rs"]
mod support;

#[cfg(test)]
mod tests {
    use super::support::{body, by_name, facts, highlights_compile, languages};

    #[test]
    fn decision_points_follow_the_counting_rules() {
        let found = by_name(&facts("typescript", "metrics.ts"));
        let cc = |name: &str| 1 + body(&found, name).decisions;
        // if, else if, ?:, for, for of, for in, while, do, two cases (the first label falls
        // through, `default` does not count), catch, `&&`, `||`, `??`, `??=`, `||=`, `&&=`;
        // `?.`, `try` and `finally` do not count, the lambda is its own function.
        assert_eq!(cc("rules"), 18);
        assert_eq!(cc("lambda"), 2);
        assert_eq!(cc("arrow"), 1);
        assert_eq!(cc("nested"), 2);
        assert_eq!(cc("tick"), 2);
    }

    #[test]
    fn the_loc_of_a_callable_leaves_out_its_local_function() {
        let found = by_name(&facts("typescript", "metrics.ts"));
        assert_eq!(found["arrow"].loc.0, 3);
        assert_eq!(found["nested"].loc.0, 1);
    }

    #[test]
    fn halstead_counts_of_a_small_body() {
        let found = by_name(&facts("typescript", "metrics.ts"));
        // `x || 1`: the operator `||`, the operands x and 1.
        let nested = body(&found, "nested").halstead;
        assert_eq!((nested.distinct_operators, nested.total_operators), (2, 2));
        assert_eq!((nested.distinct_operands, nested.total_operands), (2, 2));
    }

    #[test]
    fn highlights_compile_against_their_grammars() {
        highlights_compile(
            "typescript",
            &tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
        );
        highlights_compile("tsx", &tree_sitter_typescript::LANGUAGE_TSX.into());
        highlights_compile("javascript", &tree_sitter_javascript::LANGUAGE.into());
    }

    #[test]
    fn the_tsx_pack_shares_the_typescript_queries() {
        let read = |pack: &str, file: &str| {
            std::fs::read_to_string(languages().join(pack).join(file)).expect("file")
        };
        // The TSX pack adds the JSX values to the values of a constant.
        let jsx =
            read("tsx", "symbols.scm").replace(" (jsx_element) (jsx_self_closing_element)", "");
        assert_eq!(jsx, read("typescript", "symbols.scm"));
        assert_eq!(
            read("tsx", "metrics.scm"),
            read("typescript", "metrics.scm")
        );
        assert_eq!(
            read("javascript", "metrics.scm"),
            read("typescript", "metrics.scm")
        );
        assert!(read("tsx", "references.scm").starts_with(&read("typescript", "references.scm")));
    }
}
