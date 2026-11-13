//! What a command is.

use codeskimr_domain::{Permission, Principal};

use crate::error::CodedError;

/// Stable identifier of a command, such as `scan.start`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CommandId(&'static str);

impl CommandId {
    /// Wraps the id constant of a command.
    #[must_use]
    pub const fn new(id: &'static str) -> Self {
        Self(id)
    }

    /// The id text.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

/// The metadata every surface needs about a command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandDescriptor {
    /// Unique id.
    pub id: CommandId,
    /// Fluent key of the title (a constant from `codeskimr_codes::i18n`).
    pub title_key: &'static str,
    /// Fluent key of the description.
    pub description_key: &'static str,
    /// Default keyboard shortcut, if any.
    pub shortcut: Option<&'static str>,
    /// Whether the command only reads; MCP exposes only these by default.
    pub read_only: bool,
    /// What a caller must hold; the registry checks it before the command runs.
    pub permission: Permission,
}

/// One call of a command: who calls and with what input.
#[derive(Debug, Clone, Copy)]
pub struct Request<'a, I> {
    /// The authenticated caller.
    pub principal: &'a Principal,
    /// The command input.
    pub input: I,
}

/// A user action. It is implemented exactly once and offered on every surface.
pub trait Command: Send + Sync + 'static {
    /// What the command needs.
    type Input;
    /// What the command returns.
    type Output;

    /// The metadata of this command.
    const DESCRIPTOR: CommandDescriptor;

    /// Runs the command. The registry has already checked `DESCRIPTOR.permission`; a command that
    /// needs more (a permission on one project) checks that itself with `request.principal`.
    ///
    /// # Errors
    /// A coded failure; each command documents its own codes.
    fn execute(&self, request: Request<'_, Self::Input>) -> Result<Self::Output, CodedError>;
}
