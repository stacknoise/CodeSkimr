//! Tests of the coupling metrics of namespaces and modules (ADR 0037).

use codeskimr_domain::{
    Confidence, FileId, Metric, MetricKind, ModelError, Position, QualifiedName, Reference,
    ReferenceId, ReferenceKind, Span, Symbol, SymbolId, SymbolKind, SymbolName, coupling,
};

fn symbol(id: u64, kind: SymbolKind, name: &str) -> Result<Symbol, ModelError> {
    let at = Position { line: 0, column: 0 };
    Ok(Symbol {
        span: Span::new(at, at)?,
        qualified_name: QualifiedName::new(name)?,
        name: SymbolName::new(name)?,
        id: SymbolId::new(id),
        kind,
        file: FileId::new(id),
        doc: None,
        signature: None,
        is_static: false,
        visibility: None,
        parent: None,
    })
}

fn reference(id: u64, from: u64, to: u64, kind: ReferenceKind) -> Result<Reference, ModelError> {
    let at = Position { line: 0, column: 0 };
    Ok(Reference {
        id: ReferenceId::new(id),
        from: SymbolId::new(from),
        to: SymbolId::new(to),
        file: FileId::new(from),
        span: Span::new(at, at)?,
        kind,
        confidence: Confidence::Certain,
        verified: false,
    })
}

fn value(metrics: &[Metric], id: u64, kind: MetricKind) -> Option<f64> {
    let wanted = SymbolId::new(id);
    let found = metrics
        .iter()
        .find(|m| (m.symbol, m.kind) == (wanted, kind));
    found.map(|m| m.value.get())
}

/// `App.Views` (1) with class 3 and method 4, `App.Domain` (2) with class 5, `App.Data` (6) with
/// class 7 and an unrelated class 8 outside every namespace.
fn code() -> Result<Vec<Symbol>, ModelError> {
    Ok(vec![
        symbol(1, SymbolKind::Namespace, "App.Views")?,
        symbol(2, SymbolKind::Namespace, "App.Domain")?,
        symbol(3, SymbolKind::Class, "App.Views.Board")?,
        symbol(4, SymbolKind::Method, "App.Views.Board.Draw")?,
        symbol(5, SymbolKind::Class, "App.Domain.Piece")?,
        symbol(6, SymbolKind::Namespace, "App.Data")?,
        symbol(7, SymbolKind::Class, "App.Data.Store")?,
        symbol(8, SymbolKind::Class, "Loose")?,
    ])
}

#[test]
fn other_namespaces_are_counted_once_however_many_references_there_are() -> Result<(), ModelError> {
    let references = [
        reference(1, 4, 5, ReferenceKind::Call)?,
        reference(2, 4, 5, ReferenceKind::Read)?,
        reference(3, 3, 7, ReferenceKind::TypeUse)?,
        reference(4, 7, 5, ReferenceKind::Call)?,
    ];
    let metrics = coupling(&code()?, &references)?;
    let ce = |id| value(&metrics, id, MetricKind::EfferentCoupling);
    let ca = |id| value(&metrics, id, MetricKind::AfferentCoupling);
    assert_eq!([ce(1), ce(2), ce(6)], [Some(2.0), Some(0.0), Some(1.0)]);
    assert_eq!([ca(1), ca(2), ca(6)], [Some(0.0), Some(2.0), Some(1.0)]);
    Ok(())
}

#[test]
fn instability_is_ce_over_ca_plus_ce_and_missing_without_any_dependency() -> Result<(), ModelError>
{
    let references = [
        reference(1, 4, 5, ReferenceKind::Call)?,
        reference(2, 3, 7, ReferenceKind::TypeUse)?,
        reference(3, 7, 5, ReferenceKind::Call)?,
    ];
    let metrics = coupling(&code()?, &references)?;
    let instability = |id| value(&metrics, id, MetricKind::Instability);
    assert_eq!(instability(1), Some(1.0));
    assert_eq!(instability(2), Some(0.0));
    assert_eq!(instability(6), Some(0.5));
    let isolated = coupling(&code()?, &[])?;
    assert_eq!(value(&isolated, 1, MetricKind::EfferentCoupling), Some(0.0));
    assert_eq!(value(&isolated, 1, MetricKind::Instability), None);
    Ok(())
}

#[test]
fn references_inside_a_namespace_outside_every_namespace_and_from_tests_do_not_count()
-> Result<(), ModelError> {
    let references = [
        reference(1, 3, 4, ReferenceKind::Call)?,
        reference(2, 8, 5, ReferenceKind::Call)?,
        reference(3, 4, 8, ReferenceKind::Call)?,
        reference(4, 4, 5, ReferenceKind::Test)?,
    ];
    let metrics = coupling(&code()?, &references)?;
    for id in [1, 2, 6] {
        assert_eq!(value(&metrics, id, MetricKind::EfferentCoupling), Some(0.0));
        assert_eq!(value(&metrics, id, MetricKind::AfferentCoupling), Some(0.0));
    }
    assert_eq!(value(&metrics, 8, MetricKind::EfferentCoupling), None);
    Ok(())
}

#[test]
fn a_namespace_declared_in_several_files_is_one_unit_and_each_declaration_gets_the_value()
-> Result<(), ModelError> {
    let mut symbols = code()?;
    symbols.push(symbol(9, SymbolKind::Namespace, "App.Views")?);
    symbols.push(symbol(10, SymbolKind::Class, "App.Views.Menu")?);
    let references = [
        reference(1, 4, 5, ReferenceKind::Call)?,
        reference(2, 10, 7, ReferenceKind::Call)?,
    ];
    let metrics = coupling(&symbols, &references)?;
    assert_eq!(value(&metrics, 1, MetricKind::EfferentCoupling), Some(2.0));
    assert_eq!(value(&metrics, 9, MetricKind::EfferentCoupling), Some(2.0));
    Ok(())
}

#[test]
fn a_module_owns_itself_and_nested_modules_are_separate_units() -> Result<(), ModelError> {
    let symbols = [
        symbol(1, SymbolKind::Module, "crate::store")?,
        symbol(2, SymbolKind::Module, "crate::store::cache")?,
        symbol(3, SymbolKind::Function, "crate::store::cache::get")?,
        symbol(4, SymbolKind::Function, "crate::store::open")?,
    ];
    let references = [
        reference(1, 4, 3, ReferenceKind::Call)?,
        reference(2, 1, 2, ReferenceKind::Import)?,
    ];
    let metrics = coupling(&symbols, &references)?;
    assert_eq!(value(&metrics, 1, MetricKind::EfferentCoupling), Some(1.0));
    assert_eq!(value(&metrics, 2, MetricKind::AfferentCoupling), Some(1.0));
    assert_eq!(value(&metrics, 2, MetricKind::EfferentCoupling), Some(0.0));
    Ok(())
}
