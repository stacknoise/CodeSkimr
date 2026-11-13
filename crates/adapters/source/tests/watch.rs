//! `watch`: debounced batches of changed paths.
#![allow(missing_docs)]

mod common;

use std::sync::mpsc;
use std::time::Duration;

use codeskimr_source::{WatchConfig, watch};
use common::{TestResult, folder, write};

const DEBOUNCE: Duration = Duration::from_millis(100);
const WAIT: Duration = Duration::from_secs(10);

fn config() -> WatchConfig {
    WatchConfig {
        debounce: DEBOUNCE,
        excluded_folders: vec!["target".to_owned()],
    }
}

#[test]
fn reports_one_batch_for_a_burst_of_changes() -> TestResult {
    let dir = tempfile::tempdir()?;
    let (sender, receiver) = mpsc::channel();
    let _handle = watch(&folder(dir.path())?, config(), move |batch| {
        let _ = sender.send(batch);
    })?;
    write(dir.path(), "a.rs", "1")?;
    write(dir.path(), "b.rs", "2")?;
    write(dir.path(), "target/skipped.rs", "3")?;
    let mut seen = Vec::new();
    while !(seen.contains(&"a.rs".to_owned()) && seen.contains(&"b.rs".to_owned())) {
        let batch = receiver.recv_timeout(WAIT)?;
        seen.extend(batch.iter().map(|path| path.as_str().to_owned()));
    }
    assert!(seen.iter().all(|path| !path.starts_with("target")));
    Ok(())
}

#[test]
fn a_missing_folder_cannot_be_watched() -> TestResult {
    let dir = tempfile::tempdir()?;
    let result = watch(&folder(&dir.path().join("missing"))?, config(), |_| {});
    assert!(result.is_err());
    Ok(())
}
