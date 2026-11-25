; C syntax highlighting for the UI (capture names follow the usual Tree-sitter convention).

(comment) @comment
[(string_literal) (char_literal) (system_lib_string)] @string
(escape_sequence) @string.escape
(number_literal) @number
[(true) (false) (null)] @constant.builtin
[(primitive_type) (sized_type_specifier)] @type.builtin
(type_identifier) @type
(function_declarator declarator: (identifier) @function)
(call_expression function: (identifier) @function.call)
(call_expression function: (field_expression field: (field_identifier) @function.call))
(field_identifier) @property
(enumerator name: (identifier) @constant)
(statement_identifier) @label
[(preproc_directive) "#include" "#define" "#if" "#ifdef" "#ifndef" "#else" "#elif"
 "#endif"] @attribute
[
  "if" "else" "for" "while" "do" "switch" "case" "default" "return" "break" "continue" "goto"
  "struct" "union" "enum" "typedef" "sizeof" "static" "extern" "inline" "register" "volatile"
  "const" "auto"
] @keyword
