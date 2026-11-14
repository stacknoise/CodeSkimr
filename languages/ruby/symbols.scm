; Ruby declarations -> symbol kinds (basic tier).
(class name: [(constant) (scope_resolution)] @name) @symbol.class
(module name: [(constant) (scope_resolution)] @name) @symbol.namespace
(program (method name: (_) @name) @symbol.function)
(body_statement (method name: (_) @name) @symbol.method)
(singleton_method name: (_) @name) @symbol.method

; A `def` directly inside a top-level or class-level `if`, `unless` or `begin` (issue #66).
(program [(if consequence: (then (method name: (_) @name) @symbol.function)) (unless consequence: (then (method name: (_) @name) @symbol.function))])
(program [(if alternative: (else (method name: (_) @name) @symbol.function)) (unless alternative: (else (method name: (_) @name) @symbol.function)) (if alternative: (elsif consequence: (then (method name: (_) @name) @symbol.function)))])
(program (begin (method name: (_) @name) @symbol.function))
(body_statement [(if consequence: (then (method name: (_) @name) @symbol.method)) (unless consequence: (then (method name: (_) @name) @symbol.method))])
(body_statement [(if alternative: (else (method name: (_) @name) @symbol.method)) (unless alternative: (else (method name: (_) @name) @symbol.method)) (if alternative: (elsif consequence: (then (method name: (_) @name) @symbol.method)))])
(body_statement (begin (method name: (_) @name) @symbol.method))
