; C# declarations -> symbol kinds (blueprint 8.1). One pattern per declaration kind; `///` lines
; directly above a declaration are its documentation, modifiers give visibility and static.

; Namespaces. A file-scoped namespace holds the rest of the file, so the compilation unit closes it.
(namespace_declaration name: (_) @name) @symbol.namespace
(compilation_unit
  (file_scoped_namespace_declaration name: (_) @name) @symbol.namespace) @scope_end

; Types
((comment)* @doc
 .
 (class_declaration (modifier)* @modifier name: (identifier) @name) @symbol.class)
((comment)* @doc
 .
 (interface_declaration (modifier)* @modifier name: (identifier) @name) @symbol.interface)
((comment)* @doc
 .
 (struct_declaration (modifier)* @modifier name: (identifier) @name) @symbol.struct)
((comment)* @doc
 .
 (record_declaration (modifier)* @modifier name: (identifier) @name) @symbol.record)
((comment)* @doc
 .
 (enum_declaration (modifier)* @modifier name: (identifier) @name) @symbol.enum)
((comment)* @doc
 .
 (delegate_declaration (modifier)* @modifier name: (identifier) @name) @symbol.type_alias)

; Positional record parameters are public init properties; the opening parenthesis stands for the
; implicit `public` (see `[visibility.keywords]` in language.toml).
(record_declaration
  (parameter_list "(" @modifier (parameter name: (identifier) @name) @symbol.property))

; Callables
((comment)* @doc
 .
 (method_declaration (modifier)* @modifier name: (identifier) @name) @symbol.method)
((comment)* @doc
 .
 (operator_declaration (modifier)* @modifier operator: _ @name) @symbol.method)
((comment)* @doc
 .
 (constructor_declaration (modifier)* @modifier name: (identifier) @name) @symbol.constructor)
(local_function_statement (modifier)* @modifier name: (identifier) @name) @symbol.function

; Data members
((comment)* @doc
 .
 (property_declaration (modifier)* @modifier name: (identifier) @name) @symbol.property)
((comment)* @doc
 .
 (indexer_declaration (modifier)* @modifier "this" @name) @symbol.property)
((comment)* @doc
 .
 (event_declaration (modifier)* @modifier name: (identifier) @name) @symbol.event)
((comment)* @doc
 .
 (event_field_declaration
   (modifier)* @modifier
   (variable_declaration (variable_declarator name: (identifier) @name) @symbol.event)))
((comment)* @doc
 .
 (field_declaration
   (modifier)* @modifier
   (variable_declaration (variable_declarator name: (identifier) @name) @symbol.field)))
; A `const` field shows its whole declaration (modifiers, type and value) as signature (issue #123).
((comment)* @doc
 .
 (field_declaration
   (modifier "const")
   (variable_declaration (variable_declarator name: (identifier) @name) @symbol.field)) @signature)
((comment)* @doc
 .
 (enum_member_declaration name: (identifier) @name) @symbol.enum_member)
