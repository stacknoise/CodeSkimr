; TypeScript declarations -> symbol kinds (blueprint 8.1, ADR 0022).
; A declaration may be wrapped in `export` (the keyword is a modifier and gives visibility) and in
; `declare`. A JSDoc comment directly above the declaration, or above its `export`, is its
; documentation. Several matches of one declaration are merged by the adapter.

; Declarations of a module: exported ones, then every other one.
((comment)* @doc
 .
 (export_statement "export" @modifier declaration:
  [(class_declaration name: (type_identifier) @name) @symbol.class
   (abstract_class_declaration name: (type_identifier) @name) @symbol.class
   (interface_declaration name: (type_identifier) @name) @symbol.interface
   (type_alias_declaration name: (type_identifier) @name) @symbol.type_alias
   (enum_declaration name: (identifier) @name) @symbol.enum
   (internal_module name: [(identifier) (nested_identifier)] @name) @symbol.namespace
   (function_declaration name: (identifier) @name) @symbol.function
   (generator_function_declaration name: (identifier) @name) @symbol.function
   (ambient_declaration
    [(class_declaration name: (type_identifier) @name) @symbol.class
     (abstract_class_declaration name: (type_identifier) @name) @symbol.class
     (interface_declaration name: (type_identifier) @name) @symbol.interface
     (type_alias_declaration name: (type_identifier) @name) @symbol.type_alias
     (enum_declaration name: (identifier) @name) @symbol.enum
     (internal_module name: [(identifier) (nested_identifier)] @name) @symbol.namespace
     (function_signature name: (identifier) @name) @symbol.function])]))
((comment)* @doc
 .
 [(class_declaration name: (type_identifier) @name) @symbol.class
  (abstract_class_declaration name: (type_identifier) @name) @symbol.class
  (interface_declaration name: (type_identifier) @name) @symbol.interface
  (type_alias_declaration name: (type_identifier) @name) @symbol.type_alias
  (enum_declaration name: (identifier) @name) @symbol.enum
  (internal_module name: [(identifier) (nested_identifier)] @name) @symbol.namespace
  (function_declaration name: (identifier) @name) @symbol.function
  (generator_function_declaration name: (identifier) @name) @symbol.function
  (ambient_declaration
   [(class_declaration name: (type_identifier) @name) @symbol.class
    (abstract_class_declaration name: (type_identifier) @name) @symbol.class
    (interface_declaration name: (type_identifier) @name) @symbol.interface
    (type_alias_declaration name: (type_identifier) @name) @symbol.type_alias
    (enum_declaration name: (identifier) @name) @symbol.enum
    (internal_module name: [(identifier) (nested_identifier)] @name) @symbol.namespace
    (function_signature name: (identifier) @name) @symbol.function])])

; Constants of a module (`const` with a value that is no function) and declared constants.
(program
 (comment)+ @doc
 .
 (lexical_declaration "const" @modifier
  (variable_declarator name: (identifier) @name
   value: [(number) (string) (template_string) (true) (false) (null) (undefined) (array)
            (object) (new_expression) (call_expression) (member_expression)
            (subscript_expression) (identifier) (regex) (unary_expression) (binary_expression)
            (ternary_expression) (await_expression) (as_expression) (satisfies_expression)
            (non_null_expression) (parenthesized_expression) (class)]) @symbol.field))
(program
 (lexical_declaration "const" @modifier
  (variable_declarator name: (identifier) @name
   value: [(number) (string) (template_string) (true) (false) (null) (undefined) (array)
            (object) (new_expression) (call_expression) (member_expression)
            (subscript_expression) (identifier) (regex) (unary_expression) (binary_expression)
            (ternary_expression) (await_expression) (as_expression) (satisfies_expression)
            (non_null_expression) (parenthesized_expression) (class)]) @symbol.field))
(program
 (comment)+ @doc
 .
 (export_statement "export" @modifier declaration: (lexical_declaration "const" @modifier
  (variable_declarator name: (identifier) @name
   value: [(number) (string) (template_string) (true) (false) (null) (undefined) (array)
            (object) (new_expression) (call_expression) (member_expression)
            (subscript_expression) (identifier) (regex) (unary_expression) (binary_expression)
            (ternary_expression) (await_expression) (as_expression) (satisfies_expression)
            (non_null_expression) (parenthesized_expression) (class)]) @symbol.field)))
(program
 (export_statement "export" @modifier declaration: (lexical_declaration "const" @modifier
  (variable_declarator name: (identifier) @name
   value: [(number) (string) (template_string) (true) (false) (null) (undefined) (array)
            (object) (new_expression) (call_expression) (member_expression)
            (subscript_expression) (identifier) (regex) (unary_expression) (binary_expression)
            (ternary_expression) (await_expression) (as_expression) (satisfies_expression)
            (non_null_expression) (parenthesized_expression) (class)]) @symbol.field)))
(program
 (comment)+ @doc
 .
 (ambient_declaration (lexical_declaration "const" @modifier
  (variable_declarator name: (identifier) @name) @symbol.field)))
(program
 (ambient_declaration (lexical_declaration "const" @modifier
  (variable_declarator name: (identifier) @name) @symbol.field)))
(program
 (comment)+ @doc
 .
 (export_statement "export" @modifier declaration: (ambient_declaration (lexical_declaration "const" @modifier
  (variable_declarator name: (identifier) @name) @symbol.field))))
(program
 (export_statement "export" @modifier declaration: (ambient_declaration (lexical_declaration "const" @modifier
  (variable_declarator name: (identifier) @name) @symbol.field))))

; A function value is a function (its symbol is the function expression, so its body is measured).
; Local ones count too, like local functions.
((comment)* @doc
 .
 (lexical_declaration
  (variable_declarator name: (identifier) @name
   value: [(arrow_function) (function_expression) (generator_function)] @symbol.function)))
((comment)* @doc
 .
 (export_statement "export" @modifier declaration: (lexical_declaration
  (variable_declarator name: (identifier) @name
   value: [(arrow_function) (function_expression) (generator_function)] @symbol.function))))

; Members of a class. Modifiers (`public`, `static`, `readonly`, `abstract`, `async`, `get`, `set`)
; are captured as they come; `get` and `set` make a property, an arrow function value a method.
(class_body
 (comment)+ @doc
 .
 (method_definition
  [(accessibility_modifier) "static" "override" "async" "get" "set" "*"]* @modifier
  name: [(property_identifier) (private_property_identifier)] @name
  (#not-eq? @name "constructor")) @symbol.method)
(class_body
 (comment)+ @doc
 .
 (method_definition
  (accessibility_modifier)? @modifier
  name: (property_identifier) @name
  (#eq? @name "constructor")) @symbol.constructor)
(class_body
 (method_definition
  [(accessibility_modifier) "static" "override" "async" "get" "set" "*"]* @modifier
  name: [(property_identifier) (private_property_identifier)] @name
  (#not-eq? @name "constructor")) @symbol.method)
(class_body
 (method_definition
  (accessibility_modifier)? @modifier
  name: (property_identifier) @name
  (#eq? @name "constructor")) @symbol.constructor)
((comment)* @doc
 .
 (abstract_method_signature
  [(accessibility_modifier) "override" "get" "set"]* @modifier
  name: [(property_identifier) (private_property_identifier)] @name) @symbol.method)
((comment)* @doc
 .
 (public_field_definition
  [(accessibility_modifier) "static" "readonly" "override" "abstract" "declare" "accessor"]* @modifier
  name: [(property_identifier) (private_property_identifier)] @name
  value: [(number) (string) (template_string) (true) (false) (null) (undefined) (array)
          (object) (new_expression) (call_expression) (member_expression)
          (subscript_expression) (identifier) (regex) (unary_expression) (binary_expression)
          (ternary_expression) (await_expression) (as_expression) (satisfies_expression)
          (non_null_expression) (parenthesized_expression) (class)]) @symbol.field)
((comment)* @doc
 .
 (public_field_definition
  [(accessibility_modifier) "static" "readonly" "override" "abstract" "declare" "accessor"]* @modifier
  name: [(property_identifier) (private_property_identifier)] @name
  !value) @symbol.field)
((comment)* @doc
 .
 (public_field_definition
  [(accessibility_modifier) "static" "override"]* @modifier
  name: [(property_identifier) (private_property_identifier)] @name
  value: [(arrow_function) (function_expression) (generator_function)] @symbol.method))

; Members of an interface and of a declared class, and the members of an enum.
(interface_body
 (comment)+ @doc
 .
 (method_signature name: (property_identifier) @name) @symbol.method)
(interface_body
 (method_signature name: (property_identifier) @name) @symbol.method)
(interface_body
 (comment)+ @doc
 .
 (property_signature "readonly"? @modifier name: (property_identifier) @name) @symbol.property)
(interface_body
 (property_signature "readonly"? @modifier name: (property_identifier) @name) @symbol.property)
(enum_body
 (comment)+ @doc
 .
 [(property_identifier) @name @symbol.enum_member (enum_assignment name: (property_identifier) @name) @symbol.enum_member])
(enum_body
 [(property_identifier) @name @symbol.enum_member (enum_assignment name: (property_identifier) @name) @symbol.enum_member])
(ambient_declaration
 (class_declaration
  body: (class_body
   (comment)+ @doc
   .
   (method_signature
    [(accessibility_modifier) "static" "get" "set"]* @modifier
    name: (property_identifier) @name) @symbol.method)))
(ambient_declaration
 (class_declaration
  body: (class_body
   (method_signature
    [(accessibility_modifier) "static" "get" "set"]* @modifier
    name: (property_identifier) @name) @symbol.method)))
