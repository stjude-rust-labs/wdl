//! Utility functions for cryptographically hashing files and directories.

use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::Mutex;

use anyhow::Context;
use anyhow::Result;
use blake3::Hash;
use blake3::Hasher;
use tokio::sync::Notify;
use tokio::task::spawn_blocking;
use url::Url;
use walkdir::WalkDir;

/// Keeps track of previously calculated digests.
///
/// As WDL evaluation cannot write to existing files, it is assumed that files
/// do not change their contents during evaluation.
static DIGESTS: LazyLock<Mutex<HashMap<PathBuf, DigestStatus>>> = LazyLock::new(Mutex::default);

/// Represents the status of a calculating digest.
#[derive(Debug, Clone)]
pub enum DigestStatus {
    /// The digest is currently being calculated.
    ///
    /// The provided notify can be used to wait for the completion of the
    /// calculation.
    Calculating(Arc<Notify>),
    /// The digest has been calculated.
    Calculated(Digest),
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
    let mut waited = false;
    loop {
        let notified = {
            let mut digests = DIGESTS.lock().expect("failed to lock digests");
            match digests.get(path) {
                Some(DigestStatus::Calculating(notify)) => {
                    assert!(
                        !waited,
                        "digest should not be calculating after a notification"
                    );
                    Notify::notified_owned(notify.clone())
                }
                Some(DigestStatus::Calculated(r)) => {
                    return Ok(*r);
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
        waited = true;
    }

    let p = path.to_path_buf();
    let result = spawn_blocking(move || -> Result<Digest> {
        if p.is_file() {
            let mut hasher = Hasher::new();
            hasher.update_mmap(&p).with_context(|| {
                format!(
                    "failed to calculate digest of file `{path}`",
                    path = p.display()
                )
            })?;
            return Ok(Digest::File(hasher.finalize()));
        }

        let mut hasher = Hasher::new();
        let mut count: usize = 0;
        for entry in WalkDir::new(&p).sort_by_file_name() {
            let entry = entry.with_context(|| {
                format!(
                    "failed to walk directory contents of `{path}`",
                    path = p.display()
                )
            })?;

            count += 1;

            let rel_path = entry
                .path()
                .strip_prefix(&p)
                .unwrap_or(entry.path())
                .to_str()
                .with_context(|| {
                    format!("path `{path}` is not UTF-8", path = entry.path().display())
                })?;

            hasher.update(rel_path.as_bytes());
            if entry.path().is_file() {
                hasher
                    .update(&[1])
                    .update_mmap(entry.path())
                    .with_context(|| {
                        format!(
                            "failed to calculate digest of file `{path}`",
                            path = entry.path().display()
                        )
                    })?;
            } else {
                hasher.update(&[0]);
            }
        }

        hasher.update(&count.to_le_bytes());
        Ok(Digest::Directory(hasher.finalize()))
    })
    .await
    .expect("digest task panicked");

    // Update the status and notify any waiters
    let notify = {
        let mut digests = DIGESTS.lock().expect("failed to lock digests");
        let status = digests.get_mut(path).expect("should have status");
        let notify = match status {
            DigestStatus::Calculating(notify) => notify.clone(),
            DigestStatus::Calculated(_) => panic!("expected to find a calculating status"),
        };

        match &result {
            Ok(digest) => {
                *status = DigestStatus::Calculated(*digest);
            }
            Err(_) => {
                digests.remove(path);
            }
        }

        notify
    };

    notify.notify_waiters();
    result
}
