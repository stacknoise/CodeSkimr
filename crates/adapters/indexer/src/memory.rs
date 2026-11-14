//! Memory of a process tree, to enforce `level2.max_memory_mb`.

use std::io;

use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};

/// Measures the resident memory of a process and everything it started.
pub struct TreeMemory {
    system: System,
}

impl TreeMemory {
    /// A meter without a first measurement.
    ///
    /// # Errors
    /// If the operating system gives no access to process information. A run then does not start,
    /// because its memory limit could not be enforced.
    pub fn new() -> io::Result<Self> {
        Ok(Self {
            system: System::new().map_err(io::Error::other)?,
        })
    }

    /// Resident bytes of `root` and all its descendants.
    pub fn measure(&mut self, root: u32) -> u64 {
        self.system.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing().with_memory(),
        );
        let mut members = vec![Pid::from_u32(root)];
        let mut total: u64 = 0;
        let mut next: usize = 0;
        while let Some(pid) = members.get(next).copied() {
            next = next.saturating_add(1);
            if let Some(process) = self.system.process(pid) {
                total = total.saturating_add(process.memory());
            }
            let children: Vec<Pid> = self
                .system
                .processes()
                .values()
                .filter(|child| child.parent() == Some(pid))
                .map(sysinfo::Process::pid)
                .filter(|child| !members.contains(child))
                .collect();
            members.extend(children);
        }
        total
    }
}
