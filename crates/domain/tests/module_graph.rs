//! Cycles, coupling and risks between modules (ADR 0108).

use codeskimr_domain::{
    ModelError, ModuleCoupling, NamespaceEdge, NamespaceGraph, NamespaceNode, QualifiedName,
    RiskLimits, find_cycles, find_risks, module_coupling,
};

fn graph(count: usize, edges: &[(usize, usize, u32)]) -> Result<NamespaceGraph, ModelError> {
    Ok(NamespaceGraph {
        namespaces: (0..count)
            .map(|at| {
                Ok(NamespaceNode {
                    name: QualifiedName::new(format!("M{at}"))?,
                    types: 1,
                })
            })
            .collect::<Result<_, ModelError>>()?,
        edges: edges
            .iter()
            .map(|&(from, to, count)| NamespaceEdge { from, to, count })
            .collect(),
    })
}

fn pairs(edges: &[NamespaceEdge]) -> Vec<(usize, usize)> {
    edges.iter().map(|edge| (edge.from, edge.to)).collect()
}

#[test]
fn a_graph_without_a_ring_has_no_cycle() -> Result<(), ModelError> {
    assert_eq!(
        find_cycles(&graph(3, &[(0, 1, 4), (1, 2, 1), (0, 2, 9)])?),
        Vec::new()
    );
    Ok(())
}

#[test]
fn two_modules_that_use_each_other_are_one_ring_and_the_weaker_edge_ends_it()
-> Result<(), ModelError> {
    let found = find_cycles(&graph(3, &[(0, 1, 11), (1, 0, 2), (1, 2, 5)])?);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].modules, [0, 1]);
    assert_eq!(pairs(&found[0].edges), [(0, 1), (1, 0)]);
    assert_eq!(pairs(&found[0].resolution), [(1, 0)]);
    Ok(())
}

#[test]
fn a_ring_of_three_and_a_separate_pair_are_two_cycles() -> Result<(), ModelError> {
    let found = find_cycles(&graph(
        5,
        &[(0, 1, 3), (1, 2, 4), (2, 0, 1), (3, 4, 2), (4, 3, 2)],
    )?);
    let modules: Vec<_> = found.iter().map(|cycle| cycle.modules.clone()).collect();
    assert_eq!(modules, [vec![0, 1, 2], vec![3, 4]]);
    assert_eq!(pairs(&found[0].resolution), [(2, 0)]);
    assert_eq!(pairs(&found[1].resolution), [(3, 4)]);
    Ok(())
}

#[test]
fn a_resolution_removes_more_edges_when_one_is_not_enough() -> Result<(), ModelError> {
    let edges = [(0, 1, 1), (1, 0, 5), (1, 2, 2), (2, 1, 6)];
    let found = find_cycles(&graph(3, &edges)?);
    assert_eq!(found.len(), 1);
    assert_eq!(pairs(&found[0].resolution), [(0, 1), (1, 2)]);
    Ok(())
}

#[test]
fn a_long_chain_does_not_overflow_the_stack() -> Result<(), ModelError> {
    let count = 50_000;
    let mut edges: Vec<(usize, usize, u32)> = (0..count - 1).map(|at| (at, at + 1, 1)).collect();
    edges.push((count - 1, 0, 1));
    let found = find_cycles(&graph(count, &edges)?);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].modules.len(), count);
    assert_eq!(found[0].resolution.len(), 1);
    Ok(())
}

#[test]
fn coupling_adds_up_the_references_and_the_instability_is_outgoing_over_all()
-> Result<(), ModelError> {
    let all = module_coupling(&graph(4, &[(0, 1, 3), (2, 1, 5), (1, 3, 8)])?);
    assert_eq!(
        all[1],
        ModuleCoupling {
            incoming: 8,
            outgoing: 8
        }
    );
    assert_eq!(all[1].instability(), Some(500));
    assert_eq!(all[3].instability(), Some(0));
    assert_eq!(all[0].instability(), Some(1000));
    assert_eq!(ModuleCoupling::default().instability(), None);
    Ok(())
}

#[test]
fn a_much_used_module_that_depends_on_a_less_stable_one_is_a_risk() -> Result<(), ModelError> {
    // 0 is used 19 times and 0.51 unstable, it depends on 1, which is 0.76 unstable; 2 is used
    // 60 times, 0.22 unstable, and depends on 0.
    let g = graph(3, &[(2, 0, 17), (0, 1, 20), (1, 0, 2), (1, 2, 60)])?;
    let coupling = module_coupling(&g);
    let limits = RiskLimits {
        min_incoming: 10,
        instability_gap: 100,
        max_risks: 5,
    };
    let risks = find_risks(&g, &coupling, &limits);
    let found: Vec<_> = risks.iter().map(|r| (r.module, r.depends_on)).collect();
    assert_eq!(found, [(2, 0), (0, 1)]);
    let strict = RiskLimits {
        min_incoming: 100,
        ..limits
    };
    assert_eq!(find_risks(&g, &coupling, &strict), Vec::new());
    Ok(())
}
