; TSX: what the TypeScript pack takes over (queries.from, ADR 0056) plus JSX.
; JSX: a component (capitalised tag) is a type use of the component; `{x}` children are reads.
((jsx_opening_element name: (identifier) @name) @reference.type_use
 (#match? @name "^[A-Z]"))
((jsx_self_closing_element name: (identifier) @name) @reference.type_use
 (#match? @name "^[A-Z]"))
(jsx_expression [(identifier) @name (member_expression property: (property_identifier) @name)]) @reference.read
