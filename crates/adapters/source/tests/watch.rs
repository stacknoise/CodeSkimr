//! `watch`: debounced batches of changed paths.
#![allow(missing_docs)]

mod common;

use std::sync::mpsc::{self, Receiver};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use codeskimr_source::{WatchBatch, WatchConfig, WatchHandle, watch};
use common::{TestResult, folder, write};

const WAIT: Duration = Duration::from_secs(10);

fn config(debounce_ms: u64, max_wait_ms: u64) -> WatchConfig {
    WatchConfig {
        debounce: Duration::from_millis(debounce_ms),
        max_wait: Duration::from_millis(max_wait_ms),
        excluded_folders: vec!["target".to_owned()],
    }
}

fn start(
    dir: &std::path::Path,
    config: WatchConfig,
) -> TestResult<(WatchHandle, Receiver<WatchBatch>)> {
    let (sender, receiver) = mpsc::channel();
    let handle = watch(&folder(dir)?, config, move |batch| {
        let _ = sender.send(batch);
    })?;
    Ok((handle, receiver))
}

/// Writes `relative` every 30 ms until the returned flag is lowered and the thread is joined.
fn keep_writing(
    root: &std::path::Path,
    relative: &'static str,
) -> (Arc<AtomicBool>, JoinHandle<()>) {
    let running = Arc::new(AtomicBool::new(true));
    let flag = Arc::clone(&running);
    let root = root.to_path_buf();
    let writer = std::thread::spawn(move || {
        while flag.load(Ordering::Acquire) {
            let _ = write(&root, relative, "x");
            std::thread::sleep(Duration::from_millis(30));
        }
    });
    (running, writer)
}

fn names(batch: &WatchBatch) -> Vec<&str> {
    batch
        .paths
        .iter()
        .map(codeskimr_domain::RelativePath::as_str)
        .collect()
}

#[test]
fn a_burst_of_changes_is_one_batch() -> TestResult {
    let dir = tempfile::tempdir()?;
    let (_handle, receiver) = start(dir.path(), config(400, 5_000))?;
    write(dir.path(), "a.rs", "1")?;
    write(dir.path(), "b.rs", "2")?;
    write(dir.path(), "target/skipped.rs", "3")?;
    let batch = receiver.recv_timeout(WAIT)?;
    assert_eq!(names(&batch), ["a.rs", "b.rs"]);
    assert!(!batch.rescan);
    Ok(())
}

#[test]
fn constant_writes_cannot_starve_the_listener() -> TestResult {
    let dir = tempfile::tempdir()?;
    let (_handle, receiver) = start(dir.path(), config(5_000, 300))?;
    let (running, writer) = keep_writing(dir.path(), "tick.rs");
    let started = Instant::now();
    let batch = receiver.recv_timeout(Duration::from_secs(3));
    running.store(false, Ordering::Release);
    let _ = writer.join();
    assert!(names(&batch?).contains(&"tick.rs"));
    assert!(started.elapsed() < Duration::from_secs(3));
    Ok(())
}

#[test]
fn noise_in_excluded_folders_does_not_delay_a_batch() -> TestResult {
    let dir = tempfile::tempdir()?;
    let (_handle, receiver) = start(dir.path(), config(300, 60_000))?;
    write(dir.path(), "real.rs", "1")?;
    let (running, writer) = keep_writing(dir.path(), "target/noise.rs");
    let batch = receiver.recv_timeout(Duration::from_secs(3));
    running.store(false, Ordering::Release);
    let _ = writer.join();
    assert_eq!(names(&batch?), ["real.rs"]);
    Ok(())
}

#[test]
fn dropping_the_handle_delivers_what_is_pending() -> TestResult {
    let dir = tempfile::tempdir()?;
    let (handle, receiver) = start(dir.path(), config(60_000, 60_000))?;
    write(dir.path(), "late.rs", "1")?;
    std::thread::sleep(Duration::from_millis(500));
    drop(handle);
    let batch = receiver.recv_timeout(WAIT)?;
    assert_eq!(names(&batch), ["late.rs"]);
    assert!(receiver.recv_timeout(Duration::from_millis(200)).is_err());
    Ok(())
}

#[test]
fn a_missing_folder_cannot_be_watched() -> TestResult {
    let dir = tempfile::tempdir()?;
    let result = watch(
        &folder(&dir.path().join("missing"))?,
        config(100, 1_000),
        |_| {},
    );
    assert!(result.is_err());
    Ok(())
}
