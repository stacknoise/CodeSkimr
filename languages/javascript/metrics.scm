; JavaScript additions to the TypeScript metrics queries (ADR 0056): JSX is a grammar of its own,
; so the shared file cannot name it. A function value in a JSX expression (`onClick={() => ...}`,
; or as a child) is a callback, like an argument: it nests what is in it once (ADR 0048).
(jsx_expression [(arrow_function) (function_expression) (generator_function)] @nesting)
