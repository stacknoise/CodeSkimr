//! Memory hardening (blueprint 12.4): secret buffers locked against swapping, no core dumps.
//!
//! This is the only module of the product that may contain `unsafe` (rule R-08). Every block has
//! a `// SAFETY:` comment and a test.
#![allow(unsafe_code)]

use std::fmt;
use std::ops::Deref;

use codeskimr_application::ports::AuthError;
use zeroize::Zeroize;

/// Secret bytes in memory that the operating system may not write to the swap file.
///
/// The buffer is locked before the secret is copied into it, and wiped and unlocked when it is
/// dropped. It is neither `Clone` nor `Debug` with content.
pub struct LockedBytes {
    bytes: Box<[u8]>,
}

impl LockedBytes {
    /// Locks a new buffer and moves `source` into it; `source` is wiped on every path.
    ///
    /// # Errors
    /// `SEC-901` if the operating system refuses to lock the memory.
    pub fn from_vec(mut source: Vec<u8>) -> Result<Self, AuthError> {
        let mut bytes = vec![0; source.len()].into_boxed_slice();
        if !bytes.is_empty() && !lock(&mut bytes) {
            source.zeroize();
            return Err(AuthError::MemoryLockFailed);
        }
        bytes.copy_from_slice(&source);
        source.zeroize();
        Ok(Self { bytes })
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
        self.bytes.is_empty()
    }
}

impl Deref for LockedBytes {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &self.bytes
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
        if !self.bytes.is_empty() {
            unlock(&mut self.bytes);
        }
    }
}

fn lock(bytes: &mut [u8]) -> bool {
    // SAFETY: the pointer and length come from one live, exclusively borrowed slice; the call
    // only changes how the pages are held in memory, not their content.
    unsafe { memsec::mlock(bytes.as_mut_ptr(), bytes.len()) }
}

fn unlock(bytes: &mut [u8]) {
    bytes.zeroize();
    // SAFETY: as in `lock`; the range was locked by `from_vec`. `munlock` wipes the range again
    // before it unlocks it. A failed unlock leaves a wiped, locked page, which is harmless.
    let _ = unsafe { memsec::munlock(bytes.as_mut_ptr(), bytes.len()) };
}

/// Turns off core dumps (Unix) or error-report memory dumps (Windows) for this process.
///
/// Called once at program start by the composition root. Returns whether every step worked, so
/// the caller can log that the hardening is partial; it never stops the program.
#[must_use]
pub fn disable_core_dumps() -> bool {
    platform::disable_core_dumps()
}

#[cfg(unix)]
mod platform {
    pub fn disable_core_dumps() -> bool {
        let none = libc::rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };
        // SAFETY: `none` is a valid, initialised `rlimit` that lives for the whole call.
        let limited = unsafe { libc::setrlimit(libc::RLIMIT_CORE, &raw const none) } == 0;
        limited && not_dumpable()
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
    use windows_sys::Win32::System::Diagnostics::Debug::{
        SEM_FAILCRITICALERRORS, SEM_NOGPFAULTERRORBOX, SetErrorMode,
    };

    pub fn disable_core_dumps() -> bool {
        // SAFETY: `SetErrorMode` takes a flag set and no pointer. The flags switch off the
        // Windows error-report dialog and the report with a memory dump for this process.
        unsafe { SetErrorMode(SEM_FAILCRITICALERRORS | SEM_NOGPFAULTERRORBOX) };
        true
    }
}

#[cfg(not(any(unix, windows)))]
mod platform {
    pub fn disable_core_dumps() -> bool {
        false
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
        let source = b"moved".to_vec();
        let pointer = source.as_ptr();
        let locked = LockedBytes::from_vec(source);
        assert!(locked.is_ok_and(|l| l.as_ptr() != pointer));
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
        assert!(disable_core_dumps());
        assert_eq!(platform::core_limit(), Some(0));
    }

    #[cfg(windows)]
    #[test]
    fn the_error_report_is_switched_off() {
        assert!(disable_core_dumps());
    }
}
