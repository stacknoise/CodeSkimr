//! Fluent catalogue: loading, completeness check and rendering of the key constants
//! (blueprint section 13).

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;
use std::path::Path;

use crate::consts::{
    ERRORS_FTL, FTL_EXTENSION, GENERATED_NOTICE_I18N, LOCALES_DIR, REFERENCE_LANGUAGE,
};
use crate::ftl::message_ids;

/// All Fluent files: language, then file name (such as `app.ftl`), then file text.
#[derive(Debug, Clone, Default)]
pub struct Catalog {
    /// Text of every file, ordered by language and file name.
    pub files: BTreeMap<String, BTreeMap<String, String>>,
}

/// A key and the file it is defined in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Key {
    /// File name such as `app.ftl`.
    pub file: String,
    /// Message id such as `app-name`.
    pub id: String,
}

impl Key {
    /// Name of the generated constant: upper case, separators replaced by underscores.
    #[must_use]
    pub fn const_name(&self) -> String {
        self.id.to_ascii_uppercase().replace('-', "_")
    }
}

/// Reads the `.ftl` files of one language folder.
fn load_language(dir: &Path) -> Result<BTreeMap<String, String>, String> {
    let mut files = BTreeMap::new();
    let entries = std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some(FTL_EXTENSION) {
            continue;
        }
        let text =
            std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        files.insert(entry.file_name().to_string_lossy().into_owned(), text);
    }
    Ok(files)
}

/// Loads every language under `<root>/locales` and checks completeness against English.
///
/// # Errors
/// Returns every problem found: unreadable files, missing or extra files and keys, duplicate
/// keys and constant name collisions.
pub fn load(root: &Path) -> Result<Catalog, Vec<String>> {
    let dir = root.join(LOCALES_DIR);
    let entries = std::fs::read_dir(&dir).map_err(|e| vec![format!("{}: {e}", dir.display())])?;
    let mut catalog = Catalog::default();
    let mut problems = Vec::new();
    for entry in entries.flatten().filter(|e| e.path().is_dir()) {
        let lang = entry.file_name().to_string_lossy().into_owned();
        match load_language(&entry.path()) {
            Ok(files) => {
                catalog.files.insert(lang, files);
            }
            Err(e) => problems.push(e),
        }
    }
    problems.extend(catalog.problems());
    if problems.is_empty() {
        Ok(catalog)
    } else {
        Err(problems)
    }
}

impl Catalog {
    /// Languages found, sorted.
    #[must_use]
    pub fn languages(&self) -> Vec<&str> {
        self.files.keys().map(String::as_str).collect()
    }

    /// Keys of the reference language, without the error messages (those are reached through
    /// `ErrorCode::message_key`), in file and definition order.
    #[must_use]
    pub fn keys(&self) -> Vec<Key> {
        let Some(files) = self.files.get(REFERENCE_LANGUAGE) else {
            return Vec::new();
        };
        files
            .iter()
            .filter(|(file, _)| file.as_str() != ERRORS_FTL)
            .flat_map(|(file, text)| {
                message_ids(text).into_iter().map(|id| Key {
                    file: file.clone(),
                    id,
                })
            })
            .collect()
    }

    /// Completeness rules: same files, same keys in every language, no key twice.
    #[must_use]
    pub fn problems(&self) -> Vec<String> {
        let mut problems = Vec::new();
        let Some(reference) = self.files.get(REFERENCE_LANGUAGE) else {
            return vec![format!("{LOCALES_DIR}/{REFERENCE_LANGUAGE} does not exist")];
        };
        for (lang, files) in &self.files {
            problems.extend(duplicate_ids(lang, files));
            if lang != REFERENCE_LANGUAGE {
                problems.extend(compare(lang, files, reference));
            }
        }
        problems.extend(self.const_collisions());
        problems
    }

    /// Reports keys whose constant names collide.
    fn const_collisions(&self) -> Vec<String> {
        let mut seen: BTreeMap<String, String> = BTreeMap::new();
        let mut problems = Vec::new();
        for key in self.keys() {
            if let Some(other) = seen.insert(key.const_name(), key.id.clone())
                && other != key.id
            {
                problems.push(format!("keys {other} and {} share one constant", key.id));
            }
        }
        problems
    }
}

/// Reports ids that are defined more than once within one language.
fn duplicate_ids(lang: &str, files: &BTreeMap<String, String>) -> Vec<String> {
    let mut seen = BTreeSet::new();
    let mut problems = Vec::new();
    for (file, text) in files {
        for id in message_ids(text) {
            if !seen.insert(id.clone()) {
                problems.push(format!("{lang}/{file}: key {id} is defined twice"));
            }
        }
    }
    problems
}

/// Compares one language with the reference: files and keys per file.
fn compare(
    lang: &str,
    files: &BTreeMap<String, String>,
    reference: &BTreeMap<String, String>,
) -> Vec<String> {
    let mut problems = Vec::new();
    for file in reference.keys().filter(|f| !files.contains_key(*f)) {
        problems.push(format!("{lang}/{file}: file is missing"));
    }
    for (file, text) in files {
        let Some(reference_text) = reference.get(file) else {
            problems.push(format!(
                "{lang}/{file}: file does not exist in {REFERENCE_LANGUAGE}"
            ));
            continue;
        };
        let own: BTreeSet<String> = message_ids(text).into_iter().collect();
        let expected: BTreeSet<String> = message_ids(reference_text).into_iter().collect();
        for id in expected.difference(&own) {
            problems.push(format!("{lang}/{file}: key {id} is missing"));
        }
        for id in own.difference(&expected) {
            problems.push(format!(
                "{lang}/{file}: key {id} does not exist in {REFERENCE_LANGUAGE}"
            ));
        }
    }
    problems
}

/// Rust constants for `codeskimr_codes::i18n`.
#[must_use]
pub fn render_rust(catalog: &Catalog) -> String {
    let keys = catalog.keys();
    let mut out = format!("// {GENERATED_NOTICE_I18N}\n\n");
    for key in &keys {
        let _ = writeln!(
            out,
            "/// Message key `{id}` ({file}).\npub const {}: &str = {id:?};",
            key.const_name(),
            id = key.id,
            file = key.file,
        );
    }
    out.push_str("\n/// Every message key listed above, for completeness tests.\n");
    let _ = writeln!(out, "pub const ALL: [&str; {}] = [", keys.len());
    for key in &keys {
        let _ = writeln!(out, "    {},", key.const_name());
    }
    out.push_str("];\n");
    out
}

/// TypeScript constants and the key union type.
#[must_use]
pub fn render_ts(catalog: &Catalog) -> String {
    let mut out = format!("// {GENERATED_NOTICE_I18N}\n\n");
    out.push_str("/** All message keys by constant name. */\nexport const I18N_KEYS = {\n");
    for key in catalog.keys() {
        let _ = writeln!(out, "  {}: {:?},", key.const_name(), key.id);
    }
    out.push_str("} as const;\n\n/** A message key that exists in every language. */\n");
    out.push_str("export type I18nKey = (typeof I18N_KEYS)[keyof typeof I18N_KEYS];\n");
    out
}

/// Rust source with every Fluent file embedded into the binary (used by `codeskimr_i18n`).
///
/// `root` is the repository root as the including crate sees it; the paths are written so that
/// `include_str!` finds the files.
#[must_use]
pub fn render_embedded(catalog: &Catalog, root: &Path) -> String {
    let mut out = format!("// {GENERATED_NOTICE_I18N}\n\n");
    out.push_str("/// Every language with its Fluent files as `(file name, text)`.\n");
    out.push_str("pub const EMBEDDED: &[(&str, &[(&str, &str)])] = &[\n");
    for (lang, files) in &catalog.files {
        let _ = writeln!(out, "    ({lang:?}, &[");
        for file in files.keys() {
            let path = root.join(LOCALES_DIR).join(lang).join(file);
            let _ = writeln!(
                out,
                "        ({file:?}, include_str!({:?})),",
                path.display().to_string()
            );
        }
        out.push_str("    ]),\n");
    }
    out.push_str("];\n");
    out
}
