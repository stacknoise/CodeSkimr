//! Golden tests of the real language packs in `languages/` (blueprint 8.1 and 17.1).
//!
//! Every file in `languages/<id>/tests/` with an extension of the pack is analyzed and compared
//! with the `<file>.json` next to it. Set `CODESKIMR_BLESS=1` to write the expected files.

#[cfg(test)]
mod common;

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::Duration;

    use super::common::analyze_file;
    use codeskimr_application::ids::IdSequence;
    use codeskimr_application::ports::{FileFacts, Receiver};
    use codeskimr_domain::{FolderPath, Position, SymbolId, Visibility};
    use codeskimr_parsing::{LanguagePack, LanguagePacks, TreeSitterAnalyzer};
    use serde_json::{Value, json};

    fn languages() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../languages")
    }

    fn at(position: Position) -> String {
        format!("{}:{}", position.line, position.column)
    }

    fn receiver(receiver: &Receiver) -> String {
        match receiver {
            Receiver::Typed(name) => format!("typed {name}"),
            Receiver::Named(name) => format!("named {name}"),
            Receiver::Own => "own".to_owned(),
            Receiver::Base => "base".to_owned(),
        }
    }

    fn describe_counts(facts: &FileFacts, names: &HashMap<SymbolId, String>) -> Vec<Value> {
        facts
            .counts
            .iter()
            .map(|c| {
                let h = c.body.map(|b| b.halstead);
                json!({
                    "symbol": names.get(&c.symbol),
                    "loc": c.loc.0,
                    "decisions": c.body.map(|b| b.decisions),
                    "operators": h.map(|h| [h.distinct_operators, h.total_operators]),
                    "operands": h.map(|h| [h.distinct_operands, h.total_operands]),
                })
            })
            .collect()
    }

    fn describe(facts: &FileFacts) -> Value {
        let names: HashMap<_, _> = facts
            .symbols
            .iter()
            .map(|s| (s.id, s.qualified_name.to_string()))
            .collect();
        let symbols: Vec<Value> = facts
            .symbols
            .iter()
            .map(|s| {
                json!({
                    "kind": s.kind.label(),
                    "name": s.qualified_name.to_string(),
                    "parent": s.parent.and_then(|p| names.get(&p)),
                    "visibility": s.visibility.map(Visibility::label),
                    "static": s.is_static,
                    "span": format!("{}-{}", at(s.span.start()), at(s.span.end())),
                    "doc": s.doc,
                })
            })
            .collect();
        let references: Vec<Value> = facts
            .pending
            .iter()
            .map(|r| {
                json!({
                    "from": names.get(&r.from),
                    "kind": r.kind.label(),
                    "target": r.target.to_string(),
                    "receiver": r.receiver.as_ref().map(receiver),
                    "at": at(r.span.start()),
                })
            })
            .collect();
        let counts = describe_counts(facts, &names);
        json!({
            "loc": facts.file.loc.0,
            "symbols": symbols,
            "references": references,
            "counts": counts,
        })
    }

    fn check(expected: &Path, actual: &Value, bless: bool) -> Option<String> {
        let text = serde_json::to_string_pretty(actual).expect("json") + "\n";
        if bless {
            fs::write(expected, text).expect("write expected");
            return None;
        }
        let stored = fs::read_to_string(expected).unwrap_or_default();
        (stored != text).then(|| format!("{} differs from the analysis", expected.display()))
    }

    #[test]
    fn every_pack_matches_its_golden_files() {
        let bless = std::env::var_os("CODESKIMR_BLESS").is_some();
        let analyzer = TreeSitterAnalyzer::new(
            LanguagePacks::load(&languages()).expect("packs load"),
            Duration::from_secs(10),
        );
        let mut failures = Vec::new();
        let mut checked = 0;
        for entry in fs::read_dir(languages()).expect("languages folder") {
            let pack_dir = entry.expect("entry").path();
            if !pack_dir.is_dir() {
                continue;
            }
            let pack = LanguagePack::load(&pack_dir)
                .unwrap_or_else(|error| panic!("{}: {error}", pack_dir.display()));
            let tests = pack_dir.join("tests");
            for file in fs::read_dir(&tests).expect("tests folder") {
                let path = file.expect("entry").path();
                let extension = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or_default();
                if !pack.extensions().iter().any(|e| e == extension) {
                    continue;
                }
                let name = path
                    .file_name()
                    .expect("name")
                    .to_string_lossy()
                    .into_owned();
                let root = FolderPath::new(tests.to_string_lossy()).expect("root");
                let ids = IdSequence::starting_at(1);
                let facts = analyze_file(&analyzer, &ids, &root, &name, pack.id().as_str());
                let expected = tests.join(format!("{name}.json"));
                failures.extend(check(&expected, &describe(&facts), bless));
                checked += 1;
            }
        }
        assert!(checked > 0, "no golden sample found");
        assert!(failures.is_empty(), "{failures:#?}");
    }
}
