//! Golden test of `languages/python/metrics.scm`: one case per counting rule of ADR 0015, run
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
        let found = by_name(&facts("python", "metrics.py"));
        let cc = |name: &str| 1 + body(&found, name).decisions;
        assert_eq!(cc("plain"), 1);
        // if, and, elif, or; the plain `else` does not count
        assert_eq!(cc("branches"), 5);
        assert_eq!(cc("conditional"), 2); // `x if c else y`
        // for, while (its `else` does not count), comprehension `for` and `if`
        assert_eq!(cc("loops"), 5);
        assert_eq!(cc("cases"), 3); // two cases, the wildcard `case _` does not count
        // two `except` clauses; `try`, `else` and `finally` do not count
        assert_eq!(cc("handlers"), 3);
        assert_eq!(cc("outer"), 2); // the conditional in the lambda counts for outer
        assert_eq!(cc("local"), 2); // the nested function counts for itself only
    }

    #[test]
    fn halstead_counts_of_a_small_body() {
        let found = by_name(&facts("python", "metrics.py"));
        // `return 1` is one operator and one operand.
        let plain = body(&found, "plain").halstead;
        assert_eq!((plain.distinct_operators, plain.total_operators), (1, 1));
        assert_eq!((plain.distinct_operands, plain.total_operands), (1, 1));
        // A call counts as the operator `(`, a subscript as `[`.
        let calls = body(&found, "calls").halstead;
        assert!(calls.distinct_operators >= 5 && calls.total_operands > calls.distinct_operands);
    }

    #[test]
    fn the_loc_of_a_callable_leaves_out_its_nested_function() {
        let found = by_name(&facts("python", "metrics.py"));
        assert_eq!(found["local"].loc.0, 4);
        assert_eq!(found["outer"].loc.0, 3);
    }

    #[test]
    fn highlights_compile_against_the_grammar() {
        highlights_compile("python", &tree_sitter_python::LANGUAGE.into());
    }
}
