//! The commands the server offers over REST (blueprint 10.1): one route per registered command,
//! generated from the registry, with no logic of its own.

use std::collections::BTreeMap;
use std::sync::Arc;

use codeskimr_application::access;
use codeskimr_application::error::CodedError;
use codeskimr_application::registry::{Command, CommandDescriptor, Registry};
use codeskimr_codes::ErrorCode;
use codeskimr_domain::Principal;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

/// Turns a JSON input into a call of one command and its output into JSON.
type Call = dyn Fn(&Principal, Value) -> Result<Value, CodedError> + Send + Sync;

struct Route {
    descriptor: CommandDescriptor,
    call: Arc<Call>,
}

/// What a caller learns about a command it may run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CommandInfo {
    /// Id of the command, the last segment of its route.
    pub id: &'static str,
    /// Fluent key of the title.
    pub title_key: &'static str,
    /// Fluent key of the description.
    pub description_key: &'static str,
    /// Default shortcut, if any.
    pub shortcut: Option<&'static str>,
    /// Whether the command only reads. Only the others need the installation token.
    pub read_only: bool,
}

/// The commands that are reachable over REST. Commands are added by the composition root, which
/// owns the registry; a command that is registered but not exposed here is not reachable.
pub struct CommandApi {
    registry: Arc<Registry>,
    routes: BTreeMap<&'static str, Route>,
}

impl CommandApi {
    /// An API over `registry` without any exposed command.
    #[must_use]
    pub fn new(registry: Arc<Registry>) -> Self {
        Self {
            registry,
            routes: BTreeMap::new(),
        }
    }

    /// Exposes command `C`, whose input and output have a JSON form.
    ///
    /// # Errors
    /// `APP-901` if `C` is not registered or is exposed twice (wiring bugs).
    pub fn expose<C>(&mut self) -> Result<(), CodedError>
    where
        C: Command,
        C::Input: DeserializeOwned,
        C::Output: Serialize,
    {
        let descriptor = C::DESCRIPTOR;
        let wiring_bug = || CodedError::new(ErrorCode::UnexpectedInternalError);
        if self.registry.descriptor(descriptor.id).is_none()
            || self.routes.contains_key(descriptor.id.as_str())
        {
            return Err(wiring_bug());
        }
        let registry = Arc::clone(&self.registry);
        let call = move |principal: &Principal, input: Value| {
            let input = serde_json::from_value(input)
                .map_err(|_| CodedError::new(ErrorCode::RequestInvalid))?;
            let output = registry.execute::<C>(principal, input)?;
            serde_json::to_value(output).map_err(|_| wiring_bug())
        };
        self.routes.insert(
            descriptor.id.as_str(),
            Route {
                descriptor,
                call: Arc::new(call),
            },
        );
        Ok(())
    }

    /// The exposed commands that `principal` may run, in id order.
    #[must_use]
    pub fn list(&self, principal: &Principal) -> Vec<CommandInfo> {
        self.routes
            .values()
            .map(|route| &route.descriptor)
            .filter(|descriptor| principal.holds(descriptor.permission))
            .map(|descriptor| CommandInfo {
                id: descriptor.id.as_str(),
                title_key: descriptor.title_key,
                description_key: descriptor.description_key,
                shortcut: descriptor.shortcut,
                read_only: descriptor.read_only,
            })
            .collect()
    }

    /// The descriptor of the exposed command `id`.
    #[must_use]
    pub fn descriptor(&self, id: &str) -> Option<&CommandDescriptor> {
        self.routes.get(id).map(|route| &route.descriptor)
    }

    /// Runs command `id` for `principal`. The permission is checked before the input is read, so
    /// a caller without it learns nothing about the input either.
    ///
    /// The command runs on a thread of the blocking pool: commands are synchronous and may take
    /// long, and must never hold up the async runtime.
    ///
    /// # Errors
    /// - `API-201` if no such command is exposed.
    /// - `AUTH-401` if the principal lacks the permission.
    /// - `API-101` if the input does not fit the command.
    /// - `APP-901` if the command panicked, otherwise the code of the command's own failure.
    pub async fn run(
        self: &Arc<Self>,
        principal: Principal,
        id: &str,
        input: Value,
    ) -> Result<Value, CodedError> {
        let route = self
            .routes
            .get(id)
            .ok_or_else(|| CodedError::new(ErrorCode::CommandNotFound))?;
        access::require(&principal, route.descriptor.permission)?;
        let call = Arc::clone(&route.call);
        tokio::task::spawn_blocking(move || call(&principal, input))
            .await
            .map_err(|_| CodedError::new(ErrorCode::UnexpectedInternalError))?
    }
}
