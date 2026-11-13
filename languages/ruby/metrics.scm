; Ruby metrics captures (ADR 0015, basic tier: comments and decision points, no Halstead counts).
; Counted: if / elsif / unless (also as modifier), ternary, loops (also as modifier), every `when`,
; `rescue`, `&&`, `||`, `and`, `or`.
(comment) @comment

(if "if" @decision)
(elsif "elsif" @decision)
(unless "unless" @decision)
(if_modifier "if" @decision)
(unless_modifier "unless" @decision)
(conditional "?" @decision)
(while "while" @decision)
(until "until" @decision)
(while_modifier "while" @decision)
(until_modifier "until" @decision)
(for "for" @decision)
(when "when" @decision)
(rescue "rescue" @decision)
(binary operator: ["&&" "||" "and" "or"] @decision)
