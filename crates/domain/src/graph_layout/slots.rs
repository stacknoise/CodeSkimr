//! Bounds-checked access to the working arrays of the layout (R-07 forbids plain indexing).
//!
//! All indices come from the layout itself and are valid by construction; an index that is not
//! valid reads as the default value and writes nothing, so a bug here can never panic.

/// Reads the item at `index`, or the default when there is none.
pub fn value<T: Copy + Default>(items: &[T], index: usize) -> T {
    items.get(index).copied().unwrap_or_default()
}

/// Overwrites the item at `index` when it exists.
pub fn set<T>(items: &mut [T], index: usize, new: T) {
    if let Some(slot) = items.get_mut(index) {
        *slot = new;
    }
}

/// Appends to the list at `index` when it exists.
pub fn push<T>(lists: &mut [Vec<T>], index: usize, new: T) {
    if let Some(list) = lists.get_mut(index) {
        list.push(new);
    }
}

/// Reads the list at `index`, or an empty slice.
pub fn list<T>(lists: &[Vec<T>], index: usize) -> &[T] {
    lists.get(index).map_or(&[], Vec::as_slice)
}
