//! The outline of a project is the first thing the explorer asks for when a project is opened.
//! This wrapper uses that moment to build the usages index of the same snapshot in the
//! background, so that the map and the statistics do not wait for it when they are opened next.
//! Only the snapshot that is kept is built; the index of a project that is opened again is not
//! built twice.

use std::sync::Arc;

use codeskimr_storage::SqliteStore;
use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{CodeOutline, MemberRow, OutlineNode};
use codeskimr_domain::{SnapshotId, SymbolId};

use super::consts::INDEX_WARM_THREAD;

/// The outline of the store that builds the index of the snapshot it was asked about.
pub(super) struct WarmingOutline {
    pub(super) store: Arc<SqliteStore>,
}

impl CodeOutline for WarmingOutline {
    fn outline(&self, snapshot: SnapshotId) -> Result<Vec<OutlineNode>, CodedError> {
        let nodes = self.store.outline(snapshot)?;
        if self.store.kept_snapshot() != Some(snapshot) {
            let store = Arc::clone(&self.store);
            // A thread that cannot start leaves the index to the first query.
            std::thread::Builder::new()
                .name(INDEX_WARM_THREAD.to_owned())
                .spawn(move || store.warm(snapshot).ok())
                .ok();
        }
        Ok(nodes)
    }

    fn member_rows(
        &self,
        snapshot: SnapshotId,
        symbol: SymbolId,
    ) -> Result<Vec<MemberRow>, CodedError> {
        self.store.member_rows(snapshot, symbol)
    }
}
