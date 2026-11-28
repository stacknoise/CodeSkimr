//! Makes the picture of the map readable: a row with more boxes than fit a line wraps into
//! several lines, and the picture gets a margin so that the stripe of a layer surrounds its boxes.

use std::collections::{BTreeMap, BTreeSet};

use codeskimr_application::config::Graph;

use super::consts::{BAND_PAD_DIVISOR, CENTRE_DIVISOR, MARGIN_SIDES, MIN_LINE_BOXES};
use super::graph::BoxOut;

/// The size of the picture after [`arrange`].
#[derive(Clone, Copy)]
pub(super) struct Extent {
    pub(super) width: u32,
    pub(super) height: u32,
}

fn to_usize(value: u32) -> usize {
    usize::try_from(value).unwrap_or(usize::MAX)
}

fn to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

/// A row of the layout and which boxes it holds: the boxes of a layer come first, then the ones that
/// belong to no layer. Both parts are wrapped on their own, so that the stripe of a layer never
/// covers a line it shares with a box of no layer. Every layer has exactly one row, because the
/// layout fixes the row of a box by the position of its layer; so no box of no layer can lie
/// between two rows of one layer.
type RowKey = (u32, bool);

/// What the wrapped picture is for.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Purpose {
    /// Every row is centred; there are no stripes, so boxes of a layer and of none stay mixed.
    Centred,
    /// Only long rows or rows with boxes of a layer and of none change; stripes are drawn.
    Stripes,
}

/// Boxes of each row, in the order the layout gave them.
fn rows(boxes: &[BoxOut], purpose: Purpose) -> BTreeMap<RowKey, Vec<usize>> {
    let mut rows: BTreeMap<RowKey, Vec<usize>> = BTreeMap::new();
    for (index, found) in boxes.iter().enumerate() {
        rows.entry((
            found.rank,
            purpose == Purpose::Stripes && found.layer.is_none(),
        ))
        .or_default()
        .push(index);
    }
    for row in rows.values_mut() {
        row.sort_by_key(|&index| boxes.get(index).map(|found| found.order));
    }
    rows
}

/// Whether a row is too long for a line or a row holds boxes of a layer and boxes of no layer.
fn needs_change(rows: &BTreeMap<RowKey, Vec<usize>>, per_line: usize) -> bool {
    let ranks: BTreeSet<u32> = rows.keys().map(|key| key.0).collect();
    rows.values().any(|row| row.len() > per_line) || ranks.len() < rows.len()
}

/// Puts the boxes of a row that is too long into a grid; a picture with no long row stays as the
/// layout placed it. Returns the new size, or `None` when nothing changed.
fn wrap_rows(boxes: &mut [BoxOut], look: &Graph, purpose: Purpose) -> Option<Extent> {
    let per_line = to_usize(look.row_boxes.max(MIN_LINE_BOXES));
    let rows = rows(boxes, purpose);
    if purpose == Purpose::Stripes && !needs_change(&rows, per_line) {
        return None;
    }
    let (cell_width, cell_height) = (
        look.node_width + look.node_gap,
        look.node_height + look.node_gap,
    );
    let widest = rows.values().map(Vec::len).max().unwrap_or(0).min(per_line);
    let width = (to_u32(widest) * cell_width).saturating_sub(look.node_gap);
    let mut top = 0;
    for row in rows.values() {
        for (line, members) in row.chunks(per_line).enumerate() {
            let indent =
                (width + look.node_gap - to_u32(members.len()) * cell_width) / CENTRE_DIVISOR;
            let y = top + to_u32(line) * cell_height;
            for (column, &index) in members.iter().enumerate() {
                if let Some(found) = boxes.get_mut(index) {
                    found.x = indent + to_u32(column) * cell_width;
                    found.y = y;
                }
            }
        }
        top += to_u32(row.len().div_ceil(per_line)) * cell_height - look.node_gap + look.rank_gap;
    }
    Some(Extent {
        width,
        height: top.saturating_sub(look.rank_gap),
    })
}

/// Puts every row into centred lines of at most `row_boxes` boxes, even a short one, so that the
/// rows of the type level sit centred above and below the type in the middle.
pub(super) fn centre(boxes: &mut [BoxOut], look: &Graph) -> Extent {
    wrap_rows(boxes, look, Purpose::Centred).unwrap_or(Extent {
        width: 0,
        height: 0,
    })
}

/// Wraps long rows and gives the picture its margin: the boxes move down and to the right by
/// half a row gap, so that the stripe of the first layer has room above its boxes. `extent` is the
/// size the layout reported; the result has the margin on both sides.
pub(super) fn arrange(boxes: &mut [BoxOut], extent: Extent, look: &Graph) -> Extent {
    let extent = wrap_rows(boxes, look, Purpose::Stripes).unwrap_or(extent);
    let pad = look.rank_gap / BAND_PAD_DIVISOR;
    for found in boxes.iter_mut() {
        found.y += pad;
        found.x += pad;
    }
    Extent {
        width: extent.width + pad * MARGIN_SIDES,
        height: extent.height + pad * MARGIN_SIDES,
    }
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
            row_boxes: 3,
            risk_min_incoming: 10,
            risk_instability_gap: 100,
            risk_max: 5,
            max_modules: 200,
            max_places: 100,
        }
    }

    fn owned(mut all: Vec<BoxOut>, count: usize) -> Vec<BoxOut> {
        for found in all.iter_mut().take(count) {
            found.layer = Some("layer".to_owned());
        }
        all
    }

    fn boxes(ranks: &[u32]) -> Vec<BoxOut> {
        let mut seen = BTreeMap::new();
        ranks
            .iter()
            .map(|&rank| {
                let order = seen.entry(rank).or_insert(0_u32);
                *order += 1;
                BoxOut {
                    name: String::new(),
                    label: String::new(),
                    context: String::new(),
                    x: *order * 500,
                    y: rank * 100,
                    width: 100,
                    height: 40,
                    rank,
                    order: *order - 1,
                    layer: None,
                    types: None,
                    symbol: None,
                }
            })
            .collect()
    }

    #[test]
    fn a_short_row_keeps_its_places_and_gets_only_the_margin() {
        let mut placed = boxes(&[0, 0]);
        let out = arrange(
            &mut placed,
            Extent {
                width: 1000,
                height: 40,
            },
            &look(),
        );
        assert_eq!((placed[0].x, placed[0].y), (530, 30));
        assert_eq!((out.width, out.height), (1060, 100));
    }

    #[test]
    fn a_long_row_wraps_into_lines_of_at_most_the_configured_length() {
        let mut placed = boxes(&[0; 7]);
        let out = arrange(
            &mut placed,
            Extent {
                width: 9999,
                height: 40,
            },
            &look(),
        );
        let lines: BTreeMap<u32, usize> = placed.iter().fold(BTreeMap::new(), |mut all, b| {
            *all.entry(b.y).or_default() += 1;
            all
        });
        assert_eq!(lines.values().copied().collect::<Vec<_>>(), [3, 3, 1]);
        assert!(out.width <= 400, "wrapped width {}", out.width);
        assert!(placed.iter().all(|b| b.x + b.width <= out.width));
        assert!(placed.iter().all(|b| b.y + b.height <= out.height));
    }

    #[test]
    fn centred_rows_share_one_axis() {
        let mut placed = boxes(&[0, 0, 0, 1]);
        let out = centre(&mut placed, &look());
        let middle = |found: &BoxOut| found.x + found.width / 2;
        assert_eq!(middle(&placed[1]), out.width / 2);
        assert_eq!(middle(&placed[3]), out.width / 2);
    }

    #[test]
    fn the_rows_stay_below_each_other_after_wrapping() {
        let mut placed = boxes(&[0, 0, 0, 0, 1]);
        arrange(
            &mut placed,
            Extent {
                width: 9999,
                height: 140,
            },
            &look(),
        );
        let first_bottom = placed[..4].iter().map(|b| b.y + b.height).max().unwrap();
        assert!(placed[4].y >= first_bottom + 60);
    }

    #[test]
    fn boxes_of_no_layer_do_not_share_a_line_with_the_boxes_of_a_layer() {
        let mut placed = owned(boxes(&[0, 0, 0, 0]), 2);
        arrange(
            &mut placed,
            Extent {
                width: 1000,
                height: 40,
            },
            &look(),
        );
        let bottom = |own: bool| {
            placed
                .iter()
                .filter(|b| b.layer.is_some() == own)
                .map(|b| (b.y, b.y + b.height))
                .collect::<Vec<_>>()
        };
        let (own, other) = (bottom(true), bottom(false));
        let own_end = own.iter().map(|r| r.1).max().unwrap();
        let other_start = other.iter().map(|r| r.0).min().unwrap();
        assert!(other_start >= own_end, "{other_start} < {own_end}");
    }

    #[test]
    fn centred_rows_keep_boxes_of_a_layer_and_of_none_together() {
        let mut placed = owned(boxes(&[0, 0, 0, 0]), 2);
        centre(&mut placed, &look());
        let lines: std::collections::BTreeSet<u32> = placed.iter().map(|b| b.y).collect();
        assert_eq!(lines.len(), 2, "four boxes at three per line");
    }
}
