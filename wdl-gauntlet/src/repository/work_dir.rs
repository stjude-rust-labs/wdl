//! WorkDir for storing `Repository` files.

use indexmap::IndexMap;
use temp_dir::TempDir;

use crate::repository::identifier::Identifier;
use crate::repository::Repository;

/// A working directory for storing `Repository` files.
pub struct WorkDir {
    /// The root directory of the `WorkDir`.
    root: TempDir,

    /// The repositories stored in the `WorkDir`.
    repositories: IndexMap<Identifier, Repository>,
}

impl WorkDir {
    /// Create a new `WorkDir`.
    pub fn new() -> Self {
        let root = TempDir::new().expect("failed to create temporary directory");

        Self {
            root,
            repositories: IndexMap::new(),
        }
    }

    /// Get the root directory of the `WorkDir`.
    pub fn root(&self) -> &TempDir {
        &self.root
    }

    /// Get the repositories stored in the `WorkDir`.
    pub fn repositories(&self) -> &IndexMap<Identifier, Repository> {
        &self.repositories
    }

    /// Add a repository to the `WorkDir` from an [`Identifier`].
    /// By a guarantee of [`Repository::new()`], the added repository will
    /// _always_ have `Some(commit_hash)`.
    pub fn add_by_identifier(&mut self, identifier: &Identifier) {
        let repository = Repository::new(
            self.root
                .path()
                .join(identifier.organization())
                .join(identifier.name()),
            identifier.clone(),
            None,
        );

        self.repositories.insert(identifier.clone(), repository);
    }

    /// Get a repository from the `WorkDir` by its identifier.
    pub fn get_repository(&self, identifier: &Identifier) -> Option<&Repository> {
        self.repositories.get(identifier)
    }
}
