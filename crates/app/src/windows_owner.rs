//! Who owns the launch folder and the folders above it on Windows (issue #372).
//!
//! The access rules (see `private_acl`) cannot take rights away from the owner of a folder that
//! someone else made beforehand, and the owner of a folder above can replace a part of the path.
//! So the owner of the launch folder, of every folder above it and of the launch file is read
//! from an open handle and must be the user, the administrators, the system or the service that
//! installs Windows (the usual owner of a drive root), like the Unix check accepts the user and
//! root.

use std::fs::{File, OpenOptions};
use std::io;
use std::os::windows::fs::OpenOptionsExt;
use std::path::Path;

use windows_permissions::constants::{SeObjectType, SecurityInformation};
use windows_permissions::utilities::current_process_sid;
use windows_permissions::wrappers::GetSecurityInfo;
use windows_permissions::{LocalBox, Sid};

use crate::consts::{
    FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT, LOG_LAUNCH_PARENT_REFUSED,
    READ_CONTROL, TRUSTED_OWNER_SIDS,
};

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

/// Fails unless the folder belongs to a trusted owner. A link is not followed: it would count with
/// the owner of the link itself and not of its target (links are refused elsewhere). The folder is
/// opened with the right to read its owner only, so a folder the user may pass through but not
/// list (a folder above the own one) can be checked too.
pub(crate) fn check_folder(dir: &Path) -> io::Result<()> {
    let handle = OpenOptions::new()
        .access_mode(READ_CONTROL)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
        .open(dir)?;
    check_handle(&handle)
}

/// Fails unless every folder above `dir`, up to the drive root, belongs to a trusted owner: the
/// owner of a folder can always take the right to replace what lies in it, and with that lead the
/// path of the launch folder somewhere else. `dir` must be a resolved path, so that no part of it
/// is a link. A volume that is mounted into a folder is not seen as such: the owner of that folder
/// is read, not the owner of the root of the mounted volume (only an administrator can mount one).
/// The first folder that is refused is written to the log with the kind of the failure (a folder
/// of someone else and a folder that cannot be opened both end here), because the error that the
/// user sees cannot name it.
pub(crate) fn check_parents(dir: &Path) -> io::Result<()> {
    for parent in dir.ancestors().skip(1) {
        if let Err(error) = check_folder(parent) {
            tracing::warn!(
                event = LOG_LAUNCH_PARENT_REFUSED,
                folder = %parent.display(),
                kind = ?error.kind(),
            );
            return Err(error);
        }
    }
    Ok(())
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
    fn the_folders_above_a_temporary_folder_are_trusted_up_to_the_drive_root() {
        let dir = tempfile::tempdir().unwrap();
        let resolved = std::fs::canonicalize(dir.path()).unwrap();
        assert!(resolved.ancestors().count() > 1);
        assert!(check_parents(&resolved).is_ok());
    }

    /// Gives `dir` to the group of all users, which is not a trusted owner. Only an elevated
    /// process may give a folder away (as on the CI runner); false if the system refused.
    fn give_away(dir: &Path) -> bool {
        std::process::Command::new("icacls")
            .arg(dir)
            .args(["/setowner", "*S-1-5-32-545"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .is_ok_and(|status| status.success())
    }

    #[test]
    fn a_folder_above_that_belongs_to_someone_else_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let outer = std::fs::canonicalize(dir.path()).unwrap().join("outer");
        let inner = outer.join("inner");
        std::fs::create_dir_all(&inner).unwrap();
        assert!(check_parents(&inner).is_ok());
        if !give_away(&outer) {
            // Without elevation the owner cannot be changed, so there is nothing to refuse on the
            // machine of a developer. The CI runner is elevated: there the test must not run empty.
            assert!(
                std::env::var_os("CI").is_none(),
                "the CI runner could not give the folder to another owner"
            );
            return;
        }
        assert!(check_folder(&outer).is_err());
        assert!(check_folder(&inner).is_ok());
        assert!(check_parents(&inner).is_err());
        assert!(check_parents(&outer).is_ok());
    }

    #[test]
    fn a_folder_that_does_not_exist_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        assert!(check_folder(&dir.path().join("missing")).is_err());
    }

    #[test]
    fn every_trusted_owner_is_a_valid_sid() {
        for text in TRUSTED_OWNER_SIDS {
            assert!(text.parse::<LocalBox<Sid>>().is_ok(), "{text}");
        }
    }

    #[test]
    fn an_account_that_is_not_known_is_not_trusted() {
        let stranger: LocalBox<Sid> = "S-1-5-21-1-2-3-1001".parse().unwrap();
        assert!(!is_trusted(&stranger).unwrap());
        let administrators: LocalBox<Sid> = "S-1-5-32-544".parse().unwrap();
        assert!(is_trusted(&administrators).unwrap());
    }
}
