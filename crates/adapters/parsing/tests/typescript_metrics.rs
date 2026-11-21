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
    fn cognitive_complexity_counts_structures_with_their_nesting() {
        let found = by_name(&facts("typescript", "cognitive.ts"));
        let expected = [
            ("straight", 0),
            ("nested", 6),    // if (+1), for (+2), inner if (+3)
            ("chain", 3),     // if (+1), else if (+1 for its `else`), else (+1)
            ("runs", 2),      // the `&&` run (+1) and `||` (+1)
            ("handler", 3),   // catch (+1), the if inside it (+2); `try` adds nothing
            ("callbacks", 2), // the conditional in the callback: +2 (nested once by the callback)
            ("arrow", 1),     // a `const` arrow function starts at depth zero: if (+1)
            ("branches", 1),  // switch (+1); the cases add nothing
            ("loops", 9), // while (+1), do (+1), for of (+1), inner for of (+2), if (+3), `continue outer` (+1)
            ("method", 1), // the conditional (+1)
        ];
        for (name, value) in expected {
            assert_eq!(body(&found, name).cognitive, Some(value), "{name}");
        }
    }

    #[test]
    fn literals_follow_the_exceptions_of_the_rule() {
        let found = by_name(&facts("typescript", "literals.ts"));
        let literals = |name: &str| body(&found, name).literals;
        // 0 and 1 are exceptions
        assert_eq!(literals("clean"), Some(0));
        // 42 and 2.5; 0 and 1 are exceptions
        assert_eq!(literals("numbers"), Some(2));
        // "x", 'y', the template with a hole and the plain template
        assert_eq!(literals("strings"), Some(4));
        // the literal types and the default value stand outside the value positions
        assert_eq!(literals("types"), Some(0));
        // a key is no value; true, false, null and undefined are no literals
        assert_eq!(literals("flags"), Some(0));
        // "added" counts; the directive does not
        assert_eq!(literals("register"), Some(1));
        // 2 and 3 (the 1 is an exception); the static field counts for nobody
        assert_eq!(literals("method"), Some(2));
    }

    #[test]
    fn parameters_are_counted_without_the_this_type() {
        let found = by_name(&facts("typescript", "cognitive.ts"));
        let parameters = |name: &str| body(&found, name).parameters;
        assert_eq!(parameters("straight"), Some(1));
        assert_eq!(parameters("runs"), Some(4));
        // `this: Shop` is only a type, then a, b?, c = 3 and ...rest
        assert_eq!(parameters("method"), Some(4));
        // the callback `(x, i) => ...` and the function type of `cb` are not parameters of
        // the function around them
        assert_eq!(parameters("callbacks"), Some(1));
        assert_eq!(parameters("typed"), Some(1));
        // the unparenthesised parameter, and a function value in a class field
        assert_eq!(parameters("single"), Some(1));
        assert_eq!(parameters("bare"), Some(1));
        assert_eq!(parameters("field"), Some(2));
    }

    #[test]
    fn a_function_value_in_jsx_is_nested_once_by_its_callback_position() {
        // `onClick={() => { if ... }}` and `{() => { if ... }}` as a child: the `if` is +2 (nested
        // once). The conditional of `Plain` stays +1.
        let tsx = by_name(&facts("tsx", "cognitive.tsx"));
        let javascript = by_name(&facts("javascript", "jsx_cognitive.jsx"));
        for (found, name, value) in [
            (&tsx, "Form", 2),
            (&tsx, "Plain", 1),
            (&tsx, "Child", 2),
            (&javascript, "Form", 2),
            (&javascript, "Plain", 1),
        ] {
            assert_eq!(body(found, name).cognitive, Some(value), "{name}");
        }
    }

    #[test]
    fn a_function_value_behind_a_cast_is_nested_once() {
        // `(f) as T`, `(f) satisfies T` and `(f)!` as arguments: every `if` is +2 (the
        // parentheses already make it a callback). A bare `function () {} as T` has no parentheses
        // and is only known to the TSX pack (see docs/language-packs.md).
        let typescript = by_name(&facts("typescript", "cognitive.ts"));
        assert_eq!(body(&typescript, "casts").cognitive, Some(6));
        let tsx = by_name(&facts("tsx", "cognitive.tsx"));
        assert_eq!(body(&tsx, "Cast").cognitive, Some(6));
    }

    #[test]
    fn the_javascript_pack_counts_with_the_same_queries() {
        let found = by_name(&facts("javascript", "cognitive.js"));
        assert_eq!(body(&found, "nested").cognitive, Some(6));
        assert_eq!(body(&found, "callbacks").cognitive, Some(2));
        assert_eq!(body(&found, "callbacks").parameters, Some(1));
        assert_eq!(body(&found, "single").parameters, Some(1));
        assert_eq!(body(&found, "field").parameters, Some(2));
        assert!(
            body(&found, "nested").literals.is_some(),
            "the pack reports literals"
        );
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
