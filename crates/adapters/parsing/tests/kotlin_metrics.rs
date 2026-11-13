//! Golden test of `languages/kotlin/metrics.scm`: one case per counting rule of ADR 0015, run
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
        let found = by_name(&facts("kotlin", "Metrics.kt"));
        let cc = |name: &str| 1 + body(&found, name).decisions;
        assert_eq!(cc("plain"), 1);
        // if, &&, else if, ||, ?:, if-expression
        assert_eq!(cc("branches"), 7);
        assert_eq!(cc("loops"), 4); // for, while, do
        assert_eq!(cc("cases"), 3); // `1, 2 ->` counts once, `in 3..4 ->`; `else` does not
        assert_eq!(cc("guards"), 2); // `k > 1 ->`; `else` does not
        assert_eq!(cc("catches"), 3); // two catches; try and finally do not
        assert_eq!(cc("outer"), 2); // the lambda's `if` counts for outer
        assert_eq!(cc("inner"), 1); // a local function counts for itself only
        assert_eq!(cc("local"), 2);
    }

    #[test]
    fn halstead_counts_of_a_small_body() {
        let found = by_name(&facts("kotlin", "Metrics.kt"));
        let counts = |name: &str| {
            let h = body(&found, name).halstead;
            [
                h.distinct_operators,
                h.total_operators,
                h.distinct_operands,
                h.total_operands,
            ]
        };
        // `= 1` is one operand and no operator.
        assert_eq!(counts("plain"), [0, 0, 1, 1]);
        // A call counts as the operator `(`, a lambda as `->`; the lambda belongs to outer.
        let outer = counts("outer");
        assert!(outer[0] >= 3 && outer[3] > outer[2]);
    }

    #[test]
    fn callables_without_a_body_are_not_measured() {
        let found = by_name(&facts("kotlin", "Metrics.kt"));
        assert!(found["noBody"].body.is_none());
        assert!(found["area"].body.is_none());
    }

    #[test]
    fn a_secondary_constructor_is_measured() {
        let found = by_name(&facts("kotlin", "Metrics.kt"));
        assert!(found["M"].body.is_some());
    }

    #[test]
    fn the_loc_of_a_function_leaves_out_its_local_function() {
        let found = by_name(&facts("kotlin", "Metrics.kt"));
        assert_eq!(found["inner"].loc.0, 3);
        assert_eq!(found["local"].loc.0, 1);
    }

    #[test]
    fn highlights_compile_against_the_grammar() {
        highlights_compile("kotlin", &tree_sitter_kotlin_sg::LANGUAGE.into());
    }
}
