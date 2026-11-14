//! Cleaning up after a hard stop of the application (issue #223).
//!
//! A run kills its process group at every normal end. If `CodeSkimr` itself is killed (`kill -9`, a
//! crash, a power cut) nothing of that runs, and neither a process group on Unix nor the job object
//! of `process-wrap` on Windows (which has no `KILL_ON_JOB_CLOSE`) ends the indexer. Without
//! `unsafe` code there is no way to ask the operating system for that, so the runner leaves a
//! note in every output folder: who owns it (the application) and which process it started. The
//! next start reads the notes, ends what is still running of an owner that is gone, and removes
//! the folder.
//!
//! A process is told apart by its id **and** its start time, so a process that got the same id
//! later is never touched.

use std::fs;
use std::io;
use std::path::Path;
use std::time::{Duration, SystemTime};

use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};

use crate::consts::{
    INDEXER_NOTE_NAME, NOTE_SEPARATOR, ORPHAN_GRACE, OUTPUT_DIR_PREFIX, OWNER_NOTE_NAME,
};

/// A process as it is written in a note: its id and the second it started.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Identity {
    pid: u32,
    started: u64,
}

impl Identity {
    fn of(system: &System, pid: u32) -> Option<Self> {
        let process = system.process(Pid::from_u32(pid))?;
        Some(Self {
            pid,
            started: process.start_time(),
        })
    }

    fn parse(text: &str) -> Option<Self> {
        let (pid, started) = text.trim().split_once(NOTE_SEPARATOR)?;
        Some(Self {
            pid: pid.parse().ok()?,
            started: started.parse().ok()?,
        })
    }

    fn text(self) -> String {
        [
            self.pid.to_string(),
            NOTE_SEPARATOR.to_string(),
            self.started.to_string(),
        ]
        .concat()
    }

    /// Whether the process of this identity is running.
    fn is_running(self, system: &System) -> bool {
        Self::of(system, self.pid) == Some(self)
    }
}

fn processes() -> Option<System> {
    let mut system = System::new().ok()?;
    system.refresh_processes_specifics(ProcessesToUpdate::All, true, ProcessRefreshKind::nothing());
    Some(system)
}

/// Writes who owns `folder`: the running application.
///
/// # Errors
/// If the process list or the file is not accessible.
pub fn note_owner(folder: &Path) -> io::Result<()> {
    note(folder.join(OWNER_NOTE_NAME), std::process::id())
}

/// Writes which process the run started in `folder`.
///
/// # Errors
/// If the process list or the file is not accessible.
pub fn note_indexer(folder: &Path, pid: u32) -> io::Result<()> {
    note(folder.join(INDEXER_NOTE_NAME), pid)
}

fn note(file: std::path::PathBuf, pid: u32) -> io::Result<()> {
    let system = processes().ok_or_else(|| io::Error::other("no process list"))?;
    let identity = Identity::of(&system, pid).ok_or_else(|| io::Error::other("no such process"))?;
    fs::write(file, identity.text())
}

fn read_note(folder: &Path, name: &str) -> Option<Identity> {
    Identity::parse(&fs::read_to_string(folder.join(name)).ok()?)
}

/// Ends what orphaned run folders of `data_dir` left running and removes them. A folder of a
/// running application (this one or another instance) stays; a folder without a note is left
/// alone for [`ORPHAN_GRACE`], because it may just have been created. Best effort: whatever cannot
/// be ended or removed stays for the next start.
pub fn sweep(data_dir: &Path) {
    let Ok(entries) = fs::read_dir(data_dir) else {
        return;
    };
    let Some(system) = processes() else {
        return;
    };
    for entry in entries.filter_map(Result::ok) {
        let is_run_folder = entry
            .file_name()
            .to_string_lossy()
            .starts_with(OUTPUT_DIR_PREFIX)
            && entry.file_type().is_ok_and(|kind| kind.is_dir());
        if is_run_folder && is_orphan(&entry.path(), &system) {
            end_indexer(&entry.path(), &system);
            // A folder that cannot be removed now is tried again at the next start.
            drop(fs::remove_dir_all(entry.path()));
        }
    }
}

fn is_orphan(folder: &Path, system: &System) -> bool {
    match read_note(folder, OWNER_NOTE_NAME) {
        Some(owner) => !owner.is_running(system),
        None => is_older_than(folder, ORPHAN_GRACE),
    }
}

fn is_older_than(folder: &Path, age: Duration) -> bool {
    fs::metadata(folder)
        .and_then(|meta| meta.modified())
        .ok()
        .and_then(|modified| SystemTime::now().duration_since(modified).ok())
        .is_some_and(|elapsed| elapsed >= age)
}

/// Kills the process the run noted and every process that is still below it.
fn end_indexer(folder: &Path, system: &System) {
    let Some(indexer) = read_note(folder, INDEXER_NOTE_NAME).filter(|note| note.is_running(system))
    else {
        return;
    };
    let mut members = vec![Pid::from_u32(indexer.pid)];
    let mut next: usize = 0;
    while let Some(pid) = members.get(next).copied() {
        next = next.saturating_add(1);
        let children = system
            .processes()
            .values()
            .filter(|child| child.parent() == Some(pid))
            .map(sysinfo::Process::pid)
            .filter(|child| !members.contains(child))
            .collect::<Vec<_>>();
        members.extend(children);
    }
    // The leader first, so that it cannot start more.
    for pid in members {
        if let Some(process) = system.process(pid) {
            process.kill();
        }
    }
}
