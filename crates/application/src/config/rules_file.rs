//! Writes the architecture rules into the text of a project file (ADR 0112): everything but the
//! `rules` tables stays as it is, the tables are written anew at the end.

use serde::Serialize;

use super::consts::{RULES_ARRAY_HEADER, RULES_NAME_END, RULES_TABLE_HEADER, keys};
use super::error::{ConfigError, Violation};
use super::layer_file::{rewrite_with, without_tables_where};
use super::model::{CycleRule, ForbidRule, PublicApiRule, Rules};

/// The shape of the written tables.
#[derive(Serialize)]
struct Written<'a> {
    rules: Table<'a>,
}

#[derive(Serialize)]
struct Table<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    cycles: Option<CycleRule>,
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    forbid: &'a [ForbidRule],
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    public_api: &'a [PublicApiRule],
}

/// Whether a header line opens `[rules]` or a table below it, such as `[[rules.forbid]]`; a
/// table that only starts with the same letters, such as `[rules_notes]`, is not meant.
fn is_rules_header(head: &str) -> bool {
    [RULES_TABLE_HEADER, RULES_ARRAY_HEADER]
        .iter()
        .any(|start| {
            head.strip_prefix(start)
                .is_some_and(|rest| rest.starts_with(RULES_NAME_END))
        })
}

/// The text of a project file with `wanted` as its rules (`cycles`, `forbid` and `public_api`).
/// The rest of the file, comments included, stays; the old `rules` tables are replaced.
///
/// # Errors
/// - `CFG-102` if `text` is not valid TOML (the file is then left alone).
/// - `CFG-101` if the rules are not valid: a prefix is not a name, a rule names a module inside
///   the other, a module is named twice, or a module has no inside.
pub fn with_rules(text: &str, wanted: &Rules) -> Result<String, ConfigError> {
    let table = Table {
        cycles: (wanted.cycles == CycleRule::Deny).then_some(CycleRule::Deny),
        forbid: &wanted.forbid,
        public_api: &wanted.public_api,
    };
    let block =
        if table.cycles.is_none() && wanted.forbid.is_empty() && wanted.public_api.is_empty() {
            String::new()
        } else {
            toml::to_string(&Written { rules: table }).map_err(|_| ConfigError::OutOfRange {
                key: keys::RULES,
                violation: Violation::MustBeArchitectureRule,
            })?
        };
    let kept = without_tables_where(text, is_rules_header);
    rewrite_with(text, &kept, &block)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, ConfigLayer, LayerSource};

    fn forbid(from: &str, to: &str) -> ForbidRule {
        ForbidRule {
            from: from.to_owned(),
            to: to.to_owned(),
            reason: None,
        }
    }

    fn read(text: &str) -> Rules {
        let layer = ConfigLayer::parse(LayerSource::Project, text).unwrap();
        Config::resolve(&[layer]).unwrap().rules
    }

    #[test]
    fn rules_are_written_at_the_end_and_read_back() {
        let wanted = Rules {
            cycles: CycleRule::Deny,
            forbid: vec![forbid("A.B", "C.D")],
            public_api: vec![PublicApiRule {
                module: "A".to_owned(),
                internal: vec!["A.Internal".to_owned()],
            }],
        };
        let text = "# mine\n[keys]\nmax_files = 100\n";
        let written = with_rules(text, &wanted).unwrap();
        assert!(written.starts_with("# mine\n[keys]\nmax_files = 100"));
        assert_eq!(read(&written), wanted);
    }

    #[test]
    fn old_rules_are_replaced_and_the_other_tables_stay() {
        let text = "[[rules.forbid]]\nfrom = \"A\"\nto = \"B\"\n\n[rules]\ncycles = \"deny\"\n\n[[layers]]\nname = \"x\"\nnamespaces = [\"X\"]\n";
        let wanted = Rules {
            forbid: vec![forbid("C", "D")],
            ..Rules::default()
        };
        let written = with_rules(text, &wanted).unwrap();
        assert!(written.contains("[[layers]]"));
        assert_eq!(read(&written), wanted);
        let none = with_rules(&written, &Rules::default()).unwrap();
        assert!(!none.contains("rules"));
        assert!(none.contains("[[layers]]"));
    }

    #[test]
    fn a_table_that_only_starts_like_rules_is_not_taken_for_one_and_no_limits_leave_layers_alone() {
        let text = "# note\n[rules_notes]\nx = 1\n\n[rules]\ncycles = \"deny\"\n";
        let written = with_rules(text, &Rules::default());
        assert!(
            written.is_err(),
            "a table the model does not know is refused, not removed"
        );
        let kept = "[[layers]]\nname = \"a\"\nnamespaces = [\"x\"]\n# keep me\n";
        let same = with_layer_limits_unchanged(kept);
        assert_eq!(same, kept);
    }

    fn with_layer_limits_unchanged(text: &str) -> String {
        crate::config::with_layer_limits(text, &[]).unwrap()
    }

    #[test]
    fn invalid_rules_and_files_that_are_no_toml_are_refused() {
        let twin = Rules {
            forbid: vec![forbid("A", "B"), forbid("A", "B")],
            ..Rules::default()
        };
        assert!(with_rules("", &twin).is_err());
        let inside = Rules {
            forbid: vec![forbid("A", "A.B")],
            ..Rules::default()
        };
        assert!(with_rules("", &inside).is_err());
        assert!(with_rules("[keys", &Rules::default()).is_err());
    }
}
