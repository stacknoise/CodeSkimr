//! Golden test of `languages/rust/metrics.scm`: one case per counting rule of ADR 0015, run
//! through the analyzer, which assigns every capture to the innermost callable.

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
        let found = by_name(&facts("rust", "metrics.rs"));
        let cc = |name: &str| 1 + body(&found, name).decisions;
        assert_eq!(cc("plain"), 1);
        // if, `&&`, else if, `||`; the plain `else` does not count
        assert_eq!(cc("branches"), 5);
        assert_eq!(cc("patterns"), 3); // `if let` and `while let`
        assert_eq!(cc("loops"), 3); // for and while; `loop` does not count
        assert_eq!(cc("arms"), 3); // two arms, the wildcard arm does not count
        assert_eq!(cc("wildcard_guard"), 2); // the `&&` of the guard counts, the arm does not
        // `?`, `let ... else` and `unsafe` do not count
        assert_eq!(cc("not_counted"), 1);
        assert_eq!(cc("outer"), 2); // the closure counts for outer
        assert_eq!(cc("local"), 2); // the nested function counts for itself only
    }

    #[test]
    fn halstead_counts_of_a_small_body() {
        let found = by_name(&facts("rust", "metrics.rs"));
        // `1` is one operand and no operator.
        let plain = body(&found, "plain").halstead;
        assert_eq!((plain.distinct_operators, plain.total_operators), (0, 0));
        assert_eq!((plain.distinct_operands, plain.total_operands), (1, 1));
        // A call counts as the operator `(`, an index expression as `[`.
        let calls = body(&found, "calls").halstead;
        assert!(calls.distinct_operators >= 5 && calls.total_operands > calls.distinct_operands);
    }

    #[test]
    fn the_loc_of_a_callable_leaves_out_its_nested_function() {
        let found = by_name(&facts("rust", "metrics.rs"));
        assert_eq!(found["local"].loc.0, 3);
        assert_eq!(found["outer"].loc.0, 3);
    }

    #[test]
    fn highlights_compile_against_the_grammar() {
        highlights_compile("rust", &tree_sitter_rust::LANGUAGE.into());
    }
}
