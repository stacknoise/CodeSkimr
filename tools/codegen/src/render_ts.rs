//! TypeScript output: `ErrorCode` union and lookup table.

use std::fmt::Write;

use crate::consts::{GENERATED_NOTICE, MESSAGE_KEY_PREFIX};
use crate::registry::Registry;

/// Class names in TypeScript, keyed by digit (see blueprint 11.2).
fn class_name(digit: u8) -> &'static str {
    match digit {
        1 => "invalid_input",
        2 => "not_found",
        3 => "conflict",
        4 => "permission",
        5 => "limit",
        6 => "external",
        _ => "internal",
    }
}

/// Renders the complete generated TypeScript file.
#[must_use]
pub fn render(registry: &Registry) -> String {
    let mut out = format!("// {GENERATED_NOTICE}\n\n");
    out.push_str(
        "/** Id of a registered error code, e.g. `SRC-201`. */\nexport type ErrorCode =\n",
    );
    for code in &registry.codes {
        let _ = writeln!(out, "  | {:?}", code.id);
    }
    out.push_str(";\n\n/** Static data of one error code. */\nexport interface ErrorCodeInfo {\n");
    out.push_str("  id: ErrorCode;\n  name: string;\n  group: string;\n  class: string;\n");
    out.push_str("  severity: \"error\" | \"warning\" | \"info\";\n  retryable: boolean;\n");
    out.push_str("  description: string;\n  since: string;\n  deprecated: boolean;\n  messageKey: string;\n}\n\n");
    out.push_str("/** All registered error codes by id. */\nexport const ERROR_CODES: Record<ErrorCode, ErrorCodeInfo> = {\n");
    for code in &registry.codes {
        let _ = writeln!(
            out,
            "  {id:?}: {{ id: {id:?}, name: {:?}, group: {:?}, class: {:?}, severity: {:?}, retryable: {}, description: {:?}, since: {:?}, deprecated: {}, messageKey: {:?} }},",
            code.name,
            code.prefix(),
            class_name(code.class_digit()),
            code.severity.label(),
            code.retryable,
            code.description,
            code.since,
            code.deprecated,
            format!("{MESSAGE_KEY_PREFIX}{}", code.id),
            id = code.id,
        );
    }
    out.push_str("};\n");
    out
}
