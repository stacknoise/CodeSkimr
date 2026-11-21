//! Golden test of `languages/csharp/metrics.scm`: one case per counting rule of ADR 0015, run
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
        let found = by_name(&facts("csharp", "metrics.cs"));
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
    fn cognitive_complexity_counts_structures_with_their_nesting() {
        let found = by_name(&facts("csharp", "cognitive.cs"));
        let expected = [
            ("Straight", 0),
            ("Nested", 6),  // if (+1), for (+2), inner if (+3)
            ("Chain", 3),   // if (+1), else if (+1 for its `else`), else (+1)
            ("Runs", 2),    // the `&&` run (+1) and `||` (+1)
            ("Handler", 3), // catch (+1), the if inside it (+2); `try` adds nothing
            ("Cases", 1), // switch (+1); the cases add nothing, nor do the arms of a switch expression
            ("Arms", 1),
            ("Lambdas", 2), // the conditional in the lambda: +2 (nested once by the lambda)
            ("Inner", 1), // a local function counts for itself: if (+1); the outer method has nothing
            ("Local", 0),
            ("Loops", 8), // while (+1), foreach (+1), inner foreach (+2), if (+3), goto (+1)
        ];
        for (name, value) in expected {
            assert_eq!(body(&found, name).cognitive, Some(value), "{name}");
        }
    }

    #[test]
    fn parameters_are_counted_with_the_params_array() {
        let found = by_name(&facts("csharp", "cognitive.cs"));
        let parameters = |name: &str| body(&found, name).parameters;
        assert_eq!(parameters("Straight"), Some(1));
        assert_eq!(parameters("Runs"), Some(4));
        assert_eq!(parameters("Defaults"), Some(2));
        assert_eq!(parameters("Variadic"), Some(2));
    }

    #[test]
    fn literals_follow_the_exceptions_of_the_rule() {
        let found = by_name(&facts("csharp", "literals.cs"));
        let literals = |name: &str| body(&found, name).literals;
        // 0 and 1 are exceptions
        assert_eq!(literals("Clean"), Some(0));
        // 42 and 2.5; 0 and 1 are exceptions
        assert_eq!(literals("Numbers"), Some(2));
        // "x", 'y', the interpolated string and @"raw"
        assert_eq!(literals("Strings"), Some(4));
        // the value of a local const is an exception, the 7 is not
        assert_eq!(literals("Local"), Some(1));
        // the attribute argument is an exception; true, false and null are no literals
        assert_eq!(literals("Flags"), Some(0));
    }

    #[test]
    fn halstead_counts_of_a_small_body() {
        let found = by_name(&facts("csharp", "metrics.cs"));
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
        let found = by_name(&facts("csharp", "metrics.cs"));
        assert!(found["NoBody"].body.is_none());
        assert!(found["Shape"].counted);
        assert!(found["Shape"].body.is_none());
    }

    #[test]
    fn the_loc_of_a_callable_leaves_out_its_local_function() {
        let found = by_name(&facts("csharp", "metrics.cs"));
        // Outer: signature, two braces, the lambda line and the return; `Local` has its own line.
        assert_eq!(found["Local"].loc.0, 1);
        assert_eq!(found["Outer"].loc.0, 5);
    }

    #[test]
    fn highlights_compile_against_the_grammar() {
        highlights_compile("csharp", &tree_sitter_c_sharp::LANGUAGE.into());
    }
}
