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
; One pattern per parent: an alternation in front of the children never matches (issue #77).
(argument
  [(identifier) @name
   (member_access_expression name: (identifier) @name)]) @reference.read
(return_statement
  [(identifier) @name
   (member_access_expression name: (identifier) @name)]) @reference.read
(arrow_expression_clause
  [(identifier) @name
   (member_access_expression name: (identifier) @name)]) @reference.read
(interpolation
  [(identifier) @name
   (member_access_expression name: (identifier) @name)]) @reference.read
(parenthesized_expression
  [(identifier) @name
   (member_access_expression name: (identifier) @name)]) @reference.read
(await_expression
  [(identifier) @name
   (member_access_expression name: (identifier) @name)]) @reference.read
(throw_expression
  [(identifier) @name
   (member_access_expression name: (identifier) @name)]) @reference.read
(attribute_argument
  [(identifier) @name
   (member_access_expression name: (identifier) @name)]) @reference.read
(constant_pattern
  [(identifier) @name
   (member_access_expression name: (identifier) @name)]) @reference.read
(switch_section
  [(identifier) @name
   (member_access_expression name: (identifier) @name)]) @reference.read
(initializer_expression
  [(identifier) @name
   (member_access_expression name: (identifier) @name)]) @reference.read
(anonymous_object_creation_expression
  [(identifier) @name
   (member_access_expression name: (identifier) @name)]) @reference.read
(yield_statement
  [(identifier) @name
   (member_access_expression name: (identifier) @name)]) @reference.read
(cast_expression
  [(identifier) @name
   (member_access_expression name: (identifier) @name)]) @reference.read
(lambda_expression
  body: [(identifier) @name
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

; ---------------------------------------------------------------------------------------------
; Receivers: what the syntax says about the thing in front of the dot (blueprint 9.2, step 3).
; These patterns add facts to the occurrences above, they do not create occurrences.
(member_access_expression
  expression: ["this" @receiver.own
               "base" @receiver.base
               (identifier) @receiver
               (object_creation_expression type: (identifier) @receiver.type)
               (_) @receiver.other]
  name: [(identifier) @name (generic_name . (identifier) @name)])
(conditional_access_expression
  condition: ["this" @receiver.own
              "base" @receiver.base
              (identifier) @receiver
              (_) @receiver.other]
  (member_binding_expression name: [(identifier) @name (generic_name . (identifier) @name)]))

; ---------------------------------------------------------------------------------------------
; Scopes: nodes in which a declared name is visible.
([(block) (for_statement) (foreach_statement) (using_statement) (lambda_expression)
  (anonymous_method_expression) (method_declaration) (constructor_declaration)
  (operator_declaration) (local_function_statement) (accessor_declaration) (catch_clause)
  (switch_section) (class_declaration) (struct_declaration) (record_declaration)
  (interface_declaration)] @scope)

; ---------------------------------------------------------------------------------------------
; Locals and parameters hide members of the same name (step 1). A declared type, or the type of
; `new Foo()` behind `var`, tells the type of a receiver. A type outside the project is a type
; too; its members are not project symbols. Positional record parameters are properties and are
; not listed here.
(_ parameters: (parameter_list (parameter name: (identifier) @local)))
(_ parameters: (parameter_list
  (parameter
    type: [(identifier) @declared_type
           (generic_name . (identifier) @declared_type)
           (qualified_name name: (identifier) @declared_type)
           (nullable_type [(identifier) @declared_type (generic_name . (identifier) @declared_type)])
           (predefined_type) @declared_type
           (array_type) @declared_type]
    name: (identifier) @local)))
(class_declaration (parameter_list (parameter name: (identifier) @local)))
(struct_declaration (parameter_list (parameter name: (identifier) @local)))
(indexer_declaration parameters: (bracketed_parameter_list (parameter name: (identifier) @local)))
(lambda_expression parameters: (implicit_parameter) @local)

(local_declaration_statement
  (variable_declaration (variable_declarator name: (identifier) @local)))
(for_statement
  initializer: (variable_declaration (variable_declarator name: (identifier) @local)))
(using_statement
  (variable_declaration (variable_declarator name: (identifier) @local)))
(local_declaration_statement
  (variable_declaration
    type: [(identifier) @declared_type
           (generic_name . (identifier) @declared_type)
           (qualified_name name: (identifier) @declared_type)
           (nullable_type [(identifier) @declared_type (generic_name . (identifier) @declared_type)])
           (predefined_type) @declared_type
           (array_type) @declared_type]
    (variable_declarator name: (identifier) @local)))
(for_statement initializer:
  (variable_declaration
    type: [(identifier) @declared_type
           (generic_name . (identifier) @declared_type)
           (qualified_name name: (identifier) @declared_type)
           (nullable_type [(identifier) @declared_type (generic_name . (identifier) @declared_type)])
           (predefined_type) @declared_type
           (array_type) @declared_type]
    (variable_declarator name: (identifier) @local)))
(using_statement
  (variable_declaration
    type: [(identifier) @declared_type
           (generic_name . (identifier) @declared_type)
           (qualified_name name: (identifier) @declared_type)
           (nullable_type [(identifier) @declared_type (generic_name . (identifier) @declared_type)])
           (predefined_type) @declared_type
           (array_type) @declared_type]
    (variable_declarator name: (identifier) @local)))
; `var x = new T()`, `var x = (T)e` and `var x = e as T`: the initializer names the type.
(variable_declaration
  type: (implicit_type)
  (variable_declarator
    name: (identifier) @local
    [(object_creation_expression
       type: [(identifier) @declared_type
              (generic_name . (identifier) @declared_type)
              (qualified_name name: (identifier) @declared_type)])
     (cast_expression
       type: [(identifier) @declared_type
              (generic_name . (identifier) @declared_type)
              (qualified_name name: (identifier) @declared_type)])
     (as_expression
       right: [(identifier) @declared_type
               (generic_name . (identifier) @declared_type)
               (qualified_name name: (identifier) @declared_type)])]))
(foreach_statement left: (identifier) @local)
(foreach_statement
  type: [(identifier) @declared_type
         (generic_name . (identifier) @declared_type)
         (qualified_name name: (identifier) @declared_type)
         (predefined_type) @declared_type]
  left: (identifier) @local)
(catch_declaration name: (identifier) @local)
(catch_declaration
  type: [(identifier) @declared_type (qualified_name name: (identifier) @declared_type)]
  name: (identifier) @local)
(declaration_pattern name: (identifier) @local)
(declaration_expression name: (identifier) @local)

; Fields and properties tell the type of a receiver, but an occurrence of them is a reference.
(field_declaration
  (variable_declaration
    type: [(identifier) @declared_type
           (generic_name . (identifier) @declared_type)
           (qualified_name name: (identifier) @declared_type)
           (nullable_type [(identifier) @declared_type (generic_name . (identifier) @declared_type)])
           (predefined_type) @declared_type
           (array_type) @declared_type]
    (variable_declarator name: (identifier) @field)))
(property_declaration
  type: [(identifier) @declared_type
         (generic_name . (identifier) @declared_type)
         (qualified_name name: (identifier) @declared_type)
         (nullable_type [(identifier) @declared_type (generic_name . (identifier) @declared_type)])
         (predefined_type) @declared_type
         (array_type) @declared_type]
  name: (identifier) @field)
