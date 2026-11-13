//! Tests of the rollup of callable metrics to types, namespaces and modules (ADR 0015).

use codeskimr_domain::{
    BodyCounts, FileId, HalsteadCounts, Loc, Metric, MetricKind, ModelError, Position,
    QualifiedName, Span, Symbol, SymbolCounts, SymbolId, SymbolKind, SymbolName, measure,
};

const EPSILON: f64 = 1e-9;

fn symbol(id: u64, kind: SymbolKind, parent: Option<u64>) -> Result<Symbol, ModelError> {
    let at = Position { line: 0, column: 0 };
    Ok(Symbol {
        id: SymbolId::new(id),
        file: FileId::new(1),
        kind,
        name: SymbolName::new("n")?,
        qualified_name: QualifiedName::new("n")?,
        parent: parent.map(SymbolId::new),
        visibility: None,
        is_static: false,
        span: Span::new(at, at)?,
        signature: None,
        doc: None,
    })
}

fn counts(id: u64, loc: u32, decisions: Option<u32>) -> SymbolCounts {
    SymbolCounts {
        symbol: SymbolId::new(id),
        loc: Loc(loc),
        counted: true,
        body: decisions.map(|decisions| BodyCounts {
            decisions,
            halstead: HalsteadCounts {
                distinct_operators: 2,
                distinct_operands: 2,
                total_operators: 4,
                total_operands: 4,
            },
        }),
    }
}

fn value(metrics: &[Metric], id: u64, kind: MetricKind) -> Option<f64> {
    metrics
        .iter()
        .find(|m| m.symbol == SymbolId::new(id) && m.kind == kind)
        .map(|m| m.value.get())
}

/// Module 1 holds class 2 with methods 3 and 4, an abstract method 5, a local function 6 inside
/// method 3, and an empty class 7.
fn file() -> Result<Vec<Symbol>, ModelError> {
    Ok(vec![
        symbol(1, SymbolKind::Module, None)?,
        symbol(2, SymbolKind::Class, Some(1))?,
        symbol(3, SymbolKind::Method, Some(2))?,
        symbol(4, SymbolKind::Method, Some(2))?,
        symbol(5, SymbolKind::Method, Some(2))?,
        symbol(6, SymbolKind::Function, Some(3))?,
        symbol(7, SymbolKind::Class, Some(1))?,
    ])
}

fn counted() -> Vec<SymbolCounts> {
    vec![
        counts(1, 30, None),
        counts(2, 20, None),
        counts(3, 6, Some(2)),
        counts(4, 10, Some(0)),
        counts(5, 1, None),
        counts(6, 4, Some(1)),
        counts(7, 2, None),
    ]
}

#[test]
fn callables_get_loc_cc_and_mi_and_bodyless_ones_nothing() -> Result<(), ModelError> {
    let metrics = measure(&file()?, &counted())?;
    assert_eq!(
        value(&metrics, 3, MetricKind::CyclomaticComplexity),
        Some(3.0)
    );
    assert_eq!(
        value(&metrics, 4, MetricKind::CyclomaticComplexity),
        Some(1.0)
    );
    assert!(value(&metrics, 3, MetricKind::MaintainabilityIndex).is_some());
    assert_eq!(value(&metrics, 5, MetricKind::Loc), None);
    assert_eq!(value(&metrics, 5, MetricKind::CyclomaticComplexity), None);
    assert_eq!(value(&metrics, 5, MetricKind::MaintainabilityIndex), None);
    Ok(())
}

#[test]
fn types_and_modules_sum_cc_take_the_maximum_and_weight_mi_by_loc() -> Result<(), ModelError> {
    let metrics = measure(&file()?, &counted())?;
    // 3 (method) + 1 (method) + 2 (local function, reaches the class through its parent chain)
    for id in [2, 1] {
        assert_eq!(
            value(&metrics, id, MetricKind::CyclomaticComplexity),
            Some(6.0)
        );
        assert_eq!(
            value(&metrics, id, MetricKind::CyclomaticComplexityMax),
            Some(3.0)
        );
    }
    let mi = |id| value(&metrics, id, MetricKind::MaintainabilityIndex).unwrap();
    let expected = (mi(3) * 6.0 + mi(4) * 10.0 + mi(6) * 4.0) / 20.0;
    assert!((mi(2) - expected).abs() < EPSILON);
    Ok(())
}

#[test]
fn a_type_without_callables_has_cc_zero_and_nothing_else() -> Result<(), ModelError> {
    let metrics = measure(&file()?, &counted())?;
    assert_eq!(value(&metrics, 7, MetricKind::Loc), Some(2.0));
    assert_eq!(
        value(&metrics, 7, MetricKind::CyclomaticComplexity),
        Some(0.0)
    );
    assert_eq!(
        value(&metrics, 7, MetricKind::CyclomaticComplexityMax),
        None
    );
    assert_eq!(value(&metrics, 7, MetricKind::MaintainabilityIndex), None);
    Ok(())
}

#[test]
fn counts_of_unknown_symbols_and_other_kinds_are_ignored() -> Result<(), ModelError> {
    let mut symbols = file()?;
    symbols.push(symbol(8, SymbolKind::Field, Some(2))?);
    let metrics = measure(&symbols, &[counts(8, 1, None), counts(99, 1, None)])?;
    assert_eq!(metrics, Vec::<codeskimr_domain::Metric>::new());
    Ok(())
}

#[test]
fn without_counting_types_and_modules_only_get_loc() -> Result<(), ModelError> {
    let uncounted: Vec<SymbolCounts> = counted()
        .into_iter()
        .map(|c| SymbolCounts {
            counted: false,
            body: None,
            ..c
        })
        .collect();
    let metrics = measure(&file()?, &uncounted)?;
    assert_eq!(value(&metrics, 2, MetricKind::Loc), Some(20.0));
    assert_eq!(value(&metrics, 2, MetricKind::CyclomaticComplexity), None);
    assert_eq!(value(&metrics, 3, MetricKind::Loc), None);
    Ok(())
}
