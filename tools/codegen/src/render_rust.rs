//! Rust output: `Group` and `ErrorCode` enums with their lookup tables.

use std::fmt::Write;

use crate::consts::GENERATED_NOTICE;
use crate::registry::{Code, Group, Registry, pascal_case};

/// Escapes a string for use inside a Rust string literal.
fn lit(text: &str) -> String {
    format!("{text:?}")
}

/// Variant name of a group prefix.
fn group_variant(prefix: &str) -> String {
    pascal_case(&prefix.to_lowercase())
}

/// Renders the `Group` enum.
fn group_enum(groups: &[(&str, &Group)], out: &mut String) {
    out.push_str("/// Code group, identified by the id prefix.\n");
    out.push_str(
        "#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]\npub enum Group {\n",
    );
    for (prefix, group) in groups {
        let _ = writeln!(
            out,
            "    /// {}.\n    {},",
            group.title,
            group_variant(prefix)
        );
    }
    out.push_str("}\n\nimpl Group {\n    /// All groups in registry order.\n");
    let _ = writeln!(out, "    pub const ALL: [Self; {}] = [", groups.len());
    for (prefix, _) in groups {
        let _ = writeln!(out, "        Self::{},", group_variant(prefix));
    }
    out.push_str("    ];\n\n");
    out.push_str("    /// Static data of this group.\n    #[must_use]\n    pub const fn info(self) -> GroupInfo {\n        match self {\n");
    for (prefix, group) in groups {
        let _ = writeln!(
            out,
            "            Self::{} => GroupInfo {{ prefix: {}, title: {}, owner: {} }},",
            group_variant(prefix),
            lit(prefix),
            lit(&group.title),
            lit(&group.owner),
        );
    }
    out.push_str("        }\n    }\n}\n\n");
}

/// Renders the `Entry` literal returned by `ErrorCode::entry`.
fn entry_literal(code: &Code) -> String {
    format!(
        "Entry {{ id: {}, group: Group::{}, class: Class::{}, severity: Severity::{}, retryable: {}, \
         description: {}, since: {}, deprecated: {}, message_key: {} }}",
        lit(&code.id),
        group_variant(code.prefix()),
        class_variant(code.class_digit()),
        code.severity.variant(),
        code.retryable,
        lit(&code.description),
        lit(&code.since),
        code.deprecated,
        lit(&format!("{}{}", crate::consts::MESSAGE_KEY_PREFIX, code.id)),
    )
}

/// Maps a class digit to the `Class` variant name.
fn class_variant(digit: u8) -> &'static str {
    match digit {
        1 => "InvalidInput",
        2 => "NotFound",
        3 => "Conflict",
        4 => "Permission",
        5 => "Limit",
        6 => "External",
        _ => "Internal",
    }
}

/// Renders the `ErrorCode` enum.
fn code_enum(registry: &Registry, out: &mut String) {
    out.push_str("/// One entry of the error code registry.\n");
    out.push_str("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]\npub enum ErrorCode {\n");
    for code in &registry.codes {
        let _ = writeln!(
            out,
            "    /// `{}`: {}\n    {},",
            code.id,
            code.description,
            code.variant()
        );
    }
    out.push_str("}\n\nimpl ErrorCode {\n    /// All codes in registry order.\n");
    let _ = writeln!(
        out,
        "    pub const ALL: [Self; {}] = [",
        registry.codes.len()
    );
    for code in &registry.codes {
        let _ = writeln!(out, "        Self::{},", code.variant());
    }
    out.push_str("    ];\n\n    /// Static data of this code.\n    #[must_use]\n    pub const fn entry(self) -> Entry {\n        match self {\n");
    for code in &registry.codes {
        let _ = writeln!(
            out,
            "            Self::{} => {},",
            code.variant(),
            entry_literal(code)
        );
    }
    out.push_str("        }\n    }\n\n");
    from_id(registry, out);
    out.push_str("}\n");
}

/// Renders `ErrorCode::from_id`.
fn from_id(registry: &Registry, out: &mut String) {
    out.push_str("    /// Looks a code up by its id such as `SRC-201`.\n    #[must_use]\n    pub fn from_id(id: &str) -> Option<Self> {\n        match id {\n");
    for code in &registry.codes {
        let _ = writeln!(
            out,
            "            {} => Some(Self::{}),",
            lit(&code.id),
            code.variant()
        );
    }
    out.push_str("            _ => None,\n        }\n    }\n");
}

/// Renders the complete generated Rust file.
#[must_use]
pub fn render(registry: &Registry) -> String {
    let mut out = format!("// {GENERATED_NOTICE}\n\n");
    group_enum(&registry.group_entries(), &mut out);
    code_enum(registry, &mut out);
    out
}
