//! Corpus test (blueprint 17.2): scans the pinned public C# project of `corpus/csharp` and
//! compares the symbol counts and a hand-checked sample with `corpus/csharp/corpus.toml`.
//!
//! The project is fetched with `git` at the pinned commit into `target/corpus/` on first use.
//! Set `CODESKIMR_CORPUS_DIR` to use an existing checkout instead. Set `CODESKIMR_BLESS=1` to
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
        if let Some(dir) = std::env::var_os("CODESKIMR_CORPUS_DIR") {
            return PathBuf::from(dir);
        }
        let dir = workspace()
            .join("target/corpus/csharp")
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

    fn sources(dir: &Path, skip: &[String], found: &mut Vec<PathBuf>) {
        let mut entries: Vec<_> = fs::read_dir(dir).expect("folder").flatten().collect();
        entries.sort_by_key(fs::DirEntry::path);
        for entry in entries {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            if path.is_dir() {
                if !skip.contains(&name) && name != ".git" {
                    sources(&path, skip, found);
                }
            } else if path.extension().is_some_and(|e| e == "cs") && !name.ends_with(".g.cs") {
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

    fn scan(root: &Path, skip: &[String]) -> Scan {
        let analyzer = TreeSitterAnalyzer::new(
            LanguagePacks::load(&workspace().join("languages")).expect("packs"),
            Duration::from_secs(10),
        );
        let mut files = Vec::new();
        sources(root, skip, &mut files);
        let folder = FolderPath::new(root.to_string_lossy()).expect("root");
        let ids = IdSequence::starting_at(1);
        let mut scan = Scan::default();
        scan.counts.insert("files".to_owned(), files.len());
        for path in &files {
            let relative = path.strip_prefix(root).expect("inside root");
            let relative = relative.to_string_lossy().replace('\\', "/");
            let facts = analyze_file(&analyzer, &ids, &folder, &relative, "csharp");
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
        let text = fs::read_to_string(workspace().join("corpus/csharp/corpus.toml"));
        let corpus: Corpus =
            toml::from_str(&text.expect("corpus.toml")).expect("valid corpus.toml");
        let found = scan(&checkout(&corpus.project), &corpus.project.skip_folders);
        if std::env::var_os("CODESKIMR_BLESS").is_some() {
            println!("MEASURED {:#?}", found.counts);
            println!("SUMMARY {:?}", summary(&found.counts));
            return;
        }
        assert_eq!(found.counts, corpus.counts, "symbol counts");
        check_samples(&corpus, &found);
    }
}
