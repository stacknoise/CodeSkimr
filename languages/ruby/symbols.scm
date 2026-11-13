; Ruby declarations -> symbol kinds (basic tier).
(class name: [(constant) (scope_resolution)] @name) @symbol.class
(module name: [(constant) (scope_resolution)] @name) @symbol.namespace
(program (method name: (_) @name) @symbol.function)
(body_statement (method name: (_) @name) @symbol.method)
(singleton_method name: (_) @name) @symbol.method
