; Python metrics captures (ADR 0015, ADR 0023): comments, decision points, Halstead operators
; and operands.
; Mapping of the counting rules to nodes (golden test: crates/adapters/parsing/tests/python_metrics.rs):
;   if / elif               if_statement, elif_clause
;   conditional expression  conditional_expression (`a if c else b`)
;   loops                   for_statement, while_statement; a comprehension counts per `for` clause
;                           and per `if` clause (not the guard of a `case`)
;   case                    case_clause, but not the wildcard `case _` (its guard adds nothing)
;   except                  except_clause (`try`, `else`, `finally` do not)
;   short circuit           `and`, `or`
; Not counted (ADR 0015): `else` of if / for / while / try, `assert`, `with`, `not`, the
; walrus operator, `match` itself, a `lambda`. Decorators and default values count for the
; enclosing callable of the `def` line like every expression.

(comment) @comment

; Decision points
(if_statement "if" @decision)
(elif_clause "elif" @decision)
(conditional_expression "if" @decision)
(for_statement "for" @decision)
(while_statement "while" @decision)
(for_in_clause "for" @decision)
[
  (list_comprehension (if_clause "if" @decision))
  (set_comprehension (if_clause "if" @decision))
  (dictionary_comprehension (if_clause "if" @decision))
  (generator_expression (if_clause "if" @decision))]
(case_clause . "case" . (case_pattern) @decision (#not-eq? @decision "_"))
(except_clause "except" @decision)
(boolean_operator operator: ["and" "or"] @decision)

; Operators: tokens that compute, compare, assign, access, call and control the flow. A call
; counts as the operator `(` and a subscript as `[`.
(binary_operator operator: _ @operator)
(boolean_operator operator: _ @operator)
(comparison_operator operators: _ @operator)
(not_operator "not" @operator)
(unary_operator operator: _ @operator)
(assignment "=" @operator)
(augmented_assignment operator: _ @operator)
(named_expression ":=" @operator)
(attribute "." @operator)
(call arguments: (argument_list "(" @operator))
(subscript "[" @operator)
(conditional_expression ["if" "else"] @operator)
(lambda "lambda" @operator)
(if_statement "if" @operator)
(elif_clause "elif" @operator)
(for_statement "for" @operator)
(while_statement "while" @operator)
(for_in_clause "for" @operator)
[
  (list_comprehension (if_clause "if" @operator))
  (set_comprehension (if_clause "if" @operator))
  (dictionary_comprehension (if_clause "if" @operator))
  (generator_expression (if_clause "if" @operator))]
(match_statement "match" @operator)
(case_clause "case" @operator)
(return_statement "return" @operator)
(raise_statement "raise" @operator)
(try_statement "try" @operator)
(except_clause "except" @operator)
(break_statement) @operator
(continue_statement) @operator
(await "await" @operator)
(yield "yield" @operator)
(with_statement "with" @operator)
(assert_statement "assert" @operator)

; Operands: literals and names in value positions
[(integer) (float) (string) (concatenated_string) (true) (false) (none)] @operand
(assignment left: (identifier) @operand)
(assignment right: (identifier) @operand)
(augmented_assignment left: (identifier) @operand)
(call function: (identifier) @operand)
(attribute object: (identifier) @operand)
(attribute attribute: (identifier) @operand)
[(argument_list (identifier) @operand)
 (return_statement (identifier) @operand)
 (parenthesized_expression (identifier) @operand)
 (await (identifier) @operand)
 (raise_statement (identifier) @operand)
 (expression_statement (identifier) @operand)
 (subscript (identifier) @operand)
 (list (identifier) @operand)
 (tuple (identifier) @operand)
 (set (identifier) @operand)
 (pair (identifier) @operand)]
(binary_operator [left: (identifier) @operand right: (identifier) @operand])
(boolean_operator [left: (identifier) @operand right: (identifier) @operand])
(comparison_operator (identifier) @operand)
(not_operator argument: (identifier) @operand)
(unary_operator argument: (identifier) @operand)
(conditional_expression (identifier) @operand)
(subscript [value: (identifier) @operand subscript: (identifier) @operand])
(if_statement condition: (identifier) @operand)
(elif_clause condition: (identifier) @operand)
(while_statement condition: (identifier) @operand)
(for_statement [left: (identifier) @operand right: (identifier) @operand])
(for_in_clause [left: (identifier) @operand right: (identifier) @operand])
[
  (list_comprehension (if_clause (identifier) @operand))
  (set_comprehension (if_clause (identifier) @operand))
  (dictionary_comprehension (if_clause (identifier) @operand))
  (generator_expression (if_clause (identifier) @operand))]
(lambda body: (identifier) @operand)
(keyword_argument value: (identifier) @operand)
