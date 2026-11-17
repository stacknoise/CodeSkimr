//! Pages of a list: where it starts and how long it may be, held to the configured limits.

use crate::config::Annotations;

/// Where a page starts and how many rows it may hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageRequest {
    /// Rows to skip.
    pub offset: u32,
    /// Rows to return, at least one and at most `annotations.max_page_size`.
    pub limit: u32,
}

impl PageRequest {
    /// The page a caller asked for. A missing limit is `annotations.default_page_size`; a limit
    /// beyond `annotations.max_page_size` is held to it, and zero becomes one.
    #[must_use]
    pub fn resolve(offset: Option<u32>, limit: Option<u32>, config: &Annotations) -> Self {
        Self {
            offset: offset.unwrap_or_default(),
            limit: limit
                .unwrap_or(config.default_page_size)
                .clamp(1, config.max_page_size),
        }
    }
}

/// One page of a list and how long the whole list is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Page<T> {
    /// The rows of this page.
    pub items: Vec<T>,
    /// Rows of the whole list.
    pub total: u32,
    /// Rows skipped.
    pub offset: u32,
    /// The limit that was applied.
    pub limit: u32,
}

impl<T> Page<T> {
    /// The page `request` names of `all`.
    #[must_use]
    pub fn of(all: Vec<T>, request: PageRequest) -> Self {
        let total = u32::try_from(all.len()).unwrap_or(u32::MAX);
        let skip = usize::try_from(request.offset).unwrap_or(usize::MAX);
        let take = usize::try_from(request.limit).unwrap_or(usize::MAX);
        Self {
            items: all.into_iter().skip(skip).take(take).collect(),
            total,
            offset: request.offset,
            limit: request.limit,
        }
    }

    /// The offset of the next page, `None` on the last one.
    #[must_use]
    pub fn next_offset(&self) -> Option<u32> {
        let next = self
            .offset
            .saturating_add(u32::try_from(self.items.len()).unwrap_or(u32::MAX));
        (next < self.total).then_some(next)
    }

    /// The same page with each row turned into another.
    #[must_use]
    pub fn map<U>(self, change: impl FnMut(T) -> U) -> Page<U> {
        Page {
            items: self.items.into_iter().map(change).collect(),
            total: self.total,
            offset: self.offset,
            limit: self.limit,
        }
    }
}
