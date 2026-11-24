//! The analysis of a C# file and the resolver together (blueprint 9.2).

#[cfg(test)]
mod common;

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, HashSet};
    use std::path::Path;
    use std::time::Duration;

    use super::common::analyze_file;
    use codeskimr_application::cancel::CancelToken;
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
        let found = ScopedResolver::default()
            .resolve(
                &facts.symbols,
                &facts.pending,
                &HashSet::new(),
                (&ids, &CancelToken::new()),
            )
            .expect("resolves")
            .references;
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
        receiver_usages_in("receivers.cs", name)
    }

    /// The `(line, confidence)` of every usage in `file` of the member `name`.
    fn receiver_usages_in(file: &str, name: &str) -> Vec<(u32, Confidence)> {
        let (facts, found) = resolved(file);
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
        let applied = receiver_usages("Sensor.Apply");
        // Lines (0-based) 30 (primary constructor parameter `tool`), 33 and 34 (deconstruction), 35
        // (`let` variable) and 36 (the variable of `join ... into`) were lost before: the receiver name
        // was no symbol and no declaration, so it counted as a type outside the project.
        assert_eq!(lines(&applied), [30, 33, 34, 35, 36]);
    }

    #[test]
    fn an_object_initializer_names_members_of_its_type() {
        let event = receiver_usages("PinEvent.Index");
        assert!(
            event
                .iter()
                .any(|(line, c)| *line == 38 && *c == Confidence::Certain)
        );
        assert_eq!(lines(&receiver_usages("Pin.Index")), Vec::<u32>::new());
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

    #[test]
    fn a_null_conditional_access_is_a_usage_through_the_type_without_the_question_mark() {
        // `a?.B`, `a?.B?.C`, `a?.B()`, `a?[0]`, `a!?.B` and `this.f?.B` (0-based lines 57 to 61).
        let leaf = receiver_usages("Leaf.Weight");
        assert_eq!(lines(&leaf), [57, 58, 61]);
        assert_eq!(lines(&receiver_usages("Branch.Tip")), [57, 58]);
        // The receiver of `?.Measure()` is a `Leaf?`: the call is never also one of `Twig`.
        assert_eq!(lines(&receiver_usages("Leaf.Measure")), [59]);
        assert_eq!(lines(&receiver_usages("Twig.Measure")), Vec::<u32>::new());
        // The receiver `branch?.Tip` of the chain has no known type: `Twig.Weight` stays, unchecked.
        let twig = receiver_usages("Twig.Weight");
        assert_eq!(lines(&twig), [58]);
        assert!(twig.iter().all(|(_, c)| *c != Confidence::Certain));
        // `_field?[0]` reads the field.
        let field = receiver_usages("NullConditional._field");
        assert!(field.contains(&(60, Confidence::Certain)));
    }

    #[test]
    fn a_property_that_only_starts_a_chain_is_a_usage() {
        // `Guard.Against.Null(count)` uses the property `Against` (0-based line 71), issue #183.
        let against = receiver_usages("Guard.Against");
        assert_eq!(lines(&against), [71]);
        assert_eq!(lines(&receiver_usages("Clause.Null")), [71]);
    }

    #[test]
    fn negated_and_chained_record_properties_are_usages() {
        // Positional record properties reached through chains (`h.Settings.Enabled`,
        // `a.Child.IsOpen`) and negated (`!x.Settings.Enabled`, `!c.IsActive`); lines are
        // 0-based.
        let through_chain = lines(&receiver_usages_in("records.cs", "Settings.Enabled"));
        assert_eq!(through_chain, [14, 17, 19]);
        let direct = lines(&receiver_usages_in("records.cs", "Entry.IsActive"));
        assert_eq!(direct, [15, 18, 20, 21]);
        let second_chain = lines(&receiver_usages_in("records.cs", "Inner.IsOpen"));
        assert_eq!(second_chain, [16, 18]);
    }

    #[test]
    fn calls_through_fields_and_interfaces_in_tests_are_usages() {
        // `_store.Load`, `store.Load(..).Returns(..)` and `_stub.Load(Arg.Any<string>())`.
        let load = lines(&receiver_usages_in("records.cs", "IStore.Load"));
        assert_eq!(load, [30, 36, 37]);
    }

    #[test]
    fn members_of_types_outside_the_project_are_no_usages() {
        // `File.Exists`, `Directory.Exists` and `List<string>.IndexOf` are not project symbols;
        // only the typed `disk.Exists` and the receiver of unknown type are kept.
        assert_eq!(
            lines(&receiver_usages_in("records.cs", "IDisk.Exists")),
            [49, 50]
        );
        assert_eq!(
            lines(&receiver_usages_in("records.cs", "Index.IndexOf")),
            [48]
        );
    }

    #[test]
    fn an_increment_is_one_write_and_no_extra_read() {
        // `c.Total++`, `++c.Step` and `c.Step--` (0-based line 57) stay writes only.
        let (facts, found) = resolved("records.cs");
        let at_bump: Vec<_> = found
            .iter()
            .filter(|r| r.span.start().line == 57)
            .filter(|r| {
                facts.symbols.iter().any(|s| {
                    s.id == r.to
                        && (s.qualified_name.to_string() == "Demo.Counter.Total"
                            || s.qualified_name.to_string() == "Demo.Counter.Step")
                })
            })
            .collect();
        assert_eq!(at_bump.len(), 3);
        assert!(at_bump.iter().all(|r| r.kind == ReferenceKind::Write));
    }

    #[test]
    fn typed_receivers_are_certain_and_chains_and_unknown_receivers_are_not() {
        let direct = receiver_usages_in("records.cs", "Entry.IsActive");
        assert!(direct.contains(&(15, Confidence::Certain)));
        let chain = receiver_usages_in("records.cs", "Settings.Enabled");
        assert!(chain.iter().all(|(_, c)| *c != Confidence::Certain));
        // `d.Exists(p)` on a receiver of unknown type (0-based line 50) is never certain.
        let unknown = receiver_usages_in("records.cs", "IDisk.Exists");
        assert!(unknown.contains(&(50, Confidence::Probable)));
        assert!(unknown.iter().all(|(_, c)| *c != Confidence::Certain));
    }

    #[test]
    fn calls_on_framework_types_are_no_usages_of_a_same_named_project_method() {
        // `Directory.Exists` (a property of that name sits in an unrelated type), `File.Exists`
        // (the same) and `List.IndexOf` on a field, a parameter and a local name
        // something outside the project: none reaches `Store.Exists` or `Store.IndexOf`.
        let exists = receiver_usages_in("framework.cs", "Store.Exists");
        assert_eq!(lines(&exists), [23, 24, 25, 36]);
        let index = receiver_usages_in("framework.cs", "Store.IndexOf");
        assert!(index.is_empty(), "{index:?}");
    }

    #[test]
    fn a_receiver_of_unknown_inferred_type_keeps_the_fallback_unchecked() {
        let interface = receiver_usages_in("framework.cs", "IStore.Exists");
        assert_eq!(lines(&interface), [23, 25, 36]);
        let own = receiver_usages_in("framework.cs", "Store.Exists");
        // `Own(Store s)` is typed; `var x = Make(); x.Exists(..)` keeps the name as unchecked.
        assert!(own.contains(&(24, Confidence::Certain)));
        assert!(own.contains(&(25, Confidence::Probable)));
    }

    #[test]
    fn a_value_of_the_enclosing_type_or_of_its_other_part_keeps_the_fallback() {
        // `Config` and `Settings` are declared in the second `partial` part, the call is in the
        // first: the name stands for a value, so the member name alone stays, as unchecked.
        let kept = receiver_usages_in("framework.cs", "Store.Exists");
        assert!(kept.contains(&(36, Confidence::Probable)), "{kept:?}");
        // `File` of `Strangers` is declared in the file with its type: the call reaches `Options`.
        assert!(!kept.iter().any(|(line, _)| *line == 48), "{kept:?}");
    }

    #[test]
    fn calls_and_members_behind_a_negation_or_an_await_are_usages() {
        // Issue #705: `if (!await _field.Method())` lists the call, like the forms around it.
        let line_of = |name: &str| {
            receiver_usages_in("negation.cs", name)
                .iter()
                .map(|(line, _)| *line)
                .collect::<Vec<_>>()
        };
        assert_eq!(line_of("IGate.Negated"), [23]);
        assert_eq!(line_of("IGate.Awaited"), [24]);
        assert_eq!(line_of("IGate.Plain"), [25]);
        assert_eq!(line_of("IGate.Grouped"), [26]);
        assert_eq!(line_of("IGate.Chained"), [27]);
        assert_eq!(line_of("IGate.Safe"), [28]);
        assert_eq!(line_of("IGate.Mixed"), [29]);
        assert_eq!(line_of("IGate.Primary"), [36]);
        assert_eq!(line_of("IFlag.Ready"), [30, 31]);
        assert_eq!(line_of("Printer.TryShowAsync"), [43, 44]);
        // Calls in a constructor lambda, a property, an event handler, a local function and a
        // static initializer, and a call of an own method without receiver in another `partial` part.
        let lines = |name| {
            receiver_usages_in("froms.cs", name)
                .iter()
                .map(|(l, _)| *l)
                .collect::<Vec<_>>()
        };
        assert_eq!(lines("ISave.Offer"), [7, 9, 10, 11, 12, 13]);
        assert_eq!(lines("Model.Helper"), [15]);
    }
}
