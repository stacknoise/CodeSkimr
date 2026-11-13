; Java occurrences -> reference kinds (blueprint 9.2, level 1: names only, the resolver decides).
; Declarations are not occurrences. A base type is reported as `inherit`; the resolver turns it
; into `implement` when the name resolves to an interface. `new Foo()` is reported as a type use of
; Foo (the `type:` field below), like `new Foo()` in C#. Alternatives are always written inside
; their parent node.

; Calls
(method_invocation name: (identifier) @name) @reference.call

; Inheritance: `extends` of a class, `implements` of a class or enum, `extends` of an interface
(superclass [(type_identifier) @name
             (generic_type . (type_identifier) @name)
             (scoped_type_identifier (type_identifier) @name .)]) @reference.inherit
(super_interfaces
  (type_list [(type_identifier) @name
              (generic_type . (type_identifier) @name)
              (scoped_type_identifier (type_identifier) @name .)])) @reference.inherit
(extends_interfaces
  (type_list [(type_identifier) @name
              (generic_type . (type_identifier) @name)
              (scoped_type_identifier (type_identifier) @name .)])) @reference.inherit

; Overrides: the member name is the target
(method_declaration
  (modifiers (marker_annotation) @modifier (#eq? @modifier "@Override"))
  name: (identifier) @name) @reference.override

; Imports: the imported type or member (not a wildcard import, whose last child is the asterisk)
(import_declaration (scoped_identifier name: (identifier) @name) .) @reference.import
(import_declaration (identifier) @name .) @reference.import

; Type uses
(_ type: [(type_identifier) @name
          (generic_type . (type_identifier) @name)
          (scoped_type_identifier (type_identifier) @name .)]
   (#not-eq? @name "var")) @reference.type_use
(_ type: (array_type element: [(type_identifier) @name
                               (generic_type . (type_identifier) @name)])) @reference.type_use
(type_arguments [(type_identifier) @name
                 (generic_type . (type_identifier) @name)
                 (scoped_type_identifier (type_identifier) @name .)]) @reference.type_use
(type_bound [(type_identifier) @name (generic_type . (type_identifier) @name)]) @reference.type_use
(throws [(type_identifier) @name (scoped_type_identifier (type_identifier) @name .)]) @reference.type_use
(catch_type [(type_identifier) @name (scoped_type_identifier (type_identifier) @name .)]) @reference.type_use
(instanceof_expression right: [(type_identifier) @name (generic_type . (type_identifier) @name)]) @reference.type_use
(class_literal (type_identifier) @name) @reference.type_use
(method_reference . (type_identifier) @name) @reference.type_use
(marker_annotation name: (identifier) @name) @reference.type_use
(annotation name: (identifier) @name) @reference.type_use

; Writes
(assignment_expression left: (identifier) @name) @reference.write
(assignment_expression left: (field_access field: (identifier) @name)) @reference.write
(update_expression (identifier) @name) @reference.write
(update_expression (field_access field: (identifier) @name)) @reference.write

; Reads: names in value positions (not the name of a call, not an assignment target)
(argument_list [(identifier) @name (field_access field: (identifier) @name)]) @reference.read
(return_statement [(identifier) @name (field_access field: (identifier) @name)]) @reference.read
(variable_declarator value: [(identifier) @name (field_access field: (identifier) @name)]) @reference.read
(parenthesized_expression [(identifier) @name (field_access field: (identifier) @name)]) @reference.read
(binary_expression
  [left: (identifier) @name
   right: (identifier) @name
   left: (field_access field: (identifier) @name)
   right: (field_access field: (identifier) @name)]) @reference.read
(assignment_expression
  [right: (identifier) @name
   right: (field_access field: (identifier) @name)]) @reference.read
(ternary_expression
  [condition: (identifier) @name
   consequence: (identifier) @name
   alternative: (identifier) @name]) @reference.read
(unary_expression operand: (identifier) @name) @reference.read
(array_access [array: (identifier) @name index: (identifier) @name]) @reference.read
(cast_expression value: (identifier) @name) @reference.read
(enhanced_for_statement value: (identifier) @name) @reference.read
(lambda_expression body: (identifier) @name) @reference.read
; The receiver of a member access: a variable or a type
(field_access object: (identifier) @name) @reference.read
(method_invocation object: (identifier) @name) @reference.read

; ---------------------------------------------------------------------------------------------
; Receivers: what the syntax says about the thing in front of the dot (blueprint 9.2, step 3).
; These patterns add facts to the occurrences above, they do not create occurrences.
(method_invocation
  object: [(this) @receiver.own
           (super) @receiver.base
           (identifier) @receiver
           (object_creation_expression
             type: [(type_identifier) @receiver.type (generic_type . (type_identifier) @receiver.type)])
           (_) @receiver.other]
  name: (identifier) @name)
(field_access
  object: [(this) @receiver.own
           (super) @receiver.base
           (identifier) @receiver
           (_) @receiver.other]
  field: (identifier) @name)

; ---------------------------------------------------------------------------------------------
; Scopes: nodes in which a declared name is visible.
[(block) (method_declaration) (constructor_declaration) (compact_constructor_declaration)
 (lambda_expression) (for_statement) (enhanced_for_statement) (catch_clause)
 (try_with_resources_statement) (switch_block) (class_declaration) (interface_declaration)
 (enum_declaration) (record_declaration)] @scope

; ---------------------------------------------------------------------------------------------
; Locals and parameters hide members of the same name (step 1). A declared type, or the type of
; `new Foo()`, tells the type of a receiver. A type outside the project is a type too; its
; members are not project symbols. Record components are properties and are not listed here.
(formal_parameter name: (identifier) @local)
(spread_parameter (variable_declarator name: (identifier) @local))
(formal_parameter
  type: [(type_identifier) @declared_type
         (generic_type . (type_identifier) @declared_type)
         (scoped_type_identifier (type_identifier) @declared_type .)
         (array_type) @declared_type
         (integral_type) @declared_type
         (floating_point_type) @declared_type
         (boolean_type) @declared_type]
  name: (identifier) @local)
(lambda_expression parameters: (identifier) @local)
(lambda_expression parameters: (inferred_parameters (identifier) @local))

(local_variable_declaration declarator: (variable_declarator name: (identifier) @local))
(local_variable_declaration
  type: [(type_identifier) @declared_type
         (generic_type . (type_identifier) @declared_type)
         (scoped_type_identifier (type_identifier) @declared_type .)
         (array_type) @declared_type
         (integral_type) @declared_type
         (floating_point_type) @declared_type
         (boolean_type) @declared_type]
  declarator: (variable_declarator name: (identifier) @local))
(local_variable_declaration
  type: (type_identifier) @modifier (#eq? @modifier "var")
  declarator: (variable_declarator
    name: (identifier) @local
    value: (object_creation_expression
             type: [(type_identifier) @declared_type
                    (generic_type . (type_identifier) @declared_type)])))
(enhanced_for_statement name: (identifier) @local)
(enhanced_for_statement
  type: [(type_identifier) @declared_type
         (generic_type . (type_identifier) @declared_type)
         (integral_type) @declared_type]
  name: (identifier) @local)
(catch_formal_parameter name: (identifier) @local)
(resource name: (identifier) @local)
(instanceof_expression name: (identifier) @local)
(type_pattern (identifier) @local)

; Fields tell the type of a receiver, but an occurrence of them is a reference.
(field_declaration
  type: [(type_identifier) @declared_type
         (generic_type . (type_identifier) @declared_type)
         (scoped_type_identifier (type_identifier) @declared_type .)
         (array_type) @declared_type
         (integral_type) @declared_type
         (floating_point_type) @declared_type
         (boolean_type) @declared_type]
  declarator: (variable_declarator name: (identifier) @field))
