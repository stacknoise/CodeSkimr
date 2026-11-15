; Go declarations -> symbol kinds (basic tier: types and functions).
(type_spec name: (type_identifier) @name type: (struct_type)) @symbol.struct
(type_spec name: (type_identifier) @name type: (interface_type)) @symbol.interface
(type_spec name: (type_identifier) @name type: [
  (type_identifier) (qualified_type) (pointer_type) (slice_type) (array_type) (map_type)
  (channel_type) (function_type) (generic_type)]) @symbol.type_alias
(type_alias name: (type_identifier) @name) @symbol.type_alias
(function_declaration name: (identifier) @name) @symbol.function
(method_declaration name: (field_identifier) @name) @symbol.method

; A method belongs to the type of its receiver (`func (a *A) Run()` is `A.Run`), also a pointer or
; generic receiver (ADR 0042).
(method_declaration
  receiver: (parameter_list
    (parameter_declaration
      type: [(type_identifier) @owner
             (generic_type type: (type_identifier) @owner)
             (pointer_type [(type_identifier) @owner (generic_type type: (type_identifier) @owner)])]))
  name: (field_identifier) @name) @symbol.method
