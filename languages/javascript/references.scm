; JavaScript occurrences -> reference kinds (blueprint 9.2, level 1: names only, the resolver
; decides). Declarations are not occurrences. A class `extends` is reported as `inherit`.

; Calls. `new Foo()` is reported as a type use of Foo.
(call_expression function: (identifier) @name) @reference.call
(call_expression function: (member_expression property: (property_identifier) @name)) @reference.call
(new_expression constructor: (identifier) @name) @reference.type_use
(new_expression constructor: (member_expression property: (property_identifier) @name)) @reference.type_use

; Inheritance
(class_heritage (identifier) @name) @reference.inherit
(class_heritage (member_expression property: (property_identifier) @name)) @reference.inherit

; Imports: the imported names (`import X`, `import { X as Y }`, `import * as X`)
(import_clause (identifier) @name) @reference.import
(import_specifier name: (identifier) @name) @reference.import
(namespace_import (identifier) @name) @reference.import

; JSX: a component (capitalised tag) is a type use of the component.
((jsx_opening_element name: (identifier) @name) @reference.type_use
 (#match? @name "^[A-Z]"))
((jsx_self_closing_element name: (identifier) @name) @reference.type_use
 (#match? @name "^[A-Z]"))

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
 (unary_expression [(identifier) @name (member_expression property: (property_identifier) @name)])
 (subscript_expression [(identifier) @name (member_expression property: (property_identifier) @name)])
 (jsx_expression [(identifier) @name (member_expression property: (property_identifier) @name)])] @reference.read
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
  (class_declaration)] @scope)

; ---------------------------------------------------------------------------------------------
; Locals and parameters hide members of the same name (step 1). The type of `new Foo()` tells the
; type of a receiver. Variables of the module itself are symbols, not locals, so only
; declarations inside a block count.
(_ parameters: (formal_parameters (identifier) @local))
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
      value: (new_expression constructor: (identifier) @declared_type))))
(for_statement initializer: (lexical_declaration (variable_declarator name: (identifier) @local)))
(for_in_statement left: (identifier) @local)

; Fields tell the type of a receiver, but an occurrence of them is a reference.
(field_definition
  property: (property_identifier) @field
  value: (new_expression constructor: (identifier) @declared_type))
