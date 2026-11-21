; TSX: what the TypeScript pack takes over (queries.from, ADR 0056) plus JSX.
; JSX: a component (capitalised tag, or the property of `<Ui.Item />`) is a use of the component. It is a
; `read`: the resolver lets a read point to a value, a type or a callable, and reports it as `type_use`
; when the target is a type. A function component is a function, which a `type_use` never reaches
; (issue #609). `<div>` and the other lower-case tags are HTML and stay out. `{x}` children are reads.
((jsx_opening_element name: (identifier) @name) @reference.read
 (#match? @name "^[A-Z]"))
((jsx_self_closing_element name: (identifier) @name) @reference.read
 (#match? @name "^[A-Z]"))
(jsx_opening_element name: (member_expression property: (property_identifier) @name)) @reference.read
(jsx_self_closing_element name: (member_expression property: (property_identifier) @name)) @reference.read
(jsx_expression [(identifier) @name (member_expression property: (property_identifier) @name)]) @reference.read
