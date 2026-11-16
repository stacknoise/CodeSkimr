//! The namespace level as nested frames: every segment of a namespace name that several
//! namespaces share becomes a frame around them (`App` around `App.Views` and `App.Models`, and
//! so on). The frames of one layer are packed into lines, so a project with hundreds of
//! namespaces stays readable.

use std::collections::BTreeMap;

use codeskimr_application::config::Graph;
use codeskimr_domain::name_parts;

use super::consts::{CENTRE_DIVISOR, NAME_JOIN};
use super::graph::{BoxOut, GroupOut};
use super::graph_wrap::Extent;

/// The namespaces below one name segment.
#[derive(Default)]
struct Tree {
    own: Option<usize>,
    kids: BTreeMap<String, Tree>,
}

impl Tree {
    fn insert(&mut self, parts: &[&str], index: usize) {
        match parts.split_first() {
            None => self.own = Some(index),
            Some((first, rest)) => self
                .kids
                .entry((*first).to_owned())
                .or_default()
                .insert(rest, index),
        }
    }
}

/// Sizes of the picture, in layout units.
struct Metrics {
    node: (u32, u32),
    gap: u32,
    title: u32,
}

enum Content {
    Leaf {
        index: usize,
        label: String,
    },
    Frame {
        path: String,
        label: String,
        items: Vec<Placed>,
    },
}

struct Block {
    width: u32,
    height: u32,
    content: Content,
}

struct Placed {
    x: u32,
    y: u32,
    block: Block,
}

fn join(head: &str, tail: &str) -> String {
    if head.is_empty() {
        tail.to_owned()
    } else {
        [head, tail].join(NAME_JOIN)
    }
}

/// Puts the blocks left to right and starts a new line when the next one does not fit.
fn shelf(blocks: Vec<Block>, max: u32, gap: u32) -> (Vec<Placed>, u32, u32) {
    let (mut x, mut y, mut line, mut width) = (0, 0, 0, 0);
    let mut placed = Vec::new();
    for block in blocks {
        if x > 0 && x + block.width > max {
            y += line + gap;
            (x, line) = (0, 0);
        }
        let (own_width, own_height) = (block.width, block.height);
        placed.push(Placed { x, y, block });
        x += own_width + gap;
        line = line.max(own_height);
        width = width.max(x - gap);
    }
    (placed, width, y + line)
}

fn leaf(index: usize, label: String, metrics: &Metrics) -> Block {
    Block {
        width: metrics.node.0,
        height: metrics.node.1,
        content: Content::Leaf { index, label },
    }
}

/// A frame (or, for a namespace without children, a box) for `tree`. A chain of segments with one
/// child each and no namespace of its own is written as one title.
fn tree_block(label: &str, path: &str, tree: &Tree, max: u32, metrics: &Metrics) -> Block {
    let (mut label, mut path, mut tree) = (label.to_owned(), path.to_owned(), tree);
    while tree.own.is_none() && tree.kids.len() == 1 {
        let Some((segment, only)) = tree.kids.iter().next() else {
            break;
        };
        (label, path, tree) = (join(&label, segment), join(&path, segment), only);
    }
    if let (Some(index), true) = (tree.own, tree.kids.is_empty()) {
        return leaf(index, label, metrics);
    }
    let pad = metrics.gap;
    let inner = max.saturating_sub(pad + pad);
    let mut blocks: Vec<Block> = tree
        .own
        .map(|index| leaf(index, label.clone(), metrics))
        .into_iter()
        .collect();
    blocks.extend(
        tree.kids
            .iter()
            .map(|(segment, kid)| tree_block(segment, &join(&path, segment), kid, inner, metrics)),
    );
    let (mut items, width, height) = shelf(blocks, inner, pad);
    for item in &mut items {
        item.x += pad;
        item.y += metrics.title;
    }
    Block {
        width: width + pad + pad,
        height: metrics.title + height + pad,
        content: Content::Frame { path, label, items },
    }
}

fn emit(
    block: &Block,
    at: (u32, u32),
    depth: u32,
    boxes: &mut [BoxOut],
    groups: &mut Vec<GroupOut>,
) {
    match &block.content {
        Content::Leaf { index, label } => {
            if let Some(found) = boxes.get_mut(*index) {
                (found.x, found.y) = at;
                found.label.clone_from(label);
            }
        }
        Content::Frame { path, label, items } => {
            groups.push(GroupOut {
                name: path.clone(),
                label: label.clone(),
                x: at.0,
                y: at.1,
                width: block.width,
                height: block.height,
                depth,
            });
            for item in items {
                emit(
                    &item.block,
                    (at.0 + item.x, at.1 + item.y),
                    depth + 1,
                    boxes,
                    groups,
                );
            }
        }
    }
}

fn metrics(look: &Graph) -> Metrics {
    Metrics {
        node: (look.node_width, look.node_height),
        gap: look.node_gap / CENTRE_DIVISOR,
        title: look.node_height / CENTRE_DIVISOR,
    }
}

/// Places the namespace boxes of every row (layer) into nested frames, rows below each other.
/// Returns the frames and the size of the picture.
pub(super) fn arrange(boxes: &mut [BoxOut], look: &Graph) -> (Vec<GroupOut>, Extent) {
    let metrics = metrics(look);
    let mut rows: BTreeMap<u32, Tree> = BTreeMap::new();
    for (index, found) in boxes.iter().enumerate() {
        rows.entry(found.rank)
            .or_default()
            .insert(&name_parts(&found.name), index);
    }
    let per_line = look.row_boxes.max(1);
    let max = per_line * (look.node_width + look.node_gap) - look.node_gap;
    let (mut groups, mut top, mut width) = (Vec::new(), 0, 0);
    for tree in rows.values() {
        let blocks = tree
            .kids
            .iter()
            .map(|(segment, kid)| tree_block(segment, segment, kid, max, &metrics))
            .collect();
        let (items, row_width, row_height) = shelf(blocks, max, metrics.gap);
        for item in &items {
            emit(&item.block, (item.x, top + item.y), 0, boxes, &mut groups);
        }
        width = width.max(row_width);
        top += row_height + look.rank_gap;
    }
    let height = top.saturating_sub(look.rank_gap);
    (groups, Extent { width, height })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn look() -> Graph {
        Graph {
            node_width: 100,
            node_height: 40,
            rank_gap: 60,
            node_gap: 20,
            sweeps: 1,
            placement_passes: 1,
            row_boxes: 4,
        }
    }

    fn boxes(names: &[&str]) -> Vec<BoxOut> {
        names
            .iter()
            .map(|name| BoxOut {
                name: (*name).to_owned(),
                label: String::new(),
                x: 0,
                y: 0,
                width: 100,
                height: 40,
                rank: 0,
                order: 0,
                layer: None,
                types: Some(1),
                symbol: None,
            })
            .collect()
    }

    fn inside(inner: (u32, u32, u32, u32), outer: &GroupOut) -> bool {
        inner.0 >= outer.x
            && inner.1 >= outer.y
            && inner.0 + inner.2 <= outer.x + outer.width
            && inner.1 + inner.3 <= outer.y + outer.height
    }

    #[test]
    fn a_shared_first_segment_becomes_a_frame_around_its_namespaces() {
        let mut placed = boxes(&["Shop.Cart", "Shop.Pay", "Shop.Pay.Card", "Lone"]);
        let (groups, _) = arrange(&mut placed, &look());
        let names: Vec<_> = groups.iter().map(|group| group.name.as_str()).collect();
        assert_eq!(names, ["Shop", "Shop.Pay"]);
        let labels: Vec<_> = placed.iter().map(|found| found.label.as_str()).collect();
        assert_eq!(labels, ["Cart", "Pay", "Card", "Lone"]);
        assert_eq!(groups.iter().map(|g| g.depth).collect::<Vec<_>>(), [0, 1]);
    }

    #[test]
    fn every_box_lies_inside_its_frames_and_nothing_overlaps() {
        let mut placed = boxes(&["A.B.C", "A.B.D", "A.E", "F.G", "F.H", "I"]);
        let (groups, extent) = arrange(&mut placed, &look());
        for found in &placed {
            let area = (found.x, found.y, found.width, found.height);
            let frames = groups
                .iter()
                .filter(|group| found.name.starts_with(group.name.as_str()))
                .filter(|group| inside(area, group))
                .count();
            let wanted = groups
                .iter()
                .filter(|group| found.name.starts_with(group.name.as_str()))
                .count();
            assert_eq!(frames, wanted, "{}", found.name);
            assert!(found.x + found.width <= extent.width);
            assert!(found.y + found.height <= extent.height);
        }
        for (at, one) in placed.iter().enumerate() {
            for other in &placed[at + 1..] {
                let apart = one.x + one.width <= other.x
                    || other.x + other.width <= one.x
                    || one.y + one.height <= other.y
                    || other.y + other.height <= one.y;
                assert!(apart, "{} and {}", one.name, other.name);
            }
        }
    }

    #[test]
    fn a_chain_of_single_segments_is_one_title() {
        let mut placed = boxes(&["A.B.C.X", "A.B.C.Y"]);
        let (groups, _) = arrange(&mut placed, &look());
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].label, "A.B.C");
    }

    #[test]
    fn a_long_list_of_siblings_wraps_below_the_configured_width() {
        let names: Vec<String> = (0..30).map(|n| format!("P.N{n:02}")).collect();
        let refs: Vec<&str> = names.iter().map(String::as_str).collect();
        let mut placed = boxes(&refs);
        let (_, extent) = arrange(&mut placed, &look());
        assert!(extent.width <= 4 * 120);
        assert!(extent.height > 40 * 3);
    }
}
