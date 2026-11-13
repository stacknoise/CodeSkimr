; JavaScript declarations -> symbol kinds (blueprint 8.1, ADR 0022). A declaration may be wrapped in
; `export`; the keyword is a modifier and gives visibility. A JSDoc comment directly above the
; declaration, or above its `export`, is its documentation.

; Declarations of a module: exported ones, then every other one.
((comment)* @doc
 .
 (export_statement "export" @modifier declaration:
  [(class_declaration name: (identifier) @name) @symbol.class
   (function_declaration name: (identifier) @name) @symbol.function
   (generator_function_declaration name: (identifier) @name) @symbol.function]))
((comment)* @doc
 .
 [(class_declaration name: (identifier) @name) @symbol.class
  (function_declaration name: (identifier) @name) @symbol.function
  (generator_function_declaration name: (identifier) @name) @symbol.function])

; Constants of a module: a `const` with a value that is no function.
(program
 (comment)+ @doc
 .
 (lexical_declaration "const" @modifier
  (variable_declarator name: (identifier) @name
   value: [(number) (string) (template_string) (true) (false) (null) (undefined) (array)
           (object) (new_expression) (call_expression) (member_expression)
           (subscript_expression) (identifier) (regex) (unary_expression) (binary_expression)
           (ternary_expression) (await_expression) (parenthesized_expression) (class) (jsx_element) (jsx_self_closing_element)]) @symbol.field))
(program
 (lexical_declaration "const" @modifier
  (variable_declarator name: (identifier) @name
   value: [(number) (string) (template_string) (true) (false) (null) (undefined) (array)
           (object) (new_expression) (call_expression) (member_expression)
           (subscript_expression) (identifier) (regex) (unary_expression) (binary_expression)
           (ternary_expression) (await_expression) (parenthesized_expression) (class) (jsx_element) (jsx_self_closing_element)]) @symbol.field))
(program
 (comment)+ @doc
 .
 (export_statement "export" @modifier declaration:
  (lexical_declaration "const" @modifier
   (variable_declarator name: (identifier) @name
    value: [(number) (string) (template_string) (true) (false) (null) (undefined) (array)
            (object) (new_expression) (call_expression) (member_expression)
            (subscript_expression) (identifier) (regex) (unary_expression) (binary_expression)
            (ternary_expression) (await_expression) (parenthesized_expression) (class) (jsx_element) (jsx_self_closing_element)]) @symbol.field)))
(program
 (export_statement "export" @modifier declaration:
  (lexical_declaration "const" @modifier
   (variable_declarator name: (identifier) @name
    value: [(number) (string) (template_string) (true) (false) (null) (undefined) (array)
            (object) (new_expression) (call_expression) (member_expression)
            (subscript_expression) (identifier) (regex) (unary_expression) (binary_expression)
            (ternary_expression) (await_expression) (parenthesized_expression) (class) (jsx_element) (jsx_self_closing_element)]) @symbol.field)))

; A function value is a function (its symbol is the function expression, so its body is measured).
; Local ones count too, like local functions.
((comment)* @doc
 .
 (lexical_declaration
  (variable_declarator name: (identifier) @name
   value: [(arrow_function) (function_expression) (generator_function)] @symbol.function)))
((comment)* @doc
 .
 (export_statement "export" @modifier declaration:
  (lexical_declaration
   (variable_declarator name: (identifier) @name
    value: [(arrow_function) (function_expression) (generator_function)] @symbol.function))))

; Members of a class. Modifiers (`static`, `async`, `get`, `set`) are captured as they come; `get`
; and `set` make a property, an arrow function value a method.
(class_body
 (comment)+ @doc
 .
 (method_definition
  ["static" "async" "get" "set" "*"]* @modifier
  name: [(property_identifier) (private_property_identifier)] @name
  (#not-eq? @name "constructor")) @symbol.method)
(class_body
 (comment)+ @doc
 .
 (method_definition
  name: (property_identifier) @name
  (#eq? @name "constructor")) @symbol.constructor)
(class_body
 (method_definition
  ["static" "async" "get" "set" "*"]* @modifier
  name: [(property_identifier) (private_property_identifier)] @name
  (#not-eq? @name "constructor")) @symbol.method)
(class_body
 (method_definition
  name: (property_identifier) @name
  (#eq? @name "constructor")) @symbol.constructor)
((comment)* @doc
 .
 (field_definition
  "static"? @modifier
  property: [(property_identifier) (private_property_identifier)] @name
  value: [(number) (string) (template_string) (true) (false) (null) (undefined) (array)
          (object) (new_expression) (call_expression) (member_expression)
          (subscript_expression) (identifier) (regex) (unary_expression) (binary_expression)
          (ternary_expression) (await_expression) (parenthesized_expression) (class) (jsx_element) (jsx_self_closing_element)]) @symbol.field)
((comment)* @doc
 .
 (field_definition
  "static"? @modifier
  property: [(property_identifier) (private_property_identifier)] @name
  !value) @symbol.field)
((comment)* @doc
 .
 (field_definition
  "static"? @modifier
  property: [(property_identifier) (private_property_identifier)] @name
  value: [(arrow_function) (function_expression) (generator_function)] @symbol.method))
