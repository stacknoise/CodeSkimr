; TSX additions to the TypeScript metrics queries (ADR 0056), for what the shared file cannot name
; because the TypeScript grammar lacks it (JSX) or JavaScript does (casts).
; A function value in a JSX expression (`onClick={() => ...}`, or as a child) is a callback, like an
; argument: it nests what is in it once (ADR 0048).
(jsx_expression [(arrow_function) (function_expression) (generator_function)] @nesting)
; A function value behind `as`, `satisfies` or `!` where it can be a callback. With parentheses the
; shared `parenthesized_expression` pattern has it already.
(arguments (as_expression [(function_expression) (generator_function)] @nesting))
(arguments (satisfies_expression [(function_expression) (generator_function)] @nesting))
(arguments (non_null_expression [(function_expression) (generator_function)] @nesting))

; Literals (ADR 0058) in the places the shared file cannot name: a literal in a JSX expression, and
; one behind `as`, `satisfies` or `!`.
[
  (jsx_expression [(number) (string) (template_string)] @literal)
  (as_expression . [(number) (string) (template_string)] @literal)
  (satisfies_expression . [(number) (string) (template_string)] @literal)
  (non_null_expression [(number) (string) (template_string)] @literal)]
