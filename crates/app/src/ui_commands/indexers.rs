//! The command behind the indexer section of the settings.

use std::sync::Arc;

use codeskimr_application::access;
use codeskimr_application::error::CodedError;
use codeskimr_application::registry::{Command, CommandDescriptor, CommandId, Request};
use codeskimr_application::tools::{IndexerCheck, ToolsView};
use codeskimr_domain::Permission;

use super::consts::{INDEXERS_DESCRIPTION, INDEXERS_TITLE, TOOLS_INDEXERS};

/// Lists the indexers of level 2 on this computer, with where each was found, its version and the
/// error line of one that does not start. It needs no project and starts version queries only,
/// never project code (ADR 0033).
pub struct ListIndexers {
    pub(super) indexers: Arc<dyn IndexerCheck>,
}

impl Command for ListIndexers {
    type Input = ();
    type Output = ToolsView;

    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new(TOOLS_INDEXERS),
        title_key: INDEXERS_TITLE,
        description_key: INDEXERS_DESCRIPTION,
        shortcut: None,
        read_only: false,
        permission: Permission::Scan,
    };

    fn execute(&self, request: Request<'_, Self>) -> Result<ToolsView, CodedError> {
        access::require(request.principal, Permission::Scan)?;
        Ok(self.indexers.check())
    }
}
