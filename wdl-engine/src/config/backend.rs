//! Configuration for different backends.

use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;

mod crankshaft;
mod local;

pub use crankshaft::CrankshaftBackendConfig;
pub use local::LocalBackendConfig;

/// Represents supported task execution backends.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum BackendConfig {
    /// Use the local task execution backend.
    Local(LocalBackendConfig),
    /// Use the Crankshaft task execution backend.
    Crankshaft(CrankshaftBackendConfig),
}

impl Default for BackendConfig {
    fn default() -> Self {
        Self::Crankshaft(Default::default())
    }
}

impl BackendConfig {
    /// Validates the backend configuration.
    pub fn validate(&self) -> Result<()> {
        match self {
            Self::Local(config) => config.validate(),
            Self::Crankshaft(config) => config.validate(),
        }
    }
}
