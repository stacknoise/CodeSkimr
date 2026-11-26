//! Finding the lines of a project folder that hold a name (`NameMentions`, ADR 0104).

use std::collections::HashSet;
use std::io::Read;
use std::path::Path;

use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{Mention, MentionLimits, Mentions, NameMentions};
use codeskimr_application::rename::find_word;
use codeskimr_domain::{FolderPath, Scope};
use codeskimr_fsguard::open_regular;
use ignore::DirEntry;

use crate::consts::{BINARY_MARKER, LINE_END_CHAR, MENTION_TEXT_CHARS};
use crate::discovery::builder;
use crate::folder::open_folder;
use crate::paths::relative_path;

/// Reads every text file below a folder, in the order of the paths, with the walk of a scan:
/// `.gitignore` files inside the folder count, the `.git` folder and the excluded folders are
/// never entered, links are never followed.
#[derive(Debug, Clone, Default)]
pub struct FolderMentions {
    excluded_folders: Vec<String>,
}

impl FolderMentions {
    /// Creates the adapter; `excluded_folders` are the folder names a scan never enters
    /// (`scan.excluded_folders`).
    #[must_use]
    pub const fn new(excluded_folders: Vec<String>) -> Self {
        Self { excluded_folders }
    }
}

/// What one search keeps to.
struct Search<'a> {
    root: &'a Path,
    name: &'a str,
    limits: &'a MentionLimits,
    except: &'a HashSet<(String, u32)>,
}

impl NameMentions for FolderMentions {
    fn mentions(
        &self,
        root: &FolderPath,
        name: &str,
        limits: &MentionLimits,
        except: &HashSet<(String, u32)>,
    ) -> Result<Mentions, CodedError> {
        open_folder(root)?;
        let search = Search {
            root: Path::new(root.as_str()),
            name,
            limits,
            except,
        };
        let mut found = Mentions::default();
        let walk = builder(search.root, &Scope::default(), &self.excluded_folders)
            .sort_by_file_path(Ord::cmp)
            .build();
        for entry in walk {
            match entry {
                Ok(entry) if entry.file_type().is_some_and(|kind| kind.is_file()) => {
                    read_file(&search, &entry, &mut found);
                }
                Ok(_) => {}
                Err(_) => found.skipped += 1,
            }
        }
        Ok(found)
    }
}

/// A file that could not be read or is too large.
struct Unreadable;

/// The text of a file: `None` if it is a binary file, `Err` if it could not be read or is too large.
fn text_of(path: &Path, max_bytes: u64) -> Result<Option<String>, Unreadable> {
    let file = open_regular(path).map_err(|_| Unreadable)?;
    if file.metadata().map_err(|_| Unreadable)?.len() > max_bytes {
        return Err(Unreadable);
    }
    let mut bytes = Vec::new();
    file.take(max_bytes)
        .read_to_end(&mut bytes)
        .map_err(|_| Unreadable)?;
    if bytes.contains(&BINARY_MARKER) {
        return Ok(None);
    }
    Ok(Some(String::from_utf8_lossy(&bytes).into_owned()))
}

/// Adds the lines of one file that hold the name.
fn read_file(search: &Search<'_>, entry: &DirEntry, found: &mut Mentions) {
    let Some(path) = relative_path(search.root, entry.path()) else {
        found.skipped += 1;
        return;
    };
    let text = match text_of(entry.path(), search.limits.max_file_bytes) {
        Ok(Some(text)) => text,
        Ok(None) => return,
        Err(Unreadable) => {
            found.skipped += 1;
            return;
        }
    };
    for (index, line) in text.split(LINE_END_CHAR).enumerate() {
        let number = u32::try_from(index).unwrap_or(u32::MAX);
        if find_word(line, search.name).is_none()
            || search.except.contains(&(path.as_str().to_owned(), number))
        {
            continue;
        }
        found.total += 1;
        if found.items.len() < search.limits.max_items {
            found.items.push(Mention {
                path: path.clone(),
                line: number,
                text: line.trim().chars().take(MENTION_TEXT_CHARS).collect(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn write_file(base: &std::path::Path, name: &str, text: &str) {
        let target = base.join(name);
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::write(target, text).unwrap();
    }

    fn folder(files: &[(&str, &str)]) -> (tempfile::TempDir, FolderPath) {
        let dir = tempfile::tempdir().unwrap();
        for (name, text) in files {
            write_file(dir.path(), name, text);
        }
        let root = FolderPath::new(dir.path().to_str().unwrap()).unwrap();
        (dir, root)
    }

    fn find(root: &FolderPath, except: &[(&str, u32)], max_items: usize) -> Mentions {
        let except = except
            .iter()
            .map(|(path, line)| ((*path).to_owned(), *line))
            .collect();
        let limits = MentionLimits {
            max_items,
            max_file_bytes: 1000,
        };
        FolderMentions::new(vec!["node_modules".to_owned()])
            .mentions(root, "PlaceOrder", &limits, &except)
            .unwrap()
    }

    #[test]
    fn it_finds_whole_words_in_every_kind_of_text_file_in_path_order() {
        let (_dir, root) = folder(&[
            ("docs/api.md", "intro\nPOST /orders calls PlaceOrder\n"),
            ("Api/appsettings.json", "  \"audit\": [\"PlaceOrder\"]\n"),
            (
                "Api/Other.cs",
                "PlaceOrderAsync(); my_PlaceOrder(); PlaceOrder2();\n",
            ),
        ]);
        let found = find(&root, &[], 10);
        let places: Vec<(&str, u32)> = found
            .items
            .iter()
            .map(|mention| (mention.path.as_str(), mention.line))
            .collect();
        assert_eq!(places, [("Api/appsettings.json", 0), ("docs/api.md", 1)]);
        assert_eq!(found.total, 2);
        assert_eq!(found.items[0].text, "\"audit\": [\"PlaceOrder\"]");
    }

    #[test]
    fn it_leaves_out_the_known_lines_and_cuts_the_list_but_counts_on() {
        let (_dir, root) = folder(&[("a.cs", "PlaceOrder\nPlaceOrder\nPlaceOrder\n")]);
        let found = find(&root, &[("a.cs", 0)], 1);
        assert_eq!(found.total, 2);
        assert_eq!(found.items.len(), 1);
        assert_eq!(found.items[0].line, 1);
    }

    #[test]
    fn it_skips_ignored_and_excluded_folders_binary_and_large_files() {
        let (_dir, root) = folder(&[
            (".gitignore", "build/\n"),
            ("build/out.txt", "PlaceOrder"),
            ("node_modules/x/index.js", "PlaceOrder"),
            ("image.bin", "PlaceOrder\0\u{1}"),
            ("big.txt", &format!("PlaceOrder{}", "x".repeat(2000))),
            ("ok.txt", "PlaceOrder"),
        ]);
        let found = find(&root, &[], 10);
        let paths: Vec<&str> = found
            .items
            .iter()
            .map(|mention| mention.path.as_str())
            .collect();
        assert_eq!(paths, ["ok.txt"]);
        assert_eq!(found.skipped, 1);
    }

    #[test]
    fn a_missing_folder_is_not_found() {
        let missing = FolderPath::new("/definitely/not/here").unwrap();
        let limits = MentionLimits {
            max_items: 1,
            max_file_bytes: 1,
        };
        let error = FolderMentions::default()
            .mentions(&missing, "x", &limits, &HashSet::new())
            .unwrap_err();
        assert_eq!(codeskimr_codes::Coded::code(&error).entry().id, "SRC-201");
    }
}
