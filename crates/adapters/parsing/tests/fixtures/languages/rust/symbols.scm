((line_comment)* @doc
 .
 (struct_item (visibility_modifier)? @visibility name: (type_identifier) @name) @symbol.struct)
(enum_item (visibility_modifier)? @visibility name: (type_identifier) @name) @symbol.enum
(trait_item (visibility_modifier)? @visibility name: (type_identifier) @name) @symbol.trait
(const_item (visibility_modifier)? @visibility name: (identifier) @name) @symbol.constant
(mod_item (visibility_modifier)? @visibility name: (identifier) @name) @symbol.namespace
(source_file
  (function_item (visibility_modifier)? @visibility name: (identifier) @name) @symbol.function)
(impl_item
  body: (declaration_list
    (function_item (visibility_modifier)? @visibility name: (identifier) @name) @symbol.method))
