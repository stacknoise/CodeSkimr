//! The analysis of a C# file and the resolver together (blueprint 9.2).

#[cfg(test)]
mod common;

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::Path;
    use std::time::Duration;

    use super::common::analyze_file;
    use codeskimr_application::ids::IdSequence;
    use codeskimr_application::ports::ReferenceResolver;
    use codeskimr_application::resolve::ScopedResolver;
    use codeskimr_domain::{Confidence, FolderPath, ReferenceKind};
    use codeskimr_parsing::{LanguagePacks, TreeSitterAnalyzer};

    #[test]
    fn locals_hide_members_and_members_of_the_enclosing_type_are_certain() {
        let languages = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../languages");
        let analyzer = TreeSitterAnalyzer::new(
            LanguagePacks::load(&languages).expect("packs"),
            Duration::from_secs(10),
        );
        let root = FolderPath::new(languages.join("csharp/tests").to_string_lossy()).expect("root");
        let ids = IdSequence::starting_at(1);
        let facts = analyze_file(&analyzer, &ids, &root, "scopes.cs", "csharp");
        let found = ScopedResolver
            .resolve(&facts.symbols, &facts.pending, &ids)
            .expect("resolves");
        let name_of = |id| {
            facts
                .symbols
                .iter()
                .find(|s| s.id == id)
                .map(|s| s.qualified_name.to_string())
                .unwrap_or_default()
        };
        let writes: Vec<(String, Confidence)> = found
            .iter()
            .filter(|r| r.kind == ReferenceKind::Write)
            .map(|r| (name_of(r.to), r.confidence))
            .collect();
        assert!(writes.contains(&(
            "scopes.Shop.Orders.Order.Total".to_owned(),
            Confidence::Certain
        )));
        assert!(writes.contains(&(
            "scopes.Shop.Orders.Order.printer".to_owned(),
            Confidence::Certain
        )));
        let reads_of_parameters = found
            .iter()
            .filter(|r| name_of(r.to).contains("Order.total"))
            .count();
        assert_eq!(reads_of_parameters, 0);
    }

    /// Resolves `usages.cs` once and lists the `(line, confidence)` of every usage per target.
    fn usages() -> BTreeMap<String, Vec<(u32, Confidence)>> {
        let languages = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../languages");
        let analyzer = TreeSitterAnalyzer::new(
            LanguagePacks::load(&languages).expect("packs"),
            Duration::from_secs(10),
        );
        let root = FolderPath::new(languages.join("csharp/tests").to_string_lossy()).expect("root");
        let ids = IdSequence::starting_at(1);
        let facts = analyze_file(&analyzer, &ids, &root, "usages.cs", "csharp");
        let found = ScopedResolver
            .resolve(&facts.symbols, &facts.pending, &ids)
            .expect("resolves");
        let mut by_target: BTreeMap<String, Vec<(u32, Confidence)>> = BTreeMap::new();
        for reference in &found {
            let name = facts
                .symbols
                .iter()
                .find(|s| s.id == reference.to)
                .map(|s| s.qualified_name.to_string())
                .unwrap_or_default();
            let usage = (reference.span.start().line, reference.confidence);
            by_target.entry(name).or_default().push(usage);
        }
        by_target
    }

    #[test]
    fn values_in_arguments_returns_cases_attributes_and_initializers_are_usages() {
        let found = usages();
        let lines = |name: &str| -> Vec<u32> {
            found
                .get(&format!("usages.Demo.{name}"))
                .map(|all| all.iter().map(|(line, _)| *line).collect())
                .unwrap_or_default()
        };
        assert_eq!(lines("Names.Arg"), [27]);
        assert_eq!(lines("Names.Ret"), [28]);
        assert_eq!(lines("Names.Case"), [33]);
        assert_eq!(lines("Names.Attr"), [37]);
        assert_eq!(lines("Names.Lambda"), [41]);
        assert_eq!(lines("Item.Count"), [39]);
        assert_eq!(lines("IOne.Size"), [40]);
    }

    #[test]
    fn a_receiver_of_unknown_type_does_not_pick_between_unrelated_types() {
        let found = usages();
        let lines = |name: &str| -> Vec<u32> {
            found
                .get(&format!("usages.Demo.{name}"))
                .map(|all| all.iter().map(|(line, _)| *line).collect())
                .unwrap_or_default()
        };
        // `var x = svc.Make()` and `var y = Svc.Load<ITwo>()` have no type the resolver knows:
        // `Close` is neither IOne's nor ITwo's. `var z = (IOne)...` is typed by the cast.
        assert_eq!(lines("IOne.Close"), [54]);
        assert!(lines("ITwo.Close").is_empty());
    }
}
