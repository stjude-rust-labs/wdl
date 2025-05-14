//! Configuration for the Docker backend.

use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;

/// Gets the default value for the docker `cleanup` field.
const fn cleanup_default() -> bool {
    true
}

/// Represents configuration for the Docker backend.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct DockerBackendConfig {
    /// Whether or not to remove a task's container after the task completes.
    ///
    /// Defaults to `true`.
    #[serde(default = "cleanup_default")]
    pub cleanup: bool,
}

impl DockerBackendConfig {
    /// Validates the Docker backend configuration.
    pub fn validate(&self) -> Result<()> {
        Ok(())
    }
}

impl Default for DockerBackendConfig {
    fn default() -> Self {
        Self { cleanup: true }
    }
}
