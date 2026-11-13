//! The platforms whose OAuth device flow the program can use (blueprint 12.2, point 2).

use crate::consts::{
    GITHUB_DEVICE_PATH, GITHUB_HOST, GITHUB_SCOPE, GITHUB_TOKEN_PATH, GITHUB_USER,
    GITLAB_DEVICE_PATH, GITLAB_HOST, GITLAB_SCOPE, GITLAB_TOKEN_PATH, GITLAB_USER, PATH_SEPARATOR,
};
use crate::device_flow::DeviceFlow;
use crate::device_flow::DeviceFlowError;

/// A platform that hosts Git repositories and knows the device flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    /// GitHub, or GitHub Enterprise Server with a host.
    GitHub,
    /// GitLab, or a self-managed GitLab with a host.
    GitLab,
}

impl Provider {
    /// The flow of this platform for the application `client_id`. `host` replaces the public
    /// host for a self-managed installation (`https://git.example.com`).
    ///
    /// # Errors
    /// `NET-601` if the host is not an HTTPS address.
    pub fn flow(self, client_id: &str, host: Option<&str>) -> Result<DeviceFlow, DeviceFlowError> {
        let (public_host, device, token, scope, user) = match self {
            Self::GitHub => (
                GITHUB_HOST,
                GITHUB_DEVICE_PATH,
                GITHUB_TOKEN_PATH,
                GITHUB_SCOPE,
                GITHUB_USER,
            ),
            Self::GitLab => (
                GITLAB_HOST,
                GITLAB_DEVICE_PATH,
                GITLAB_TOKEN_PATH,
                GITLAB_SCOPE,
                GITLAB_USER,
            ),
        };
        let base = host.unwrap_or(public_host).trim_end_matches(PATH_SEPARATOR);
        DeviceFlow::new(
            format!("{base}{device}"),
            format!("{base}{token}"),
            client_id,
            scope,
            user,
        )
    }
}
