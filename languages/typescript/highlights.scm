; TypeScript syntax highlighting for the UI (capture names follow the usual Tree-sitter convention).

(comment) @comment
[(string) (template_string) (regex)] @string
(number) @number
[(true) (false) (null) (undefined)] @constant.builtin
(predefined_type) @type.builtin
(type_identifier) @type
(function_declaration name: (identifier) @function)
(method_definition name: (property_identifier) @function)
(call_expression function: (identifier) @function)
(call_expression function: (member_expression property: (property_identifier) @function))
(property_identifier) @property
["if" "else" "for" "while" "do" "switch" "case" "default" "return" "throw" "try" "catch"
 "finally" "break" "continue" "new" "class" "function" "const" "let" "var" "import" "export"
 "from" "async" "await" "extends" "implements" "interface" "type" "enum" "static" "readonly"
 "abstract" "namespace" "declare" "of" "in" "as" "typeof"] @keyword
(accessibility_modifier) @keyword
[(this) (super)] @keyword
