; C++ declarations -> symbol kinds (basic tier). A type counts only where it is defined (`body`).
(namespace_definition name: (namespace_identifier) @name) @symbol.namespace
(class_specifier name: (type_identifier) @name body: (_)) @symbol.class
(struct_specifier name: (type_identifier) @name body: (_)) @symbol.struct
(union_specifier name: (type_identifier) @name body: (_)) @symbol.union
(enum_specifier name: (type_identifier) @name body: (_)) @symbol.enum

; Functions. A definition outside a class body is a free function, `Class::name` a method; the ones
; inside a class body are methods (constructors and destructors included).
(translation_unit
  (function_definition declarator: (function_declarator declarator: (identifier) @name)) @symbol.function)
(translation_unit
  (function_definition
    declarator: (pointer_declarator declarator: (function_declarator declarator: (identifier) @name))) @symbol.function)
(translation_unit
  (function_definition
    declarator: (reference_declarator (function_declarator declarator: (identifier) @name))) @symbol.function)
(declaration_list
  (function_definition declarator: (function_declarator declarator: (identifier) @name)) @symbol.function)
(declaration_list
  (function_definition
    declarator: (pointer_declarator declarator: (function_declarator declarator: (identifier) @name))) @symbol.function)
(declaration_list
  (function_definition
    declarator: (reference_declarator (function_declarator declarator: (identifier) @name))) @symbol.function)
(template_declaration
  (function_definition declarator: (function_declarator declarator: (identifier) @name)) @symbol.function)
(template_declaration
  (function_definition
    declarator: (pointer_declarator declarator: (function_declarator declarator: (identifier) @name))) @symbol.function)
(template_declaration
  (function_definition
    declarator: (reference_declarator (function_declarator declarator: (identifier) @name))) @symbol.function)
(function_definition
  declarator: (function_declarator declarator: (qualified_identifier name: (identifier) @name))) @symbol.method
(field_declaration_list
  (function_definition
    declarator: (function_declarator declarator: [(identifier) (field_identifier) (destructor_name)] @name)) @symbol.method)
