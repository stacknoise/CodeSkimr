//! Corpus test (blueprint 17.2): scans the pinned public project of every `corpus/<language>` and
//! compares the symbol counts and a hand-checked sample with its `corpus.toml`.
//!
//! The project is fetched with `git` at the pinned commit into `target/corpus/` on first use.
//! Set `CODESKIMR_CORPUS_DIR` to use an existing checkout of the C# project instead (or of the
//! language named in `CODESKIMR_CORPUS_LANGUAGE`). Set `CODESKIMR_BLESS=1` to
//! print the measured counts instead of comparing.

#[cfg(test)]
mod common;

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::time::Duration;

    use super::common::analyze_file;
    use codeskimr_application::ids::IdSequence;
    use codeskimr_application::ports::FileFacts;
    use codeskimr_domain::{FolderPath, Visibility};
    use codeskimr_parsing::{LanguagePacks, TreeSitterAnalyzer};
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct Corpus {
        project: Project,
        counts: BTreeMap<String, usize>,
        #[serde(default)]
        sample: Vec<Sample>,
        #[serde(default)]
        references: Vec<Occurrence>,
    }

    #[derive(Deserialize)]
    struct Project {
        repository: String,
        commit: String,
        skip_folders: Vec<String>,
        /// Id of the language pack that analyzes the files.
        language: String,
        /// File extensions of the sources, without the dot.
        extensions: Vec<String>,
        /// File name endings that mark generated files to skip.
        #[serde(default)]
        skip_suffixes: Vec<String>,
    }

    /// A symbol that was checked by hand in the source.
    #[derive(Deserialize)]
    struct Sample {
        file: String,
        kind: String,
        name: String,
        visibility: Option<String>,
        parent: Option<String>,
    }

    /// A reference that was checked by hand in the source.
    #[derive(Deserialize)]
    struct Occurrence {
        file: String,
        kind: String,
        target: String,
        line: u32,
    }

    fn workspace() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
    }

    fn git(dir: &Path, args: &[&str]) -> bool {
        Command::new("git")
            .args(args)
            .current_dir(dir)
            .status()
            .is_ok_and(|status| status.success())
    }

    /// Fetches the pinned commit into a fresh folder.
    fn fetch(dir: &Path, project: &Project) -> bool {
        fs::remove_dir_all(dir).ok();
        fs::create_dir_all(dir).is_ok()
            && git(dir, &["init", "-q"])
            && git(dir, &["remote", "add", "origin", &project.repository])
            && git(
                dir,
                &["fetch", "-q", "--depth", "1", "origin", &project.commit],
            )
            && git(dir, &["checkout", "-q", "-f", "FETCH_HEAD"])
    }

    /// The pinned commit in `target/corpus`; a folder that cannot be checked out again (for example
    /// pruned by a build cache) is fetched anew.
    fn checkout(project: &Project) -> PathBuf {
        let language = std::env::var("CODESKIMR_CORPUS_LANGUAGE").unwrap_or("csharp".to_owned());
        if language == project.language
            && let Some(dir) = std::env::var_os("CODESKIMR_CORPUS_DIR")
        {
            return PathBuf::from(dir);
        }
        let dir = workspace()
            .join("target/corpus")
            .join(&project.language)
            .join(&project.commit);
        let present =
            dir.join(".git").exists() && git(&dir, &["checkout", "-q", "-f", &project.commit]);
        assert!(
            present || fetch(&dir, project),
            "cannot fetch {} at {}; without network set CODESKIMR_CORPUS_DIR to a checkout of the pinned commit",
            project.repository,
            project.commit
        );
        dir
    }

    fn is_source(project: &Project, path: &Path, name: &str) -> bool {
        path.extension()
            .is_some_and(|e| project.extensions.iter().any(|x| e == x.as_str()))
            && !project
                .skip_suffixes
                .iter()
                .any(|s| name.ends_with(s.as_str()))
    }

    fn sources(dir: &Path, project: &Project, found: &mut Vec<PathBuf>) {
        let mut entries: Vec<_> = fs::read_dir(dir).expect("folder").flatten().collect();
        entries.sort_by_key(fs::DirEntry::path);
        for entry in entries {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            if path.is_dir() {
                if !project.skip_folders.contains(&name) && name != ".git" {
                    sources(&path, project, found);
                }
            } else if is_source(project, &path, &name) {
                found.push(path);
            }
        }
    }

    /// A symbol of the scan: file, kind label, name, visibility label and parent name.
    type Found = (
        String,
        &'static str,
        String,
        Option<&'static str>,
        Option<String>,
    );

    #[derive(Default)]
    struct Scan {
        counts: BTreeMap<String, usize>,
        symbols: Vec<Found>,
        /// File, kind label, target and 1-based line.
        references: Vec<(String, &'static str, String, u32)>,
    }

    fn scan(root: &Path, project: &Project) -> Scan {
        let analyzer = TreeSitterAnalyzer::new(
            LanguagePacks::load(&workspace().join("languages")).expect("packs"),
            Duration::from_secs(10),
        );
        let mut files = Vec::new();
        sources(root, project, &mut files);
        let folder = FolderPath::new(root.to_string_lossy()).expect("root");
        let ids = IdSequence::starting_at(1);
        let mut scan = Scan::default();
        scan.counts.insert("files".to_owned(), files.len());
        for path in &files {
            let relative = path.strip_prefix(root).expect("inside root");
            let relative = relative.to_string_lossy().replace('\\', "/");
            let facts = analyze_file(&analyzer, &ids, &folder, &relative, &project.language);
            record(&mut scan, &relative, &facts);
        }
        scan
    }

    fn record(scan: &mut Scan, file: &str, facts: &FileFacts) {
        let by_id: BTreeMap<_, _> = facts.symbols.iter().map(|s| (s.id, s)).collect();
        for symbol in &facts.symbols {
            *scan
                .counts
                .entry(symbol.kind.label().to_owned())
                .or_default() += 1;
            let parent = symbol.parent.and_then(|p| by_id.get(&p));
            scan.symbols.push((
                file.to_owned(),
                symbol.kind.label(),
                symbol.name.to_string(),
                symbol.visibility.map(Visibility::label),
                parent.map(|p| p.name.to_string()),
            ));
        }
        for pending in &facts.pending {
            scan.references.push((
                file.to_owned(),
                pending.kind.label(),
                pending.target.to_string(),
                pending.span.start().line + 1,
            ));
        }
    }

    fn check_samples(corpus: &Corpus, scan: &Scan) {
        for sample in &corpus.sample {
            let hit = scan
                .symbols
                .iter()
                .any(|(file, kind, name, visibility, parent)| {
                    *file == sample.file
                        && *kind == sample.kind
                        && *name == sample.name
                        && *visibility == sample.visibility.as_deref()
                        && *parent == sample.parent
                });
            assert!(
                hit,
                "symbol {} {} in {}",
                sample.kind, sample.name, sample.file
            );
        }
        for want in &corpus.references {
            let hit = scan.references.iter().any(|(file, kind, target, line)| {
                *file == want.file
                    && *kind == want.kind
                    && *target == want.target
                    && *line == want.line
            });
            assert!(
                hit,
                "reference {} {} in {}:{}",
                want.kind, want.target, want.file, want.line
            );
        }
    }

    /// The counting rule of the private reference (ADR 0017): groups of symbol kinds.
    const GROUPS: [(&str, &[&str]); 6] = [
        (
            "types",
            &[
                "class",
                "struct",
                "interface",
                "record",
                "enum",
                "type_alias",
            ],
        ),
        ("methods", &["method", "constructor", "function"]),
        ("properties", &["property"]),
        ("fields", &["field", "constant"]),
        ("events", &["event"]),
        ("enum_members", &["enum_member"]),
    ];

    fn summary(counts: &BTreeMap<String, usize>) -> BTreeMap<&'static str, usize> {
        GROUPS
            .iter()
            .map(|(group, kinds)| {
                let total = kinds.iter().filter_map(|kind| counts.get(*kind)).sum();
                (*group, total)
            })
            .collect()
    }

    #[test]
    fn scan_matches_the_expected_counts_and_sample() {
        let mut folders: Vec<_> = fs::read_dir(workspace().join("corpus"))
            .expect("corpus folder")
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| path.join("corpus.toml").is_file())
            .collect();
        folders.sort();
        assert!(!folders.is_empty(), "no corpus.toml found");
        for folder in folders {
            let text = fs::read_to_string(folder.join("corpus.toml")).expect("corpus.toml");
            let corpus: Corpus = toml::from_str(&text).expect("valid corpus.toml");
            let found = scan(&checkout(&corpus.project), &corpus.project);
            if std::env::var_os("CODESKIMR_BLESS").is_some() {
                println!("MEASURED {} {:#?}", corpus.project.language, found.counts);
                println!("SUMMARY {:?}", summary(&found.counts));
                continue;
            }
            assert_eq!(
                found.counts,
                corpus.counts,
                "symbol counts of {}",
                folder.display()
            );
            check_samples(&corpus, &found);
        }
    }
}
