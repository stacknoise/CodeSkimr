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
; One pattern per parent: an alternation in front of the children never matches (issue #77).
(arguments (identifier) @operand)
(return_statement (identifier) @operand)
(throw_statement (identifier) @operand)
(parenthesized_expression (identifier) @operand)
(await_expression (identifier) @operand)
(yield_expression (identifier) @operand)
(template_substitution (identifier) @operand)
(array (identifier) @operand)
(expression_statement (identifier) @operand)
(unary_expression (identifier) @operand)
(update_expression (identifier) @operand)
(spread_element (identifier) @operand)
(subscript_expression (identifier) @operand)
(variable_declarator name: (_) value: (identifier) @operand)
(pair value: (identifier) @operand)
(arrow_function body: (identifier) @operand)
(binary_expression [left: (identifier) @operand right: (identifier) @operand])
(assignment_expression [left: (identifier) @operand right: (identifier) @operand])
(augmented_assignment_expression [left: (identifier) @operand right: (identifier) @operand])
(ternary_expression
  [condition: (identifier) @operand consequence: (identifier) @operand
   alternative: (identifier) @operand])

; Cognitive complexity (ADR 0048). A structure adds one and one more for every nesting level it
; stands in; `else` and a run of the same boolean operator add one at any depth. `else if` is the
; `if_statement` in an `else_clause`: only its `else` counts, and its body is no deeper than the
; first `if`. A function value that is not a declaration of its own (a callback, a returned or
; assigned function) nests what is in it but adds nothing; a `const f = () => ...` is a function of
; its own and starts at depth zero. A labelled `break` or `continue` adds one at any depth. Not
; counted: `try`, `finally`, `case` clauses (the `switch` counts once), `?.`, recursion.
(if_statement) @cognitive
(if_statement) @nesting
(else_clause (if_statement) @continuation)
(else_clause "else" @cognitive_flat)
(ternary_expression) @cognitive
(ternary_expression) @nesting
(for_statement) @cognitive
(for_statement) @nesting
(for_in_statement) @cognitive
(for_in_statement) @nesting
(while_statement) @cognitive
(while_statement) @nesting
(do_statement) @cognitive
(do_statement) @nesting
(switch_statement) @cognitive
(switch_statement) @nesting
(catch_clause) @cognitive
(catch_clause) @nesting
(break_statement label: (statement_identifier)) @cognitive_flat
(continue_statement label: (statement_identifier)) @cognitive_flat
(binary_expression operator: ["&&" "||" "??"] @cognitive_sequence)
(arguments [(arrow_function) (function_expression) (generator_function)] @nesting)
(return_statement [(arrow_function) (function_expression) (generator_function)] @nesting)
(array [(arrow_function) (function_expression) (generator_function)] @nesting)
(parenthesized_expression [(arrow_function) (function_expression) (generator_function)] @nesting)
(await_expression [(arrow_function) (function_expression) (generator_function)] @nesting)
(pair value: [(arrow_function) (function_expression) (generator_function)] @nesting)
(assignment_expression right: [(arrow_function) (function_expression) (generator_function)] @nesting)
(ternary_expression [(arrow_function) (function_expression) (generator_function)] @nesting)
(binary_expression [(arrow_function) (function_expression) (generator_function)] @nesting)

; Parameters, only those of a function of its own: a declaration, a method, and a function value in
; a `const` or in a class field. The parameters of a callback or of a function type are not the
; ones of the function around them. A list holds every child but a comment and the `this` type.
(function_declaration parameters: (formal_parameters (_) @parameter (#not-match? @parameter "^(//|/\\*|this[ :])")))
(generator_function_declaration parameters: (formal_parameters (_) @parameter (#not-match? @parameter "^(//|/\\*|this[ :])")))
(method_definition parameters: (formal_parameters (_) @parameter (#not-match? @parameter "^(//|/\\*|this[ :])")))
(variable_declarator value: (arrow_function parameters: (formal_parameters (_) @parameter (#not-match? @parameter "^(//|/\\*|this[ :])"))))
(variable_declarator value: (function_expression parameters: (formal_parameters (_) @parameter (#not-match? @parameter "^(//|/\\*|this[ :])"))))
(variable_declarator value: (generator_function parameters: (formal_parameters (_) @parameter (#not-match? @parameter "^(//|/\\*|this[ :])"))))
(variable_declarator value: (arrow_function parameter: (identifier) @parameter))
(class_body (_ value: (arrow_function parameters: (formal_parameters (_) @parameter (#not-match? @parameter "^(//|/\\*|this[ :])")))))
(class_body (_ value: (function_expression parameters: (formal_parameters (_) @parameter (#not-match? @parameter "^(//|/\\*|this[ :])")))))
(class_body (_ value: (generator_function parameters: (formal_parameters (_) @parameter (#not-match? @parameter "^(//|/\\*|this[ :])")))))
(class_body (_ value: (arrow_function parameter: (identifier) @parameter)))
