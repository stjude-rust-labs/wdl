//! Cache for storing `Repository` files.

use std::path::PathBuf;

use indexmap::IndexMap;
use log::info;

use crate::repository::identifier::Identifier;
use crate::repository::Repository;

/// A cache for storing `Repository` files.
pub struct Cache {
    /// The root directory of the `Cache`.
    root: PathBuf,

    /// The repositories stored in the `Cache`.
    repositories: IndexMap<Identifier, Repository>,
}

impl Cache {
    /// Create a new `Cache`.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        let root = root.into();

        // Ensure the root directory exists.
        if !root.exists() {
            info!("creating cache root directory: {:?}", root);
            std::fs::create_dir_all(&root).expect("failed to create cache root directory");
        }

        Self {
            root,
            repositories: IndexMap::new(),
        }
    }

    /// Get the root directory of the `Cache`.
    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    /// Get the repositories stored in the `Cache`.
    pub fn repositories(&self) -> &IndexMap<Identifier, Repository> {
        &self.repositories
    }

    /// Add a repository to the `Cache` from an `Identifier`
    /// and optionally a commit hash.
    pub fn add_by_identifier(
        &mut self,
        identifier: &Identifier,
        commit_hash: Option<[u8; 20]>,
    ) -> &Repository {
        let repository = Repository::new(
            self.root
                .join(identifier.organization())
                .join(identifier.name()),
            identifier.clone(),
            commit_hash,
        );

        self.repositories.insert(identifier.clone(), repository);
        self.get_repository(identifier)
            .expect("failed to add repository")
    }

    /// Get a repository from the `Cache` by its identifier.
    pub fn get_repository(&self, identifier: &Identifier) -> Option<&Repository> {
        self.repositories.get(identifier)
    }
}
