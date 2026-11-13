; PHP declarations -> symbol kinds (basic tier).
; A namespace without braces holds the rest of the file, so the program closes it.
(namespace_definition name: (namespace_name) @name body: (_)) @symbol.namespace
(program (namespace_definition name: (namespace_name) @name !body) @symbol.namespace) @scope_end
(class_declaration name: (name) @name) @symbol.class
(interface_declaration name: (name) @name) @symbol.interface
(trait_declaration name: (name) @name) @symbol.trait
(enum_declaration name: (name) @name) @symbol.enum
(function_definition name: (name) @name) @symbol.function
(method_declaration name: (name) @name) @symbol.method
