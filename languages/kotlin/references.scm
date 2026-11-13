; Kotlin occurrences -> reference kinds (blueprint 9.2, level 1: names only, the resolver decides).
; Declarations are not occurrences. A base type in the supertype list is reported as `inherit`; the
; resolver turns it into `implement` when the name resolves to an interface. `Foo()` is a call of
; Foo (the resolver finds the constructor, which is named like its class). Alternatives are always
; written inside their parent node (issues #77 and #81). A qualified type `a.b.C` reports every
; part of its name as a type use.

; Calls: `f(x)` and `receiver.f(x)`
(call_expression (simple_identifier) @name) @reference.call
(call_expression (navigation_expression (navigation_suffix (simple_identifier) @name))) @reference.call

; Inheritance: the supertypes of a class or object, also `Foo by delegate`
(delegation_specifier
  [(user_type (type_identifier) @name)
   (constructor_invocation (user_type (type_identifier) @name))
   (explicit_delegation (user_type (type_identifier) @name))]) @reference.inherit

; Overrides: the member name is the target
(function_declaration
  (modifiers (member_modifier) @modifier (#eq? @modifier "override"))
  (simple_identifier) @name) @reference.override

; Imports: the imported name, not a wildcard import; an alias import reports the original name
(import_header (identifier (simple_identifier) @name .) .) @reference.import
(import_header (identifier (simple_identifier) @name .) (import_alias)) @reference.import

; Type uses
(parameter (user_type (type_identifier) @name)) @reference.type_use
(class_parameter (user_type (type_identifier) @name)) @reference.type_use
(variable_declaration (user_type (type_identifier) @name)) @reference.type_use
(function_declaration (user_type (type_identifier) @name)) @reference.type_use
(function_declaration (receiver_type (user_type (type_identifier) @name))) @reference.type_use
(property_declaration (receiver_type (user_type (type_identifier) @name))) @reference.type_use
(nullable_type (user_type (type_identifier) @name)) @reference.type_use
(type_projection (user_type (type_identifier) @name)) @reference.type_use
(type_constraint (user_type (type_identifier) @name)) @reference.type_use
(type_parameter (user_type (type_identifier) @name)) @reference.type_use
(type_alias (user_type (type_identifier) @name)) @reference.type_use
(check_expression (user_type (type_identifier) @name)) @reference.type_use
(as_expression (user_type (type_identifier) @name)) @reference.type_use
(type_test (user_type (type_identifier) @name)) @reference.type_use
(callable_reference . (type_identifier) @name) @reference.type_use
(annotation [(user_type (type_identifier) @name)
             (constructor_invocation (user_type (type_identifier) @name))]) @reference.type_use

; Writes
(assignment (directly_assignable_expression (simple_identifier) @name)) @reference.write
(assignment
  (directly_assignable_expression (navigation_suffix (simple_identifier) @name) .)) @reference.write
(postfix_expression (simple_identifier) @name ["++" "--"]) @reference.write
(prefix_expression ["++" "--"] (simple_identifier) @name) @reference.write

; Reads: names in value positions (not the callee of a call, not an assignment target)
(value_argument
  [(simple_identifier) @name
   (navigation_expression (navigation_suffix (simple_identifier) @name .))]) @reference.read
(jump_expression
  [(simple_identifier) @name
   (navigation_expression (navigation_suffix (simple_identifier) @name .))]) @reference.read
(property_declaration
  [(simple_identifier) @name
   (navigation_expression (navigation_suffix (simple_identifier) @name .))]) @reference.read
(assignment
  [(simple_identifier) @name
   (navigation_expression (navigation_suffix (simple_identifier) @name .))]) @reference.read
(parenthesized_expression [(simple_identifier) @name]) @reference.read
(comparison_expression [(simple_identifier) @name]) @reference.read
(equality_expression [(simple_identifier) @name]) @reference.read
(conjunction_expression [(simple_identifier) @name]) @reference.read
(disjunction_expression [(simple_identifier) @name]) @reference.read
(elvis_expression [(simple_identifier) @name]) @reference.read
(additive_expression [(simple_identifier) @name]) @reference.read
(multiplicative_expression [(simple_identifier) @name]) @reference.read
(prefix_expression [(simple_identifier) @name]) @reference.read
(indexing_suffix [(simple_identifier) @name]) @reference.read
(if_expression condition: (simple_identifier) @name) @reference.read
(when_subject (simple_identifier) @name) @reference.read
(for_statement "in" . (simple_identifier) @name) @reference.read
(control_structure_body (simple_identifier) @name) @reference.read
(statements (simple_identifier) @name) @reference.read
(function_body (simple_identifier) @name) @reference.read
(check_expression . (simple_identifier) @name) @reference.read
(as_expression . (simple_identifier) @name) @reference.read
(callable_reference "::" . (simple_identifier) @name) @reference.read
; The receiver of a member access: a variable or a type
(navigation_expression . (simple_identifier) @name) @reference.read

; ---------------------------------------------------------------------------------------------
; Receivers: what the syntax says about the thing in front of the dot (blueprint 9.2, step 3).
; These patterns add facts to the occurrences above, they do not create occurrences.
(navigation_expression
  .
  [(this_expression) @receiver.own
   (super_expression) @receiver.base
   (simple_identifier) @receiver
   (_) @receiver.other]
  .
  (navigation_suffix (simple_identifier) @name))

; ---------------------------------------------------------------------------------------------
; Scopes: nodes in which a declared name is visible.
[(statements) (function_declaration) (secondary_constructor) (anonymous_function)
 (lambda_literal) (for_statement) (catch_block) (class_declaration) (object_declaration)
 (companion_object)] @scope

; ---------------------------------------------------------------------------------------------
; Locals and parameters hide members of the same name (step 1). A declared type, or the type of
; `val x = Foo()` (a capitalized call), tells the type of a receiver.
(parameter (simple_identifier) @local)
(parameter
  (simple_identifier) @local
  .
  [(user_type (type_identifier) @declared_type)
   (nullable_type (user_type (type_identifier) @declared_type))])
(lambda_parameters (variable_declaration (simple_identifier) @local))
(statements (property_declaration (variable_declaration (simple_identifier) @local)))
(statements
  (property_declaration
    (variable_declaration
      (simple_identifier) @local
      .
      [(user_type (type_identifier) @declared_type)
       (nullable_type (user_type (type_identifier) @declared_type))])))
(statements
  (property_declaration
    (variable_declaration (simple_identifier) @local)
    (call_expression (simple_identifier) @declared_type (#match? @declared_type "^[A-Z]"))))
(for_statement (variable_declaration (simple_identifier) @local))
(catch_block (simple_identifier) @local)

; Fields tell the type of a receiver, but an occurrence of them is a reference.
(class_body
  (property_declaration
    (variable_declaration
      (simple_identifier) @field
      .
      [(user_type (type_identifier) @declared_type)
       (nullable_type (user_type (type_identifier) @declared_type))])))
(class_body
  (property_declaration
    (variable_declaration (simple_identifier) @field)
    (call_expression (simple_identifier) @declared_type (#match? @declared_type "^[A-Z]"))))
(primary_constructor
  (class_parameter
    (binding_pattern_kind)
    .
    (simple_identifier) @field
    .
    [(user_type (type_identifier) @declared_type)
     (nullable_type (user_type (type_identifier) @declared_type))]))
