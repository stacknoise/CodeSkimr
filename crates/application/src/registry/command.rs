//! What a command is.

use std::marker::PhantomData;

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
/// (issue #70). The request is also tied to its command type (`Request<'_, C>`), so a command
/// cannot hand the request it received on to another command (issue #344).
///
/// A crate outside the application layer cannot build one:
///
/// ```compile_fail,E0639
/// use codeskimr_application::registry::{Command, Request};
/// use codeskimr_domain::Principal;
///
/// fn forge<C: Command>(principal: &Principal, input: C::Input) {
///     let _ = Request::<C> { principal, input, command: std::marker::PhantomData };
/// }
/// ```
#[non_exhaustive]
pub struct Request<'a, C: Command> {
    /// The authenticated caller.
    pub principal: &'a Principal,
    /// The command input.
    pub input: C::Input,
    /// Ties the request to the command it was made for, so that a command cannot hand its own
    /// request on to another one (issue #344).
    command: PhantomData<fn() -> C>,
}

impl<'a, C: Command> Request<'a, C> {
    /// A request of `principal` for the command `C`; only the permission check builds one.
    pub(crate) const fn new(principal: &'a Principal, input: C::Input) -> Self {
        Self {
            principal,
            input,
            command: PhantomData,
        }
    }
}

/// A user action. It is implemented exactly once and offered on every surface.
///
/// The request a command receives belongs to that command; passing it on to another command does
/// not compile (issue #344):
///
/// ```compile_fail,E0308
/// use codeskimr_application::error::CodedError;
/// use codeskimr_application::registry::{Command, CommandDescriptor, CommandId, Request};
/// use codeskimr_domain::Permission;
///
/// const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
///     id: CommandId::new("x"),
///     title_key: "t",
///     description_key: "d",
///     shortcut: None,
///     read_only: true,
///     permission: Permission::Read,
/// };
/// struct Low;
/// struct High;
/// impl Command for High {
///     type Input = ();
///     type Output = ();
///     const DESCRIPTOR: CommandDescriptor = DESCRIPTOR;
///     fn execute(&self, _request: Request<'_, Self>) -> Result<(), CodedError> {
///         Ok(())
///     }
/// }
/// impl Command for Low {
///     type Input = ();
///     type Output = ();
///     const DESCRIPTOR: CommandDescriptor = DESCRIPTOR;
///     fn execute(&self, request: Request<'_, Self>) -> Result<(), CodedError> {
///         High.execute(request)
///     }
/// }
/// ```
pub trait Command: Sized + Send + Sync + 'static {
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
    fn execute(&self, request: Request<'_, Self>) -> Result<Self::Output, CodedError>;
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
        self.execute(Request::new(principal, input))
    }
}

impl<C: Command> CommandExt for C {}
