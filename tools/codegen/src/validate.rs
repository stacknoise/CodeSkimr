//! Registry validation (blueprint rule E-02, the parts that need no source scan).

use std::collections::BTreeSet;
use std::path::Path;

use crate::consts::{CLASS_DIGITS, ERRORS_FTL, LOCALES_DIR, MESSAGE_KEY_PREFIX, PREFIX_LEN};
use crate::registry::{Code, Registry};

/// Checks the id format `<PREFIX>-<C><NN>` and the registry-level rules for one code.
fn check_code(code: &Code, registry: &Registry, problems: &mut Vec<String>) {
    let id = &code.id;
    let prefix = code.prefix();
    let number = code.number();
    let prefix_ok = PREFIX_LEN.contains(&prefix.len())
        && prefix.starts_with(|c: char| c.is_ascii_uppercase())
        && prefix
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit());
    let number_ok = number.len() == 3
        && number.bytes().all(|b| b.is_ascii_digit())
        && CLASS_DIGITS.contains(&code.class_digit())
        && !number.ends_with("00");
    if !prefix_ok || !number_ok {
        problems.push(format!("{id}: id does not match <PREFIX>-<C><NN>"));
    }
    if !registry.groups.contains_key(prefix) {
        problems.push(format!(
            "{id}: group {prefix} is not declared under [groups]"
        ));
    }
    if !is_snake_case(&code.name) {
        problems.push(format!("{id}: name {} is not snake_case", code.name));
    }
    if code.description.trim().is_empty() {
        problems.push(format!("{id}: description is empty"));
    }
    if code.since.split('.').count() != 3 {
        problems.push(format!("{id}: since {} is not a version", code.since));
    }
}

/// `true` for non-empty lower-case words joined by single underscores.
fn is_snake_case(name: &str) -> bool {
    !name.is_empty()
        && name.starts_with(|c: char| c.is_ascii_lowercase())
        && name.split('_').all(|w| {
            !w.is_empty()
                && w.chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        })
}

/// Reports ids, names and enum variants that occur more than once.
fn check_duplicates(registry: &Registry, problems: &mut Vec<String>) {
    let mut ids = BTreeSet::new();
    let mut names = BTreeSet::new();
    let mut variants = BTreeSet::new();
    for code in &registry.codes {
        if !ids.insert(code.id.as_str()) {
            problems.push(format!("{}: duplicate id", code.id));
        }
        if !names.insert(code.name.clone()) {
            problems.push(format!("{}: duplicate name {}", code.id, code.name));
        }
        if !variants.insert(code.variant()) {
            problems.push(format!(
                "{}: name {} collides after Pascal-casing",
                code.id, code.name
            ));
        }
    }
}

/// Reports groups that are declared but have no code.
fn check_unused_groups(registry: &Registry, problems: &mut Vec<String>) {
    let used: BTreeSet<&str> = registry.codes.iter().map(Code::prefix).collect();
    for prefix in registry
        .groups
        .keys()
        .filter(|p| !used.contains(p.as_str()))
    {
        problems.push(format!("group {prefix} has no codes"));
    }
}

/// Validates the registry on its own.
///
/// # Errors
/// Returns every problem found, one message per problem.
pub fn validate_registry(registry: &Registry) -> Result<(), Vec<String>> {
    let mut problems = Vec::new();
    for code in &registry.codes {
        check_code(code, registry, &mut problems);
    }
    check_duplicates(registry, &mut problems);
    check_unused_groups(registry, &mut problems);
    if problems.is_empty() {
        Ok(())
    } else {
        Err(problems)
    }
}

/// Message keys defined in the text of an `errors.ftl` file.
#[must_use]
pub fn message_keys(ftl: &str) -> BTreeSet<String> {
    ftl.lines()
        .filter(|line| line.starts_with(MESSAGE_KEY_PREFIX))
        .filter_map(|line| line.split_once('=').map(|(key, _)| key.trim().to_string()))
        .collect()
}

/// Checks that every non-deprecated code has a message and that no message is orphaned.
#[must_use]
pub fn check_messages(registry: &Registry, lang: &str, ftl: &str) -> Vec<String> {
    let defined = message_keys(ftl);
    let mut problems = Vec::new();
    let mut expected = BTreeSet::new();
    for code in &registry.codes {
        let key = format!("{MESSAGE_KEY_PREFIX}{}", code.id);
        if !defined.contains(&key) && !code.deprecated {
            problems.push(format!("{}: no message in {lang}/{ERRORS_FTL}", code.id));
        }
        expected.insert(key);
    }
    for key in defined.difference(&expected) {
        problems.push(format!("{lang}/{ERRORS_FTL}: message {key} has no code"));
    }
    problems
}

/// Checks all languages found under `<root>/locales`.
///
/// # Errors
/// Returns every problem found, including unreadable locale files.
pub fn validate_locales(registry: &Registry, root: &Path) -> Result<(), Vec<String>> {
    let dir = root.join(LOCALES_DIR);
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Err(vec![format!("cannot read {}", dir.display())]);
    };
    let mut problems = Vec::new();
    let mut langs: Vec<_> = entries.flatten().filter(|e| e.path().is_dir()).collect();
    langs.sort_by_key(std::fs::DirEntry::file_name);
    for entry in langs {
        let lang = entry.file_name().to_string_lossy().into_owned();
        match std::fs::read_to_string(entry.path().join(ERRORS_FTL)) {
            Ok(ftl) => problems.extend(check_messages(registry, &lang, &ftl)),
            Err(e) => problems.push(format!("{lang}/{ERRORS_FTL}: {e}")),
        }
    }
    if problems.is_empty() {
        Ok(())
    } else {
        Err(problems)
    }
}
