; C# occurrences -> reference kinds (blueprint 9.2, level 1: names only, the resolver decides).
; Declarations are not occurrences. A base type is reported as `inherit`; the resolver turns it
; into `implement` when the name resolves to an interface.

; Calls. `new Foo()` is reported as a type use of Foo (the `type:` field below).
(invocation_expression function: (identifier) @name) @reference.call
(invocation_expression function: (generic_name . (identifier) @name)) @reference.call
(invocation_expression function: (member_access_expression name: (identifier) @name)) @reference.call
(invocation_expression
  function: (member_access_expression name: (generic_name . (identifier) @name))) @reference.call
(invocation_expression
  function: (conditional_access_expression
    (member_binding_expression name: (identifier) @name))) @reference.call

; Inheritance
(base_list (identifier) @name) @reference.inherit
(base_list (generic_name . (identifier) @name)) @reference.inherit
(base_list (qualified_name name: (identifier) @name)) @reference.inherit

; Overrides: the member name is the target
(method_declaration (modifier "override") name: (identifier) @name) @reference.override
(property_declaration (modifier "override") name: (identifier) @name) @reference.override

; Imports
(using_directive . (identifier) @name .) @reference.import
(using_directive (qualified_name) @name) @reference.import

; Type uses
(_ type: (identifier) @name) @reference.type_use
(_ type: (generic_name . (identifier) @name)) @reference.type_use
(_ type: (qualified_name name: (identifier) @name)) @reference.type_use
(_ returns: (identifier) @name) @reference.type_use
(_ returns: (generic_name . (identifier) @name)) @reference.type_use
(_ returns: (qualified_name name: (identifier) @name)) @reference.type_use
(type_argument_list (identifier) @name) @reference.type_use
(type_argument_list (generic_name . (identifier) @name)) @reference.type_use
(type_parameter_constraint (type (identifier) @name)) @reference.type_use

; Writes
(assignment_expression left: (identifier) @name) @reference.write
(assignment_expression left: (member_access_expression name: (identifier) @name)) @reference.write
(postfix_unary_expression (identifier) @name) @reference.write
(prefix_unary_expression ["++" "--"] (identifier) @name) @reference.write

; Reads: names in value positions (not the function of a call, not an assignment target)
([(argument) (return_statement) (arrow_expression_clause)
  (interpolation) (parenthesized_expression) (await_expression) (throw_expression)]
  [(identifier) @name
   (member_access_expression name: (identifier) @name)]) @reference.read
(variable_declarator
  name: (_)
  .
  [(identifier) @name
   (member_access_expression name: (identifier) @name)]) @reference.read
(binary_expression
  [left: (identifier) @name
   right: (identifier) @name
   left: (member_access_expression name: (identifier) @name)
   right: (member_access_expression name: (identifier) @name)]) @reference.read
(assignment_expression
  [right: (identifier) @name
   right: (member_access_expression name: (identifier) @name)]) @reference.read
(conditional_expression
  [condition: (identifier) @name
   consequence: (identifier) @name
   alternative: (identifier) @name]) @reference.read
; The receiver of a member access: a variable or a type
(member_access_expression expression: (identifier) @name) @reference.read
