//! The command behind the button "Show the places" of the tab "Modules" (ADR 0108): the lines
//! where something in one module refers to something in another, so that a dependency that ends a
//! ring can be followed to the code that has to change.

use std::sync::Arc;

use codeskimr_application::access;
use codeskimr_application::error::CodedError;
use codeskimr_application::module_places::{Between, PlacesOut, places_between};
use codeskimr_application::registry::{Command, CommandDescriptor, CommandId, Request};
use codeskimr_domain::Permission;
use serde::{Deserialize, Serialize};

use super::consts::{
    GRAPH_MODULE_PLACES, GRAPH_MODULE_PLACES_DESCRIPTION, GRAPH_MODULE_PLACES_TITLE,
};
use super::graph::{GraphPorts, newest};

/// The input of `graph.module_places`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ModulePlacesInput {
    /// Id of the project.
    pub project: u64,
    /// The depending module.
    pub from: String,
    /// The module it depends on.
    pub to: String,
    /// The depth the modules were merged to; zero (or left out) keeps all.
    #[serde(default)]
    pub depth: u32,
}

/// The answer of `graph.module_places`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct ModulePlacesOut {
    /// Id of the snapshot read, `None` before the first scan.
    pub snapshot: Option<u64>,
    /// The places, at most `graph.max_places`, and how many there are.
    #[serde(flatten)]
    pub found: PlacesOut,
}

/// Lists the places of one dependency between two modules.
pub struct GraphModulePlaces {
    pub(super) ports: Arc<GraphPorts>,
}

impl Command for GraphModulePlaces {
    type Input = ModulePlacesInput;
    type Output = ModulePlacesOut;

    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new(GRAPH_MODULE_PLACES),
        title_key: GRAPH_MODULE_PLACES_TITLE,
        description_key: GRAPH_MODULE_PLACES_DESCRIPTION,
        shortcut: None,
        read_only: true,
        permission: Permission::Read,
    };

    fn execute(&self, request: Request<'_, Self>) -> Result<ModulePlacesOut, CodedError> {
        access::require(request.principal, Permission::Read)?;
        let ports = &self.ports;
        let Some((_, snapshot)) = newest(ports, request.input.project)? else {
            return Ok(ModulePlacesOut::default());
        };
        let between = Between {
            from: &request.input.from,
            to: &request.input.to,
            depth: usize::try_from(request.input.depth).unwrap_or_default(),
        };
        let limit = usize::try_from(ports.look.max_places).unwrap_or(usize::MAX);
        Ok(ModulePlacesOut {
            snapshot: Some(snapshot.id.get()),
            found: places_between(ports.references.as_ref(), snapshot.id, &between, limit)?,
        })
    }
}
