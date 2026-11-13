; Rust metrics captures (ADR 0015, ADR 0026): comments, decision points, Halstead operators and
; operands.
; Mapping of the counting rules to nodes (golden test: crates/adapters/parsing/tests/rust_metrics.rs):
;   if / else if            if_expression (an `else if` is an if_expression in the else clause);
;                           `if let` is an if_expression with a let condition
;   loops                   for_expression, while_expression (`while let` too)
;   match arm               match_arm, but not the wildcard arm `_` (also with a guard)
;   short circuit           `&&`, `||` (also inside a guard or a let chain)
; Not counted (ADR 0015): `else`, `loop`, the `?` operator, `let ... else`, `unsafe`, `async`,
; closures (they count for their function), a `match` itself.
; A call counts as the operator `(` and an index expression as `[`. Identifiers count as operands
; only in value positions, never as type names; `self` is no operand.

[(line_comment) (block_comment)] @comment

; Decision points
(if_expression "if" @decision)
(for_expression "for" @decision)
(while_expression "while" @decision)
(match_arm pattern: (match_pattern) @decision (#not-match? @decision "^_(\\s|$)"))
(binary_expression operator: ["&&" "||"] @decision)

; Operators: tokens that compute, compare, assign, access, call and control the flow
(binary_expression operator: _ @operator)
(unary_expression ["-" "!" "*"] @operator)
(reference_expression "&" @operator)
(assignment_expression "=" @operator)
(compound_assignment_expr operator: _ @operator)
(type_cast_expression "as" @operator)
(range_expression [".." "..="] @operator)
(field_expression "." @operator)
(try_expression "?" @operator)
(call_expression arguments: (arguments "(" @operator))
(index_expression "[" @operator)
(macro_invocation "!" @operator)
(let_declaration "let" @operator)
(let_condition "let" @operator)
(if_expression "if" @operator)
(for_expression "for" @operator)
(while_expression "while" @operator)
(loop_expression "loop" @operator)
(match_expression "match" @operator)
(match_arm "=>" @operator)
(return_expression "return" @operator)
(break_expression "break" @operator)
(continue_expression "continue" @operator)
(await_expression "await" @operator)
(closure_parameters "|" @operator)

; Operands: literals and names in value positions
[(integer_literal) (float_literal) (string_literal) (raw_string_literal) (char_literal)
 (boolean_literal)] @operand
(let_declaration pattern: (identifier) @operand)
(let_declaration value: (identifier) @operand)
(call_expression function: (identifier) @operand)
(call_expression function: (scoped_identifier name: (identifier) @operand))
(field_expression value: (identifier) @operand)
(field_expression field: (field_identifier) @operand)
(field_initializer field: (field_identifier) @operand)
(shorthand_field_initializer (identifier) @operand)
([(arguments) (return_expression) (break_expression) (parenthesized_expression)
  (await_expression) (try_expression) (array_expression) (tuple_expression)
  (expression_statement) (unary_expression) (reference_expression) (index_expression)
  (field_initializer) (range_expression) (type_cast_expression)] (identifier) @operand)
(binary_expression [left: (identifier) @operand right: (identifier) @operand])
(assignment_expression [left: (identifier) @operand right: (identifier) @operand])
(compound_assignment_expr [left: (identifier) @operand right: (identifier) @operand])
(for_expression [pattern: (identifier) @operand value: (identifier) @operand])
(match_expression value: (identifier) @operand)
(match_arm value: (identifier) @operand)
(if_expression condition: (identifier) @operand)
(while_expression condition: (identifier) @operand)
(let_condition [pattern: (identifier) @operand value: (identifier) @operand])
(closure_expression body: (identifier) @operand)
