; Kotlin declarations -> symbol kinds (blueprint 8.1, ADR 0027). A KDoc comment directly above a
; declaration is its documentation. Of the modifiers only the keywords that matter are captured:
; the visibility, and the class and property modifiers that change the kind (`enum` is a keyword
; of its own in this grammar; `annotation`, `data`, `const`; see `[modifier_kinds]` in
; language.toml). A `package` header closes with the file. Alternatives are written inside their parent node (issues #77 and #81).

; Package: the file closes it.
(source_file
  (package_header (identifier) @name) @symbol.namespace) @scope_end

; Types
((multiline_comment)* @doc
 .
 (class_declaration
   (modifiers [(visibility_modifier) (class_modifier)] @modifier)?
   "enum"? @modifier
   "class"
   (type_identifier) @name) @symbol.class)
((multiline_comment)* @doc
 .
 (class_declaration
   (modifiers [(visibility_modifier) (class_modifier)] @modifier)?
   "interface"
   (type_identifier) @name) @symbol.interface)
((multiline_comment)* @doc
 .
 (object_declaration
   (modifiers (visibility_modifier) @modifier)?
   (type_identifier) @name) @symbol.class)
; A companion object has a name only when it is written; without one its members belong to the
; surrounding class.
((multiline_comment)* @doc
 .
 (companion_object
   (modifiers (visibility_modifier) @modifier)?
   (type_identifier) @name) @symbol.class)
((multiline_comment)* @doc
 .
 (type_alias
   (modifiers (visibility_modifier) @modifier)?
   (type_identifier) @name) @symbol.type_alias)

; Callables. A function of a type becomes a method (`[kinds_by_parent]` in language.toml).
((multiline_comment)* @doc
 .
 (function_declaration
   (modifiers (visibility_modifier) @modifier)?
   (simple_identifier) @name) @symbol.function)
; A secondary constructor is named like its class.
(class_declaration
  (type_identifier) @name
  [(class_body (multiline_comment)+ @doc . (secondary_constructor
                                         (modifiers (visibility_modifier) @modifier)?) @symbol.constructor)
   (class_body (secondary_constructor
                 (modifiers (visibility_modifier) @modifier)?) @symbol.constructor)
   (enum_class_body (secondary_constructor
                      (modifiers (visibility_modifier) @modifier)?) @symbol.constructor)])

; Properties: declared in a file, a class, an object or an enum, never inside a function. A
; `const` property is a constant.
[(source_file
   (multiline_comment)+ @doc
   .
   (property_declaration
     (modifiers [(visibility_modifier) (property_modifier)] @modifier)?
     (variable_declaration (simple_identifier) @name)) @symbol.property)
 (class_body
   (multiline_comment)+ @doc
   .
   (property_declaration
     (modifiers [(visibility_modifier) (property_modifier)] @modifier)?
     (variable_declaration (simple_identifier) @name)) @symbol.property)
 (enum_class_body
   (multiline_comment)+ @doc
   .
   (property_declaration
     (modifiers [(visibility_modifier) (property_modifier)] @modifier)?
     (variable_declaration (simple_identifier) @name)) @symbol.property)]
[(source_file
   (property_declaration
     (modifiers [(visibility_modifier) (property_modifier)] @modifier)?
     (variable_declaration (simple_identifier) @name)) @symbol.property)
 (class_body
   (property_declaration
     (modifiers [(visibility_modifier) (property_modifier)] @modifier)?
     (variable_declaration (simple_identifier) @name)) @symbol.property)
 (enum_class_body
   (property_declaration
     (modifiers [(visibility_modifier) (property_modifier)] @modifier)?
     (variable_declaration (simple_identifier) @name)) @symbol.property)]

; Parameters of the primary constructor that say `val` or `var` are properties.
(primary_constructor
  (class_parameter
    (modifiers (visibility_modifier) @modifier)?
    (binding_pattern_kind)
    .
    (simple_identifier) @name) @symbol.property)

; Enum entries
(enum_class_body
  (enum_entry (simple_identifier) @name) @symbol.enum_member)
