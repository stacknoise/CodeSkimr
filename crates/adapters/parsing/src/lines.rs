//! Classification of source lines as blank, comment or code, the input of the LOC metric.

use std::collections::HashSet;

use codeskimr_domain::LineKind;
use tree_sitter::Node;

/// Classifies every line of the tree. A line with any code token is `Code`, one with only comment
/// tokens is `Comment`, the rest is `Blank`.
#[must_use]
pub fn classify(root: Node<'_>, comment_nodes: &HashSet<String>) -> Vec<LineKind> {
    let mut lines = vec![LineKind::Blank; root.end_position().row.saturating_add(1)];
    let mut cursor = root.walk();
    loop {
        let node = cursor.node();
        let is_comment = comment_nodes.contains(node.kind());
        let is_leaf = is_comment || node.child_count() == 0;
        if is_leaf && node.end_byte() > node.start_byte() {
            mark(&mut lines, node, is_comment);
        }
        if !is_leaf && cursor.goto_first_child() {
            continue;
        }
        while !cursor.goto_next_sibling() {
            if !cursor.goto_parent() {
                return lines;
            }
        }
    }
}

fn mark(lines: &mut [LineKind], node: Node<'_>, is_comment: bool) {
    let start = node.start_position();
    let end = node.end_position();
    // A token that ends at column zero does not touch its last row.
    let last = if end.column == 0 && end.row > start.row {
        end.row - 1
    } else {
        end.row
    };
    let kind = if is_comment {
        LineKind::Comment
    } else {
        LineKind::Code
    };
    for slot in lines.iter_mut().take(last + 1).skip(start.row) {
        *slot = stronger(*slot, kind);
    }
}

const fn stronger(old: LineKind, new: LineKind) -> LineKind {
    match (old, new) {
        (LineKind::Code, _) | (_, LineKind::Code) => LineKind::Code,
        (LineKind::Comment, _) | (_, LineKind::Comment) => LineKind::Comment,
        _ => LineKind::Blank,
    }
}
