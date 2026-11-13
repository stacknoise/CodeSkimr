//! The analysis of a C# file and the resolver together (blueprint 9.2).

#[cfg(test)]
mod common;

#[cfg(test)]
mod tests {
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
}
