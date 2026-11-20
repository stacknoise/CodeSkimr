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
    fn cognitive_complexity_counts_structures_with_their_nesting() {
        let found = by_name(&facts("python", "cognitive.py"));
        let cognitive = |name: &str| body(&found, name).cognitive;
        assert_eq!(cognitive("straight"), Some(0));
        // if (+1), for (+2), inner if (+3)
        assert_eq!(cognitive("nested"), Some(6));
        // if (+1), elif (+1), else (+1)
        assert_eq!(cognitive("chain"), Some(3));
        // `and` run (+1) and `or` (+1)
        assert_eq!(cognitive("runs"), Some(2));
        // except (+1), the if inside it (+2)
        assert_eq!(cognitive("handler"), Some(3));
        // the conditional expression in the lambda: +2 (nested once)
        assert_eq!(cognitive("lambdas"), Some(2));
        // match (+1); the cases add nothing
        assert_eq!(cognitive("method"), Some(1));
        assert_eq!(cognitive("looped"), Some(1));
    }

    #[test]
    fn literals_follow_the_exceptions_of_the_rule() {
        let found = by_name(&facts("python", "literals.py"));
        let literals = |name: &str| body(&found, name).literals;
        // the docstring, 0 and 1 are exceptions
        assert_eq!(literals("clean"), Some(0));
        // 42, 2.5; 0 and 1 are exceptions
        assert_eq!(literals("numbers"), Some(2));
        // "x", 'y' and the f-string
        assert_eq!(literals("strings"), Some(3));
        // the default 7; the two strings in annotations are exceptions
        assert_eq!(literals("annotated"), Some(1));
        // every part of a concatenation counts; so does a string inside an interpolation
        assert_eq!(literals("joined"), Some(2));
        assert_eq!(literals("interpolated"), Some(2));
        assert_eq!(literals("flags"), Some(0));
        // 1, 2 and 3: the 1 is an exception, a class field outside a function counts for nobody
        assert_eq!(literals("method"), Some(2));
    }

    #[test]
    fn parameters_are_counted_without_the_receiver() {
        let found = by_name(&facts("python", "cognitive.py"));
        let parameters = |name: &str| body(&found, name).parameters;
        assert_eq!(parameters("straight"), Some(1));
        assert_eq!(parameters("runs"), Some(4));
        // a, b: int, c=3, *rest, **named; `self` does not count
        assert_eq!(parameters("method"), Some(5));
        assert_eq!(parameters("make"), Some(1));
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
