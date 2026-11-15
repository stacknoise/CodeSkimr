//! Finding the intervals that contain a position without looking at all of them (issue #220).
//!
//! A minified or generated file has thousands of symbols and occurrences on a few lines. A search
//! through all of them for every occurrence is quadratic. The intervals are kept sorted by their
//! start as an implicit balanced tree whose nodes know the largest end below them, so a query
//! skips every part that cannot reach the position: `O(log n + hits)`.

use std::ops::Range;

use codeskimr_domain::Position;

/// One interval with what it stands for and its place in the order it was given in.
struct Entry<T> {
    start: Position,
    end: Position,
    order: usize,
    value: T,
}

/// Intervals of positions, searchable by a position they contain.
pub(crate) struct IntervalIndex<T> {
    entries: Vec<Entry<T>>,
    /// For the node in the middle of a range of `entries`: the largest end in that range.
    max_end: Vec<Position>,
}

impl<T> IntervalIndex<T> {
    /// Builds the index of `(start, end, value)` triples.
    pub(crate) fn new(items: impl IntoIterator<Item = (Position, Position, T)>) -> Self {
        let mut entries: Vec<Entry<T>> = items
            .into_iter()
            .enumerate()
            .map(|(order, (start, end, value))| Entry {
                start,
                end,
                order,
                value,
            })
            .collect();
        entries.sort_by_key(|entry| (entry.start, entry.order));
        let mut max_end: Vec<Position> = entries.iter().map(|entry| entry.end).collect();
        Self::fill(&entries, &mut max_end, 0, entries.len());
        Self { entries, max_end }
    }

    /// Writes the largest end of each range into the slot of its middle node; returns the
    /// largest end of `lo..hi`.
    fn fill(
        entries: &[Entry<T>],
        max_end: &mut [Position],
        lo: usize,
        hi: usize,
    ) -> Option<Position> {
        if lo >= hi {
            return None;
        }
        let mid = lo.midpoint(hi);
        let left = Self::fill(entries, max_end, lo, mid);
        let right = Self::fill(entries, max_end, mid.saturating_add(1), hi);
        let own = entries.get(mid)?.end;
        let largest = [left, right, Some(own)].into_iter().flatten().max()?;
        if let Some(slot) = max_end.get_mut(mid) {
            *slot = largest;
        }
        Some(largest)
    }

    /// The values whose interval starts at or before `at` and whose end `reaches`, in the order
    /// they were given in. `reaches` must hold for a larger end whenever it holds for a smaller.
    pub(crate) fn containing(&self, at: Position, reaches: impl Fn(Position) -> bool) -> Vec<&T> {
        let mut found: Vec<&Entry<T>> = Vec::new();
        let query = Query { at, reaches };
        self.visit(0..self.entries.len(), &query, &mut found);
        found.sort_by_key(|entry| entry.order);
        found.into_iter().map(|entry| &entry.value).collect()
    }

    fn visit<'s>(
        &'s self,
        range: Range<usize>,
        query: &Query<impl Fn(Position) -> bool>,
        found: &mut Vec<&'s Entry<T>>,
    ) {
        let Range { start: lo, end: hi } = range;
        if lo >= hi {
            return;
        }
        let mid = lo.midpoint(hi);
        let (Some(entry), Some(largest)) = (self.entries.get(mid), self.max_end.get(mid)) else {
            return;
        };
        if !(query.reaches)(*largest) {
            return;
        }
        self.visit(lo..mid, query, found);
        // Everything right of the middle starts at or after its start.
        if entry.start <= query.at {
            if (query.reaches)(entry.end) {
                found.push(entry);
            }
            self.visit(mid.saturating_add(1)..hi, query, found);
        }
    }
}

/// What a search asks for: a position and the ends that are far enough.
struct Query<F> {
    at: Position,
    reaches: F,
}

#[cfg(test)]
mod tests {
    use codeskimr_domain::Position;

    use super::IntervalIndex;

    fn at(column: u32) -> Position {
        Position { line: 0, column }
    }

    fn found<'a>(index: &IntervalIndex<&'a str>, column: u32) -> Vec<&'a str> {
        index
            .containing(at(column), |end| at(column) < end)
            .into_iter()
            .copied()
            .collect()
    }

    #[test]
    fn it_finds_every_interval_around_a_position_in_the_order_it_was_given() {
        let index = IntervalIndex::new([
            (at(5), at(9), "inner"),
            (at(0), at(20), "outer"),
            (at(5), at(9), "twin"),
            (at(12), at(15), "later"),
        ]);
        assert_eq!(found(&index, 6), ["inner", "outer", "twin"]);
        assert_eq!(found(&index, 13), ["outer", "later"]);
        // The end is not part of the interval.
        assert_eq!(found(&index, 9), ["outer"]);
        assert_eq!(found(&index, 20), Vec::<&str>::new());
    }

    #[test]
    fn an_empty_index_finds_nothing() {
        let index: IntervalIndex<&str> = IntervalIndex::new([]);
        assert_eq!(found(&index, 3), Vec::<&str>::new());
    }

    #[test]
    fn it_agrees_with_a_scan_over_all_intervals() {
        let items: Vec<(Position, Position, usize)> = (0..200_u32)
            .map(|n| {
                (
                    at((n * 7) % 90),
                    at((n * 7) % 90 + 1 + (n * 13) % 40),
                    n as usize,
                )
            })
            .collect();
        let index = IntervalIndex::new(items.clone());
        for column in 0..140 {
            let expected: Vec<usize> = items
                .iter()
                .filter(|(start, end, _)| *start <= at(column) && at(column) < *end)
                .map(|(_, _, value)| *value)
                .collect();
            let got: Vec<usize> = index
                .containing(at(column), |end| at(column) < end)
                .into_iter()
                .copied()
                .collect();
            assert_eq!(got, expected, "column {column}");
        }
    }
}
