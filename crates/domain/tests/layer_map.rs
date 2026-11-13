//! The layers of the dependency map as a layout.

#[cfg(test)]
mod tests {
    use codeskimr_domain::{
        Layer, LayerEdge, LayerName, LayoutParams, NodeSize, QualifiedName, layer_map,
    };

    const SIZE: NodeSize = NodeSize {
        width: 100,
        height: 40,
    };
    const PARAMS: LayoutParams = LayoutParams {
        rank_gap: 30,
        node_gap: 20,
        sweeps: 4,
        placement_passes: 2,
    };

    fn name(text: &str) -> LayerName {
        LayerName::new(text).unwrap()
    }

    fn layer(text: &str) -> Layer {
        Layer {
            name: name(text),
            namespaces: vec![QualifiedName::new(text).unwrap()],
        }
    }

    fn edge(from: &str, to: &str) -> LayerEdge {
        LayerEdge {
            from: name(from),
            to: name(to),
            count: 1,
            violates_layering: false,
            in_cycle: false,
        }
    }

    #[test]
    fn declared_layers_sit_in_their_declared_order_from_top_to_bottom() {
        let layers = [layer("ui"), layer("app"), layer("domain")];
        let map = layer_map(&layers, &[edge("domain", "ui")], SIZE, &PARAMS).unwrap();
        let rows: Vec<u32> = map.layout.placements.iter().map(|p| p.rank).collect();
        assert_eq!(rows, [0, 1, 2]);
        let tops: Vec<u32> = map.layout.placements.iter().map(|p| p.y).collect();
        assert!(tops[0] < tops[1] && tops[1] < tops[2]);
    }

    #[test]
    fn a_layer_without_edges_still_gets_a_box() {
        let layers = [layer("ui"), layer("app")];
        let map = layer_map(&layers, &[], SIZE, &PARAMS).unwrap();
        assert_eq!(map.names.len(), 2);
        assert_eq!(map.layout.placements.len(), 2);
    }

    #[test]
    fn a_layer_only_the_edges_name_is_added_after_the_declared_ones() {
        let layers = [layer("ui")];
        let map = layer_map(&layers, &[edge("ui", "gone")], SIZE, &PARAMS).unwrap();
        let names: Vec<&str> = map.names.iter().map(LayerName::as_str).collect();
        assert_eq!(names, ["ui", "gone"]);
        assert!(map.layout.placements[1].rank > map.layout.placements[0].rank);
    }

    #[test]
    fn a_name_that_is_declared_twice_is_one_box() {
        let layers = [layer("ui"), layer("ui")];
        let map = layer_map(&layers, &[], SIZE, &PARAMS).unwrap();
        assert_eq!(map.names.len(), 1);
    }

    #[test]
    fn no_layers_and_no_edges_give_an_empty_picture() {
        let map = layer_map(&[], &[], SIZE, &PARAMS).unwrap();
        assert_eq!(map.names.len(), 0);
        assert_eq!(map.layout.width, 0);
    }

    #[test]
    fn the_same_input_gives_the_same_picture() {
        let layers = [layer("ui"), layer("app"), layer("domain")];
        let edges = [
            edge("ui", "app"),
            edge("app", "domain"),
            edge("ui", "domain"),
        ];
        let first = layer_map(&layers, &edges, SIZE, &PARAMS).unwrap();
        assert_eq!(first, layer_map(&layers, &edges, SIZE, &PARAMS).unwrap());
    }
}
