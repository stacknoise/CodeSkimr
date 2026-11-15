; Java metrics captures (ADR 0015, ADR 0025): comments, decision points, Halstead operators and
; operands.
; Mapping of the counting rules to nodes (golden test: crates/adapters/parsing/tests/java_metrics.rs):
;   if / else if            if_statement (an `else if` is an if_statement inside the else)
;   conditional expression  ternary_expression
;   loops                   for_statement, enhanced_for_statement, while_statement, do_statement
;   case                    switch_block_statement_group that has statements and starts with a
;                           `case` label (a label without statements falls through into the next
;                           group and shares its arm), and switch_rule whose label starts with
;                           `case` (not `default`); the guard `when` adds nothing
;   catch                   catch_clause (also in try-with-resources; `try`, `finally` do not)
;   short circuit           `&&`, `||`
; Not counted (ADR 0015): `default`, `assert`, `synchronized`, `instanceof` patterns, `try`,
; `finally`. A call counts as the operator `(` and an array access as `[`. Identifiers count as
; operands only in value positions, never as type names, annotations or `this`.

[(line_comment) (block_comment)] @comment

; Decision points
(if_statement "if" @decision)
(ternary_expression "?" @decision)
(for_statement "for" @decision)
(enhanced_for_statement "for" @decision)
(while_statement "while" @decision)
(do_statement "do" @decision)
; a label-only group falls through into the next one and shares its arm
(switch_block_statement_group . (switch_label . "case") @decision (statement))
(switch_rule . (switch_label . "case") @decision)
(catch_clause "catch" @decision)
(binary_expression operator: ["&&" "||"] @decision)

; Operators: tokens that compute, compare, assign, access, call and control the flow
(binary_expression operator: _ @operator)
(assignment_expression operator: _ @operator)
(unary_expression operator: _ @operator)
(update_expression ["++" "--"] @operator)
(field_access "." @operator)
(method_invocation "." @operator)
(method_invocation arguments: (argument_list "(" @operator))
(array_access "[" @operator)
(ternary_expression ["?" ":"] @operator)
(lambda_expression "->" @operator)
(object_creation_expression "new" @operator)
(array_creation_expression "new" @operator)
(instanceof_expression "instanceof" @operator)
(cast_expression "(" @operator)
(if_statement "if" @operator)
(for_statement "for" @operator)
(enhanced_for_statement "for" @operator)
(while_statement "while" @operator)
(do_statement "do" @operator)
(switch_expression "switch" @operator)
(switch_label "case" @operator)
(return_statement "return" @operator)
(throw_statement "throw" @operator)
(try_statement "try" @operator)
(try_with_resources_statement "try" @operator)
(catch_clause "catch" @operator)
(break_statement "break" @operator)
(continue_statement "continue" @operator)
(synchronized_statement "synchronized" @operator)
(assert_statement "assert" @operator)

; Operands: literals and names in value positions
[(decimal_integer_literal) (hex_integer_literal) (octal_integer_literal) (binary_integer_literal)
 (decimal_floating_point_literal) (hex_floating_point_literal) (string_literal)
 (character_literal) (true) (false) (null_literal)] @operand
(variable_declarator name: (identifier) @operand)
(variable_declarator value: (identifier) @operand)
(method_invocation name: (identifier) @operand)
(method_invocation object: (identifier) @operand)
(field_access [object: (identifier) @operand field: (identifier) @operand])
(argument_list (identifier) @operand)
(return_statement (identifier) @operand)
(throw_statement (identifier) @operand)
(parenthesized_expression (identifier) @operand)
(binary_expression [left: (identifier) @operand right: (identifier) @operand])
(assignment_expression [left: (identifier) @operand right: (identifier) @operand])
(unary_expression operand: (identifier) @operand)
(update_expression (identifier) @operand)
(ternary_expression
  [condition: (identifier) @operand consequence: (identifier) @operand
   alternative: (identifier) @operand])
(array_access [array: (identifier) @operand index: (identifier) @operand])
(cast_expression value: (identifier) @operand)
(instanceof_expression left: (identifier) @operand)
(enhanced_for_statement [name: (identifier) @operand value: (identifier) @operand])
(lambda_expression body: (identifier) @operand)
(array_initializer (identifier) @operand)

; Cognitive complexity (ADR 0048). A structure adds one and one more for every nesting level it
; stands in; `else` and a run of the same boolean operator add one at any depth. `else if` is an
; `if_statement` in the `alternative` of another: only its `else` counts, and its body is no
; deeper than the first `if`. A lambda nests what is in it but adds nothing. A labelled `break` or
; `continue` adds one at any depth. Not counted: `try`, `finally`, `case` labels (the `switch`
; counts once), `synchronized`, `assert`, recursion.
(if_statement) @cognitive
(if_statement) @nesting
(if_statement alternative: (if_statement) @continuation)
(if_statement "else" @cognitive_flat)
(ternary_expression) @cognitive
(ternary_expression) @nesting
(for_statement) @cognitive
(for_statement) @nesting
(enhanced_for_statement) @cognitive
(enhanced_for_statement) @nesting
(while_statement) @cognitive
(while_statement) @nesting
(do_statement) @cognitive
(do_statement) @nesting
(switch_expression) @cognitive
(switch_expression) @nesting
(catch_clause) @cognitive
(catch_clause) @nesting
(break_statement (identifier)) @cognitive_flat
(continue_statement (identifier)) @cognitive_flat
(binary_expression operator: ["&&" "||"] @cognitive_sequence)
(lambda_expression) @nesting

; Parameters of a method or constructor, without the `this` receiver; a lambda has its own list
(method_declaration parameters: (formal_parameters [(formal_parameter) (spread_parameter)] @parameter))
(constructor_declaration parameters: (formal_parameters [(formal_parameter) (spread_parameter)] @parameter))
