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
    fn cognitive_complexity_counts_structures_with_their_nesting() {
        let found = by_name(&facts("java", "Cognitive.java"));
        let cognitive = |name: &str| body(&found, name).cognitive;
        assert_eq!(cognitive("straight"), Some(0));
        // if (+1), for (+2), inner if (+3)
        assert_eq!(cognitive("nested"), Some(6));
        // if (+1), else if (+1 for its `else`), else (+1)
        assert_eq!(cognitive("chain"), Some(3));
        // the `&&` run (+1) and `||` (+1)
        assert_eq!(cognitive("runs"), Some(2));
        // catch (+1), the if inside it (+2); `try` adds nothing
        assert_eq!(cognitive("handler"), Some(3));
        // the if in the lambda: +2 (nested once by the lambda)
        assert_eq!(cognitive("callbacks"), Some(2));
        // switch (+1); the labels add nothing
        assert_eq!(cognitive("branches"), Some(1));
        // while (+1), do (+1), for-each (+1), inner for-each (+2), if (+3), `continue outer` (+1)
        assert_eq!(cognitive("loops"), Some(9));
        // the conditional (+1)
        assert_eq!(cognitive("method"), Some(1));
        // a switch expression with arrows counts once, like the statement
        assert_eq!(cognitive("arrows"), Some(1));
        // outer conditional (+1), the one in its branch (+2)
        assert_eq!(cognitive("ternaries"), Some(3));
        // if (+1), else if (+1), the if in its body at depth one (+2), else (+1)
        assert_eq!(cognitive("deep"), Some(5));
        // the conditional (+1); the if in the anonymous class counts for its method `run`
        assert_eq!(cognitive("anonymous"), Some(1));
        assert_eq!(cognitive("run"), Some(1));
        // the constructor (named like its class, so it is the entry of that name): if (+1), && (+1)
        assert_eq!(cognitive("Cognitive"), Some(2));
    }

    #[test]
    fn parameters_are_counted_with_varargs_and_without_lambda_parameters() {
        let found = by_name(&facts("java", "Cognitive.java"));
        let parameters = |name: &str| body(&found, name).parameters;
        assert_eq!(parameters("straight"), Some(1));
        assert_eq!(parameters("runs"), Some(4));
        // a, b, c and the varargs `rest`
        assert_eq!(parameters("method"), Some(4));
        // the lambda parameter `x` is not a parameter of `callbacks`
        assert_eq!(parameters("callbacks"), Some(1));
        // a constructor counts its parameters too
        assert_eq!(parameters("Cognitive"), Some(2));
    }

    #[test]
    fn highlights_compile_against_the_grammar() {
        highlights_compile("java", &tree_sitter_java::LANGUAGE.into());
    }
}
