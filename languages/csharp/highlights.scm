; C# syntax highlighting for the UI (capture names follow the usual Tree-sitter convention).

(comment) @comment
[(string_literal) (character_literal) (verbatim_string_literal) (raw_string_literal)
 (interpolated_string_expression)] @string
[(integer_literal) (real_literal)] @number
[(boolean_literal) (null_literal)] @constant.builtin
(predefined_type) @type.builtin
(modifier) @keyword
(class_declaration name: (identifier) @type)
(interface_declaration name: (identifier) @type)
(struct_declaration name: (identifier) @type)
(record_declaration name: (identifier) @type)
(enum_declaration name: (identifier) @type)
(method_declaration name: (identifier) @function)
(constructor_declaration name: (identifier) @function)
(invocation_expression function: (identifier) @function)
(invocation_expression function: (member_access_expression name: (identifier) @function))
(property_declaration name: (identifier) @property)
(parameter name: (identifier) @variable.parameter)
(attribute name: (identifier) @attribute)
[
  "if" "else" "for" "foreach" "while" "do" "switch" "case" "default" "return" "break" "continue"
  "try" "catch" "finally" "throw" "new" "using" "namespace" "class" "struct" "interface" "enum"
  "record" "await" "async" "var" "is" "as" "in" "out" "ref" "typeof" "yield" "when" "lock"
] @keyword
