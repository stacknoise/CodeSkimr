; Java declarations -> symbol kinds (blueprint 8.1, ADR 0025). A Javadoc comment directly above a
; declaration is its documentation. Of the modifiers only the keywords that matter are captured;
; `final` is captured only together with `static`, which makes the field a constant
; (`[modifier_kinds]` in language.toml). A `package` declaration closes with the file.

; Package: the compilation unit closes it.
(program
  (package_declaration [(identifier) (scoped_identifier)] @name) @symbol.namespace) @scope_end

; Types
((block_comment)* @doc
 .
 (class_declaration
   (modifiers ["public" "protected" "private" "static"] @modifier)?
   name: (identifier) @name) @symbol.class)
((block_comment)* @doc
 .
 (interface_declaration
   (modifiers ["public" "protected" "private" "static"] @modifier)?
   name: (identifier) @name) @symbol.interface)
((block_comment)* @doc
 .
 (annotation_type_declaration
   (modifiers ["public" "protected" "private" "static"] @modifier)?
   name: (identifier) @name) @symbol.interface)
((block_comment)* @doc
 .
 (enum_declaration
   (modifiers ["public" "protected" "private" "static"] @modifier)?
   name: (identifier) @name) @symbol.enum)
((block_comment)* @doc
 .
 (record_declaration
   (modifiers ["public" "protected" "private" "static"] @modifier)?
   name: (identifier) @name) @symbol.record)

; Record components are properties; the kind default of language.toml makes them public.
(record_declaration
  parameters: (formal_parameters (formal_parameter name: (identifier) @name) @symbol.property))

; Callables
((block_comment)* @doc
 .
 (method_declaration
   (modifiers ["public" "protected" "private" "static"] @modifier)?
   name: (identifier) @name) @symbol.method)
((block_comment)* @doc
 .
 (annotation_type_element_declaration
   (modifiers ["public" "protected" "private" "static"] @modifier)?
   name: (identifier) @name) @symbol.method)
((block_comment)* @doc
 .
 (constructor_declaration
   (modifiers ["public" "protected" "private"] @modifier)?
   name: (identifier) @name) @symbol.constructor)
((block_comment)* @doc
 .
 (compact_constructor_declaration
   (modifiers ["public" "protected" "private"] @modifier)?
   name: (identifier) @name) @symbol.constructor)

; Data members. One symbol per declarator (`int a, b;`).
((block_comment)* @doc
 .
 (field_declaration
   (modifiers ["public" "protected" "private" "static"] @modifier)?
   declarator: (variable_declarator name: (identifier) @name) @symbol.field))
((block_comment)* @doc
 .
 (field_declaration
   (modifiers "static" @modifier "final" @modifier)
   declarator: (variable_declarator name: (identifier) @name) @symbol.field) @signature)
((block_comment)* @doc
 .
 (field_declaration
   (modifiers "final" @modifier "static" @modifier)
   declarator: (variable_declarator name: (identifier) @name) @symbol.field) @signature)
((block_comment)* @doc
 .
 (constant_declaration
   declarator: (variable_declarator name: (identifier) @name) @symbol.constant) @signature)
((block_comment)* @doc
 .
 (enum_constant name: (identifier) @name) @symbol.enum_member)
