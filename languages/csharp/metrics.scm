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
; Not counted (ADR 0015): the pattern combinators `and` / `or` of C# 9 (`is` patterns outside a
; match arm are no decision points); `&&`, `||` and `??` are the short-circuit operators.
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
; One pattern per parent: an alternation in front of the children never matches (issue #77).
(argument (identifier) @operand)
(return_statement (identifier) @operand)
(arrow_expression_clause (identifier) @operand)
(interpolation (identifier) @operand)
(parenthesized_expression (identifier) @operand)
(await_expression (identifier) @operand)
(throw_expression (identifier) @operand)
(variable_declarator name: (_) . (identifier) @operand)
(binary_expression [left: (identifier) @operand right: (identifier) @operand])
(assignment_expression [left: (identifier) @operand right: (identifier) @operand])
(conditional_expression
  [condition: (identifier) @operand consequence: (identifier) @operand
   alternative: (identifier) @operand])
(prefix_unary_expression (identifier) @operand)
(postfix_unary_expression (identifier) @operand)

; Cognitive complexity (ADR 0048). A structure adds one and one more for every nesting level it
; stands in; `else` and a run of the same boolean operator add one at any depth. `else if` is an
; `if_statement` in the `alternative` of another: only its `else` counts, and its body is no
; deeper than the first `if`. A lambda nests what is in it but adds nothing; a local function is a
; function of its own. A `goto` adds one at any depth. Not counted: `try`, `finally`, `case`
; labels and the arms of a `switch` expression (the `switch` counts once), `?.`, recursion.
(if_statement) @cognitive
(if_statement) @nesting
(if_statement alternative: (if_statement) @continuation)
(if_statement "else" @cognitive_flat)
(conditional_expression) @cognitive
(conditional_expression) @nesting
(for_statement) @cognitive
(for_statement) @nesting
(foreach_statement) @cognitive
(foreach_statement) @nesting
(while_statement) @cognitive
(while_statement) @nesting
(do_statement) @cognitive
(do_statement) @nesting
(switch_statement) @cognitive
(switch_statement) @nesting
(switch_expression) @cognitive
(switch_expression) @nesting
(catch_clause) @cognitive
(catch_clause) @nesting
(lambda_expression) @nesting
(anonymous_method_expression) @nesting
(goto_statement) @cognitive_flat
(binary_expression operator: ["&&" "||" "??"] @cognitive_sequence)

; Parameters of a method, an operator, a constructor and a local function. The grammar leaves the
; parameter of `params` without a `parameter` node, so its keyword stands for it.
(method_declaration parameters: (parameter_list (parameter) @parameter))
(operator_declaration parameters: (parameter_list (parameter) @parameter))
(constructor_declaration parameters: (parameter_list (parameter) @parameter))
(local_function_statement parameters: (parameter_list (parameter) @parameter))
(method_declaration parameters: (parameter_list "params" @parameter))
(operator_declaration parameters: (parameter_list "params" @parameter))
(constructor_declaration parameters: (parameter_list "params" @parameter))
(local_function_statement parameters: (parameter_list "params" @parameter))

; Literals the rule of no literals reports (ADR 0058, R-06): numbers, strings and characters in a
; method. `true`, `false` and `null` are no literals here. Exceptions: the integers 0 and 1, the
; value of a local `const`, and a literal that is a direct argument of an attribute. An
; interpolated string counts once as a string, and the literals inside its holes count on top of
; that. The initialiser of a local `const` is exempt when it is a literal itself (`const int X = 99`; the
; parts of `2 * 50` still count).
[(integer_literal) (real_literal) (string_literal) (verbatim_string_literal) (raw_string_literal) (character_literal) (interpolated_string_expression)] @literal
((integer_literal) @literal_exempt (#any-of? @literal_exempt "0" "1"))
(local_declaration_statement (modifier "const") (variable_declaration (variable_declarator (_) @literal_exempt)))
(attribute_argument (_) @literal_exempt)
