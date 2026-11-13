; Java syntax highlighting for the UI (capture names follow the usual Tree-sitter convention).

[(line_comment) (block_comment)] @comment
(string_literal) @string
(character_literal) @string
[(decimal_integer_literal) (hex_integer_literal) (octal_integer_literal) (binary_integer_literal)
 (decimal_floating_point_literal) (hex_floating_point_literal)] @number
[(true) (false) (null_literal)] @constant.builtin
[(integral_type) (floating_point_type) (boolean_type) (void_type)] @type.builtin
(type_identifier) @type
(class_declaration name: (identifier) @type)
(interface_declaration name: (identifier) @type)
(enum_declaration name: (identifier) @type)
(record_declaration name: (identifier) @type)
(annotation_type_declaration name: (identifier) @type)
(method_declaration name: (identifier) @function)
(constructor_declaration name: (identifier) @function)
(method_invocation name: (identifier) @function.call)
(field_declaration declarator: (variable_declarator name: (identifier) @property))
(enum_constant name: (identifier) @constant)
(formal_parameter name: (identifier) @variable.parameter)
[(annotation) (marker_annotation)] @attribute
[(this) (super)] @variable.builtin
(modifiers) @keyword
[
  "if" "else" "for" "while" "do" "switch" "case" "default" "return" "break" "continue" "try"
  "catch" "finally" "throw" "throws" "new" "class" "interface" "enum" "record" "extends"
  "implements" "import" "package" "instanceof" "synchronized" "assert" "yield" "permits"
] @keyword
