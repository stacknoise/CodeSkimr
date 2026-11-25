; Ruby syntax highlighting for the UI (capture names follow the usual Tree-sitter convention).

(comment) @comment
[(string) (bare_string) (heredoc_body) (character)] @string
(escape_sequence) @string.escape
[(simple_symbol) (delimited_symbol) (hash_key_symbol)] @string
[(integer) (float) (rational) (complex)] @number
[(true) (false) (nil)] @constant.builtin
[(self) (super)] @variable.builtin
(constant) @type
(method name: (identifier) @function)
(singleton_method name: (identifier) @function)
(call method: (identifier) @function.call)
[(instance_variable) (class_variable) (global_variable)] @property
(regex) @string
[
  "alias" "and" "begin" "break" "case" "class" "def" "do" "else" "elsif" "end" "ensure" "for"
  "if" "in" "module" "next" "not" "or" "redo" "rescue" "retry" "return" "then" "undef" "unless"
  "until" "when" "while" "yield"
] @keyword
