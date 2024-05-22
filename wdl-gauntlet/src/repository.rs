//! A local repository of files from a remote GitHub repository.

use std::path::PathBuf;

use faster_hex;
use git2::build::RepoBuilder;
use git2::FetchOptions;
use indexmap::IndexMap;
use log::info;
use serde::Deserialize;
use serde::Serialize;

pub mod cache;
pub mod identifier;

pub use cache::Cache;
pub use identifier::Identifier;

/// A byte slice that can be converted to a [`git2::Oid`].
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct RawHash([u8; 20]);

impl Serialize for RawHash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        faster_hex::hex_string(&self.0).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for RawHash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s.len() != 40 {
            return Err(serde::de::Error::custom(
                "a commit hash must have 40 characters",
            ));
        }

        let mut hash = [0u8; 20];
        faster_hex::hex_decode(s.as_bytes(), &mut hash).map_err(serde::de::Error::custom)?;
        Ok(Self(hash))
    }
}

/// A repository of GitHub files.
#[derive(Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Debug)]
pub struct Repository {
    /// The root directory of the [`Repository`].
    root: PathBuf,

    /// The name for the [`Repository`] expressed as an [`Identifier`].
    identifier: Identifier,

    /// The commit hash for the [`Repository`].
    commit_hash: RawHash,
}

impl Repository {
    /// Create a new [`Repository`].
    pub fn new(
        root: impl Into<PathBuf>,
        identifier: Identifier,
        commit_hash: Option<RawHash>,
    ) -> Self {
        let root = root.into();

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
        let commit_hash = match commit_hash {
            Some(hash) => {
                let obj = git_repo
                    .find_object(
                        git2::Oid::from_bytes(&hash.0).expect("failed to convert hash"),
                        Some(git2::ObjectType::Commit),
                    )
                    .expect("failed to find object");
                git_repo
                    .set_head_detached(obj.id())
                    .expect("failed to set head detached");
                hash
            }
            None => {
                let head = git_repo.head().expect("failed to get head");
                let commit = head.peel_to_commit().expect("failed to peel to commit");

                let mut bytes = [0u8; 20];
                bytes.copy_from_slice(commit.id().as_bytes());
                RawHash(bytes)
            }
        };

        Self {
            root,
            identifier,
            commit_hash,
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

    /// Gets the commit hash from the [`Repository`] by reference.
    pub fn commit_hash(&self) -> &RawHash {
        &self.commit_hash
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

    /// Update to the latest commit hash for the [`Repository`].
    pub fn update(&mut self) {
        // Clear the repo's root directory.
        std::fs::remove_dir_all(&self.root).expect("failed to remove root directory");

        // Re-clone the repository.
        info!("cloning repository: {:?}", self.identifier);
        let mut fo = FetchOptions::new();
        fo.depth(1);
        let git_repo = RepoBuilder::new()
            .fetch_options(fo)
            .clone(
                format!("https://github.com/{}.git", self.identifier).as_str(),
                &self.root,
            )
            .expect("failed to clone repository");

        // Update the commit hash.
        let head = git_repo.head().expect("failed to get head");
        let commit = head.peel_to_commit().expect("failed to peel to commit");

        let mut bytes = [0u8; 20];
        bytes.copy_from_slice(commit.id().as_bytes());
        self.commit_hash = RawHash(bytes)
    }
}
