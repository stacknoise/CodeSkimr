//! What a command is.

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
}

/// A user action. It is implemented exactly once and offered on every surface.
pub trait Command: Send + Sync + 'static {
    /// What the command needs.
    type Input;
    /// What the command returns.
    type Output;

    /// The metadata of this command.
    const DESCRIPTOR: CommandDescriptor;

    /// Runs the command.
    ///
    /// # Errors
    /// A coded failure; each command documents its own codes.
    fn execute(&self, input: Self::Input) -> Result<Self::Output, CodedError>;
}
