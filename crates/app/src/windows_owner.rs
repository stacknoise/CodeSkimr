//! Who owns the launch folder on Windows (issue #372).
//!
//! The access rules (see `private_acl`) cannot take rights away from the owner of a folder that
//! someone else made beforehand, so the owner is read from the open handle and must be the user,
//! the administrators or the system, like the Unix check accepts the user and root.

use std::fs::{File, OpenOptions};
use std::io;
use std::os::windows::fs::OpenOptionsExt;
use std::path::Path;

use windows_permissions::constants::{SeObjectType, SecurityInformation};
use windows_permissions::utilities::current_process_sid;
use windows_permissions::wrappers::GetSecurityInfo;
use windows_permissions::{LocalBox, Sid};

use crate::consts::{FILE_FLAG_BACKUP_SEMANTICS, TRUSTED_OWNER_SIDS};

fn refused() -> io::Error {
    io::Error::from(io::ErrorKind::PermissionDenied)
}

/// Whether `owner` is the user of this process or one of the trusted accounts.
fn is_trusted(owner: &Sid) -> io::Result<bool> {
    if owner == &*current_process_sid()? {
        return Ok(true);
    }
    for text in TRUSTED_OWNER_SIDS {
        let known: LocalBox<Sid> = text.parse().map_err(|_| refused())?;
        if owner == &*known {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Fails unless the file or folder behind `handle` belongs to a trusted owner. An owner that
/// cannot be read is refused.
pub(crate) fn check_handle(handle: &File) -> io::Result<()> {
    let descriptor = GetSecurityInfo(
        handle,
        SeObjectType::SE_FILE_OBJECT,
        SecurityInformation::Owner,
    )?;
    let owner = descriptor.owner().ok_or_else(refused)?;
    if is_trusted(owner)? {
        Ok(())
    } else {
        Err(refused())
    }
}

/// Fails unless the folder belongs to a trusted owner.
pub(crate) fn check_folder(dir: &Path) -> io::Result<()> {
    let handle = OpenOptions::new()
        .read(true)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS)
        .open(dir)?;
    check_handle(&handle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_folder_and_a_file_that_this_user_made_are_trusted() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("made");
        let file = File::create(&file_path).unwrap();
        assert!(check_folder(dir.path()).is_ok());
        assert!(check_handle(&file).is_ok());
    }

    #[test]
    fn a_folder_that_does_not_exist_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        assert!(check_folder(&dir.path().join("missing")).is_err());
    }

    #[test]
    fn an_account_that_is_not_known_is_not_trusted() {
        let stranger: LocalBox<Sid> = "S-1-5-21-1-2-3-1001".parse().unwrap();
        assert!(!is_trusted(&stranger).unwrap());
        let administrators: LocalBox<Sid> = "S-1-5-32-544".parse().unwrap();
        assert!(is_trusted(&administrators).unwrap());
    }
}
