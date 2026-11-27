; ABAP declarations -> symbol kinds (basic tier, ADR 0113).
(class_definition name: (identifier) @name) @symbol.class
(interface_definition name: (identifier) @name) @symbol.interface
(form_definition name: (identifier) @name) @symbol.function
(function_definition name: (identifier) @name) @symbol.function
(module_definition name: (identifier) @name) @symbol.function

; `CLASS c IMPLEMENTATION` is no symbol but a container (ADR 0039): its methods belong to the class
; the block names, which is declared before it in the same file or in another one.
(class_implementation name: (identifier) @name) @container

; A method is a symbol where it is implemented. `METHOD zif~name` has two names; the last one is the
; method, the interface only qualifies it.
(method_implementation name: (identifier) @name) @symbol.method
(method_implementation "~" . (identifier) @name) @symbol.method

; The methods of an interface have no implementation, so the declaration is the symbol.
(interface_definition (methods_declaration (method_entry name: (identifier) @name) @symbol.method))
