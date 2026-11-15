//! Golden test of `languages/typescript/metrics.scm` (the TSX and JavaScript packs take the file over,
//! ADR 0056): one case per counting rule of ADR 0015, and the highlights of the three packs.

#[cfg(test)]
mod common;
#[cfg(test)]
#[path = "common/metrics.rs"]
mod support;

#[cfg(test)]
mod tests {
    use super::support::{body, by_name, facts, highlights_compile};

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
}
