; Go metrics captures (ADR 0015, basic tier: comments and decision points, no Halstead counts).
; Counted: if, for, every `case` of a switch or select (not `default`), `&&` and `||`.
(comment) @comment

(if_statement "if" @decision)
(for_statement "for" @decision)
(expression_case "case" @decision)
(type_case "case" @decision)
(communication_case "case" @decision)
(binary_expression operator: ["&&" "||"] @decision)
