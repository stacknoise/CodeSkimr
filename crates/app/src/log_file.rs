//! The log file in the data folder: it rotates by size and keeps a few old files (ADR 0115).
//!
//! Every line that reaches it has passed the redaction already (`log_writer`).

use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use crate::consts::{
    LOG_DIR_MODE, LOG_FILE_MAX_BYTES, LOG_FILE_MODE, LOG_FILES_KEPT, LOG_OLD_SEPARATOR,
    LOG_TAIL_BYTES,
};

/// An open log file that knows its size.
pub struct LogFile {
    path: PathBuf,
    file: File,
    size: u64,
    max_bytes: u64,
}

/// The name of the `index`-th old file of `path`: `codeskimr.log.1`.
fn old_name(path: &Path, index: u32) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(LOG_OLD_SEPARATOR);
    name.push(index.to_string());
    PathBuf::from(name)
}

/// Opens `path` for appending. A link or anything but a plain file of that name is refused, so
/// that the log never writes through something that points elsewhere. On Unix the open itself
/// does not follow a link (`O_NOFOLLOW`) and the file is made readable by its owner only, also if
/// it existed with wider rights.
fn open_append(path: &Path) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.create(true).append(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(LOG_FILE_MODE).custom_flags(libc::O_NOFOLLOW);
    }
    #[cfg(not(unix))]
    if fs::symlink_metadata(path).is_ok_and(|meta| !meta.is_file()) {
        return Err(io::Error::from(io::ErrorKind::InvalidInput));
    }
    let file = options.open(path)?;
    if !file.metadata()?.is_file() {
        return Err(io::Error::from(io::ErrorKind::InvalidInput));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        file.set_permissions(fs::Permissions::from_mode(LOG_FILE_MODE))?;
    }
    Ok(file)
}

/// Opens `path` with `options` without following a link on Unix.
fn open_plain(options: &mut OpenOptions, path: &Path) -> io::Result<File> {
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::custom_flags(options, libc::O_NOFOLLOW);
    options.open(path)
}

/// Creates the folder of the log; on Unix only its owner may enter it.
fn create_folder(folder: &Path) -> io::Result<()> {
    let mut builder = fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    std::os::unix::fs::DirBuilderExt::mode(&mut builder, LOG_DIR_MODE);
    builder.create(folder)
}

impl LogFile {
    /// Opens the file at `path` and creates its folder.
    ///
    /// # Errors
    /// The folder or the file cannot be created, or `path` is a link or a folder.
    pub fn open(path: &Path) -> io::Result<Self> {
        Self::open_with_limit(path, LOG_FILE_MAX_BYTES)
    }

    /// Like [`Self::open`] with another size at which the file rotates.
    pub(crate) fn open_with_limit(path: &Path, max_bytes: u64) -> io::Result<Self> {
        if let Some(parent) = path.parent() {
            create_folder(parent)?;
        }
        let file = open_append(path)?;
        let size = file.metadata()?.len();
        Ok(Self {
            path: path.to_path_buf(),
            file,
            size,
            max_bytes,
        })
    }

    /// The path of the active file.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The size of the active file in bytes.
    #[must_use]
    pub const fn size(&self) -> u64 {
        self.size
    }

    /// Appends `bytes`; a file that would pass the limit rotates first.
    ///
    /// # Errors
    /// The file cannot be rotated or written.
    pub fn write(&mut self, bytes: &[u8]) -> io::Result<()> {
        let length = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
        if self.size > 0 && self.size.saturating_add(length) > self.max_bytes {
            self.rotate()?;
        }
        self.file.write_all(bytes)?;
        self.size = self.size.saturating_add(length);
        Ok(())
    }

    /// Moves every file one place up, drops the oldest and starts a new active file.
    fn rotate(&mut self) -> io::Result<()> {
        let last = LOG_FILES_KEPT.saturating_sub(1);
        let _ = fs::remove_file(old_name(&self.path, last));
        for index in (1..last).rev() {
            let _ = fs::rename(old_name(&self.path, index), old_name(&self.path, index + 1));
        }
        let _ = fs::rename(&self.path, old_name(&self.path, 1));
        self.file = open_append(&self.path)?;
        self.size = 0;
        Ok(())
    }

    /// Empties the active file and removes the old ones.
    ///
    /// # Errors
    /// The active file cannot be emptied.
    pub fn clear(&mut self) -> io::Result<()> {
        for index in 1..LOG_FILES_KEPT {
            let _ = fs::remove_file(old_name(&self.path, index));
        }
        // The append handle has no right to cut the file on Windows, so a handle of its own does.
        OpenOptions::new()
            .write(true)
            .open(&self.path)?
            .set_len(0)?;
        self.size = 0;
        Ok(())
    }

    /// The last `lines` lines of the active file, oldest first. Only the end of the file is read.
    ///
    /// # Errors
    /// The file cannot be read.
    pub fn tail(&self, lines: usize) -> io::Result<Vec<String>> {
        let mut file = open_plain(OpenOptions::new().read(true), &self.path)?;
        let length = file.metadata()?.len();
        let start = length.saturating_sub(LOG_TAIL_BYTES);
        file.seek(SeekFrom::Start(start))?;
        let mut bytes = Vec::new();
        file.take(LOG_TAIL_BYTES).read_to_end(&mut bytes)?;
        let text = String::from_utf8_lossy(&bytes);
        let mut all: Vec<&str> = text.lines().collect();
        if start > 0 && !all.is_empty() {
            // The first line may start before the part that was read.
            all.remove(0);
        }
        let from = all.len().saturating_sub(lines);
        let last = all.get(from..).unwrap_or_default();
        Ok(last.iter().map(|line| (*line).to_owned()).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::{LogFile, old_name};

    #[test]
    fn lines_are_appended_and_the_tail_gives_the_last_ones() {
        let dir = tempfile::tempdir().unwrap();
        let mut log = LogFile::open(&dir.path().join("logs").join("a.log")).unwrap();
        for number in 0..5 {
            log.write(format!("line {number}\n").as_bytes()).unwrap();
        }
        assert_eq!(log.tail(2).unwrap(), ["line 3", "line 4"]);
        assert_eq!(log.size(), 35);
    }

    #[test]
    fn a_full_file_rotates_and_only_a_few_old_files_stay() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.log");
        let mut log = LogFile::open_with_limit(&path, 10).unwrap();
        for number in 0..12 {
            log.write(format!("line {number:02}\n").as_bytes()).unwrap();
        }
        assert_eq!(log.tail(10).unwrap(), ["line 11"]);
        assert!(old_name(&path, 1).exists());
        assert!(!old_name(&path, crate::consts::LOG_FILES_KEPT).exists());
        assert!(
            std::fs::read_to_string(old_name(&path, 1))
                .unwrap()
                .contains("line 10")
        );
    }

    #[test]
    fn clearing_empties_the_active_file_and_removes_the_old_ones() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.log");
        let mut log = LogFile::open_with_limit(&path, 10).unwrap();
        for number in 0..4 {
            log.write(format!("line {number}\n").as_bytes()).unwrap();
        }
        log.clear().unwrap();
        assert_eq!(log.size(), 0);
        assert_eq!(log.tail(5).unwrap(), Vec::<String>::new());
        log.write(b"after\n").unwrap();
        assert_eq!(log.tail(5).unwrap(), ["after"]);
        assert!(!old_name(&path, 1).exists());
    }

    #[cfg(unix)]
    #[test]
    fn a_link_is_not_written_through() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("target");
        std::fs::write(&target, "x").unwrap();
        let link = dir.path().join("a.log");
        std::os::unix::fs::symlink(&target, &link).unwrap();
        assert!(LogFile::open(&link).is_err());
    }
}
