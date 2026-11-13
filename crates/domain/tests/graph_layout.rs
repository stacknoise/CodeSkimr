//! Layered graph layout: ranks, crossings, spacing and robustness.

#[cfg(test)]
mod tests {
    use codeskimr_domain::{
        Layout, LayoutEdge, LayoutGraph, LayoutNode, LayoutParams, ModelError, NodeIndex, NodeSize,
    };

    const PARAMS: LayoutParams = LayoutParams {
        rank_gap: 40,
        node_gap: 20,
        sweeps: 8,
        placement_passes: 4,
    };

    fn node(rank: Option<u32>) -> LayoutNode {
        LayoutNode {
            size: NodeSize {
                width: 100,
                height: 30,
            },
            rank,
        }
    }

    fn edge(from: usize, to: usize) -> LayoutEdge {
        LayoutEdge {
            from: NodeIndex::new(from),
            to: NodeIndex::new(to),
        }
    }

    fn graph(count: usize, edges: &[(usize, usize)]) -> LayoutGraph {
        let nodes = (0..count).map(|_| node(None)).collect();
        LayoutGraph::new(nodes, edges.iter().map(|&(a, b)| edge(a, b)).collect()).unwrap()
    }

    fn no_overlap(layout: &Layout) {
        for (i, a) in layout.placements.iter().enumerate() {
            for b in &layout.placements[i + 1..] {
                let apart_x = a.x + 100 <= b.x || b.x + 100 <= a.x;
                let apart_y = a.y + 30 <= b.y || b.y + 30 <= a.y;
                assert!(apart_x || apart_y, "boxes overlap: {a:?} {b:?}");
            }
        }
    }

    #[test]
    fn empty_graph_gives_empty_layout() {
        let layout = graph(0, &[]).layout(&PARAMS);
        assert_eq!(layout.placements.len(), 0);
        assert_eq!((layout.width, layout.height), (0, 0));
    }

    #[test]
    fn edge_to_unknown_node_is_rejected() {
        let result = LayoutGraph::new(vec![node(None)], vec![edge(0, 1)]);
        assert_eq!(result.unwrap_err(), ModelError::UnknownNode);
    }

    #[test]
    fn chain_goes_down_one_row_per_step() {
        let layout = graph(3, &[(0, 1), (1, 2)]).layout(&PARAMS);
        let ranks: Vec<u32> = layout.placements.iter().map(|p| p.rank).collect();
        assert_eq!(ranks, [0, 1, 2]);
        assert_eq!(layout.placements[1].y, 30 + 40);
        assert_eq!(layout.height, 3 * 30 + 2 * 40);
    }

    #[test]
    fn cycle_and_self_loop_still_get_distinct_rows() {
        let layout = graph(3, &[(0, 1), (1, 2), (2, 0), (1, 1)]).layout(&PARAMS);
        let mut ranks: Vec<u32> = layout.placements.iter().map(|p| p.rank).collect();
        ranks.sort_unstable();
        assert_eq!(ranks, [0, 1, 2]);
    }

    #[test]
    fn fixed_ranks_are_kept_even_against_the_edge_direction() {
        let nodes = vec![node(Some(2)), node(Some(0))];
        let layout = LayoutGraph::new(nodes, vec![edge(0, 1)])
            .unwrap()
            .layout(&PARAMS);
        assert_eq!(layout.placements[0].rank, 2);
        assert_eq!(layout.placements[1].rank, 0);
    }

    #[test]
    fn crossing_edges_are_untangled() {
        // 0 and 1 on top, 2 and 3 below; the input order would cross 0->3 with 1->2.
        let layout = graph(4, &[(0, 3), (1, 2)]).layout(&PARAMS);
        let order = |i: usize| layout.placements[i].order;
        assert_eq!(order(0) < order(1), order(3) < order(2));
    }

    #[test]
    fn long_edge_does_not_push_nodes_apart_or_overlap() {
        let layout = graph(4, &[(0, 1), (1, 2), (2, 3), (0, 3)]).layout(&PARAMS);
        no_overlap(&layout);
        assert_eq!(layout.placements[3].rank, 3);
    }

    #[test]
    fn same_input_gives_same_output() {
        let g = graph(6, &[(0, 1), (0, 2), (1, 3), (2, 3), (3, 4), (3, 5), (5, 0)]);
        assert_eq!(g.layout(&PARAMS), g.layout(&PARAMS));
    }

    #[test]
    fn random_graphs_never_overlap_and_stay_in_the_picture() {
        let mut seed: u64 = 7;
        let mut next = |limit: usize| {
            seed = seed
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            usize::try_from(seed >> 33).unwrap() % limit
        };
        for count in [1, 2, 5, 40, 120] {
            let edges: Vec<(usize, usize)> =
                (0..count * 3).map(|_| (next(count), next(count))).collect();
            let layout = graph(count, &edges).layout(&PARAMS);
            no_overlap(&layout);
            for p in &layout.placements {
                assert!(p.x + 100 <= layout.width && p.y + 30 <= layout.height);
            }
        }
    }
}
