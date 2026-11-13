//! Tests of the level-1 reference resolver (blueprint 9.2) on a small symbol table.

#[cfg(test)]
mod tests {
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
            previous: None,
        }
    }

    fn typed(mut pending: PendingReference, type_name: &str) -> PendingReference {
        pending.receiver = Some(Receiver::Typed(
            QualifiedName::new(type_name).expect("type"),
        ));
        pending
    }

    fn resolve(table: &Table, pending: &[PendingReference]) -> Vec<Reference> {
        ScopedResolver
            .resolve(&table.symbols, pending, &IdSequence::starting_at(1000))
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
    fn a_reference_that_is_resolved_again_keeps_its_confidence() {
        let shop = shop();
        let mut again = occurrence(shop.save, 2, ReferenceKind::TypeUse, "Shop.Printer");
        again.previous = Some(Confidence::Probable);
        let found = resolve(&shop.table, &[again]);
        assert_eq!(targets(&found), [shop.printer]);
        assert_eq!(found[0].confidence, Confidence::Probable);
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
    fn a_receiver_of_unknown_type_is_probable_for_one_type_and_dropped_for_unrelated_types() {
        let mut table = Table::default();
        let one = table.add(SymbolKind::Interface, "Demo.IOne", None, 1);
        let one_close = table.add(SymbolKind::Method, "Demo.IOne.Close", Some(one), 1);
        let caller = table.add(SymbolKind::Method, "Demo.Client.Go", None, 3);
        let call = unknown(occurrence(caller, 3, ReferenceKind::Call, "Close"));
        let found = resolve(&table, std::slice::from_ref(&call));
        assert_eq!(targets(&found), [one_close]);
        assert_eq!(found[0].confidence, Confidence::Probable);
        let two = table.add(SymbolKind::Interface, "Demo.ITwo", None, 2);
        table.add(SymbolKind::Method, "Demo.ITwo.Close", Some(two), 2);
        assert!(resolve(&table, &[call]).is_empty());
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
        assert_eq!(calls, [shop.printer_print, fancy_print]);
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
    fn a_receiver_name_of_unknown_type_does_not_pick_between_unrelated_types() {
        let mut shop = shop();
        shop.table
            .add(SymbolKind::Field, "Shop.Order.sink", Some(shop.order), 2);
        let call = named(
            occurrence(shop.save, 2, ReferenceKind::Call, "Print"),
            "sink",
        );
        // `Print` exists in two unrelated types, so no one of them is reported.
        assert!(resolve(&shop.table, &[call]).is_empty());
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
    fn base_lists_resolved_again_still_tell_the_hierarchy() {
        let shop = shop();
        let mut seed = occurrence(shop.printer, 1, ReferenceKind::Implement, "Shop.IPrinter");
        seed.previous = Some(Confidence::Certain);
        let call = typed(
            occurrence(shop.save, 2, ReferenceKind::Call, "Print"),
            "IPrinter",
        );
        let found = resolve(&shop.table, &[seed, call]);
        let calls = of_kind(&found, ReferenceKind::Call);
        assert_eq!(calls, [shop.interface_print, shop.printer_print]);
    }
}
