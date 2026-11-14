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
    use codeskimr_application::ports::{FileFacts, ReferenceResolver};
    use codeskimr_application::resolve::ScopedResolver;
    use codeskimr_domain::{Confidence, FolderPath, Reference, ReferenceKind};
    use codeskimr_parsing::{LanguagePacks, TreeSitterAnalyzer};

    /// Analyzes and resolves one sample file of the C# pack.
    fn resolved(file: &str) -> (FileFacts, Vec<Reference>) {
        let languages = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../languages");
        let analyzer = TreeSitterAnalyzer::new(
            LanguagePacks::load(&languages).expect("packs"),
            Duration::from_secs(10),
        );
        let root = FolderPath::new(languages.join("csharp/tests").to_string_lossy()).expect("root");
        let ids = IdSequence::starting_at(1);
        let facts = analyze_file(&analyzer, &ids, &root, file, "csharp");
        let found = ScopedResolver
            .resolve(&facts.symbols, &facts.pending, &ids)
            .expect("resolves");
        (facts, found)
    }

    #[test]
    fn locals_hide_members_and_members_of_the_enclosing_type_are_certain() {
        let (facts, found) = resolved("scopes.cs");
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
        assert!(writes.contains(&("Shop.Orders.Order.Total".to_owned(), Confidence::Certain)));
        assert!(writes.contains(&("Shop.Orders.Order.printer".to_owned(), Confidence::Certain)));
        let reads_of_parameters = found
            .iter()
            .filter(|r| name_of(r.to).contains("Order.total"))
            .count();
        assert_eq!(reads_of_parameters, 0);
    }

    /// Resolves `usages.cs` once and lists the `(line, confidence)` of every usage per target.
    fn usages() -> BTreeMap<String, Vec<(u32, Confidence)>> {
        let (facts, found) = resolved("usages.cs");
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

    fn lines_of(found: &BTreeMap<String, Vec<(u32, Confidence)>>, name: &str) -> Vec<u32> {
        found
            .get(&format!("Demo.{name}"))
            .map(|all| all.iter().map(|(line, _)| *line).collect())
            .unwrap_or_default()
    }

    #[test]
    fn values_in_arguments_returns_cases_attributes_and_initializers_are_usages() {
        let found = usages();
        let lines = |name: &str| lines_of(&found, name);
        assert_eq!(lines("Names.Arg"), [27]);
        assert_eq!(lines("Names.Ret"), [28]);
        assert_eq!(lines("Names.Case"), [33]);
        assert_eq!(lines("Names.Attr"), [37]);
        assert_eq!(lines("Names.Lambda"), [41]);
        assert_eq!(lines("Item.Count"), [39]);
        assert_eq!(lines("IOne.Size"), [40]);
    }

    #[test]
    fn member_accesses_in_the_branches_of_a_conditional_are_usages() {
        let found = usages();
        let lines = |name: &str| lines_of(&found, name);
        assert_eq!(lines("Keys.First"), [66]);
        assert_eq!(lines("Keys.Second"), [66]);
    }

    #[test]
    fn a_receiver_of_unknown_type_keeps_every_unrelated_type() {
        let found = usages();
        let lines = |name: &str| lines_of(&found, name);
        // `var x = svc.Make()` and `var y = Svc.Load<ITwo>()` have no type the resolver knows:
        // `Close` could be IOne's or ITwo's: both are kept, as unchecked. `var z = (IOne)...` is
        // typed by the cast.
        assert_eq!(lines("IOne.Close"), [50, 52, 54]);
        assert_eq!(lines("ITwo.Close"), [50, 52]);
    }

    /// The `(line, confidence)` of every usage in `receivers.cs` of the member `name`.
    fn receiver_usages(name: &str) -> Vec<(u32, Confidence)> {
        let (facts, found) = resolved("receivers.cs");
        let target = format!("Demo.{name}");
        found
            .iter()
            .filter(|r| {
                facts
                    .symbols
                    .iter()
                    .any(|s| s.id == r.to && s.qualified_name.to_string() == target)
            })
            .map(|r| (r.span.start().line, r.confidence))
            .collect()
    }

    fn lines(usages: &[(u32, Confidence)]) -> Vec<u32> {
        let mut all: Vec<u32> = usages.iter().map(|(line, _)| *line).collect();
        all.sort_unstable();
        all.dedup();
        all
    }

    #[test]
    fn receivers_in_every_written_form_name_their_type() {
        // `r!`, `(IReader)o`, `(o as IReader)!`, `this._reader`, `await r?.`, `is IReader p`,
        // `case IReader s`, `out IReader got` and a primary constructor parameter all tell the
        // type: the call is never also a usage of the unrelated `IWriter.Open`.
        let writer = lines(&receiver_usages("IWriter.Open"));
        // Only the call on a receiver of unknown type (0-based line 31) keeps it.
        assert_eq!(writer, [31]);
        let reader = lines(&receiver_usages("IReader.Open"));
        assert_eq!(reader, [22, 23, 24, 25, 26, 27, 28, 29, 30, 31]);
    }

    #[test]
    fn names_that_queries_and_deconstruction_declare_hide_members_and_type_receivers() {
        let applied = receiver_usages("Editor.Apply");
        // Lines (0-based) 30 (primary constructor parameter `tool`), 33 and 34 (deconstruction), 35
        // (`let` variable) and 36 (the variable of `join ... into`) were lost before: the receiver name
        // was no symbol and no declaration, so it counted as a type outside the project.
        assert_eq!(lines(&applied), [30, 33, 34, 35, 36]);
    }

    #[test]
    fn an_object_initializer_names_members_of_its_type() {
        let event = receiver_usages("RowEvent.Index");
        assert!(
            event
                .iter()
                .any(|(line, c)| *line == 38 && *c == Confidence::Certain)
        );
        assert_eq!(lines(&receiver_usages("Row.Index")), Vec::<u32>::new());
    }

    #[test]
    fn a_base_call_reaches_the_base_member_only() {
        let base = receiver_usages("Base.Key");
        assert_ne!(base, Vec::new());
        assert!(
            base.iter()
                .all(|(_, confidence)| *confidence == Confidence::Certain)
        );
        // `base.Key` in the accessors of `Derived.Key` is no usage of `Derived.Key` itself.
        assert_eq!(lines(&receiver_usages("Derived.Key")), Vec::<u32>::new());
    }
}
