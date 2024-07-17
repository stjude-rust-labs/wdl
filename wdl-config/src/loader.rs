use std::collections::VecDeque;
use std::convert::Infallible;
use std::path::PathBuf;

use config::ConfigError;
use config::Environment;
use config::File;

use crate::providers::EnvProvider;
use crate::providers::FileProvider;
use crate::BoxedProvider;
use crate::Config;
use crate::Provider;
use crate::CONFIG_SEARCH_PATHS;

#[derive(Debug)]
pub enum Error {
    /// An error from the `config` crate.
    Config(ConfigError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Config(err) => write!(f, "`config` error: {err}"),
        }
    }
}

impl std::error::Error for Error {}

/// A [`Result`](std::result::Result) with an [`Error`].
pub type Result<T> = std::result::Result<T, Error>;

pub struct Loader(VecDeque<BoxedProvider>);

impl Loader {
    /// Creates an empty [`Loader`].
    pub fn empty() -> Self {
        Self(VecDeque::new())
    }

    /// Adds the default configuration to the front of the provider stack.
    pub fn with_default_configuration(mut self) -> Self {
        // NOTE: default configuration should always be the first provider evaluated.
        self.0.push_front(Config::default().into());
        self
    }

    /// Adds a file to the search path of the [`Loader`].
    ///
    /// Note that the file is not required to be present.
    pub fn add_optional_file(mut self, path: PathBuf) -> Self {
        self.0.push_back(FileProvider::optional(path).into());
        self
    }

    /// Adds a file to the search path of the [`Loader`].
    ///
    /// Note that the file is required to be present.
    pub fn add_required_file(mut self, path: PathBuf) -> Self {
        self.0.push_back(FileProvider::required(path).into());
        self
    }

    /// Adds the default search paths to the [`Loader`].
    pub fn with_default_search_paths(mut self) -> Self {
        for path in CONFIG_SEARCH_PATHS.clone().into_iter() {
            self = self.add_optional_file(path);
        }

        self
    }

    /// Adds a new environment prefix to the [`Loader`].
    pub fn add_env_prefix(mut self, prefix: &str) -> Self {
        self.0.push_back(EnvProvider::new(prefix).into());
        self
    }

    /// Adds the default environment prefix to the [`Loader`].
    pub fn with_default_env_prefix(mut self) -> Self {
        self.0.push_back(EnvProvider::default().into());
        self
    }

    /// Gets a reference to the inner [`ConfigBuilder`].
    pub fn inner(&self) -> &VecDeque<BoxedProvider> {
        &self.0
    }

    /// Consumes `self` and returns the inner [`ConfigBuilder`].
    pub fn into_inner(self) -> VecDeque<BoxedProvider> {
        self.0
    }

    /// Consumes `self` and attempts to load the [`Config`].
    pub fn try_load(self) -> std::result::Result<Config, Box<dyn std::error::Error>> {
        for provider in self.0 {
            let config = provider.provide().map_err(|e| );
        }

        self.0
            .build()
            .map_err(Error::Config)?
            .try_deserialize()
            .map_err(Error::Config)
    }
}

impl Default for Loader {
    fn default() -> Self {
        Self::empty()
            .with_default_search_paths()
            .with_default_env_prefix()
    }
}

#[cfg(test)]
mod tests {
    use crate::Loader;

    #[test]
    fn an_empty_loader_unwraps() {
        Loader::empty();
    }
}
