; ABAP occurrences (basic tier: calls by name only, ADR 0113).
; `PERFORM form`
(statement . (identifier) @_keyword . (identifier) @name
  (#match? @_keyword "^(?i)perform$")) @reference.call

; `CALL METHOD cls=>method` and `CALL METHOD obj->method`
(statement . (identifier) @_call . (identifier) @_method (operator) . (identifier) @name
  (#match? @_call "^(?i)call$")
  (#match? @_method "^(?i)method$")) @reference.call

; `name( ... )`, `cls=>name( ... )` and `obj->name( ... )`: an identifier right before a bracket
(statement (identifier) @name . (parenthesized_group)
  (#not-match? @name "^(?i)(data|final|field-symbol|new|cond|switch|conv|ref|value|reduce|filter|exact|corresponding|boolc|xsdbool|lines|line_exists|line_index|cast|let|for|abs|strlen|to_upper|to_lower)$")) @reference.call
