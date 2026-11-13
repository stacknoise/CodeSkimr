; Kotlin metrics captures (ADR 0015, ADR 0027): comments, decision points, Halstead operators and
; operands.
; Mapping of the counting rules to nodes (golden test: crates/adapters/parsing/tests/kotlin_metrics.rs):
;   if / else if            if_expression (an `else if` is an if_expression inside the else); the
;                           `if` expression takes the place of the conditional expression
;   loops                   for_statement, while_statement, do_while_statement
;   case                    when_entry that has a condition (not `else`); several conditions of one
;                           entry (`1, 2 ->`) count once
;   catch                   catch_block (`try` and `finally` do not)
;   short circuit           `&&`, `||` and the Elvis operator `?:`
; Not counted (ADR 0015): `else`, `try`, `finally`, `?.`, `is` and `as`, `!!`, a lambda.
; A call counts as the operator `(` and an index expression as `[`. Identifiers count as operands
; in value positions only, never as type names or declared names of types and functions.
; Alternatives are written inside their parent node (issues #77 and #81).

[(line_comment) (multiline_comment)] @comment

; Decision points
(if_expression "if" @decision)
(for_statement "for" @decision)
(while_statement "while" @decision)
(do_while_statement "do" @decision)
(when_entry . (when_condition) @decision)
(catch_block "catch" @decision)
(conjunction_expression "&&" @decision)
(disjunction_expression "||" @decision)
(elvis_expression "?:" @decision)

; Operators: tokens that compute, compare, assign, access, call and control the flow
(comparison_expression ["<" ">" "<=" ">="] @operator)
(equality_expression ["==" "!=" "===" "!=="] @operator)
(conjunction_expression "&&" @operator)
(disjunction_expression "||" @operator)
(elvis_expression "?:" @operator)
(additive_expression ["+" "-"] @operator)
(multiplicative_expression ["*" "/" "%"] @operator)
(range_expression [".." "..<"] @operator)
(infix_expression (simple_identifier) @operator .)
(prefix_expression ["-" "+" "!" "++" "--"] @operator)
(postfix_expression ["++" "--" "!!"] @operator)
(assignment ["=" "+=" "-=" "*=" "/=" "%="] @operator)
(navigation_suffix ["." "?."] @operator)
(call_suffix (value_arguments "(" @operator))
(indexing_suffix "[" @operator)
(lambda_literal "->" @operator)
(check_expression "is" @operator)
(as_expression ["as" "as?"] @operator)
(range_test "in" @operator)
(if_expression "if" @operator)
(for_statement "for" @operator)
(while_statement "while" @operator)
(do_while_statement "do" @operator)
(when_expression "when" @operator)
(when_entry "->" @operator)
(jump_expression ["return" "throw" "break" "continue"] @operator)
(try_expression "try" @operator)
(catch_block "catch" @operator)

; Operands: literals and names in value positions
[(integer_literal) (long_literal) (hex_literal) (bin_literal) (real_literal) (string_literal)
 (character_literal) (boolean_literal) (null_literal)] @operand
[(variable_declaration (simple_identifier) @operand)
 (property_declaration (simple_identifier) @operand)
 (value_argument (simple_identifier) @operand)
 (jump_expression (simple_identifier) @operand)
 (parenthesized_expression (simple_identifier) @operand)
 (comparison_expression (simple_identifier) @operand)
 (equality_expression (simple_identifier) @operand)
 (conjunction_expression (simple_identifier) @operand)
 (disjunction_expression (simple_identifier) @operand)
 (elvis_expression (simple_identifier) @operand)
 (additive_expression (simple_identifier) @operand)
 (multiplicative_expression (simple_identifier) @operand)
 (range_expression (simple_identifier) @operand)
 (prefix_expression (simple_identifier) @operand)
 (postfix_expression (simple_identifier) @operand)
 (assignment (simple_identifier) @operand)
 (navigation_expression (simple_identifier) @operand)
 (navigation_suffix (simple_identifier) @operand)
 (call_expression (simple_identifier) @operand)
 (indexing_suffix (simple_identifier) @operand)
 (control_structure_body (simple_identifier) @operand)
 (statements (simple_identifier) @operand)
 (function_body (simple_identifier) @operand)
 (when_subject (simple_identifier) @operand)
 (when_condition (simple_identifier) @operand)
 (if_expression condition: (simple_identifier) @operand)
 (check_expression . (simple_identifier) @operand)
 (as_expression . (simple_identifier) @operand)
 (for_statement "in" . (simple_identifier) @operand)]
