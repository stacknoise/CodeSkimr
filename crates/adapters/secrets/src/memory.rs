//! Memory hardening (blueprint 12.4): secret buffers locked against swapping, no core dumps.
//!
//! This is the only module of the product that may contain `unsafe` (rule R-08). Every block has
//! a `// SAFETY:` comment and a test.
#![allow(unsafe_code)]

use std::fmt;
use std::ops::Deref;
use std::ptr::NonNull;

use codeskimr_application::ports::AuthError;
use zeroize::Zeroize;

/// Secret bytes in memory that the operating system may not write to the swap file.
///
/// Every buffer has its own pages (`memsec::malloc_sized`: page-aligned, guard pages, locked), so
/// freeing one buffer never unlocks the memory of another. The pages are wiped and unlocked when
/// the buffer is dropped. It is neither `Clone` nor `Debug` with content.
pub struct LockedBytes {
    /// `None` for an empty buffer, which has nothing to lock.
    block: Option<NonNull<[u8]>>,
}

// SAFETY: the block belongs to this value alone and is only read through `&self`; moving the
// owner to another thread or reading from several threads at once cannot race.
unsafe impl Send for LockedBytes {}
// SAFETY: see `Send`; no method changes the bytes after `from_vec` has returned.
unsafe impl Sync for LockedBytes {}

impl LockedBytes {
    /// Locks a new buffer and moves `source` into it; `source` is wiped on every path.
    ///
    /// # Errors
    /// `SEC-901` if the operating system refuses to lock the memory.
    pub fn from_vec(mut source: Vec<u8>) -> Result<Self, AuthError> {
        Self::take_from(&mut source)
    }

    /// Like [`from_vec`](Self::from_vec) for a vector the caller keeps: it is wiped (and emptied
    /// of content, not of length) on every path.
    ///
    /// # Errors
    /// `SEC-901` if the operating system refuses to lock the memory.
    pub fn take_from(source: &mut [u8]) -> Result<Self, AuthError> {
        Self::take_locking(source, lock)
    }

    /// The work of [`take_from`](Self::take_from) with the locking step injected, so that its
    /// refusal can be tested.
    fn take_locking(
        source: &mut [u8],
        locker: impl FnOnce(&mut [u8]) -> bool,
    ) -> Result<Self, AuthError> {
        let result = Self::allocate(source.len(), locker).map(|mut buffer| {
            buffer.bytes_mut().copy_from_slice(source);
            buffer
        });
        source.zeroize();
        result
    }

    fn allocate(len: usize, locker: impl FnOnce(&mut [u8]) -> bool) -> Result<Self, AuthError> {
        if len == 0 {
            return Ok(Self { block: None });
        }
        // SAFETY: `malloc_sized` has no preconditions; it returns `None` if it cannot allocate.
        let block = unsafe { memsec::malloc_sized(len) }.ok_or(AuthError::MemoryLockFailed)?;
        let mut buffer = Self { block: Some(block) };
        // On a refusal the drop of `buffer` wipes and frees the block again.
        if locker(buffer.bytes_mut()) {
            Ok(buffer)
        } else {
            Err(AuthError::MemoryLockFailed)
        }
    }

    fn bytes_mut(&mut self) -> &mut [u8] {
        match self.block.as_mut() {
            // SAFETY: the block is alive and exclusively borrowed through `&mut self`.
            Some(block) => unsafe { block.as_mut() },
            None => &mut [],
        }
    }

    /// Locks a new buffer holding a copy of `source`.
    ///
    /// # Errors
    /// `SEC-901` if the operating system refuses to lock the memory.
    pub fn copy_of(source: &[u8]) -> Result<Self, AuthError> {
        Self::from_vec(source.to_vec())
    }

    /// Whether the buffer holds no bytes (an empty buffer has nothing to lock).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.block.is_none()
    }
}

impl Deref for LockedBytes {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        match self.block.as_ref() {
            // SAFETY: the block is alive as long as `self` is, and nobody writes to it through
            // a shared reference.
            Some(block) => unsafe { block.as_ref() },
            None => &[],
        }
    }
}

impl fmt::Debug for LockedBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(stringify!(LockedBytes))
            .finish_non_exhaustive()
    }
}

impl Drop for LockedBytes {
    fn drop(&mut self) {
        if let Some(block) = self.block.take() {
            // SAFETY: `block` came from `malloc_sized` and is freed exactly once, here. `free`
            // wipes the bytes, unlocks the pages and returns them to the system.
            unsafe { memsec::free(block) };
        }
    }
}

/// Makes sure the pages of `bytes` are locked. `malloc_sized` already tries to; a second call
/// is harmless and shows whether it worked, which the allocator does not report.
fn lock(bytes: &mut [u8]) -> bool {
    // SAFETY: the pointer and length come from one live, exclusively borrowed slice; the call
    // only changes how the pages are held in memory, not their content.
    unsafe { memsec::mlock(bytes.as_mut_ptr(), bytes.len()) }
}

/// How far the core dumps of this process could be switched off.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoreDumps {
    /// No core dump is written.
    Off,
    /// The error dialog and report of the system are off, but the system may still write a dump
    /// by its own policy (Windows `LocalDumps`).
    Partly,
    /// Nothing could be done on this platform or a step failed.
    Unavailable,
}

/// Turns off core dumps (Unix) or the error report of Windows for this process.
///
/// Called once at program start by the composition root; it never stops the program.
#[must_use]
pub fn disable_core_dumps() -> CoreDumps {
    platform::disable_core_dumps()
}

#[cfg(unix)]
mod platform {
    use super::CoreDumps;

    pub fn disable_core_dumps() -> CoreDumps {
        let none = libc::rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };
        // SAFETY: `none` is a valid, initialised `rlimit` that lives for the whole call.
        let capped = unsafe { libc::setrlimit(libc::RLIMIT_CORE, &raw const none) } == 0;
        if capped && not_dumpable() {
            CoreDumps::Off
        } else {
            CoreDumps::Unavailable
        }
    }

    /// Also keeps other processes from reading the memory through `ptrace` or `/proc` (Linux).
    #[cfg(target_os = "linux")]
    fn not_dumpable() -> bool {
        // SAFETY: `PR_SET_DUMPABLE` takes plain integers and no pointer.
        unsafe { libc::prctl(libc::PR_SET_DUMPABLE, 0, 0, 0, 0) == 0 }
    }

    /// `PT_DENY_ATTACH` and the like are not needed; the zero core limit is enough there.
    #[cfg(not(target_os = "linux"))]
    fn not_dumpable() -> bool {
        true
    }

    /// Reads the current core limit; for the test.
    #[cfg(test)]
    pub fn core_limit() -> Option<libc::rlim_t> {
        let mut limit = libc::rlimit {
            rlim_cur: 1,
            rlim_max: 1,
        };
        // SAFETY: `limit` is a valid `rlimit` that the call fills in.
        let read = unsafe { libc::getrlimit(libc::RLIMIT_CORE, &raw mut limit) } == 0;
        read.then_some(limit.rlim_cur)
    }
}

#[cfg(windows)]
mod platform {
    use super::CoreDumps;

    use windows_sys::Win32::System::Diagnostics::Debug::{
        SEM_FAILCRITICALERRORS, SEM_NOGPFAULTERRORBOX, SetErrorMode,
    };

    pub fn disable_core_dumps() -> CoreDumps {
        // SAFETY: `SetErrorMode` takes a flag set and no pointer. The flags switch off the
        // Windows error-report dialog for this process. A machine-wide `LocalDumps` policy can
        // still write a dump, so the result is only partial.
        unsafe { SetErrorMode(SEM_FAILCRITICALERRORS | SEM_NOGPFAULTERRORBOX) };
        CoreDumps::Partly
    }
}

#[cfg(not(any(unix, windows)))]
mod platform {
    use super::CoreDumps;

    pub fn disable_core_dumps() -> CoreDumps {
        CoreDumps::Unavailable
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locked_bytes_hold_the_content_and_are_not_printed() {
        let locked = LockedBytes::copy_of(b"secret").ok();
        assert!(locked.as_deref().is_some_and(|bytes| bytes == b"secret"));
        assert!(locked.is_some_and(|l| !format!("{l:?}").contains("secret")));
    }

    #[test]
    fn the_source_is_wiped_when_it_moves_into_the_buffer() {
        let mut source = b"moved".to_vec();
        let locked = LockedBytes::take_from(&mut source);
        assert!(source.iter().all(|byte| *byte == 0));
        assert!(locked.is_ok_and(|l| &*l == b"moved"));
    }

    #[test]
    fn a_refused_lock_is_sec_901_and_the_source_is_wiped_anyway() {
        let mut source = b"refused".to_vec();
        let result = LockedBytes::take_locking(&mut source, |_| false);
        assert!(matches!(result, Err(AuthError::MemoryLockFailed)));
        assert!(source.iter().all(|byte| *byte == 0));
    }

    #[test]
    fn an_empty_buffer_needs_no_lock() {
        let locked = LockedBytes::from_vec(Vec::new());
        assert!(locked.is_ok_and(|l| l.is_empty()));
    }

    #[test]
    fn many_buffers_can_be_locked_and_released_again() {
        for _ in 0..64 {
            assert!(LockedBytes::copy_of(&[7; 256]).is_ok());
        }
    }

    #[cfg(unix)]
    #[test]
    fn core_dumps_are_off_afterwards() {
        assert_eq!(disable_core_dumps(), CoreDumps::Off);
        assert_eq!(platform::core_limit(), Some(0));
    }

    #[cfg(windows)]
    #[test]
    fn the_error_report_is_only_partly_switched_off() {
        assert_eq!(disable_core_dumps(), CoreDumps::Partly);
    }
}

/// Proof that the pages are locked, and that freeing one buffer keeps the other locked. It reads
/// `VmLck` of `/proc/self/status`, which every other test of the process would disturb, so the
/// measurement runs alone in a child process.
#[cfg(all(test, target_os = "linux"))]
mod locked_pages {
    use std::process::Command;

    use super::LockedBytes;

    const CHILD: &str = "CODESKIMR_TEST_VMLCK";
    const VMLCK: &str = "VmLck:";

    fn locked_kb() -> u64 {
        std::fs::read_to_string("/proc/self/status")
            .ok()
            .and_then(|status| {
                let line = status.lines().find(|line| line.starts_with(VMLCK))?;
                line.split_whitespace().nth(1)?.parse().ok()
            })
            .unwrap_or_default()
    }

    /// The body of the child process; it does nothing when the test runs on its own.
    #[test]
    fn child_measures_locked_pages() {
        if std::env::var(CHILD).is_err() {
            return;
        }
        let base = locked_kb();
        let first = LockedBytes::copy_of(&[1; 16]).unwrap();
        let second = LockedBytes::copy_of(&[2; 16]).unwrap();
        let both = locked_kb();
        assert!(both >= base + 2, "no pages locked: {base} -> {both}");
        drop(first);
        let after_first = locked_kb();
        assert!(after_first < both, "the first buffer kept its pages locked");
        assert!(after_first > base, "the second buffer lost its lock");
        assert_eq!(&*second, &[2; 16]);
        drop(second);
        assert_eq!(locked_kb(), base);
    }

    #[test]
    fn freeing_a_buffer_keeps_the_others_locked() {
        let status = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "memory::locked_pages::child_measures_locked_pages",
            ])
            .args(["--test-threads=1", "--nocapture"])
            .env(CHILD, "1")
            .status()
            .unwrap();
        assert!(status.success());
    }
}
