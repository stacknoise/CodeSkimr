; Rust syntax highlighting for the UI (capture names follow the usual Tree-sitter convention).
[(line_comment) (block_comment)] @comment
[(string_literal) (raw_string_literal) (char_literal)] @string
(escape_sequence) @string.escape
[(integer_literal) (float_literal)] @number
(boolean_literal) @constant.builtin
(primitive_type) @type.builtin
(type_identifier) @type
(function_item name: (identifier) @function)
(function_signature_item name: (identifier) @function)
(call_expression function: (identifier) @function.call)
(call_expression function: (field_expression field: (field_identifier) @function.call))
(call_expression function: (scoped_identifier name: (identifier) @function.call))
(macro_invocation macro: (identifier) @function.macro)
(field_identifier) @property
(attribute_item) @attribute
(lifetime) @label
["as" "async" "await" "break" "const" "continue" "else" "enum" "extern" "fn" "for" "if" "impl"
 "in" "let" "loop" "match" "mod" "move" "pub" "ref" "return" "static" "struct" "trait" "type"
 "union" "unsafe" "use" "where" "while" "dyn"] @keyword
[(self) (super) (crate) (mutable_specifier)] @keyword
