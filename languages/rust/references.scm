; Rust occurrences -> reference kinds (blueprint 9.2, level 1: names only, the resolver decides).
; Declarations are not occurrences. `impl` blocks are no symbols (ADR 0024), so the occurrences in
; the head of an `impl` (the trait that is implemented, the type) belong to the enclosing module,
; and those in a function to the function. A supertrait is reported as `inherit` from the trait, a
; trait that is implemented as `implement`; a function of a trait implementation is an `override`
; of the trait function of that name. `Foo::new()` is a call with the receiver `Foo`, `Foo { .. }`
; a type use of Foo.

; Calls
(call_expression function: (identifier) @name) @reference.call
(call_expression function: (field_expression field: (field_identifier) @name)) @reference.call
(call_expression function: (scoped_identifier name: (identifier) @name)) @reference.call
(call_expression
  function: (generic_function
    function: [(identifier) @name
               (field_expression field: (field_identifier) @name)
               (scoped_identifier name: (identifier) @name)])) @reference.call

; Inheritance and implementation
(trait_item
  bounds: (trait_bounds [(type_identifier) @name
                         (generic_type type: (type_identifier) @name)
                         (scoped_type_identifier name: (type_identifier) @name)])) @reference.inherit
(impl_item
  trait: [(type_identifier) @name
          (generic_type type: (type_identifier) @name)
          (scoped_type_identifier name: (type_identifier) @name)]) @reference.implement
(impl_item "for" body: (declaration_list (function_item name: (identifier) @name) @reference.override))

; Imports: the imported names (`use a::B`, `use a::{B, c::D}`, `use a::B as E`)
(use_declaration argument: (identifier) @name) @reference.import
(use_declaration argument: (scoped_identifier name: (identifier) @name)) @reference.import
(use_declaration argument: (use_as_clause path: (identifier) @name)) @reference.import
(use_declaration
  argument: (use_as_clause path: (scoped_identifier name: (identifier) @name))) @reference.import
(use_declaration
  argument: (scoped_use_list
    list: (use_list [(identifier) @name
                     (scoped_identifier name: (identifier) @name)
                     (use_as_clause path: (identifier) @name)
                     (use_as_clause path: (scoped_identifier name: (identifier) @name))]))) @reference.import

; Type uses: annotations, field and return types, generic arguments, struct expressions, the type
; of an `impl` and the bounds of a type parameter. The name of a generic type (`Vec` in
; `Vec<Foo>`) counts as well; the traits of `impl` and of supertrait lists have patterns of their own.
[(parameter type: [(type_identifier) @name (scoped_type_identifier name: (type_identifier) @name) (generic_type type: (type_identifier) @name)])
 (field_declaration type: [(type_identifier) @name (scoped_type_identifier name: (type_identifier) @name) (generic_type type: (type_identifier) @name)])
 (let_declaration type: [(type_identifier) @name (scoped_type_identifier name: (type_identifier) @name) (generic_type type: (type_identifier) @name)])
 (const_item type: [(type_identifier) @name (scoped_type_identifier name: (type_identifier) @name) (generic_type type: (type_identifier) @name)])
 (static_item type: [(type_identifier) @name (scoped_type_identifier name: (type_identifier) @name) (generic_type type: (type_identifier) @name)])
 (type_item type: [(type_identifier) @name (scoped_type_identifier name: (type_identifier) @name) (generic_type type: (type_identifier) @name)])
 (impl_item type: [(type_identifier) @name (scoped_type_identifier name: (type_identifier) @name) (generic_type type: (type_identifier) @name)])
 (reference_type type: [(type_identifier) @name (scoped_type_identifier name: (type_identifier) @name) (generic_type type: (type_identifier) @name)])
 (pointer_type type: [(type_identifier) @name (scoped_type_identifier name: (type_identifier) @name) (generic_type type: (type_identifier) @name)])
 (ordered_field_declaration_list type: [(type_identifier) @name (scoped_type_identifier name: (type_identifier) @name) (generic_type type: (type_identifier) @name)])
 (qualified_type type: [(type_identifier) @name (scoped_type_identifier name: (type_identifier) @name) (generic_type type: (type_identifier) @name)])
 (type_cast_expression type: [(type_identifier) @name (scoped_type_identifier name: (type_identifier) @name) (generic_type type: (type_identifier) @name)])
 (function_item return_type: [(type_identifier) @name (scoped_type_identifier name: (type_identifier) @name) (generic_type type: (type_identifier) @name)])
 (function_signature_item return_type: [(type_identifier) @name (scoped_type_identifier name: (type_identifier) @name) (generic_type type: (type_identifier) @name)])
 (closure_expression return_type: [(type_identifier) @name (scoped_type_identifier name: (type_identifier) @name) (generic_type type: (type_identifier) @name)])
 (type_arguments [(type_identifier) @name (scoped_type_identifier name: (type_identifier) @name) (generic_type type: (type_identifier) @name)])
 (array_type element: [(type_identifier) @name (scoped_type_identifier name: (type_identifier) @name) (generic_type type: (type_identifier) @name)])
 (tuple_type [(type_identifier) @name (scoped_type_identifier name: (type_identifier) @name) (generic_type type: (type_identifier) @name)])
 (dynamic_type trait: [(type_identifier) @name (scoped_type_identifier name: (type_identifier) @name) (generic_type type: (type_identifier) @name)])
 (abstract_type trait: [(type_identifier) @name (scoped_type_identifier name: (type_identifier) @name) (generic_type type: (type_identifier) @name)])
 (type_parameter bounds: (trait_bounds [(type_identifier) @name (scoped_type_identifier name: (type_identifier) @name) (generic_type type: (type_identifier) @name)]))
 (where_predicate bounds: (trait_bounds [(type_identifier) @name (scoped_type_identifier name: (type_identifier) @name) (generic_type type: (type_identifier) @name)]))
 (struct_expression name: [(type_identifier) @name (scoped_type_identifier name: (type_identifier) @name) (generic_type type: (type_identifier) @name)])] @reference.type_use

; A generic type in front of `::` (`Foo::<u8>::new()`)
(scoped_identifier path: (generic_type type: (type_identifier) @name)) @reference.type_use

; Writes
(assignment_expression left: (identifier) @name) @reference.write
(assignment_expression left: (field_expression field: (field_identifier) @name)) @reference.write
(compound_assignment_expr left: (identifier) @name) @reference.write
(compound_assignment_expr left: (field_expression field: (field_identifier) @name)) @reference.write

; Reads: names in value positions (not the function of a call, not an assignment target)
[(arguments [(identifier) @name (field_expression field: (field_identifier) @name)])
 (return_expression [(identifier) @name (field_expression field: (field_identifier) @name)])
 (parenthesized_expression [(identifier) @name (field_expression field: (field_identifier) @name)])
 (try_expression [(identifier) @name (field_expression field: (field_identifier) @name)])
 (await_expression [(identifier) @name (field_expression field: (field_identifier) @name)])
 (array_expression [(identifier) @name (field_expression field: (field_identifier) @name)])
 (tuple_expression [(identifier) @name (field_expression field: (field_identifier) @name)])
 (unary_expression [(identifier) @name (field_expression field: (field_identifier) @name)])
 (index_expression [(identifier) @name (field_expression field: (field_identifier) @name)])
 (expression_statement [(identifier) @name (field_expression field: (field_identifier) @name)])
 (binary_expression left: [(identifier) @name (field_expression field: (field_identifier) @name)])
 (let_declaration value: [(identifier) @name (field_expression field: (field_identifier) @name)])
 (reference_expression value: [(identifier) @name (field_expression field: (field_identifier) @name)])
 (match_expression value: [(identifier) @name (field_expression field: (field_identifier) @name)])
 (for_expression value: [(identifier) @name (field_expression field: (field_identifier) @name)])
 (field_initializer value: [(identifier) @name (field_expression field: (field_identifier) @name)])
 (match_arm value: [(identifier) @name (field_expression field: (field_identifier) @name)])
 (closure_expression body: [(identifier) @name (field_expression field: (field_identifier) @name)])
 (_ right: [(identifier) @name (field_expression field: (field_identifier) @name)])
 (_ condition: [(identifier) @name (field_expression field: (field_identifier) @name)])] @reference.read
(shorthand_field_initializer (identifier) @name) @reference.read
; A path in a value position (`Style::Plain`, `limits::MAX`) reads its last name and the type or
; module in front of it
[(arguments (scoped_identifier [path: (identifier) @name name: (identifier) @name]))
 (let_declaration value: (scoped_identifier [path: (identifier) @name name: (identifier) @name]))
 (return_expression (scoped_identifier [path: (identifier) @name name: (identifier) @name]))
 (match_arm
   pattern: (match_pattern (scoped_identifier [path: (identifier) @name name: (identifier) @name])))
 (call_expression
   function: (scoped_identifier path: (identifier) @name (#not-eq? @name "Self")))
 (binary_expression
   [left: (scoped_identifier [path: (identifier) @name name: (identifier) @name])
    right: (scoped_identifier [path: (identifier) @name name: (identifier) @name])])] @reference.read
; The receiver of a member access: a variable or a type
(field_expression value: (identifier) @name) @reference.read

; ---------------------------------------------------------------------------------------------
; Receivers: what the syntax says about the thing in front of the dot or the `::` (blueprint 9.2,
; step 3). These patterns add facts to the occurrences above, they do not create occurrences.
; `self` and `Self` are the enclosing object.
(field_expression
  value: [(self) @receiver.own
          (identifier) @receiver
          (call_expression function: (scoped_identifier path: (identifier) @receiver.type))
          (struct_expression name: (type_identifier) @receiver.type)
          (_) @receiver.other]
  field: (field_identifier) @name)
(scoped_identifier
  path: [((identifier) @receiver.own (#eq? @receiver.own "Self"))
         (identifier) @receiver
         (scoped_identifier name: (identifier) @receiver)
         (generic_type type: (type_identifier) @receiver)
         (_) @receiver.other]
  name: (identifier) @name)

; ---------------------------------------------------------------------------------------------
; Scopes: nodes in which a declared name is visible.
[(function_item) (closure_expression) (block) (for_expression) (match_arm) (if_expression)
 (while_expression)] @scope

; ---------------------------------------------------------------------------------------------
; Locals and parameters hide members of the same name (step 1). A declared type, a creation with
; `Foo::new()` or a struct expression tells the type of a receiver.
(parameter pattern: (identifier) @local)
(parameter
  pattern: (identifier) @local
  type: [(type_identifier) @declared_type
         (generic_type type: (type_identifier) @declared_type)
         (reference_type type: [(type_identifier) @declared_type
                                (generic_type type: (type_identifier) @declared_type)])])
(closure_parameters (identifier) @local)
(closure_parameters (parameter pattern: (identifier) @local))
(let_declaration pattern: (identifier) @local)
(let_declaration
  pattern: (identifier) @local
  type: [(type_identifier) @declared_type
         (generic_type type: (type_identifier) @declared_type)
         (reference_type type: [(type_identifier) @declared_type
                                (generic_type type: (type_identifier) @declared_type)])])
(let_declaration
  pattern: (identifier) @local
  value: (call_expression function: (scoped_identifier path: (identifier) @declared_type)))
(let_declaration
  pattern: (identifier) @local
  value: (struct_expression name: (type_identifier) @declared_type))
(let_declaration pattern: (tuple_pattern (identifier) @local))
(let_declaration pattern: (mut_pattern (identifier) @local))
(for_expression pattern: (identifier) @local)
(for_expression pattern: (tuple_pattern (identifier) @local))
(let_condition pattern: (tuple_struct_pattern type: (_) . (identifier) @local))

; Fields tell the type of a receiver, but an occurrence of them is a reference.
(field_declaration
  name: (field_identifier) @field
  type: [(type_identifier) @declared_type
         (generic_type type: (type_identifier) @declared_type)
         (reference_type type: [(type_identifier) @declared_type
                                (generic_type type: (type_identifier) @declared_type)])])
