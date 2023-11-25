//! An inner representation for the configuration object.
//!
//! This struct holds the configuration values.

use indexmap::IndexMap;
use indexmap::IndexSet;
use serde::Deserialize;
use serde::Serialize;
use serde_with::serde_as;

use wdl_core as core;

mod repr;

pub use repr::ErrorsAsReprs;

use crate::document;
use crate::repository;

/// Parsing errors as [`String`]s associated with a [document
/// identifier](document::Identifier).
pub type Errors = IndexMap<document::Identifier, String>;

/// A unique set of [repository identifiers](repository::Identifier).
pub type Repositories = IndexSet<repository::Identifier>;

/// The inner configuration object for a [`Config`](super::Config).
///
/// This object stores the actual configuration values for this subcommand.
#[serde_as]
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Inner {
    /// The WDL version.
    version: core::Version,

    /// The repositories.
    #[serde(default)]
    repositories: Repositories,

    /// The ignored errors.
    #[serde_as(as = "ErrorsAsReprs")]
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    ignored_errors: Errors,
}

impl Inner {
    /// Gets the [`Version`](core::Version) for this [`Inner`] by reference.
    ///
    /// # Examples
    ///
    /// ```
    /// use wdl_core as core;
    /// use wdl_gauntlet as gauntlet;
    ///
    /// use gauntlet::config::Inner;
    ///
    /// let config = r#"version = "v1"
    ///
    /// [[repositories]]
    /// organization = "Foo"
    /// name = "Bar""#;
    ///
    /// let inner: Inner = toml::from_str(&config).unwrap();
    /// assert_eq!(inner.version(), &core::Version::V1);
    /// ```
    pub fn version(&self) -> &core::Version {
        &self.version
    }

    /// Gets the [`Repositories`] for this [`Inner`] by reference.
    ///
    /// # Examples
    ///
    /// ```
    /// use wdl_core as core;
    /// use wdl_gauntlet as gauntlet;
    ///
    /// use gauntlet::config::Inner;
    ///
    /// let config = r#"version = "v1"
    ///
    /// [[repositories]]
    /// organization = "Foo"
    /// name = "Bar""#;
    ///
    /// let inner: Inner = toml::from_str(&config).unwrap();
    /// assert_eq!(inner.repositories().len(), 1);
    /// ```
    pub fn repositories(&self) -> &Repositories {
        &self.repositories
    }

    /// Extends the [`Repositories`] for this [`Inner`].
    ///
    /// # Examples
    ///
    /// ```
    /// use wdl_core as core;
    /// use wdl_gauntlet as gauntlet;
    ///
    /// use indexmap::IndexSet;
    ///
    /// use gauntlet::config::Inner;
    ///
    /// let config = r#"version = "v1"
    ///
    /// [[repositories]]
    /// organization = "Foo"
    /// name = "Bar""#;
    ///
    /// let mut inner: Inner = toml::from_str(&config).unwrap();
    ///
    /// let mut repositories = IndexSet::new();
    /// repositories.insert(
    ///     "Foo/Baz"
    ///         .parse::<gauntlet::repository::Identifier>()
    ///         .unwrap(),
    /// );
    ///
    /// inner.extend_repositories(repositories);
    ///
    /// assert_eq!(inner.repositories().len(), 2);
    /// ```
    pub fn extend_repositories<T: IntoIterator<Item = repository::Identifier>>(
        &mut self,
        items: T,
    ) {
        self.repositories.extend(items.into_iter());
        self.repositories.sort();
    }

    /// Gets the [`Errors`] for this [`Inner`] by reference.
    ///
    /// # Examples
    ///
    /// ```
    /// use wdl_core as core;
    /// use wdl_gauntlet as gauntlet;
    ///
    /// use gauntlet::config::Inner;
    ///
    /// let config = r#"version = "v1"
    ///
    /// [[ignored_errors]]
    /// document = "Foo/Bar:baz.wdl"
    /// error = '''an error'''"#;
    ///
    /// let mut inner: Inner = toml::from_str(&config).unwrap();
    ///
    /// assert_eq!(inner.ignored_errors().len(), 1);
    /// ```
    pub fn ignored_errors(&self) -> &Errors {
        &self.ignored_errors
    }

    /// Replaces the [`Errors`] for this [`Inner`].
    ///
    /// # Examples
    ///
    /// ```
    /// use wdl_core as core;
    /// use wdl_gauntlet as gauntlet;
    ///
    /// use indexmap::IndexMap;
    ///
    /// use gauntlet::config::Inner;
    ///
    /// let config = r#"version = "v1"
    ///
    /// [[ignored_errors]]
    /// document = "Foo/Bar:baz.wdl"
    /// error = '''an error'''"#;
    ///
    /// let mut inner: Inner = toml::from_str(&config).unwrap();
    ///
    /// let mut errors = IndexMap::new();
    /// errors.insert(
    ///     "Foo/Baz:quux.wdl"
    ///         .parse::<gauntlet::document::Identifier>()
    ///         .unwrap(),
    ///     String::from("another error"),
    /// );
    ///
    /// inner.replace_ignored_errors(errors);
    ///
    /// assert_eq!(inner.ignored_errors().len(), 1);
    /// let (document, error) = inner.ignored_errors().first().unwrap();
    /// assert_eq!(error, &String::from("another error"));
    /// ```
    pub fn replace_ignored_errors<T: IntoIterator<Item = (document::Identifier, String)>>(
        &mut self,
        items: T,
    ) {
        self.ignored_errors = items.into_iter().collect();
        self.ignored_errors.sort_keys();
    }

    /// Extends the [`Errors`] for this [`Inner`].
    ///
    /// # Examples
    ///
    /// ```
    /// use wdl_core as core;
    /// use wdl_gauntlet as gauntlet;
    ///
    /// use indexmap::IndexMap;
    ///
    /// use gauntlet::config::Inner;
    ///
    /// let config = r#"version = "v1"
    ///
    /// [[ignored_errors]]
    /// document = "Foo/Bar:baz.wdl"
    /// error = '''an error'''"#;
    ///
    /// let mut inner: Inner = toml::from_str(&config).unwrap();
    ///
    /// let mut errors = IndexMap::new();
    /// errors.insert(
    ///     "Foo/Baz:quux.wdl"
    ///         .parse::<gauntlet::document::Identifier>()
    ///         .unwrap(),
    ///     String::from("another error"),
    /// );
    ///
    /// inner.extend_ignored_errors(errors);
    ///
    /// assert_eq!(inner.ignored_errors().len(), 2);
    /// ```
    pub fn extend_ignored_errors<T: IntoIterator<Item = (document::Identifier, String)>>(
        &mut self,
        items: T,
    ) {
        self.ignored_errors.extend(items.into_iter());
        self.ignored_errors.sort_keys();
    }

    /// Sorts the [`Repositories`] and the [`Errors`] (by key).
    pub fn sort(&mut self) {
        self.repositories.sort();
        self.ignored_errors.sort_keys();
    }
}

impl From<core::Version> for Inner {
    fn from(version: core::Version) -> Self {
        Self {
            version,
            repositories: Default::default(),
            ignored_errors: Default::default(),
        }
    }
}
