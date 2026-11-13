; C++ metrics captures (ADR 0015, basic tier: comments and decision points, no Halstead counts).
; Counted: if, conditional expression, loops (also range-based), every `case` (not `default`),
; `catch`, `&&` and `||`.
(comment) @comment

(if_statement "if" @decision)
(conditional_expression "?" @decision)
(for_statement "for" @decision)
(for_range_loop "for" @decision)
(while_statement "while" @decision)
(do_statement "do" @decision)
(case_statement "case" @decision)
(catch_clause "catch" @decision)
(binary_expression operator: ["&&" "||"] @decision)
