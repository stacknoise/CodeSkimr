; TSX: what the TypeScript pack takes over (queries.from, ADR 0056) plus constants and fields whose value is JSX.
(program
 (comment)+ @doc
 .
 (lexical_declaration "const" @modifier
  (variable_declarator name: (identifier) @name
   value: [(jsx_element) (jsx_self_closing_element)]) @symbol.field))
(program
 (lexical_declaration "const" @modifier
  (variable_declarator name: (identifier) @name
   value: [(jsx_element) (jsx_self_closing_element)]) @symbol.field))
(program
 (comment)+ @doc
 .
 (export_statement "export" @modifier declaration: (lexical_declaration "const" @modifier
  (variable_declarator name: (identifier) @name
   value: [(jsx_element) (jsx_self_closing_element)]) @symbol.field)))
(program
 (export_statement "export" @modifier declaration: (lexical_declaration "const" @modifier
  (variable_declarator name: (identifier) @name
   value: [(jsx_element) (jsx_self_closing_element)]) @symbol.field)))
((comment)* @doc
 .
 (public_field_definition
  [(accessibility_modifier) "static" "readonly" "override" "abstract" "declare" "accessor"]* @modifier
  name: [(property_identifier) (private_property_identifier)] @name
  value: [(jsx_element) (jsx_self_closing_element)]) @symbol.field)
