; PHP metrics captures (ADR 0015, basic tier: comments and decision points, no Halstead counts).
; Counted: if / else if, conditional expression, loops, every `case` (not `default`), `catch`,
; `&&`, `||`, `and`, `or`, `??`.
(comment) @comment

(if_statement "if" @decision)
(else_if_clause "elseif" @decision)
(conditional_expression "?" @decision)
(for_statement "for" @decision)
(foreach_statement "foreach" @decision)
(while_statement "while" @decision)
(do_statement "do" @decision)
(case_statement "case" @decision)
(catch_clause "catch" @decision)
(binary_expression operator: ["&&" "||" "and" "or" "??"] @decision)
