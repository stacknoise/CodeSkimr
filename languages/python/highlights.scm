; Python highlighting captures (blueprint 8.1).
(comment) @comment
(string) @string
(escape_sequence) @string.escape
[(integer) (float)] @number
[(true) (false) (none)] @constant.builtin
(function_definition name: (identifier) @function)
(class_definition name: (identifier) @type)
(call function: (identifier) @function.call)
(call function: (attribute attribute: (identifier) @function.call))
(decorator) @attribute
["def" "class" "return" "if" "elif" "else" "for" "while" "try" "except" "finally" "with"
 "import" "from" "as" "raise" "pass" "break" "continue" "lambda" "yield" "await" "async"
 "match" "case" "assert" "del" "global" "nonlocal" "in" "is" "not" "and" "or"] @keyword
