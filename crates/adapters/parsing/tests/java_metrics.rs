//! Golden test of `languages/java/metrics.scm`: one case per counting rule of ADR 0015, run
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
        let found = by_name(&facts("java", "Metrics.java"));
        let cc = |name: &str| 1 + body(&found, name).decisions;
        assert_eq!(cc("plain"), 1);
        // if, &&, else if, ||, ?:
        assert_eq!(cc("branches"), 6);
        assert_eq!(cc("loops"), 5); // for, for-each, while, do
        assert_eq!(cc("cases"), 2); // case 1 and case 2 share one group; default does not count
        assert_eq!(cc("rules"), 3); // two rules; `default ->` does not count
        assert_eq!(cc("catches"), 3); // two catches; try and finally do not count
        assert_eq!(cc("resources"), 2); // try-with-resources counts its catch only
        assert_eq!(cc("outer"), 2); // the lambda's ?: counts for outer
        assert_eq!(cc("inner"), 2); // a method of an anonymous class counts for itself only
    }

    #[test]
    fn halstead_counts_of_a_small_body() {
        let found = by_name(&facts("java", "Metrics.java"));
        // `return 1;` is one operator and one operand.
        let plain = body(&found, "plain").halstead;
        assert_eq!((plain.distinct_operators, plain.total_operators), (1, 1));
        assert_eq!((plain.distinct_operands, plain.total_operands), (1, 1));
        // A call counts as the operator `(`, a lambda as `->`; the lambda belongs to outer.
        let outer = body(&found, "outer").halstead;
        assert!(outer.distinct_operators >= 3 && outer.total_operands > outer.distinct_operands);
    }

    #[test]
    fn methods_without_a_body_are_not_measured() {
        let found = by_name(&facts("java", "Metrics.java"));
        assert!(found["noBody"].body.is_none());
        assert!(found["area"].body.is_none());
    }

    #[test]
    fn the_loc_of_a_method_leaves_out_its_anonymous_class_method() {
        let found = by_name(&facts("java", "Metrics.java"));
        assert_eq!(found["inner"].loc.0, 1);
        assert_eq!(found["outer"].loc.0, 6);
    }

    #[test]
    fn highlights_compile_against_the_grammar() {
        highlights_compile("java", &tree_sitter_java::LANGUAGE.into());
    }
}
