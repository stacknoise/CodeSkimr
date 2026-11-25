; Go syntax highlighting for the UI (capture names follow the usual Tree-sitter convention).

(comment) @comment
[(interpreted_string_literal) (raw_string_literal) (rune_literal)] @string
(escape_sequence) @string.escape
[(int_literal) (float_literal) (imaginary_literal)] @number
[(true) (false) (nil) (iota)] @constant.builtin
(type_identifier) @type
(function_declaration name: (identifier) @function)
(method_declaration name: (field_identifier) @function)
(call_expression function: (identifier) @function.call)
(call_expression function: (selector_expression field: (field_identifier) @function.call))
(field_identifier) @property
(parameter_declaration name: (identifier) @variable.parameter)
(label_name) @label
[
  "package" "import" "func" "var" "const" "type" "struct" "interface" "map" "chan" "if" "else"
  "for" "range" "switch" "case" "default" "select" "return" "break" "continue" "go" "defer"
  "goto" "fallthrough"
] @keyword
