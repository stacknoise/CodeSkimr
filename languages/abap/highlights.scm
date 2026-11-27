; ABAP syntax highlighting for the UI (ADR 0064). Adapted from queries/highlights.scm of
; zooloo303/tree-sitter-abap (MIT): capture names changed to the ones the UI knows.

; Comments -----------------------------------------------------------------

(line_comment) @comment
(star_comment) @comment

; Literals -----------------------------------------------------------------

(string) @string
(backtick_string) @string
(string_template "|" @string)
(template_text) @string
(template_substitution
  "{" @string
  "}" @string)
(number) @number
(text_symbol_key) @constant

; Pragmas / macro params -----------------------------------------------------

(pragma) @attribute
(macro_parameter) @variable.builtin

; Field symbols --------------------------------------------------------------

(field_symbol) @variable.builtin

; Structural keywords (anonymous nodes from grammar rules) -------------------

[
  "class" "endclass" "definition" "implementation"
  "interface" "endinterface"
  "method" "endmethod" "methods" "class-methods"
  "data" "class-data" "types" "constants" "statics"
  "field-symbols" "parameters" "select-options" "tables" "ranges" "nodes"
  "form" "endform" "function" "endfunction" "module" "endmodule"
  "report" "program" "function-pool"
  "public" "protected" "private" "section"
  "deferred" "local" "friends" "final"
] @keyword

; Declared names --------------------------------------------------------------

(class_definition name: (identifier) @type)
(class_forward_declaration name: (identifier) @type)
(class_implementation name: (identifier) @type)
(interface_definition name: (identifier) @type)
(interface_forward_declaration name: (identifier) @type)
(method_implementation name: (identifier) @function)
(method_entry name: (identifier) @function)
(form_definition name: (identifier) @function)
(function_definition name: (identifier) @function)
(module_definition name: (identifier) @function)
(program_statement name: (identifier) @function)

; Keywords appearing inside generic statements (case-insensitive) -------------
; The grammar lexes these as plain identifiers; classification happens here.

((identifier) @keyword
  (#match? @keyword "(?i)^(if|else|elseif|endif|case|when|others|endcase|loop|endloop|do|enddo|while|endwhile|exit|continue|check|return|and|or|not|equiv|is|between|in|initial|bound|assigned|supplied|instance|select|endselect|from|where|into|up|to|rows|order|by|group|having|join|inner|left|outer|right|on|as|distinct|single|for|all|entries|of|union|fields|appending|corresponding|insert|update|modify|delete|append|clear|free|refresh|sort|read|binary|search|transporting|collect|type|ref|like|line|standard|sorted|hashed|table|with|unique|key|default|size|value|begin|end|include|structure|occurs|header|importing|exporting|changing|returning|receiving|raising|exceptions|using|optional|preferred|parameter|call|perform|submit|leave|screen|transaction|try|catch|cleanup|endtry|raise|exception|resumable|shortdump|create|object|assign|assigning|unassign|casting|component|move|write|format|at|first|last|new-line|uline|skip|concatenate|split|condense|translate|replace|shift|find|regex|pcre|respecting|occurrences|occurrence|separated|message|id|number|display|commit|rollback|work|wait|export|import|memory|database|shared|abstract|inheriting|redefinition|load|renaming|suffix|read-only|testing|risk|level|harmless|dangerous|critical|duration|short|medium|long|cond|switch|reduce|filter|conv|exact|new|then|let|until|next|init|base|members|without|when_others|assert|eq|ne|lt|gt|le|ge|co|cn|ca|na|cs|ns|cp|np|byte-co|is_initial|times|mod|div|length|lines|others|sum|get|set|bit|xsdbool|boolc|interfaces|aliases|events|class-events|partially|implemented|events)$"))

; Built-in constants / special variables ---------------------------------------

((identifier) @constant
  (#match? @constant "(?i)^(abap_true|abap_false|abap_undefined|abap_bool|space)$"))

((identifier) @variable.builtin
  (#match? @variable.builtin "(?i)^(sy|syst|me|super|textid)$"))

; Type-looking identifiers (class/interface naming conventions) ----------------

((identifier) @type
  (#match? @type "(?i)^(/[a-z0-9_]+/)?(lcl_|ltcl_|ltd_|lif_|[zy]?c[lx]_|[zy]?if_)[a-z0-9_]+$"))
