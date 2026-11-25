; C++ syntax highlighting for the UI (capture names follow the usual Tree-sitter convention).

(comment) @comment
[(string_literal) (char_literal) (raw_string_literal) (system_lib_string)] @string
(escape_sequence) @string.escape
(number_literal) @number
[(true) (false) (null)] @constant.builtin
[(primitive_type) (sized_type_specifier) (auto)] @type.builtin
[(type_identifier) (namespace_identifier)] @type
(function_declarator declarator: (identifier) @function)
(function_declarator declarator: (field_identifier) @function)
(function_declarator declarator: (qualified_identifier name: (identifier) @function))
(call_expression function: (identifier) @function.call)
(call_expression function: (field_expression field: (field_identifier) @function.call))
(call_expression function: (qualified_identifier name: (identifier) @function.call))
(field_identifier) @property
(enumerator name: (identifier) @constant)
(statement_identifier) @label
(this) @variable.builtin
[(preproc_directive) "#include" "#define" "#if" "#ifdef" "#ifndef" "#else" "#elif"
 "#endif"] @attribute
[
  "if" "else" "for" "while" "do" "switch" "case" "default" "return" "break" "continue" "goto"
  "struct" "union" "enum" "typedef" "sizeof" "static" "extern" "inline" "register" "volatile"
  "const" "class" "namespace" "template" "typename" "public" "private" "protected" "virtual"
  "override" "final" "new" "delete" "try" "catch" "throw" "using" "operator" "constexpr"
  "noexcept" "explicit" "friend"
] @keyword
