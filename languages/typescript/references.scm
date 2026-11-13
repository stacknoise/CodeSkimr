; TypeScript occurrences -> reference kinds (blueprint 9.2, level 1: names only, the resolver
; decides). Declarations are not occurrences. A class `extends` is reported as `inherit` (the
; resolver turns it into `implement` when the name resolves to an interface), `implements` as
; `implement`.

; Calls. `new Foo()` is reported as a type use of Foo.
(call_expression function: (identifier) @name) @reference.call
(call_expression function: (member_expression property: (property_identifier) @name)) @reference.call
(new_expression constructor: (identifier) @name) @reference.type_use
(new_expression constructor: (member_expression property: (property_identifier) @name)) @reference.type_use

; Inheritance
(class_heritage (extends_clause value: (identifier) @name)) @reference.inherit
(class_heritage (extends_clause value: (member_expression property: (property_identifier) @name))) @reference.inherit
(implements_clause (type_identifier) @name) @reference.implement
(implements_clause (generic_type name: (type_identifier) @name)) @reference.implement
(implements_clause (nested_type_identifier name: (type_identifier) @name)) @reference.implement
(extends_type_clause type: (type_identifier) @name) @reference.inherit
(extends_type_clause type: (generic_type name: (type_identifier) @name)) @reference.inherit

; Overrides: the member name is the target
(method_definition (override_modifier) name: (property_identifier) @name) @reference.override
(public_field_definition (override_modifier) name: (property_identifier) @name) @reference.override

; Imports: the imported names (`import X`, `import { X as Y }`, `import * as X`)
(import_clause (identifier) @name) @reference.import
(import_specifier name: (identifier) @name) @reference.import
(namespace_import (identifier) @name) @reference.import

; Type uses
[(type_annotation [(type_identifier) @name (generic_type name: (type_identifier) @name) (nested_type_identifier name: (type_identifier) @name)])
   (type_arguments [(type_identifier) @name (generic_type name: (type_identifier) @name) (nested_type_identifier name: (type_identifier) @name)])
   (union_type [(type_identifier) @name (generic_type name: (type_identifier) @name) (nested_type_identifier name: (type_identifier) @name)])
   (intersection_type [(type_identifier) @name (generic_type name: (type_identifier) @name) (nested_type_identifier name: (type_identifier) @name)])
   (array_type [(type_identifier) @name (generic_type name: (type_identifier) @name) (nested_type_identifier name: (type_identifier) @name)])
   (tuple_type [(type_identifier) @name (generic_type name: (type_identifier) @name) (nested_type_identifier name: (type_identifier) @name)])
   (optional_type [(type_identifier) @name (generic_type name: (type_identifier) @name) (nested_type_identifier name: (type_identifier) @name)])
   (rest_type [(type_identifier) @name (generic_type name: (type_identifier) @name) (nested_type_identifier name: (type_identifier) @name)])
   (parenthesized_type [(type_identifier) @name (generic_type name: (type_identifier) @name) (nested_type_identifier name: (type_identifier) @name)])
   (readonly_type [(type_identifier) @name (generic_type name: (type_identifier) @name) (nested_type_identifier name: (type_identifier) @name)])
   (conditional_type [(type_identifier) @name (generic_type name: (type_identifier) @name) (nested_type_identifier name: (type_identifier) @name)])
   (type_predicate [(type_identifier) @name (generic_type name: (type_identifier) @name) (nested_type_identifier name: (type_identifier) @name)])
   (constraint [(type_identifier) @name (generic_type name: (type_identifier) @name) (nested_type_identifier name: (type_identifier) @name)])
   (default_type [(type_identifier) @name (generic_type name: (type_identifier) @name) (nested_type_identifier name: (type_identifier) @name)])
   (as_expression [(type_identifier) @name (generic_type name: (type_identifier) @name) (nested_type_identifier name: (type_identifier) @name)])
   (satisfies_expression [(type_identifier) @name (generic_type name: (type_identifier) @name) (nested_type_identifier name: (type_identifier) @name)])
   (index_type_query [(type_identifier) @name (generic_type name: (type_identifier) @name) (nested_type_identifier name: (type_identifier) @name)])
   (lookup_type [(type_identifier) @name (generic_type name: (type_identifier) @name) (nested_type_identifier name: (type_identifier) @name)])] @reference.type_use
(type_alias_declaration value: [(type_identifier) @name
                                (generic_type name: (type_identifier) @name)]) @reference.type_use

; Writes
(assignment_expression left: (identifier) @name) @reference.write
(assignment_expression left: (member_expression property: (property_identifier) @name)) @reference.write
(augmented_assignment_expression left: (identifier) @name) @reference.write
(augmented_assignment_expression left: (member_expression property: (property_identifier) @name)) @reference.write
(update_expression argument: (identifier) @name) @reference.write
(update_expression argument: (member_expression property: (property_identifier) @name)) @reference.write

; Reads: names in value positions (not the function of a call, not an assignment target)
[(arguments [(identifier) @name (member_expression property: (property_identifier) @name)])
   (return_statement [(identifier) @name (member_expression property: (property_identifier) @name)])
   (arrow_function [(identifier) @name (member_expression property: (property_identifier) @name)])
   (template_substitution [(identifier) @name (member_expression property: (property_identifier) @name)])
   (parenthesized_expression [(identifier) @name (member_expression property: (property_identifier) @name)])
   (await_expression [(identifier) @name (member_expression property: (property_identifier) @name)])
   (throw_statement [(identifier) @name (member_expression property: (property_identifier) @name)])
   (array [(identifier) @name (member_expression property: (property_identifier) @name)])
   (spread_element [(identifier) @name (member_expression property: (property_identifier) @name)])
   (pair [(identifier) @name (member_expression property: (property_identifier) @name)])
   (yield_expression [(identifier) @name (member_expression property: (property_identifier) @name)])
   (non_null_expression [(identifier) @name (member_expression property: (property_identifier) @name)])
   (as_expression [(identifier) @name (member_expression property: (property_identifier) @name)])
   (satisfies_expression [(identifier) @name (member_expression property: (property_identifier) @name)])
   (unary_expression [(identifier) @name (member_expression property: (property_identifier) @name)])
   (subscript_expression [(identifier) @name (member_expression property: (property_identifier) @name)])] @reference.read
(variable_declarator
  name: (_)
  value: [(identifier) @name
          (member_expression property: (property_identifier) @name)]) @reference.read
(binary_expression
  [left: (identifier) @name
   right: (identifier) @name
   left: (member_expression property: (property_identifier) @name)
   right: (member_expression property: (property_identifier) @name)]) @reference.read
(assignment_expression
  [right: (identifier) @name
   right: (member_expression property: (property_identifier) @name)]) @reference.read
(ternary_expression
  [condition: (identifier) @name
   consequence: (identifier) @name
   alternative: (identifier) @name]) @reference.read
; The receiver of a member access: a variable or a type
(member_expression object: (identifier) @name) @reference.read

; ---------------------------------------------------------------------------------------------
; Receivers: what the syntax says about the thing in front of the dot (blueprint 9.2, step 3).
; These patterns add facts to the occurrences above, they do not create occurrences.
(member_expression
  object: [(this) @receiver.own
           (super) @receiver.base
           (identifier) @receiver
           (new_expression constructor: (identifier) @receiver.type)
           (_) @receiver.other]
  property: (property_identifier) @name)

; ---------------------------------------------------------------------------------------------
; Scopes: nodes in which a declared name is visible.
([(statement_block) (for_statement) (for_in_statement) (arrow_function) (function_expression)
  (function_declaration) (generator_function_declaration) (method_definition) (catch_clause)
  (class_declaration) (abstract_class_declaration) (interface_declaration)] @scope)

; ---------------------------------------------------------------------------------------------
; Locals and parameters hide members of the same name (step 1). A declared type, or the type of
; `new Foo()`, tells the type of a receiver. Variables of the module itself are symbols, not
; locals, so only declarations inside a block count.
(_ parameters: (formal_parameters (required_parameter pattern: (identifier) @local)))
(_ parameters: (formal_parameters (optional_parameter pattern: (identifier) @local)))
(_ parameters: (formal_parameters
  (required_parameter
    pattern: (identifier) @local
    type: (type_annotation [(type_identifier) @declared_type
                            (generic_type name: (type_identifier) @declared_type)
                            (predefined_type) @declared_type
                            (array_type) @declared_type]))))
(_ parameters: (formal_parameters
  (optional_parameter
    pattern: (identifier) @local
    type: (type_annotation [(type_identifier) @declared_type
                            (generic_type name: (type_identifier) @declared_type)
                            (predefined_type) @declared_type
                            (array_type) @declared_type]))))
(arrow_function parameter: (identifier) @local)
(catch_clause parameter: (identifier) @local)
(statement_block
  (lexical_declaration (variable_declarator name: (identifier) @local)))
(statement_block
  (variable_declaration (variable_declarator name: (identifier) @local)))
(statement_block
  (lexical_declaration
    (variable_declarator
      name: (identifier) @local
      type: (type_annotation [(type_identifier) @declared_type
                              (generic_type name: (type_identifier) @declared_type)
                              (predefined_type) @declared_type
                              (array_type) @declared_type]))))
(statement_block
  (lexical_declaration
    (variable_declarator
      name: (identifier) @local
      value: (new_expression constructor: (identifier) @declared_type))))
(for_statement initializer: (lexical_declaration (variable_declarator name: (identifier) @local)))
(for_in_statement left: (identifier) @local)

; Fields and parameter properties tell the type of a receiver, but an occurrence of them is a
; reference.
(public_field_definition
  name: (property_identifier) @field
  type: (type_annotation [(type_identifier) @declared_type
                          (generic_type name: (type_identifier) @declared_type)
                          (predefined_type) @declared_type
                          (array_type) @declared_type]))
(public_field_definition
  name: (property_identifier) @field
  value: (new_expression constructor: (identifier) @declared_type))
(required_parameter
  (accessibility_modifier)
  pattern: (identifier) @field
  type: (type_annotation [(type_identifier) @declared_type
                          (generic_type name: (type_identifier) @declared_type)
                          (predefined_type) @declared_type
                          (array_type) @declared_type]))
