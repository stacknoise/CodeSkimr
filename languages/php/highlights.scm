; PHP syntax highlighting for the UI (capture names follow the usual Tree-sitter convention).

(comment) @comment
[(string) (encapsed_string) (heredoc_body) (nowdoc_body)] @string
(escape_sequence) @string.escape
[(integer) (float)] @number
[(boolean) (null)] @constant.builtin
(primitive_type) @type.builtin
(named_type (name) @type)
(class_declaration name: (name) @type)
(interface_declaration name: (name) @type)
(trait_declaration name: (name) @type)
(enum_declaration name: (name) @type)
(function_definition name: (name) @function)
(method_declaration name: (name) @function)
(function_call_expression function: (name) @function.call)
(member_call_expression name: (name) @function.call)
(scoped_call_expression name: (name) @function.call)
(property_element (variable_name) @property)
(php_tag) @attribute
[
  "abstract" "and" "as" "break" "case" "catch" "class" "const" "continue" "declare" "default"
  "do" "echo" "else" "elseif" "enum" "extends" "final" "finally" "fn" "for" "foreach" "function"
  "global" "if" "implements" "include" "include_once" "instanceof" "interface" "namespace" "new"
  "or" "private" "protected" "public" "readonly" "require" "require_once" "return" "static"
  "switch" "throw" "trait" "try" "use" "while" "xor" "yield"
] @keyword
