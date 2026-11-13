; TypeScript and JavaScript metrics captures (ADR 0015, ADR 0022): comments, decision points,
; Halstead operators and operands. The file is the same for both grammars; the golden test of each
; pack runs it.
; Mapping of the counting rules to nodes:
;   if / else if            if_statement (an `else if` is an if_statement inside the else)
;   conditional expression  ternary_expression
;   loops                   for_statement, for_in_statement (also `for ... of`), while_statement,
;                           do_statement
;   case                    switch_case that has a value and a statement (a label that falls
;                           through to the next one does not count, `default` never does)
;   catch                   catch_clause
;   short circuit           `&&`, `||`, `??` and the compound `&&=`, `||=`, `??=`
; Not counted (ADR 0015): optional chaining `?.`, `try`, `finally`, `default`.
; A call counts as the operator `(` and an index expression as `[`. Identifiers count as operands
; only in value positions, never as type names.

(comment) @comment

; Decision points
(if_statement "if" @decision)
(ternary_expression "?" @decision)
(for_statement "for" @decision)
(for_in_statement "for" @decision)
(while_statement "while" @decision)
(do_statement "do" @decision)
(switch_case . "case" . (_) . (_) @decision)
(catch_clause "catch" @decision)
(binary_expression operator: ["&&" "||" "??"] @decision)
(augmented_assignment_expression operator: ["&&=" "||=" "??="] @decision)

; Operators: tokens that compute, compare, assign, access, call and control the flow
(binary_expression operator: _ @operator)
(assignment_expression "=" @operator)
(augmented_assignment_expression operator: _ @operator)
(unary_expression operator: _ @operator)
(update_expression ["++" "--"] @operator)
(member_expression "." @operator)
(member_expression (optional_chain) @operator)
(ternary_expression ["?" ":"] @operator)
(arrow_function "=>" @operator)
(call_expression arguments: (arguments "(" @operator))
(subscript_expression "[" @operator)
(new_expression "new" @operator)
(if_statement "if" @operator)
(for_statement "for" @operator)
(for_in_statement "for" @operator)
(while_statement "while" @operator)
(do_statement "do" @operator)
(switch_statement "switch" @operator)
(switch_case "case" @operator)
(return_statement "return" @operator)
(throw_statement "throw" @operator)
(try_statement "try" @operator)
(catch_clause "catch" @operator)
(break_statement "break" @operator)
(continue_statement "continue" @operator)
(await_expression "await" @operator)
(yield_expression "yield" @operator)

; Operands: literals and names in value positions
[(number) (string) (template_string) (regex) (true) (false) (null) (undefined)] @operand
(variable_declarator name: (identifier) @operand)
(call_expression function: (identifier) @operand)
(member_expression object: (identifier) @operand)
(member_expression property: (property_identifier) @operand)
([(arguments) (return_statement) (throw_statement) (parenthesized_expression)
  (await_expression) (yield_expression) (template_substitution) (array)
  (expression_statement) (unary_expression) (update_expression) (spread_element)
  (subscript_expression)] (identifier) @operand)
(variable_declarator name: (_) value: (identifier) @operand)
(pair value: (identifier) @operand)
(arrow_function body: (identifier) @operand)
(binary_expression [left: (identifier) @operand right: (identifier) @operand])
(assignment_expression [left: (identifier) @operand right: (identifier) @operand])
(augmented_assignment_expression [left: (identifier) @operand right: (identifier) @operand])
(ternary_expression
  [condition: (identifier) @operand consequence: (identifier) @operand
   alternative: (identifier) @operand])
