; Rust declarations -> symbol kinds (blueprint 8.1, ADR 0024). `///` and `/** */` comments directly
; above a declaration (attributes may sit in between) are its documentation; the tokens of a
; visibility modifier (`pub`, `crate`, `super`, `self`, `in`) are the visibility words.
; `impl` blocks are no symbols, so a function inside one is a method of the enclosing module (a
; `for` in the head makes the methods of a trait implementation public, see language.toml).

; Declarations that look the same wherever they stand.
(([(line_comment) (block_comment)]* @doc) .
 (attribute_item)* .
 [(mod_item
   (visibility_modifier ["pub" "in" (crate) (super) (self)] @modifier)*
   name: (identifier) @name) @symbol.namespace
  (struct_item
   (visibility_modifier ["pub" "in" (crate) (super) (self)] @modifier)*
   name: (type_identifier) @name) @symbol.struct
  (union_item
   (visibility_modifier ["pub" "in" (crate) (super) (self)] @modifier)*
   name: (type_identifier) @name) @symbol.union
  (enum_item
   (visibility_modifier ["pub" "in" (crate) (super) (self)] @modifier)*
   name: (type_identifier) @name) @symbol.enum
  (trait_item
   (visibility_modifier ["pub" "in" (crate) (super) (self)] @modifier)*
   name: (type_identifier) @name) @symbol.trait
  (type_item
   (visibility_modifier ["pub" "in" (crate) (super) (self)] @modifier)*
   name: (type_identifier) @name) @symbol.type_alias
  (associated_type name: (type_identifier) @name) @symbol.type_alias
  (const_item
   (visibility_modifier ["pub" "in" (crate) (super) (self)] @modifier)*
   name: (identifier) @name) @symbol.constant
  (static_item
   (visibility_modifier ["pub" "in" (crate) (super) (self)] @modifier)*
   name: (identifier) @name) @symbol.constant
  (enum_variant name: (identifier) @name) @symbol.enum_member
  (field_declaration
   (visibility_modifier ["pub" "in" (crate) (super) (self)] @modifier)*
   name: (field_identifier) @name) @symbol.field])

; Functions. A function directly in a module, a function body or at the top of a file is a
; `function`; in an `impl` or a trait it is a `method`. The adapter merges the matches of one
; declaration, so the name, the documentation, the visibility and `static` are queried by patterns
; of their own (a quantified comment in front of a declaration inside a parent matches nothing
; without a comment, ADR 0022).

; Functions of a module or a block
[(source_file (function_item name: (identifier) @name ) @symbol.function)
 (mod_item body: (declaration_list (function_item name: (identifier) @name ) @symbol.function))
 (block (function_item name: (identifier) @name ) @symbol.function)]
[(source_file [(line_comment) (block_comment)]+ @doc . (attribute_item)* . (function_item name: (identifier) @name ) @symbol.function)
 (mod_item body: (declaration_list [(line_comment) (block_comment)]+ @doc . (attribute_item)* . (function_item name: (identifier) @name ) @symbol.function))
 (block [(line_comment) (block_comment)]+ @doc . (attribute_item)* . (function_item name: (identifier) @name ) @symbol.function)]
[(source_file (function_item (visibility_modifier ["pub" "in" (crate) (super) (self)] @modifier)* name: (identifier) @name ) @symbol.function)
 (mod_item body: (declaration_list (function_item (visibility_modifier ["pub" "in" (crate) (super) (self)] @modifier)* name: (identifier) @name ) @symbol.function))
 (block (function_item (visibility_modifier ["pub" "in" (crate) (super) (self)] @modifier)* name: (identifier) @name ) @symbol.function)]

; Methods of an impl block; a `for` in its head is a trait implementation
(impl_item "for"? @modifier body: (declaration_list (function_item name: (identifier) @name ) @symbol.method))
(impl_item  body: (declaration_list [(line_comment) (block_comment)]+ @doc . (attribute_item)* . (function_item name: (identifier) @name ) @symbol.method))
(impl_item  body: (declaration_list (function_item (visibility_modifier ["pub" "in" (crate) (super) (self)] @modifier)* name: (identifier) @name ) @symbol.method))
(impl_item  body: (declaration_list (function_item name: (identifier) @name parameters: [(parameters . (parameter)) (parameters . "(" . ")")] @static) @symbol.method))

; Methods of a trait: with a default body or only a signature
(trait_item body: (declaration_list [(function_item name: (identifier) @name ) @symbol.method (function_signature_item name: (identifier) @name ) @symbol.method]))
(trait_item body: (declaration_list [(line_comment) (block_comment)]+ @doc . (attribute_item)* . [(function_item name: (identifier) @name ) @symbol.method (function_signature_item name: (identifier) @name ) @symbol.method]))
(trait_item body: (declaration_list [(function_item name: (identifier) @name parameters: [(parameters . (parameter)) (parameters . "(" . ")")] @static) @symbol.method (function_signature_item name: (identifier) @name parameters: [(parameters . (parameter)) (parameters . "(" . ")")] @static) @symbol.method]))
