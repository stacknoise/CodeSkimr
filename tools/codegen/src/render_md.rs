//! Markdown output: the readable catalogue `docs/error-codes.md`.

use std::fmt::Write;

use crate::consts::GENERATED_NOTICE;
use crate::registry::Registry;

/// Renders the catalogue, one table per group.
#[must_use]
pub fn render(registry: &Registry) -> String {
    let mut out = format!("<!-- {GENERATED_NOTICE} -->\n\n# Error code catalogue\n\n");
    out.push_str("Codes have the form `<PREFIX>-<C><NN>` (blueprint section 11). User-facing messages live in `locales/<lang>/errors.ftl` as `error-<id>`.\n");
    for (prefix, group) in registry.group_entries() {
        let _ = write!(
            out,
            "\n## {prefix} – {}\n\nOwner: `{}`\n\n",
            group.title, group.owner
        );
        out.push_str("| Code | Name | Severity | Retryable | Since | Description |\n|---|---|---|---|---|---|\n");
        for code in registry.codes.iter().filter(|c| c.prefix() == prefix) {
            let state = if code.deprecated { " (deprecated)" } else { "" };
            let _ = writeln!(
                out,
                "| {} | `{}` | {} | {} | {} | {}{state} |",
                code.id,
                code.name,
                code.severity.label(),
                if code.retryable { "yes" } else { "no" },
                code.since,
                code.description,
            );
        }
    }
    out
}
