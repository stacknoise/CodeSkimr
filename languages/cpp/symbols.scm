; C++ declarations -> symbol kinds (basic tier). A type counts only where it is defined (`body`).
(namespace_definition name: (namespace_identifier) @name) @symbol.namespace
(class_specifier name: (type_identifier) @name body: (_)) @symbol.class
(struct_specifier name: (type_identifier) @name body: (_)) @symbol.struct
(union_specifier name: (type_identifier) @name body: (_)) @symbol.union
(enum_specifier name: (type_identifier) @name body: (_)) @symbol.enum

; Functions. A definition outside a class body is a free function (also as a template), `Class::name`
; a method; the ones inside a class body (also templates) are methods, constructors, destructors and
; operators included. Pointer and reference return types are handled; the class of an out-of-line
; method belongs to its class and carries it in its name (`@owner`, ADR 0042).
(translation_unit (function_definition declarator: [
      (function_declarator declarator: (identifier) @name)
      (pointer_declarator (function_declarator declarator: (identifier) @name))
      (reference_declarator (function_declarator declarator: (identifier) @name))]) @symbol.function)
(translation_unit (template_declaration (function_definition declarator: [
      (function_declarator declarator: (identifier) @name)
      (pointer_declarator (function_declarator declarator: (identifier) @name))
      (reference_declarator (function_declarator declarator: (identifier) @name))]) @symbol.function))
(declaration_list (function_definition declarator: [
      (function_declarator declarator: (identifier) @name)
      (pointer_declarator (function_declarator declarator: (identifier) @name))
      (reference_declarator (function_declarator declarator: (identifier) @name))]) @symbol.function)
(declaration_list (template_declaration (function_definition declarator: [
      (function_declarator declarator: (identifier) @name)
      (pointer_declarator (function_declarator declarator: (identifier) @name))
      (reference_declarator (function_declarator declarator: (identifier) @name))]) @symbol.function))
(function_definition declarator: [
      (function_declarator declarator: (qualified_identifier name: [(identifier) (destructor_name) (operator_name)] @name))
      (pointer_declarator (function_declarator declarator: (qualified_identifier name: [(identifier) (destructor_name) (operator_name)] @name)))
      (reference_declarator (function_declarator declarator: (qualified_identifier name: [(identifier) (destructor_name) (operator_name)] @name)))]) @symbol.method
(field_declaration_list (function_definition declarator: [
      (function_declarator declarator: [(identifier) (field_identifier) (destructor_name) (operator_name)] @name)
      (pointer_declarator (function_declarator declarator: [(identifier) (field_identifier) (destructor_name) (operator_name)] @name))
      (reference_declarator (function_declarator declarator: [(identifier) (field_identifier) (destructor_name) (operator_name)] @name))]) @symbol.method)
(field_declaration_list
  (template_declaration (function_definition declarator: [
      (function_declarator declarator: [(identifier) (field_identifier) (destructor_name) (operator_name)] @name)
      (pointer_declarator (function_declarator declarator: [(identifier) (field_identifier) (destructor_name) (operator_name)] @name))
      (reference_declarator (function_declarator declarator: [(identifier) (field_identifier) (destructor_name) (operator_name)] @name))]) @symbol.method))

; An out-of-line method belongs to the class named before `::`, also a class template (`Box<T>::get`).
(function_definition declarator: [
      (function_declarator declarator: (qualified_identifier scope: [(namespace_identifier) @owner (template_type name: (type_identifier) @owner)] name: [(identifier) (destructor_name) (operator_name)] @name))
      (pointer_declarator (function_declarator declarator: (qualified_identifier scope: [(namespace_identifier) @owner (template_type name: (type_identifier) @owner)] name: [(identifier) (destructor_name) (operator_name)] @name)))
      (reference_declarator (function_declarator declarator: (qualified_identifier scope: [(namespace_identifier) @owner (template_type name: (type_identifier) @owner)] name: [(identifier) (destructor_name) (operator_name)] @name)))]) @symbol.method
