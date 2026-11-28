//! The commands behind the tab Diagnosis of the settings: the log level that can be changed while
//! the program runs, and the log file in the data folder (ADR 0115).
//!
//! No command takes a path. The file and its folder are built from the data folder. The log holds
//! paths and names of projects, so reading it needs the token like reading source text does
//! (ADR 0035, ADR 0115).

use std::io::ErrorKind;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use codeskimr_application::{
    access,
    error::CodedError,
    registry::{Command, CommandDescriptor, CommandId, Request},
};
use codeskimr_codes::ErrorCode;
use codeskimr_domain::Permission;
use serde::{Deserialize, Serialize};
use tracing_subscriber::filter::LevelFilter;

use super::consts::{
    LOG_CLEAR, LOG_CLEAR_DESCRIPTION, LOG_CLEAR_TITLE, LOG_OPEN_FOLDER,
    LOG_OPEN_FOLDER_DESCRIPTION, LOG_OPEN_FOLDER_TITLE, LOG_SET_LEVEL, LOG_SET_LEVEL_DESCRIPTION,
    LOG_SET_LEVEL_TITLE, LOG_STATUS, LOG_STATUS_DESCRIPTION, LOG_STATUS_TITLE, LOG_TAIL,
    LOG_TAIL_DESCRIPTION, LOG_TAIL_TITLE,
};
use crate::consts::{LOG_FILE_MAX_BYTES, LOG_FILES_KEPT, LOG_LEVELS, LOG_TAIL_MAX_LINES};
use crate::file_opener::FileOpener;
use crate::log_control;

/// What the tab Diagnosis shows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LogStatus {
    /// The level that counts now: `off`, `warn`, `info`, `debug` or `trace`.
    pub level: String,
    /// The level that counts when nothing was chosen.
    pub default_level: String,
    /// The levels that can be chosen, from silent to everything.
    pub levels: Vec<String>,
    /// Whether the level can be changed at all (it cannot if another subscriber owns the log).
    pub adjustable: bool,
    /// The log file, or none if it could not be opened.
    pub file: Option<String>,
    /// The size of the active file in bytes.
    pub size_bytes: u64,
    /// The size at which the file is renamed and a new one starts.
    pub max_file_bytes: u64,
    /// How many files are kept, the active one included.
    pub files_kept: u32,
    /// How many seconds the chosen level lasts, if it is not the default.
    pub resets_in_seconds: Option<u64>,
    /// Whether a program can open the folder on this machine.
    pub can_open_folder: bool,
}

/// The words of a level in the wire form.
fn word(level: LevelFilter) -> String {
    level.to_string().to_lowercase()
}

/// The state of the log, with `can_open_folder` from the opener.
fn status(opener: &dyn FileOpener) -> LogStatus {
    let info = log_control::file_info();
    LogStatus {
        level: word(log_control::level()),
        default_level: word(log_control::default_level()),
        levels: LOG_LEVELS.iter().map(|level| word(*level)).collect(),
        adjustable: log_control::is_installed(),
        file: info
            .as_ref()
            .map(|(path, _)| path.to_string_lossy().into_owned()),
        size_bytes: info.map_or(0, |(_, size)| size),
        max_file_bytes: LOG_FILE_MAX_BYTES,
        files_kept: LOG_FILES_KEPT,
        resets_in_seconds: log_control::resets_in().map(|left| left.as_secs()),
        can_open_folder: opener.available(),
    }
}

/// The error of a log file that is missing or cannot be used.
fn unavailable() -> CodedError {
    CodedError::new(ErrorCode::LogFileUnavailable)
}

/// What the commands share.
pub struct LogPorts {
    /// Opens the folder of the log for the person at this machine.
    pub(super) opener: Arc<dyn FileOpener>,
}

/// Shows the log level and the log file.
pub struct LogStatusCommand {
    pub(super) ports: Arc<LogPorts>,
}

impl Command for LogStatusCommand {
    type Input = ();
    type Output = LogStatus;

    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new(LOG_STATUS),
        title_key: LOG_STATUS_TITLE,
        description_key: LOG_STATUS_DESCRIPTION,
        shortcut: None,
        read_only: true,
        permission: Permission::ReadSource,
    };

    fn execute(&self, request: Request<'_, Self>) -> Result<LogStatus, CodedError> {
        access::require(request.principal, Permission::ReadSource)?;
        Ok(status(self.ports.opener.as_ref()))
    }
}

/// The input of `log.set_level`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LevelInput {
    /// `off`, `warn`, `info`, `debug` or `trace`.
    pub level: String,
}

/// Changes the log level at once. A level other than the default ends after 24 hours.
pub struct LogSetLevel {
    pub(super) ports: Arc<LogPorts>,
}

impl Command for LogSetLevel {
    type Input = LevelInput;
    type Output = LogStatus;

    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new(LOG_SET_LEVEL),
        title_key: LOG_SET_LEVEL_TITLE,
        description_key: LOG_SET_LEVEL_DESCRIPTION,
        shortcut: None,
        read_only: false,
        permission: Permission::Manage,
    };

    /// # Errors
    /// `AUTH-401` without the right to manage; `API-101` for a word that is not one of the levels or if another subscriber owns the log.
    fn execute(&self, request: Request<'_, Self>) -> Result<LogStatus, CodedError> {
        access::require(request.principal, Permission::Manage)?;
        let level = LevelFilter::from_str(&request.input.level)
            .ok()
            .filter(|level| LOG_LEVELS.contains(level))
            .ok_or_else(|| CodedError::new(ErrorCode::RequestInvalid))?;
        if !log_control::set_level(level) {
            return Err(CodedError::new(ErrorCode::RequestInvalid));
        }
        Ok(status(self.ports.opener.as_ref()))
    }
}

/// The input of `log.tail`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TailInput {
    /// How many of the last lines; at most 2000.
    pub lines: usize,
}

/// The last lines of the log file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LogLines {
    /// The lines, oldest first.
    pub lines: Vec<String>,
}

/// Reads the end of the log file.
pub struct LogTail;

impl Command for LogTail {
    type Input = TailInput;
    type Output = LogLines;

    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new(LOG_TAIL),
        title_key: LOG_TAIL_TITLE,
        description_key: LOG_TAIL_DESCRIPTION,
        shortcut: None,
        read_only: true,
        permission: Permission::ReadSource,
    };

    /// # Errors
    /// `AUTH-401` without the right to read source text (the log holds paths and names, so the
    /// token is needed like for the source view); `APP-607` if there is no log file or it cannot
    /// be read.
    fn execute(&self, request: Request<'_, Self>) -> Result<LogLines, CodedError> {
        access::require(request.principal, Permission::ReadSource)?;
        let wanted = request.input.lines.min(LOG_TAIL_MAX_LINES);
        match log_control::with_file(|file| file.tail(wanted)) {
            Some(Ok(lines)) => Ok(LogLines { lines }),
            Some(Err(error)) if error.kind() == ErrorKind::NotFound => {
                Ok(LogLines { lines: Vec::new() })
            }
            Some(Err(_)) | None => Err(unavailable()),
        }
    }
}

/// Empties the log file and removes the old ones.
pub struct LogClear {
    pub(super) ports: Arc<LogPorts>,
}

impl Command for LogClear {
    type Input = ();
    type Output = LogStatus;

    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new(LOG_CLEAR),
        title_key: LOG_CLEAR_TITLE,
        description_key: LOG_CLEAR_DESCRIPTION,
        shortcut: None,
        read_only: false,
        permission: Permission::Manage,
    };

    /// # Errors
    /// `AUTH-401` without the right to manage; `APP-607` if there is no file or it cannot be
    /// emptied.
    fn execute(&self, request: Request<'_, Self>) -> Result<LogStatus, CodedError> {
        access::require(request.principal, Permission::Manage)?;
        match log_control::with_file(crate::log_file::LogFile::clear) {
            Some(Ok(())) => Ok(status(self.ports.opener.as_ref())),
            Some(Err(_)) | None => Err(unavailable()),
        }
    }
}

/// Opens the folder of the log files with the program of the operating system.
pub struct LogOpenFolder {
    pub(super) ports: Arc<LogPorts>,
}

impl Command for LogOpenFolder {
    type Input = ();
    type Output = ();

    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new(LOG_OPEN_FOLDER),
        title_key: LOG_OPEN_FOLDER_TITLE,
        description_key: LOG_OPEN_FOLDER_DESCRIPTION,
        shortcut: None,
        read_only: false,
        permission: Permission::Manage,
    };

    /// # Errors
    /// `AUTH-401` without the right to manage; `APP-606` if nobody sits at this machine or the
    /// system starts no program; `APP-607` if there is no log file.
    fn execute(&self, request: Request<'_, Self>) -> Result<(), CodedError> {
        access::require(request.principal, Permission::Manage)?;
        let folder = log_control::file_info()
            .and_then(|(path, _)| path.parent().map(PathBuf::from))
            .ok_or_else(unavailable)?;
        if !self.ports.opener.available() {
            return Err(CodedError::new(ErrorCode::LogFolderOpenFailed));
        }
        self.ports
            .opener
            .open(&folder)
            .map_err(|_| CodedError::new(ErrorCode::LogFolderOpenFailed))
    }
}
