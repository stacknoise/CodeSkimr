//! The numbers behind the modules of a project (ADR 0108).

use codeskimr_application::module_report::ModuleReport;
use codeskimr_domain::{
    Layer, LayerName, ModelError, NamespaceEdge, NamespaceGraph, NamespaceNode, QualifiedName,
    RiskLimits,
};

fn graph(count: usize, edges: &[(usize, usize, u32)]) -> Result<NamespaceGraph, ModelError> {
    let mut all = NamespaceGraph::default();
    for at in 0..count {
        let name = QualifiedName::new(format!("M{at}"))?;
        all.namespaces.push(NamespaceNode { name, types: 1 });
    }
    for &(from, to, count) in edges {
        all.edges.push(NamespaceEdge { from, to, count });
    }
    Ok(all)
}

fn layer(name: &str, module: &str, may_use: &[&str]) -> Result<Layer, ModelError> {
    Ok(Layer {
        name: LayerName::new(name)?,
        namespaces: vec![QualifiedName::new(module)?],
        may_use: Some(
            may_use
                .iter()
                .map(|one| LayerName::new(*one))
                .collect::<Result<_, _>>()?,
        ),
    })
}

#[test]
fn the_kept_modules_are_the_most_referenced_ones_in_graph_order() -> Result<(), ModelError> {
    let all = graph(4, &[(0, 1, 1), (2, 1, 9), (2, 3, 4)])?;
    let report = ModuleReport::of(&all, &[]);
    assert_eq!(report.kept(2), [1, 2]);
    assert_eq!(report.kept(10), [0, 1, 2, 3]);
    Ok(())
}

#[test]
fn a_cut_leaves_the_rings_and_the_rows_of_the_rest_alone() -> Result<(), ModelError> {
    let all = graph(3, &[(0, 1, 5), (1, 0, 4), (2, 0, 1)])?;
    let report = ModuleReport::of(&all, &[]);
    let rows = report.rows(&report.kept(2));
    let names: Vec<&str> = rows.iter().map(|row| row.name.as_str()).collect();
    assert_eq!(names, ["M0", "M1"]);
    assert!(rows.iter().all(|row| row.in_cycle));
    assert_eq!(report.cycles().len(), 1);
    Ok(())
}

#[test]
fn a_dependency_against_the_layering_is_named_with_both_layers() -> Result<(), ModelError> {
    let all = graph(2, &[(0, 1, 3)])?;
    let layers = [layer("core", "M0", &[])?, layer("ui", "M1", &["core"])?];
    let report = ModuleReport::of(&all, &layers);
    let found = report.violation_rows();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].from_layer, "core");
    assert_eq!(found[0].to_layer, "ui");
    assert_eq!(report.rows(&[0, 1])[0].violations, 1);
    Ok(())
}

#[test]
fn a_risk_names_the_module_and_the_less_stable_one_it_depends_on() -> Result<(), ModelError> {
    let all = graph(3, &[(1, 0, 10), (2, 0, 10), (0, 2, 5), (2, 1, 40)])?;
    let limits = RiskLimits {
        min_incoming: 10,
        instability_gap: 100,
        max_risks: 5,
    };
    let risks = ModuleReport::of(&all, &[]).risks(&limits);
    assert_eq!(risks.len(), 1);
    assert_eq!(risks[0].module, "M0");
    assert_eq!(risks[0].depends_on, "M2");
    Ok(())
}
