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
///
/// Only this crate can build a request (the struct is `#[non_exhaustive]`), and the only ways to
/// get one are [`CommandExt::run`] and `Registry::execute`, which check the permission the command
/// declares first. A holder of a command value therefore cannot reach `execute` around the check
/// (issue #70). A command implementation could still hand its own request on to another command
/// with the same input type; that gap is tracked in issue #344.
///
/// A crate outside the application layer cannot build one:
///
/// ```compile_fail,E0639
/// use codeskimr_application::registry::Request;
/// use codeskimr_domain::Principal;
///
/// fn forge(principal: &Principal) {
///     let _ = Request { principal, input: () };
/// }
/// ```
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
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

/// Runs a command for a caller after the permission check. It is implemented for every command
/// and cannot be overridden, so no command can offer a way around the check.
pub trait CommandExt: Command + Sized {
    /// Checks that `principal` holds the permission of `DESCRIPTOR` and runs the command.
    ///
    /// # Errors
    /// `AUTH-401` if the permission is missing, otherwise the command's own failure.
    fn run(&self, principal: &Principal, input: Self::Input) -> Result<Self::Output, CodedError> {
        crate::access::require(principal, Self::DESCRIPTOR.permission)?;
        self.execute(Request { principal, input })
    }
}

impl<C: Command> CommandExt for C {}
