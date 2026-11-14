; C declarations -> symbol kinds (basic tier). A type counts only where it is defined (`body`).
(struct_specifier name: (type_identifier) @name body: (_)) @symbol.struct
(union_specifier name: (type_identifier) @name body: (_)) @symbol.union
(enum_specifier name: (type_identifier) @name body: (_)) @symbol.enum
(type_definition type: [(struct_specifier body: (_)) (union_specifier body: (_)) (enum_specifier body: (_))]
  declarator: (type_identifier) @name) @symbol.type_alias
(function_definition declarator: (function_declarator declarator: (identifier) @name)) @symbol.function
(function_definition
  declarator: (pointer_declarator declarator: (function_declarator declarator: (identifier) @name))) @symbol.function
; A function that returns a function pointer: `int (*getcb(void))(int)` (issue #66).
(function_definition
  declarator: (function_declarator
    declarator: (parenthesized_declarator
      (pointer_declarator declarator: (function_declarator declarator: (identifier) @name))))) @symbol.function
(function_definition
  declarator: (pointer_declarator
    declarator: (function_declarator
      declarator: (parenthesized_declarator
        (pointer_declarator declarator: (function_declarator declarator: (identifier) @name)))))) @symbol.function
