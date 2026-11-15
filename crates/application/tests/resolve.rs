//! Tests of the level-1 reference resolver (blueprint 9.2) on a small symbol table.

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use codeskimr_application::ids::IdSequence;
    use codeskimr_application::ports::{PendingReference, Receiver, ReferenceResolver};
    use codeskimr_application::resolve::ScopedResolver;
    use codeskimr_domain::{
        Confidence, FileId, Position, QualifiedName, Reference, ReferenceKind, Span, Symbol,
        SymbolId, SymbolKind, SymbolName,
    };

    /// A symbol table that is built symbol by symbol; ids are handed out in order.
    #[derive(Default)]
    struct Table {
        symbols: Vec<Symbol>,
    }

    impl Table {
        /// A namespace with one class `Tool` in it.
        fn add_tool(&mut self, namespace: &str, file: u64) -> (SymbolId, SymbolId) {
            let outer = self.add(SymbolKind::Namespace, namespace, None, file);
            let class = format!("{namespace}.Tool");
            (
                outer,
                self.add(SymbolKind::Class, &class, Some(outer), file),
            )
        }

        /// A class `Shop.Fancy` with a method `Print`.
        fn add_fancy(&mut self) -> (SymbolId, SymbolId) {
            let class = self.add(SymbolKind::Class, "Shop.Fancy", None, 5);
            (
                class,
                self.add(SymbolKind::Method, "Shop.Fancy.Print", Some(class), 5),
            )
        }

        fn add(
            &mut self,
            kind: SymbolKind,
            qualified: &str,
            parent: Option<SymbolId>,
            file: u64,
        ) -> SymbolId {
            let id = SymbolId::new(u64::try_from(self.symbols.len()).expect("small") + 1);
            let name = qualified.rsplit('.').next().expect("name");
            let position = Position { line: 0, column: 0 };
            self.symbols.push(Symbol {
                id,
                file: FileId::new(file),
                kind,
                name: SymbolName::new(name).expect("name"),
                qualified_name: QualifiedName::new(qualified).expect("qualified"),
                parent,
                visibility: None,
                is_static: false,
                span: Span::new(position, position).expect("span"),
                signature: None,
                doc: None,
            });
            id
        }
    }

    fn occurrence(
        from: SymbolId,
        file: u64,
        kind: ReferenceKind,
        target: &str,
    ) -> PendingReference {
        let position = Position { line: 1, column: 1 };
        PendingReference {
            from,
            file: FileId::new(file),
            span: Span::new(position, position).expect("span"),
            kind,
            target: QualifiedName::new(target).expect("target"),
            receiver: None,
        }
    }

    fn typed(mut pending: PendingReference, type_name: &str) -> PendingReference {
        pending.receiver = Some(Receiver::Typed(
            QualifiedName::new(type_name).expect("type"),
        ));
        pending
    }

    fn resolve(table: &Table, pending: &[PendingReference]) -> Vec<Reference> {
        resolve_with_tests(table, pending, &HashSet::new())
    }

    fn resolve_with_tests(
        table: &Table,
        pending: &[PendingReference],
        test_files: &HashSet<FileId>,
    ) -> Vec<Reference> {
        ScopedResolver
            .resolve(
                &table.symbols,
                pending,
                test_files,
                &IdSequence::starting_at(1000),
            )
            .expect("resolves")
    }

    fn of_kind(references: &[Reference], kind: ReferenceKind) -> Vec<SymbolId> {
        references
            .iter()
            .filter(|reference| reference.kind == kind)
            .map(|reference| reference.to)
            .collect()
    }

    fn targets(references: &[Reference]) -> Vec<SymbolId> {
        references.iter().map(|reference| reference.to).collect()
    }

    /// `Shop` with `IPrinter`/`Printer` (both with `Print`) and `Order` with `Save`.
    struct Shop {
        table: Table,
        interface_print: SymbolId,
        printer: SymbolId,
        printer_print: SymbolId,
        order: SymbolId,
        save: SymbolId,
    }

    fn shop() -> Shop {
        let mut table = Table::default();
        let shop = table.add(SymbolKind::Namespace, "Shop", None, 1);
        let interface = table.add(SymbolKind::Interface, "Shop.IPrinter", Some(shop), 1);
        let interface_print = table.add(
            SymbolKind::Method,
            "Shop.IPrinter.Print",
            Some(interface),
            1,
        );
        let printer = table.add(SymbolKind::Class, "Shop.Printer", Some(shop), 1);
        let printer_print = table.add(SymbolKind::Method, "Shop.Printer.Print", Some(printer), 1);
        let order = table.add(SymbolKind::Class, "Shop.Order", Some(shop), 2);
        let save = table.add(SymbolKind::Method, "Shop.Order.Save", Some(order), 2);
        Shop {
            table,
            interface_print,
            printer,
            printer_print,
            order,
            save,
        }
    }

    #[test]
    fn a_type_of_the_same_namespace_is_certain() {
        let shop = shop();
        let found = resolve(
            &shop.table,
            &[occurrence(shop.save, 2, ReferenceKind::TypeUse, "Printer")],
        );
        assert_eq!(targets(&found), [shop.printer]);
        assert_eq!(found[0].confidence, Confidence::Certain);
    }

    #[test]
    fn a_member_of_the_enclosing_type_is_certain() {
        let shop = shop();
        let found = resolve(
            &shop.table,
            &[occurrence(shop.save, 2, ReferenceKind::Call, "Save")],
        );
        assert_eq!(targets(&found), [shop.save]);
        assert_eq!(found[0].confidence, Confidence::Certain);
    }

    #[test]
    fn uses_in_a_test_file_are_test_references_and_imports_stay_imports() {
        let mut shop = shop();
        let (elsewhere, tool) = shop.table.add_tool("Elsewhere", 3);
        let pending = [
            occurrence(shop.save, 2, ReferenceKind::Import, "Elsewhere"),
            occurrence(shop.save, 2, ReferenceKind::TypeUse, "Tool"),
            occurrence(shop.save, 2, ReferenceKind::Call, "Tool"),
            occurrence(shop.save, 7, ReferenceKind::TypeUse, "Tool"),
        ];
        let tests = HashSet::from([FileId::new(2)]);
        let found = resolve_with_tests(&shop.table, &pending, &tests);
        let kinds: Vec<_> = found.iter().map(|r| (r.to, r.file, r.kind)).collect();
        assert!(kinds.contains(&(elsewhere, FileId::new(2), ReferenceKind::Import)));
        assert!(kinds.contains(&(tool, FileId::new(2), ReferenceKind::Test)));
        // The same name in a file that is no test file keeps its kind.
        assert!(kinds.contains(&(tool, FileId::new(7), ReferenceKind::TypeUse)));
        assert!(!kinds.contains(&(tool, FileId::new(2), ReferenceKind::TypeUse)));
    }

    #[test]
    fn a_name_that_only_matches_globally_is_probable() {
        let mut shop = shop();
        let (_, tool) = shop.table.add_tool("Elsewhere", 3);
        let found = resolve(
            &shop.table,
            &[occurrence(shop.save, 2, ReferenceKind::TypeUse, "Tool")],
        );
        assert_eq!(targets(&found), [tool]);
        assert_eq!(found[0].confidence, Confidence::Probable);
    }

    #[test]
    fn an_import_makes_the_name_certain() {
        let mut shop = shop();
        let (elsewhere, tool) = shop.table.add_tool("Elsewhere", 3);
        let pending = [
            occurrence(shop.save, 2, ReferenceKind::Import, "Elsewhere"),
            occurrence(shop.save, 2, ReferenceKind::TypeUse, "Tool"),
        ];
        let found = resolve(&shop.table, &pending);
        assert_eq!(targets(&found), [elsewhere, tool]);
        assert!(found.iter().all(|r| r.confidence == Confidence::Certain));
    }

    #[test]
    fn several_candidates_are_all_probable() {
        let mut shop = shop();
        let (_, first) = shop.table.add_tool("One", 3);
        let (_, second) = shop.table.add_tool("Two", 4);
        let found = resolve(
            &shop.table,
            &[occurrence(shop.save, 2, ReferenceKind::TypeUse, "Tool")],
        );
        assert_eq!(targets(&found), [first, second]);
        assert!(found.iter().all(|r| r.confidence == Confidence::Probable));
    }

    #[test]
    fn a_base_type_that_is_an_interface_is_implemented() {
        let shop = shop();
        let pending = [occurrence(
            shop.printer,
            1,
            ReferenceKind::Inherit,
            "IPrinter",
        )];
        let found = resolve(&shop.table, &pending);
        assert_eq!(found[0].kind, ReferenceKind::Implement);
    }

    #[test]
    fn a_call_through_an_interface_reaches_every_implementation() {
        let shop = shop();
        let pending = [
            occurrence(shop.printer, 1, ReferenceKind::Inherit, "IPrinter"),
            typed(
                occurrence(shop.save, 2, ReferenceKind::Call, "Print"),
                "IPrinter",
            ),
        ];
        let found = resolve(&shop.table, &pending);
        let calls: Vec<&Reference> = found
            .iter()
            .filter(|r| r.kind == ReferenceKind::Call)
            .collect();
        assert_eq!(
            calls.iter().map(|r| r.to).collect::<Vec<_>>(),
            [shop.interface_print, shop.printer_print]
        );
        assert!(calls.iter().all(|r| r.confidence == Confidence::Probable));
    }

    #[test]
    fn a_call_on_a_class_receiver_is_certain() {
        let shop = shop();
        let pending = [typed(
            occurrence(shop.save, 2, ReferenceKind::Call, "Print"),
            "Printer",
        )];
        let found = resolve(&shop.table, &pending);
        assert_eq!(targets(&found), [shop.printer_print]);
        assert_eq!(found[0].confidence, Confidence::Certain);
    }

    #[test]
    fn a_member_of_an_outside_type_is_not_taken_for_a_project_symbol() {
        let shop = shop();
        let pending = [typed(
            occurrence(shop.save, 2, ReferenceKind::Call, "Print"),
            "TextWriter",
        )];
        assert_eq!(
            targets(&resolve(&shop.table, &pending)),
            Vec::<SymbolId>::new()
        );
    }

    #[test]
    fn an_inherited_member_is_found_through_the_base_type() {
        let mut shop = shop();
        let helper = shop.table.add(
            SymbolKind::Method,
            "Shop.Printer.Flush",
            Some(shop.printer),
            1,
        );
        let pending = [
            occurrence(shop.order, 2, ReferenceKind::Inherit, "Printer"),
            occurrence(shop.save, 2, ReferenceKind::Call, "Flush"),
        ];
        let mut with_own = pending.to_vec();
        with_own[1].receiver = Some(Receiver::Own);
        let found = resolve(&shop.table, &with_own);
        let call: Vec<&Reference> = found
            .iter()
            .filter(|r| r.kind == ReferenceKind::Call)
            .collect();
        assert_eq!(call.len(), 1);
        assert_eq!(
            (call[0].to, call[0].confidence),
            (helper, Confidence::Certain)
        );
    }

    #[test]
    fn an_override_points_to_the_base_member_and_not_to_itself() {
        let mut shop = shop();
        let (derived, fancy_print) = shop.table.add_fancy();
        let pending = [
            occurrence(derived, 5, ReferenceKind::Inherit, "Printer"),
            occurrence(fancy_print, 5, ReferenceKind::Override, "Print"),
        ];
        let found = resolve(&shop.table, &pending);
        let overrides: Vec<&Reference> = found
            .iter()
            .filter(|r| r.kind == ReferenceKind::Override)
            .collect();
        assert_eq!(
            overrides.iter().map(|r| r.to).collect::<Vec<_>>(),
            [shop.printer_print]
        );
    }

    #[test]
    fn a_name_in_a_value_position_that_is_a_type_is_a_type_use() {
        let shop = shop();
        let found = resolve(
            &shop.table,
            &[occurrence(shop.save, 2, ReferenceKind::Read, "Printer")],
        );
        assert_eq!(
            (found[0].to, found[0].kind),
            (shop.printer, ReferenceKind::TypeUse)
        );
    }

    #[test]
    fn unknown_names_and_the_same_occurrence_twice_give_one_or_no_reference() {
        let shop = shop();
        let twice = occurrence(shop.save, 2, ReferenceKind::TypeUse, "Printer");
        let pending = [
            twice.clone(),
            twice,
            occurrence(shop.save, 2, ReferenceKind::TypeUse, "Nothing"),
        ];
        assert_eq!(resolve(&shop.table, &pending).len(), 1);
    }

    fn unknown(mut pending: PendingReference) -> PendingReference {
        pending.receiver = Some(Receiver::Unknown);
        pending
    }

    #[test]
    fn a_receiver_of_unknown_type_is_probable_for_one_type_and_keeps_every_unrelated_type() {
        let mut table = Table::default();
        let one = table.add(SymbolKind::Interface, "Demo.IOne", None, 1);
        let one_close = table.add(SymbolKind::Method, "Demo.IOne.Close", Some(one), 1);
        let caller = table.add(SymbolKind::Method, "Demo.Client.Go", None, 3);
        let call = unknown(occurrence(caller, 3, ReferenceKind::Call, "Close"));
        let found = resolve(&table, std::slice::from_ref(&call));
        assert_eq!(targets(&found), [one_close]);
        assert_eq!(found[0].confidence, Confidence::Probable);
        let two = table.add(SymbolKind::Interface, "Demo.ITwo", None, 2);
        let two_close = table.add(SymbolKind::Method, "Demo.ITwo.Close", Some(two), 2);
        let both = resolve(&table, &[call]);
        assert_eq!(targets(&both), [one_close, two_close]);
        assert!(
            both.iter()
                .all(|r| r.confidence == Confidence::Probable && !r.verified)
        );
    }

    #[test]
    fn a_receiver_of_unknown_type_keeps_sibling_overrides_of_one_base() {
        let mut table = Table::default();
        table.add(SymbolKind::Class, "Demo.Animal", None, 1);
        let dog = table.add(SymbolKind::Class, "Demo.Dog", None, 2);
        let cat = table.add(SymbolKind::Class, "Demo.Cat", None, 3);
        let dog_speak = table.add(SymbolKind::Method, "Demo.Dog.Speak", Some(dog), 2);
        let cat_speak = table.add(SymbolKind::Method, "Demo.Cat.Speak", Some(cat), 3);
        let caller = table.add(SymbolKind::Method, "Demo.Zoo.Go", None, 4);
        let bases = [
            occurrence(dog, 2, ReferenceKind::Inherit, "Animal"),
            occurrence(cat, 3, ReferenceKind::Inherit, "Animal"),
        ];
        let call = unknown(occurrence(caller, 4, ReferenceKind::Call, "Speak"));
        let found = resolve(&table, &[bases[0].clone(), bases[1].clone(), call]);
        let calls = of_kind(&found, ReferenceKind::Call);
        assert_eq!(calls.len(), 2);
        assert!(calls.contains(&dog_speak) && calls.contains(&cat_speak));
    }

    #[test]
    fn a_receiver_of_unknown_type_ignores_members_of_the_enclosing_type() {
        let mut table = Table::default();
        let client = table.add(SymbolKind::Class, "Demo.Client", None, 3);
        table.add(SymbolKind::Method, "Demo.Client.Close", Some(client), 3);
        let go = table.add(SymbolKind::Method, "Demo.Client.Go", Some(client), 3);
        let call = unknown(occurrence(go, 3, ReferenceKind::Call, "Close"));
        let found = resolve(&table, &[call]);
        assert_eq!(found[0].confidence, Confidence::Probable);
    }

    fn named(mut pending: PendingReference, name: &str) -> PendingReference {
        pending.receiver = Some(Receiver::Named(QualifiedName::new(name).expect("name")));
        pending
    }

    #[test]
    fn a_type_of_an_enclosing_namespace_is_certain() {
        let mut shop = shop();
        let inner = shop
            .table
            .add(SymbolKind::Namespace, "Shop.Orders", None, 6);
        let cart = shop
            .table
            .add(SymbolKind::Class, "Shop.Orders.Cart", Some(inner), 6);
        let found = resolve(
            &shop.table,
            &[occurrence(cart, 6, ReferenceKind::TypeUse, "Printer")],
        );
        assert_eq!(targets(&found), [shop.printer]);
        assert_eq!(found[0].confidence, Confidence::Certain);
    }

    #[test]
    fn a_call_on_the_base_reaches_the_base_member_only() {
        let mut shop = shop();
        let (derived, fancy_print) = shop.table.add_fancy();
        let mut call = occurrence(fancy_print, 5, ReferenceKind::Call, "Print");
        call.receiver = Some(Receiver::Base);
        let pending = [
            occurrence(derived, 5, ReferenceKind::Inherit, "Printer"),
            call,
        ];
        let found = resolve(&shop.table, &pending);
        let calls = of_kind(&found, ReferenceKind::Call);
        assert_eq!(calls, [shop.printer_print]);
    }

    #[test]
    fn a_receiver_name_that_no_project_symbol_bears_is_outside() {
        let shop = shop();
        let call = named(
            occurrence(shop.save, 2, ReferenceKind::Call, "Print"),
            "Console",
        );
        assert_eq!(
            targets(&resolve(&shop.table, &[call])),
            Vec::<SymbolId>::new()
        );
    }

    #[test]
    fn a_receiver_name_of_unknown_type_keeps_every_unrelated_type_as_probable() {
        let mut shop = shop();
        shop.table
            .add(SymbolKind::Field, "Shop.Order.sink", Some(shop.order), 2);
        let call = named(
            occurrence(shop.save, 2, ReferenceKind::Call, "Print"),
            "sink",
        );
        // `Print` exists in two unrelated types: both are kept, probable and unchecked.
        let found = resolve(&shop.table, &[call]);
        assert_eq!(found.len(), 2);
        assert!(
            found
                .iter()
                .all(|r| r.confidence == Confidence::Probable && !r.verified)
        );
    }

    #[test]
    fn a_member_call_on_a_named_receiver_is_not_certain_for_a_local_of_the_same_name() {
        let mut table = Table::default();
        let module = table.add(SymbolKind::Module, "Demo.model", None, 1);
        table.add(SymbolKind::Constant, "Demo.model.keys", Some(module), 1);
        let walk = table.add(SymbolKind::Function, "Demo.model.walk", Some(module), 3);
        table.add(SymbolKind::Function, "Demo.model.walk.room", Some(walk), 4);
        // `keys.room()` is a member of `keys`, not the local function `room` of the same name.
        let call = named(occurrence(walk, 5, ReferenceKind::Call, "room"), "keys");
        let found = resolve(&table, &[call]);
        assert_eq!(found.len(), 1);
        assert!(found[0].confidence == Confidence::Probable && !found[0].verified);
    }

    #[test]
    fn a_call_through_a_module_qualifier_resolves_by_scope_as_before() {
        let mut table = Table::default();
        let module = table.add(SymbolKind::Module, "Demo.util", None, 1);
        let helper = table.add(SymbolKind::Function, "Demo.util.helper", Some(module), 1);
        let caller = table.add(SymbolKind::Function, "Demo.util.run", Some(module), 3);
        // `util.helper()` inside the module: `util` is a module, not a variable.
        let call = named(occurrence(caller, 4, ReferenceKind::Call, "helper"), "util");
        let found = resolve(&table, &[call]);
        assert_eq!(targets(&found), [helper]);
        assert_eq!(found[0].confidence, Confidence::Certain);
    }

    /// A class `Demo.Box` with a field `count` and a method `run`, and a free read of `count`.
    fn box_with_count() -> (Table, SymbolId, PendingReference) {
        let mut table = Table::default();
        let class = table.add(SymbolKind::Class, "Demo.Box", None, 1);
        table.add(SymbolKind::Field, "Demo.Box.count", Some(class), 1);
        let method = table.add(SymbolKind::Method, "Demo.Box.run", Some(class), 1);
        let mut read = occurrence(method, 1, ReferenceKind::Read, "count");
        read.receiver = Some(Receiver::Free);
        (table, method, read)
    }

    #[test]
    fn a_free_name_in_a_function_never_means_a_member_of_the_enclosing_type() {
        let (mut table, _, read) = box_with_count();
        let module_level = table.add(SymbolKind::Constant, "Demo.count", None, 1);
        let found = resolve(&table, &[read]);
        // The field is no candidate; the constant outside the class is, by the last resort only.
        assert_eq!(targets(&found), [module_level]);
        assert_eq!(found[0].confidence, Confidence::Probable);
    }

    #[test]
    fn a_free_name_with_only_members_of_that_name_gives_no_hit() {
        let (table, method, read) = box_with_count();
        assert_eq!(targets(&resolve(&table, &[read])), []);
        // The same name without that knowledge still finds the field of the own type.
        let plain = occurrence(method, 1, ReferenceKind::Read, "count");
        assert_eq!(resolve(&table, &[plain]).len(), 1);
    }

    #[test]
    fn a_free_name_never_means_a_method_even_if_the_pack_nests_it_in_a_module() {
        let mut table = Table::default();
        let module = table.add(SymbolKind::Module, "demo", None, 1);
        // The `impl` of Rust has no symbol: the method sits in the module.
        table.add(SymbolKind::Method, "demo.from", Some(module), 1);
        let caller = table.add(SymbolKind::Function, "demo.run", Some(module), 1);
        let mut read = occurrence(caller, 1, ReferenceKind::Read, "from");
        read.receiver = Some(Receiver::Free);
        assert_eq!(targets(&resolve(&table, &[read])), []);
    }

    #[test]
    fn the_name_in_the_declaration_of_a_field_is_no_use_of_it() {
        let mut table = Table::default();
        let class = table.add(SymbolKind::Class, "Demo.Box", None, 1);
        let field = table.add(SymbolKind::Field, "Demo.Box.count", Some(class), 1);
        let other = table.add(SymbolKind::Field, "Demo.Other.count", None, 2);
        let mut write = occurrence(class, 1, ReferenceKind::Write, "count");
        // The occurrence starts where the field does: `count: int` in a class body.
        let at = Position { line: 0, column: 0 };
        write.span = Span::new(at, at).expect("span");
        let found = resolve(&table, &[write]);
        assert_eq!(targets(&found), [], "{field:?} {other:?}");
    }

    #[test]
    fn a_call_of_an_interface_member_is_probable_even_without_implementer() {
        let shop = shop();
        let call = typed(
            occurrence(shop.save, 2, ReferenceKind::Call, "Print"),
            "IPrinter",
        );
        let found = resolve(&shop.table, &[call]);
        assert_eq!(targets(&found), [shop.interface_print]);
        assert_eq!(found[0].confidence, Confidence::Probable);
    }

    #[test]
    fn base_lists_of_stored_names_still_tell_the_hierarchy() {
        let shop = shop();
        let seed = occurrence(shop.printer, 1, ReferenceKind::Implement, "IPrinter");
        let call = typed(
            occurrence(shop.save, 2, ReferenceKind::Call, "Print"),
            "IPrinter",
        );
        let found = resolve(&shop.table, &[seed, call]);
        let calls = of_kind(&found, ReferenceKind::Call);
        assert_eq!(calls, [shop.interface_print, shop.printer_print]);
    }
}
