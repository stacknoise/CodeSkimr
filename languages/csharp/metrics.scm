; C# metrics captures (ADR 0015): comments, decision points, Halstead operators and operands.
; Mapping of the counting rules to nodes (golden test: crates/adapters/parsing/tests/csharp_metrics.rs):
;   if / else if            if_statement (an `else if` is an if_statement inside the else)
;   conditional expression  conditional_expression
;   loops                   for_statement, foreach_statement, while_statement, do_statement
;   case                    switch_section that starts with `case` and has a body (not `default`,
;                           not a label that falls through to the next one)
;   match arm               switch_expression_arm without the discard pattern `_`
;   catch                   catch_clause (a `when` filter belongs to it and adds nothing)
;   short circuit           `&&`, `||`, `??` and `??=`
; A call counts as the operator `(` and an index expression as `[`, so that distinctness by text
; works. Identifiers count as operands only in value positions, never as type names.

(comment) @comment

; Decision points
(if_statement "if" @decision)
(conditional_expression "?" @decision)
(for_statement "for" @decision)
(foreach_statement "foreach" @decision)
(while_statement "while" @decision)
(do_statement "do" @decision)
; a label-only section falls through into the next one and shares its arm
(switch_section . "case" . (_) . (_) @decision)
(switch_expression_arm . (_) @decision (#not-eq? @decision "_"))
(catch_clause "catch" @decision)
(binary_expression operator: ["&&" "||" "??"] @decision)
(assignment_expression operator: "??=" @decision)

; Operators: tokens that compute, compare, assign, access, call and control the flow
(binary_expression operator: _ @operator)
(assignment_expression operator: _ @operator)
(prefix_unary_expression . _ @operator)
(postfix_unary_expression _ @operator .)
(member_access_expression "." @operator)
(conditional_access_expression "?" @operator)
(conditional_expression ["?" ":"] @operator)
(lambda_expression "=>" @operator)
(invocation_expression arguments: (argument_list "(" @operator))
(element_access_expression subscript: (bracketed_argument_list "[" @operator))
(object_creation_expression "new" @operator)
(if_statement "if" @operator)
(for_statement "for" @operator)
(foreach_statement "foreach" @operator)
(while_statement "while" @operator)
(do_statement "do" @operator)
(switch_statement "switch" @operator)
(switch_expression "switch" @operator)
(switch_section "case" @operator)
(return_statement "return" @operator)
(throw_statement "throw" @operator)
(try_statement "try" @operator)
(catch_clause "catch" @operator)
(break_statement "break" @operator)
(continue_statement "continue" @operator)
(await_expression "await" @operator)
(using_statement "using" @operator)
(lock_statement "lock" @operator)

; Operands: literals and names in value positions
[(integer_literal) (real_literal) (string_literal) (character_literal) (boolean_literal)
 (null_literal)] @operand
(variable_declarator name: (identifier) @operand)
(invocation_expression function: (identifier) @operand)
(member_access_expression name: (identifier) @operand)
(member_access_expression expression: (identifier) @operand)
([(argument) (return_statement) (arrow_expression_clause) (interpolation)
  (parenthesized_expression) (await_expression) (throw_expression)] (identifier) @operand)
(variable_declarator name: (_) . (identifier) @operand)
(binary_expression [left: (identifier) @operand right: (identifier) @operand])
(assignment_expression [left: (identifier) @operand right: (identifier) @operand])
(conditional_expression
  [condition: (identifier) @operand consequence: (identifier) @operand
   alternative: (identifier) @operand])
(prefix_unary_expression (identifier) @operand)
(postfix_unary_expression (identifier) @operand)
