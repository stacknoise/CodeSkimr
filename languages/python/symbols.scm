; Python declarations -> symbol kinds (blueprint 8.1, ADR 0023). The docstring is the first
; statement of a body; decorator names are modifiers (`@staticmethod`, `@property` ...). A `def`
; is a `function` here; language.toml turns it into a `method` inside a class.

; Classes
(class_definition
  name: (identifier) @name
  body: (block . (expression_statement (string (string_content) @doc))?)) @symbol.class

; Functions and methods. The decorators are siblings of the definition.
(function_definition
  name: (identifier) @name @modifier
  body: (block . (expression_statement (string (string_content) @doc))?)) @symbol.function
(decorated_definition
  (decorator [(identifier) @modifier (attribute attribute: (identifier) @modifier)])
  definition: (function_definition name: (identifier) @name) @symbol.function)

; Fields: assignments directly in a class body, annotated or not.
(class_definition
  body: (block
    (expression_statement
      (assignment left: (identifier) @name) @symbol.field)))

; Constants: UPPER_CASE assignments at module level.
(module
  (expression_statement
    (assignment left: (identifier) @name (#match? @name "^[A-Z][A-Z0-9_]*$")) @symbol.constant))
