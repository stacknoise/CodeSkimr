; Kotlin syntax highlighting for the UI (capture names follow the usual Tree-sitter convention).

[(line_comment) (multiline_comment)] @comment
(string_literal) @string
(character_literal) @string
[(integer_literal) (long_literal) (hex_literal) (bin_literal) (real_literal)] @number
[(boolean_literal) (null_literal)] @constant.builtin
(type_identifier) @type
(class_declaration (type_identifier) @type)
(object_declaration (type_identifier) @type)
(function_declaration (simple_identifier) @function)
(call_expression (simple_identifier) @function.call)
(call_expression (navigation_expression (navigation_suffix (simple_identifier) @function.call)))
(class_parameter (simple_identifier) @property)
(parameter (simple_identifier) @variable.parameter)
(enum_entry (simple_identifier) @constant)
(annotation) @attribute
(this_expression) @variable.builtin
(super_expression) @variable.builtin
(modifiers) @keyword
[
  "if" "else" "for" "while" "do" "when" "return" "break" "continue" "try" "catch" "finally" "throw"
  "class" "interface" "object" "fun" "val" "var" "import" "package" "typealias" "constructor"
  "companion" "enum" "is" "as" "in" "by" "init"
] @keyword
