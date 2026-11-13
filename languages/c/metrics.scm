; C metrics captures (ADR 0015, basic tier: comments and decision points, no Halstead counts).
; Counted: if, conditional expression, loops, every `case` (not `default`), `&&` and `||`.
(comment) @comment

(if_statement "if" @decision)
(conditional_expression "?" @decision)
(for_statement "for" @decision)
(while_statement "while" @decision)
(do_statement "do" @decision)
(case_statement "case" @decision)
(binary_expression operator: ["&&" "||"] @decision)
