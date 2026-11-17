//! The content hash of a symbol (ADR 0068): over the tokens of its span, whitespace and comments
//! excluded, for every language pack through the same walk of the syntax tree.

#[cfg(test)]
mod common;

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::OnceLock;
    use std::time::Duration;

    use super::common::analyze_file;
    use codeskimr_application::ids::IdSequence;
    use codeskimr_application::ports::{ContentDigest, FileFacts};
    use codeskimr_domain::{FolderPath, SymbolKind};
    use codeskimr_parsing::{Blake3Digest, LanguagePacks, TreeSitterAnalyzer};

    /// One case per language pack: the language, the comment marker, the file name and a type with a
    /// method `work` whose body adds one (a function `work` for C, which has no types).
    const CASES: &[(&str, &str, &str, &str)] = &[
        (
            "csharp",
            "//",
            "Box.cs",
            "namespace Demo;\npublic class Box\n{\n    public int Work(int x)\n    {\n        return x + 1;\n    }\n}\n",
        ),
        (
            "rust",
            "//",
            "box.rs",
            "pub struct Box;\nimpl Box {\n    pub fn work(x: u32) -> u32 {\n        x + 1\n    }\n}\n",
        ),
        (
            "python",
            "#",
            "box.py",
            "class Box:\n    def work(self, x):\n        return x + 1\n",
        ),
        (
            "java",
            "//",
            "Box.java",
            "public class Box {\n    public int work(int x) {\n        return x + 1;\n    }\n}\n",
        ),
        (
            "kotlin",
            "//",
            "Box.kt",
            "class Box {\n    fun work(x: Int): Int {\n        return x + 1\n    }\n}\n",
        ),
        (
            "typescript",
            "//",
            "box.ts",
            "export class Box {\n  work(x: number): number {\n    return x + 1;\n  }\n}\n",
        ),
        (
            "tsx",
            "//",
            "box.tsx",
            "export class Box {\n  work(x: number): number {\n    return x + 1;\n  }\n}\n",
        ),
        (
            "javascript",
            "//",
            "box.js",
            "export class Box {\n  work(x) {\n    return x + 1;\n  }\n}\n",
        ),
        (
            "go",
            "//",
            "box.go",
            "package demo\n\ntype Box struct{}\n\nfunc (b Box) work(x int) int {\n\treturn x + 1\n}\n",
        ),
        (
            "c",
            "//",
            "box.c",
            "int work(int x) {\n    return x + 1;\n}\n",
        ),
        (
            "cpp",
            "//",
            "box.cpp",
            "class Box {\npublic:\n    int work(int x) {\n        return x + 1;\n    }\n};\n",
        ),
        (
            "ruby",
            "#",
            "box.rb",
            "class Box\n  def work(x)\n    x + 1\n  end\nend\n",
        ),
        (
            "php",
            "//",
            "box.php",
            "<?php\nclass Box {\n    public function work($x) {\n        return $x + 1;\n    }\n}\n",
        ),
    ];

    /// The packs load once for all tests of this file.
    static ANALYZER: OnceLock<TreeSitterAnalyzer> = OnceLock::new();

    fn languages() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../languages")
    }

    /// The same code with a comment after every line, blank lines between them and wider gaps in
    /// the line of the method: what a formatter or a comment does, nothing else.
    fn cosmetic(source: &str, marker: &str) -> String {
        source
            .lines()
            .map(|line| format!("{line}  {marker} a comment"))
            .collect::<Vec<_>>()
            .join("\n\n")
            + "\n"
    }

    fn edited(source: &str) -> String {
        source.replace("+ 1", "+ 2")
    }

    /// The hashes of the file by simple name; a symbol without one is missing.
    fn hashes_of(language: &str, file: &str, source: &str) -> HashMap<String, String> {
        let folder = tempfile::tempdir().expect("folder");
        let root = folder.path();
        fs::write(root.join(file), source).expect("file");
        let analyzer = ANALYZER.get_or_init(|| {
            TreeSitterAnalyzer::new(
                LanguagePacks::load(&languages()).expect("packs load"),
                Duration::from_secs(10),
            )
        });
        let facts = analyze_file(
            analyzer,
            &IdSequence::starting_at(1),
            &FolderPath::new(root.to_string_lossy()).expect("root"),
            file,
            language,
        );
        by_name(&facts)
    }

    fn by_name(facts: &FileFacts) -> HashMap<String, String> {
        facts
            .hashes
            .iter()
            .filter_map(|found| {
                let symbol = facts.symbols.iter().find(|s| s.id == found.symbol)?;
                Some((symbol.name.to_string(), found.hash.to_string()))
            })
            .collect()
    }

    #[test]
    fn a_cosmetic_edit_keeps_the_hash_and_a_change_of_code_does_not() {
        for (language, marker, file, source) in CASES {
            let before = hashes_of(language, file, source);
            let same = hashes_of(language, file, &cosmetic(source, marker));
            let changed = hashes_of(language, file, &edited(source));
            assert!(
                before.contains_key("work") || before.contains_key("Work"),
                "{language}: {before:?}"
            );
            assert_eq!(
                before, same,
                "{language}: whitespace and comments are no change"
            );
            let method = if before.contains_key("work") {
                "work"
            } else {
                "Work"
            };
            assert_ne!(
                changed.get(method),
                before.get(method),
                "{language}: {method} is edited"
            );
            // A type that holds its members changes with them; in Rust and Go the methods stand
            // outside the type, which then keeps its hash.
            let outside = ["rust", "go"].contains(language);
            if before.contains_key("Box") && !outside {
                assert_ne!(
                    changed.get("Box"),
                    before.get("Box"),
                    "{language}: Box holds the method"
                );
            }
        }
    }

    #[test]
    fn a_rust_type_does_not_hash_its_impl_blocks() {
        let (language, _, file, source) = CASES[1];
        let before = hashes_of(language, file, source);
        let after = hashes_of(language, file, &edited(source));
        assert_eq!(before.get("Box"), after.get("Box"));
        assert_ne!(before.get("work"), after.get("work"));
    }

    #[test]
    fn every_pack_hashes_its_types_and_members() {
        for (language, _, file, source) in CASES {
            let found = hashes_of(language, file, source);
            let has_type = source.contains("class") || source.contains("struct");
            let expected = if has_type { 2 } else { 1 };
            assert!(found.len() >= expected, "{language}: {found:?}");
        }
    }

    #[test]
    fn a_type_hashes_its_own_span_and_a_member_only_its_own() {
        let source = "namespace Demo;\npublic class Box\n{\n    public int One() { return 1; }\n    public int Two() { return 2; }\n}\n";
        let before = hashes_of("csharp", "Box.cs", source);
        let after = hashes_of("csharp", "Box.cs", &source.replace("return 2", "return 3"));
        assert_eq!(before.get("One"), after.get("One"));
        assert_ne!(before.get("Two"), after.get("Two"));
        assert_ne!(before.get("Box"), after.get("Box"));
    }

    #[test]
    fn a_namespace_and_a_file_have_no_hash_of_their_own() {
        let source = "namespace Demo;\npublic class Box\n{\n}\n";
        let found = hashes_of("csharp", "Box.cs", source);
        assert_eq!(found.get("Demo"), None);
        assert_eq!(found.len(), 1);
        assert!(!SymbolKind::Namespace.has_content_hash());
        assert!(!SymbolKind::Module.has_content_hash());
    }

    #[test]
    fn two_scans_of_the_same_file_give_the_same_hashes() {
        let (language, _, file, source) = CASES[0];
        assert_eq!(
            hashes_of(language, file, source),
            hashes_of(language, file, source)
        );
    }

    #[test]
    fn the_digest_keeps_the_parts_apart() {
        let digest = Blake3Digest;
        let split = digest.digest(&["ab", "c"]).expect("hash");
        assert_ne!(split, digest.digest(&["a", "bc"]).expect("hash"));
        assert_eq!(split, digest.digest(&["ab", "c"]).expect("hash"));
        assert_eq!(split.as_str().len(), 64);
    }
}
