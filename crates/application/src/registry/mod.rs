//! The command registry (blueprint section 10.1): every user-facing action is registered once and
//! feeds the quick search, keyboard shortcuts, CLI, REST and MCP.
//!
//! The registry stores the descriptor of each command, which the generators read, and the typed
//! handler, which the driving adapters fetch by command type. Turning wire formats into `Input`
//! and `Output` is the job of each adapter.
//!
//! Handlers are not handed out. Every call goes through [`Registry::execute`], which checks the
//! permission the command declares before it runs, so no surface can skip the check.

mod command;
mod error;

pub use command::{Command, CommandDescriptor, CommandId, Request};
pub use error::RegistryError;

use std::any::Any;
use std::collections::BTreeMap;
use std::sync::Arc;

use codeskimr_codes::ErrorCode;
use codeskimr_domain::{Permission, Principal};

use crate::access;
use crate::error::CodedError;

struct Entry {
    descriptor: CommandDescriptor,
    handler: Arc<dyn Any + Send + Sync>,
}

/// Holds all commands, ordered by id.
#[derive(Default)]
pub struct Registry {
    entries: BTreeMap<CommandId, Entry>,
}

impl Registry {
    /// Creates an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a command under the id of its descriptor.
    ///
    /// # Errors
    /// `RegistryError::DuplicateId` if the id is taken, `RegistryError::PermissionMismatch` if
    /// `read_only` and the declared permission contradict each other; nothing changes then.
    pub fn register<C: Command>(&mut self, command: C) -> Result<(), RegistryError> {
        let descriptor = C::DESCRIPTOR;
        let reads = matches!(
            descriptor.permission,
            Permission::Read | Permission::ReadSource
        );
        if descriptor.read_only != reads {
            return Err(RegistryError::PermissionMismatch(descriptor.id));
        }
        if self.entries.contains_key(&descriptor.id) {
            return Err(RegistryError::DuplicateId(descriptor.id));
        }
        let handler: Arc<dyn Any + Send + Sync> = Arc::new(command);
        self.entries.insert(
            descriptor.id,
            Entry {
                descriptor,
                handler,
            },
        );
        Ok(())
    }

    /// All descriptors in id order, for generating menus, CLI, REST and MCP.
    pub fn descriptors(&self) -> impl Iterator<Item = &CommandDescriptor> {
        self.entries.values().map(|entry| &entry.descriptor)
    }

    /// The descriptor registered under `id`.
    #[must_use]
    pub fn descriptor(&self, id: CommandId) -> Option<&CommandDescriptor> {
        self.entries.get(&id).map(|entry| &entry.descriptor)
    }

    /// Runs command `C` for `principal`, after checking the permission it declares.
    ///
    /// # Errors
    /// `AUTH-401` if the principal lacks the permission, `APP-901` if `C` is not registered
    /// (a wiring bug), otherwise the code of the command's own failure.
    pub fn execute<C: Command>(
        &self,
        principal: &Principal,
        input: C::Input,
    ) -> Result<C::Output, CodedError> {
        access::require(principal, C::DESCRIPTOR.permission)?;
        let command = self
            .entries
            .get(&C::DESCRIPTOR.id)
            .and_then(|entry| entry.handler.downcast_ref::<C>())
            .ok_or_else(|| CodedError::new(ErrorCode::UnexpectedInternalError))?;
        command.execute(Request { principal, input })
    }
}
