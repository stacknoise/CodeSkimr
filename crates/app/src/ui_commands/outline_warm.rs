//! The outline of a project is the first thing the explorer asks for when a project is opened.
//! This wrapper uses that moment to build the usages index of the same snapshot in the
//! background, so that the map and the statistics do not wait for it when they are opened next.
//!
//! The outline is only asked for the newest snapshot of a project, so that is the one that is
//! built. What was started is remembered here and never read from the store, because the store
//! holds its lock for the whole build and the outline must not wait for it. At most one build
//! runs at a time; a call that finds one running starts nothing.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{CodeOutline, MemberRow, OutlineNode};
use codeskimr_domain::{SnapshotId, SymbolId};

use super::consts::INDEX_WARM_THREAD;

/// What builds the index of a snapshot; the store in the program.
pub(super) trait IndexBuild: Send + Sync + 'static {
    /// Builds the index of `snapshot` and keeps it; whether that worked.
    fn build(&self, snapshot: SnapshotId) -> bool;
}

impl IndexBuild for codeskimr_storage::SqliteStore {
    fn build(&self, snapshot: SnapshotId) -> bool {
        self.warm(snapshot).is_ok()
    }
}

/// The outline of the store that builds the index of the snapshot it was asked about.
pub(super) struct WarmingOutline<B: IndexBuild> {
    outline: Arc<dyn CodeOutline>,
    builder: Arc<B>,
    /// The snapshot whose index was started last and did not fail.
    started: Arc<Mutex<Option<SnapshotId>>>,
    /// Whether a build is running.
    busy: Arc<AtomicBool>,
}

impl<B: IndexBuild> WarmingOutline<B> {
    /// Wraps `outline`; `builder` builds the indexes.
    pub(super) fn new(outline: Arc<dyn CodeOutline>, builder: Arc<B>) -> Self {
        Self {
            outline,
            builder,
            started: Arc::new(Mutex::new(None)),
            busy: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Starts the build of `snapshot` unless it was started already or another one runs.
    fn warm(&self, snapshot: SnapshotId) {
        {
            let mut started = self.started.lock().unwrap_or_else(PoisonError::into_inner);
            if *started == Some(snapshot) || self.busy.swap(true, Ordering::AcqRel) {
                return;
            }
            *started = Some(snapshot);
        }
        let (builder, started, busy) = (
            Arc::clone(&self.builder),
            Arc::clone(&self.started),
            Arc::clone(&self.busy),
        );
        let spawned = std::thread::Builder::new()
            .name(INDEX_WARM_THREAD.to_owned())
            .spawn(move || {
                if !builder.build(snapshot) {
                    // A failed build may be tried again by the next outline.
                    *started.lock().unwrap_or_else(PoisonError::into_inner) = None;
                }
                busy.store(false, Ordering::Release);
            });
        if spawned.is_err() {
            // A thread that cannot start leaves the index to the first query.
            self.busy.store(false, Ordering::Release);
            *self.started.lock().unwrap_or_else(PoisonError::into_inner) = None;
        }
    }
}

impl<B: IndexBuild> CodeOutline for WarmingOutline<B> {
    fn outline(&self, snapshot: SnapshotId) -> Result<Vec<OutlineNode>, CodedError> {
        let nodes = self.outline.outline(snapshot)?;
        self.warm(snapshot);
        Ok(nodes)
    }

    fn member_rows(
        &self,
        snapshot: SnapshotId,
        symbol: SymbolId,
    ) -> Result<Vec<MemberRow>, CodedError> {
        self.outline.member_rows(snapshot, symbol)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::AtomicUsize;
    use std::sync::mpsc::{Receiver, Sender, channel};
    use std::time::Duration;

    use super::*;

    struct Plain;

    impl CodeOutline for Plain {
        fn outline(&self, _: SnapshotId) -> Result<Vec<OutlineNode>, CodedError> {
            Ok(Vec::new())
        }

        fn member_rows(&self, _: SnapshotId, _: SymbolId) -> Result<Vec<MemberRow>, CodedError> {
            Ok(Vec::new())
        }
    }

    /// Counts the builds and holds each one until the test lets it go.
    struct Gate {
        builds: AtomicUsize,
        works: bool,
        entered: Mutex<Sender<()>>,
        release: Mutex<Receiver<()>>,
    }

    impl IndexBuild for Gate {
        fn build(&self, _: SnapshotId) -> bool {
            self.builds.fetch_add(1, Ordering::SeqCst);
            self.entered.lock().unwrap().send(()).unwrap();
            self.release.lock().unwrap().recv().unwrap();
            self.works
        }
    }

    const WAIT: Duration = Duration::from_secs(5);

    struct Rig {
        warming: WarmingOutline<Gate>,
        gate: Arc<Gate>,
        entered: Receiver<()>,
        release: Sender<()>,
    }

    fn rig(works: bool) -> Rig {
        let (entered_tx, entered) = channel();
        let (release, release_rx) = channel();
        let gate = Arc::new(Gate {
            builds: AtomicUsize::new(0),
            works,
            entered: Mutex::new(entered_tx),
            release: Mutex::new(release_rx),
        });
        let warming = WarmingOutline::new(Arc::new(Plain), Arc::clone(&gate));
        Rig {
            warming,
            gate,
            entered,
            release,
        }
    }

    fn finished(rig: &Rig) {
        rig.release.send(()).unwrap();
        while rig.warming.busy.load(Ordering::Acquire) {
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    #[test]
    fn the_outline_does_not_wait_for_a_build_that_runs() {
        let rig = rig(true);
        rig.warming.outline(SnapshotId::new(1)).unwrap();
        rig.entered.recv_timeout(WAIT).unwrap();
        // The build is held; the outline of the same and of another snapshot answers at once.
        assert!(rig.warming.outline(SnapshotId::new(1)).is_ok());
        assert!(rig.warming.outline(SnapshotId::new(2)).is_ok());
        assert_eq!(rig.gate.builds.load(Ordering::SeqCst), 1);
        finished(&rig);
    }

    /// Asks for the outline of snapshot 1 and lets the build that starts end.
    fn built_once(rig: &Rig) {
        rig.warming.outline(SnapshotId::new(1)).unwrap();
        rig.entered.recv_timeout(WAIT).unwrap();
        finished(rig);
    }

    #[test]
    fn a_snapshot_that_was_built_is_not_built_again() {
        let rig = rig(true);
        built_once(&rig);
        rig.warming.outline(SnapshotId::new(1)).unwrap();
        assert_eq!(rig.gate.builds.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn a_failed_build_is_tried_again_by_the_next_outline() {
        let rig = rig(false);
        built_once(&rig);
        rig.warming.outline(SnapshotId::new(1)).unwrap();
        rig.entered.recv_timeout(WAIT).unwrap();
        assert_eq!(rig.gate.builds.load(Ordering::SeqCst), 2);
        finished(&rig);
    }

    #[test]
    fn the_members_are_passed_through() {
        let rig = rig(true);
        let rows = rig
            .warming
            .member_rows(SnapshotId::new(1), SymbolId::new(1))
            .unwrap();
        assert!(rows.is_empty());
        assert_eq!(rig.gate.builds.load(Ordering::SeqCst), 0);
    }
}
