//! Configuration for the Crankshaft backend.

use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;

/// Represents configuration for the Crankshaft backend.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct CrankshaftBackendConfig {
    /// The name of the Crankshaft backend to use.
    ///
    /// If no name is provided, the first backend will be used.
    pub backend: Option<String>,
}

impl CrankshaftBackendConfig {
    /// Validates the Crankshaft backend configuration.
    pub fn validate(&self) -> Result<()> {
        Ok(())
    }
}
