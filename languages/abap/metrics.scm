; ABAP metrics captures (ADR 0015, ADR 0113, basic tier: comments and decision points).
; The grammar has one flat `statement` node whose first word is the keyword, so the decision points
; are found by that word (case does not matter): IF, ELSEIF, CASE branches (WHEN), loops (LOOP, DO,
; WHILE, SELECT as a loop), CATCH, CHECK and the boolean operators AND, OR and EQUIV in a condition.
(star_comment) @comment
(line_comment) @comment

(statement . (identifier) @decision
  (#match? @decision "^(?i)(if|elseif|loop|do|while|catch|check)$"))

; Every branch of a CASE counts, `WHEN OTHERS` does not.
(statement . (identifier) @decision . (_) @_branch
  (#match? @decision "^(?i)when$")
  (#not-match? @_branch "^(?i)others$"))

; The boolean operators count wherever they stand in a statement (a condition of IF, WHILE, WHEN,
; CHECK, SELECT ... WHERE); `BETWEEN a AND b` is counted too, which makes the value a little high.
(statement (identifier) @decision
  (#match? @decision "^(?i)(and|or|equiv)$"))
