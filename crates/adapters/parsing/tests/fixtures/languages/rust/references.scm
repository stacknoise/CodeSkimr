(call_expression function: (identifier) @name) @reference.call
(call_expression function: (field_expression field: (field_identifier) @name)) @reference.call
(use_declaration argument: (_) @name) @reference.import
(parameter type: (type_identifier) @name) @reference.type_use
