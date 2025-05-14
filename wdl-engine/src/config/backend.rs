//! Configuration for different backends.

use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;

mod docker;
mod local;

pub use docker::DockerBackendConfig;
pub use local::LocalBackendConfig;

/// Represents supported task execution backends.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum BackendConfig {
    /// Use the local task execution backend.
    Local(LocalBackendConfig),
    /// Use the Docker task execution backend.
    Docker(DockerBackendConfig),
}

impl Default for BackendConfig {
    fn default() -> Self {
        Self::Docker(Default::default())
    }
}

impl BackendConfig {
    /// Validates the backend configuration.
    pub fn validate(&self) -> Result<()> {
        match self {
            Self::Local(config) => config.validate(),
            Self::Docker(config) => config.validate(),
        }
    }
}
