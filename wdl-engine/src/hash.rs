//! Utility functions for cryptographically hashing files and directories.

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::Mutex;
use std::time::Duration;
use std::time::SystemTime;

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use blake3::Hash;
use blake3::Hasher;
use tokio::sync::Notify;
use tokio::task::spawn_blocking;
use url::Url;
use walkdir::WalkDir;

/// Keeps track of previously calculated digests.
///
/// As WDL evaluation cannot write to existing files, it is assumed that files
/// and directories are not modified during evaluation.
///
/// We check for changes to files and directories when we get a cache hit and
/// error if the source has been modified.
static DIGESTS: LazyLock<Mutex<HashMap<PathBuf, DigestStatus>>> = LazyLock::new(Mutex::default);

/// Represents metadata about a cache entry.
///
/// This is used to detect changes to the source of a cached digest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Metadata {
    /// The metadata is for a file.
    File {
        /// The size of the file, in bytes.
        size: u64,
        /// The last modified time of the file.
        ///
        /// This will be `None` on platforms that don't support it.
        mtime: Option<SystemTime>,
    },
    /// The metadata is for a directory.
    Directory {
        /// A digest that is derived from:
        ///
        /// * The size of each recursive entry in the directory, in bytes.
        /// * The last modified time of each recursive entry in the directory.
        /// * The number of recursive entries in the directory.
        digest: Hash,
    },
}

impl Metadata {
    /// Constructs a new metadata for the given path.
    ///
    /// The provided callback is invoked for each recursively contained entry if
    /// the given path is a directory.
    fn new<F>(path: impl AsRef<Path>, mut on_entry: F) -> Result<Self>
    where
        F: FnMut(&Path, &fs::Metadata) -> Result<()>,
    {
        let path = path.as_ref();
        let metadata = path.metadata().with_context(|| {
            format!(
                "failed to read metadata for path `{path}`",
                path = path.display()
            )
        })?;

        // If the path is a file, return its metadata
        if metadata.is_file() {
            return Ok(Self::File {
                size: metadata.len(),
                mtime: metadata.modified().ok(),
            });
        }

        // Otherwise, walk the directory and calculate a digest.
        let mut entries: usize = 0;
        let mut hasher = Hasher::new();
        for entry in WalkDir::new(path).sort_by_file_name() {
            let entry = entry.with_context(|| {
                format!(
                    "failed to walk directory contents of `{path}`",
                    path = path.display()
                )
            })?;

            let metadata = entry.metadata().with_context(|| {
                format!(
                    "failed to read metadata for path `{path}`",
                    path = entry.path().display()
                )
            })?;

            // Hash the size of the entry
            hasher.update(&metadata.len().to_le_bytes());

            // Hash the mtime of the entry
            let duration = metadata
                .modified()
                .map(|m| {
                    m.duration_since(SystemTime::UNIX_EPOCH).with_context(|| {
                        format!(
                            "last modified time for `{path}` is before UNIX epoch",
                            path = path.display()
                        )
                    })
                })
                .unwrap_or(Ok(Duration::ZERO))?;
            hasher.update(&duration.as_nanos().to_le_bytes());

            // Call the provided callback
            on_entry(entry.path(), &metadata)?;

            entries += 1;
        }

        hasher.update(&entries.to_le_bytes());
        Ok(Self::Directory {
            digest: hasher.finalize(),
        })
    }
}

/// Represents the status of a calculating digest.
#[derive(Debug, Clone)]
enum DigestStatus {
    /// The digest is currently being calculated.
    ///
    /// The provided notify can be used to wait for the completion of the
    /// calculation.
    Calculating(Arc<Notify>),
    /// The digest has been calculated.
    Calculated {
        /// The metadata for the entry.
        metadata: Metadata,
        /// The digest that was calculated.
        digest: Digest,
    },
}

/// Represents a calculated [Blake3](https://github.com/BLAKE3-team/BLAKE3) digest of a file or directory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Digest {
    /// The digest is for a file.
    File(Hash),
    /// The digest is for a directory.
    Directory(Hash),
}

/// An extension trait for joining a digest to a URL.
pub trait UrlDigestExt: Sized {
    /// Joins the given digest to the URL.
    ///
    /// If the digest is for a file, a `file` path segment is pushed first.
    ///
    /// If the digest is for a directory, a `directory` path segment is pushed
    /// first.
    ///
    /// A path segment is then pushed for the digest as a hex string.
    fn join_digest(&self, digest: Digest) -> Self;
}

impl UrlDigestExt for Url {
    fn join_digest(&self, digest: Digest) -> Self {
        assert!(
            !self.cannot_be_a_base(),
            "invalid URL: URL is required to be a base"
        );

        let mut url = self.clone();

        {
            // SAFETY: this will always return `Ok` if the above assert passed
            let mut segments = url.path_segments_mut().unwrap();
            segments.pop_if_empty();

            let digest = match digest {
                Digest::File(digest) => {
                    segments.push("file");
                    digest
                }
                Digest::Directory(digest) => {
                    segments.push("directory");
                    digest
                }
            };

            let hex = digest.to_hex();
            segments.push(hex.as_str());
        }

        url
    }
}

/// Calculates the digest of a path.
///
/// If the path is a single file, a [blake3](blake3) digest is calculated for
/// the file.
///
/// If the path is a directory, a consistent, recursive walk of the directory is
/// performed and a digest of each directory entry is calculated.
///
/// A directory entry's digest consists of:
///
/// * The relative path to the entry.
/// * Whether or not the entry is a file or a directory.
/// * If the entry is a file, the [blake3](blake3) digest of the file's
///   contents.
/// * The total number of entries in the directory.
///
/// The returned digest string is prefixed with `file/` for file digests and
/// `directory/` for directories.
///
/// [blake3]: https://github.com/BLAKE3-team/BLAKE3
pub async fn calculate_path_digest(path: impl AsRef<Path>) -> Result<Digest> {
    let path = path.as_ref();

    // This loop exists so that all but one request to digest the same path will
    // block on a notification that the digest has been calculated.
    //
    // When the notification is received, the lookup into the digests collection is
    // retried.
    loop {
        let notified = {
            let mut digests = DIGESTS.lock().expect("failed to lock digests");
            match digests.get(path) {
                Some(DigestStatus::Calculating(notify)) => Notify::notified_owned(notify.clone()),
                Some(DigestStatus::Calculated { digest, metadata }) => {
                    let new_metadata = Metadata::new(path, |_, _| Ok(()))?;
                    if new_metadata != *metadata {
                        bail!(
                            "path `{path}` has been modified during evaluation",
                            path = path.display()
                        );
                    }

                    return Ok(*digest);
                }
                None => {
                    // Insert an entry to notify others when the calculation completes
                    digests.insert(
                        path.to_path_buf(),
                        DigestStatus::Calculating(Arc::new(Notify::new())),
                    );
                    break;
                }
            }
        };

        notified.await;
    }

    // Spawn a task to compute the path's metadata and digest
    let base_path = path.to_path_buf();
    let result = spawn_blocking(move || -> Result<(Metadata, Digest)> {
        let mut hasher = Hasher::new();
        let mut entries: usize = 0;

        // Calculate the metadata digest for the path
        // For a directory, this will also calculate the content digest from the
        // callback
        let metadata = Metadata::new(&base_path, |path, metadata| {
            // Hash the relative path to the entry
            hasher.update(
                path.strip_prefix(&base_path)
                    .unwrap_or(path)
                    .to_str()
                    .with_context(|| format!("path `{path}` is not UTF-8", path = path.display()))?
                    .as_bytes(),
            );

            // If entry is a file, hash its contents
            if metadata.is_file() {
                hasher.update(&[1]).update_mmap(path).with_context(|| {
                    format!(
                        "failed to calculate digest of file `{path}`",
                        path = path.display()
                    )
                })?;
            } else {
                hasher.update(&[0]);
            }

            entries += 1;
            Ok(())
        })?;

        match metadata {
            Metadata::File { .. } => {
                assert_eq!(hasher.count(), 0, "hasher count should be zero");

                // If the metadata is for a file, hash its contents now
                hasher.update_mmap(&base_path).with_context(|| {
                    format!(
                        "failed to calculate digest of file `{path}`",
                        path = base_path.display()
                    )
                })?;
                Ok((metadata, Digest::File(hasher.finalize())))
            }
            Metadata::Directory { .. } => {
                // Write the number of entries to the content hash for the directory
                hasher.update(&entries.to_le_bytes());
                Ok((metadata, Digest::Directory(hasher.finalize())))
            }
        }
    })
    .await
    .expect("digest task panicked");

    // Update the status and notify any waiters
    let (result, notify) = {
        let mut digests = DIGESTS.lock().expect("failed to lock digests");
        let status = digests.get_mut(path).expect("should have status");
        let notify = match status {
            DigestStatus::Calculating(notify) => notify.clone(),
            DigestStatus::Calculated { .. } => panic!("expected to find a calculating status"),
        };

        match result {
            Ok((metadata, digest)) => {
                *status = DigestStatus::Calculated { metadata, digest };
                (Ok(digest), notify)
            }
            Err(e) => {
                digests.remove(path);
                (Err(e), notify)
            }
        }
    };

    notify.notify_waiters();
    result
}
