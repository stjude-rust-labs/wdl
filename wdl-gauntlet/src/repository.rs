//! A local repository of files from a remote GitHub repository.

use std::path::PathBuf;

use git2::build::CheckoutBuilder;
use git2::build::RepoBuilder;
use git2::FetchOptions;
use git2::RemoteCallbacks;
use indexmap::IndexMap;
use log::debug;
use log::info;
use log::warn;

pub mod cache;
pub mod identifier;

pub use cache::Cache;
pub use identifier::Identifier;

/// A repository of GitHub files.
pub struct Repository {
    /// The root directory of the [`Repository`].
    root: PathBuf,

    /// The name for the [`Repository`] expressed as an [`Identifier`].
    identifier: Identifier,

    /// Raw `git2` repository.
    raw_repository: git2::Repository,
}

impl Repository {
    /// Create a new [`Repository`].
    pub fn new(root: impl Into<PathBuf>, identifier: Identifier) -> Self {
        let root = root.into().join(identifier.name());

        // Ensure the root directory exists.
        if !root.exists() {
            info!("creating repository root directory: {:?}", root);
            std::fs::create_dir_all(&root).expect("failed to create repository root directory");
        }

        let git_repo = match git2::Repository::open(&root) {
            Ok(repo) => {
                info!("opening existing repository: {:?}", root);
                repo
            }
            Err(_) => {
                info!("cloning repository: {:?}", identifier);
                let mut fo = FetchOptions::new();
                fo.depth(1);
                RepoBuilder::new()
                    .fetch_options(fo)
                    .clone(
                        format!("https://github.com/{}.git", identifier).as_str(),
                &root,
                    )
                    .expect("failed to clone repository")
            }
        };

        Self {
            root,
            identifier,
            raw_repository: git_repo,
        }
    }

    /// Get the root directory of the [`Repository`].
    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    /// Gets the repository identifier from the [`Repository`] by reference.
    #[allow(dead_code)]
    pub fn identifier(&self) -> &Identifier {
        &self.identifier
    }

    /// Retrieve all the WDL files from the [`Repository`].
    pub fn wdl_files(&self) -> IndexMap<String, String> {
        let mut wdl_files = IndexMap::new();
        self.add_wdl_files(&self.root, &mut wdl_files);
        wdl_files
    }

    /// Add to an [`IndexMap`] all the WDL files in a directory
    /// and its subdirectories.
    fn add_wdl_files(&self, path: &PathBuf, wdl_files: &mut IndexMap<String, String>) {
        if path.is_dir() {
            for entry in std::fs::read_dir(path).expect("failed to read directory") {
                let entry = entry.expect("failed to read entry");
                let path = entry.path();
                self.add_wdl_files(&path, wdl_files);
            }
        } else if path.is_file() {
            let path_str = path
                .to_str()
                .expect("failed to convert file name to string");
            if path_str.ends_with(".wdl") {
                let contents =
                    std::fs::read_to_string(&path).expect("failed to read file contents");
                wdl_files.insert(path_str.to_string(), contents);
            }
        }
    }
}
