; Python occurrences -> reference kinds (blueprint 9.2, level 1: names only, the resolver decides).
; Declarations are not occurrences. A base class is reported as `inherit`; the resolver turns it
; into `implement` when the name resolves to an interface (a `Protocol`). Python has no override
; keyword, so there is no `override` occurrence.

; Calls. `Foo()` is a call of the class; the resolver finds the constructor.
(call function: (identifier) @name) @reference.call
(call function: (attribute attribute: (identifier) @name)) @reference.call

; Inheritance
(class_definition superclasses: (argument_list (identifier) @name)) @reference.inherit
(class_definition
  superclasses: (argument_list (attribute attribute: (identifier) @name))) @reference.inherit

; Imports: the module of `import a.b` and `from a.b import c`, and the imported names
(import_statement name: (dotted_name) @name) @reference.import
(import_statement name: (aliased_import name: (dotted_name) @name)) @reference.import
(import_from_statement module_name: (dotted_name) @name) @reference.import
(import_from_statement name: (dotted_name) @name) @reference.import
(import_from_statement name: (aliased_import name: (dotted_name) @name)) @reference.import

; Type uses: annotations (also inside generics such as `list[Foo]` and unions `Foo | None`)
(type (identifier) @name) @reference.type_use
(type (attribute attribute: (identifier) @name)) @reference.type_use
(type (generic_type (identifier) @name)) @reference.type_use
(type (generic_type (type_parameter (type (identifier) @name)))) @reference.type_use
(type (binary_operator left: (identifier) @name)) @reference.type_use
(type (binary_operator right: (identifier) @name)) @reference.type_use
(type (subscript value: (identifier) @name)) @reference.type_use

; Writes
(assignment left: (identifier) @name) @reference.write
(assignment left: (attribute attribute: (identifier) @name)) @reference.write
(augmented_assignment left: (identifier) @name) @reference.write
(augmented_assignment left: (attribute attribute: (identifier) @name)) @reference.write

; Reads: names in value positions (not the function of a call, not an assignment target)
(call arguments: (argument_list (identifier) @name)) @reference.read
(call arguments: (argument_list (attribute attribute: (identifier) @name))) @reference.read
(keyword_argument value: (identifier) @name) @reference.read
(return_statement (identifier) @name) @reference.read
(return_statement (attribute attribute: (identifier) @name)) @reference.read
(assignment right: (identifier) @name) @reference.read
(assignment right: (attribute attribute: (identifier) @name)) @reference.read
(binary_operator [left: (identifier) @name right: (identifier) @name]) @reference.read
(boolean_operator [left: (identifier) @name right: (identifier) @name]) @reference.read
(comparison_operator (identifier) @name) @reference.read
(not_operator argument: (identifier) @name) @reference.read
(conditional_expression (identifier) @name) @reference.read
(parenthesized_expression (identifier) @name) @reference.read
(subscript [value: (identifier) @name subscript: (identifier) @name]) @reference.read
(for_statement right: (identifier) @name) @reference.read
(if_statement condition: (identifier) @name) @reference.read
(expression_statement (identifier) @name) @reference.read
; The receiver of a member access: a variable or a type
(attribute object: (identifier) @name) @reference.read

; ---------------------------------------------------------------------------------------------
; Receivers: what the syntax says about the thing in front of the dot (blueprint 9.2, step 3).
; These patterns add facts to the occurrences above, they do not create occurrences. `self` and
; `cls` are the enclosing object by convention.
(attribute
  object: [((identifier) @receiver.own (#any-of? @receiver.own "self" "cls"))
           (call function: (identifier) @receiver.base (#eq? @receiver.base "super"))
           (call function: (identifier) @receiver.type)
           (identifier) @receiver
           (_) @receiver.other]
  attribute: (identifier) @name)

; ---------------------------------------------------------------------------------------------
; Scopes: nodes in which a declared name is visible. A class body is no scope for its methods.
[(function_definition) (lambda) (list_comprehension) (set_comprehension)
 (dictionary_comprehension) (generator_expression)] @scope

; ---------------------------------------------------------------------------------------------
; Locals and parameters hide members of the same name (step 1). An annotation tells the type of
; a receiver; so does `x = Foo()`.
(parameters (identifier) @local)
(parameters (default_parameter name: (identifier) @local))
(parameters (list_splat_pattern (identifier) @local))
(parameters (dictionary_splat_pattern (identifier) @local))
(lambda_parameters (identifier) @local)
(lambda_parameters (default_parameter name: (identifier) @local))
(parameters
  (typed_parameter
    (identifier) @local
    type: (type [(identifier) @declared_type (generic_type (identifier) @declared_type)
                 (attribute attribute: (identifier) @declared_type)])))
(parameters
  (typed_default_parameter
    name: (identifier) @local
    type: (type [(identifier) @declared_type (generic_type (identifier) @declared_type)
                 (attribute attribute: (identifier) @declared_type)])))

; Assignments in the body of a function. Names assigned in nested blocks are rarer than the
; direct ones; the loop variables and `with ... as` names are listed below.
(function_definition
  body: (block (expression_statement (assignment left: (identifier) @local))))
(function_definition
  body: (block
    (expression_statement
      (assignment
        left: (identifier) @local
        type: (type [(identifier) @declared_type (generic_type (identifier) @declared_type)
                     (attribute attribute: (identifier) @declared_type)])))))
(function_definition
  body: (block
    (expression_statement
      (assignment
        left: (identifier) @local
        right: (call function: (identifier) @declared_type)))))
(for_statement left: (identifier) @local)
(for_in_clause left: (identifier) @local)
(as_pattern alias: (as_pattern_target (identifier) @local))
(except_clause (as_pattern alias: (as_pattern_target (identifier) @local)))
(named_expression name: (identifier) @local)

; Class attributes tell the type of a receiver, but an occurrence of them is a reference.
(class_definition
  body: (block
    (expression_statement
      (assignment
        left: (identifier) @field
        type: (type [(identifier) @declared_type (generic_type (identifier) @declared_type)
                     (attribute attribute: (identifier) @declared_type)])))))
