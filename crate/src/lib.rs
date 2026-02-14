//! # codec-corpus
//!
//! Runtime API for downloading, caching, and accessing test image datasets
//! from the [`imazen/codec-corpus`](https://github.com/imazen/codec-corpus)
//! GitHub repository.
//!
//! No data ships with the crate. Datasets are fetched lazily on first access
//! and cached locally.
//!
//! ```no_run
//! let corpus = codec_corpus::Corpus::new().unwrap();
//! let valid = corpus.get("webp-conformance/valid").unwrap();
//! for entry in std::fs::read_dir(valid).unwrap() {
//!     let path = entry.unwrap().path();
//!     println!("{}", path.display());
//! }
//! ```

#![forbid(unsafe_code)]

mod download;

use std::path::{Path, PathBuf};
use std::time::Duration;

// ---------------------------------------------------------------------------
// Embedded metadata
// ---------------------------------------------------------------------------

const REPO_URL: &str = "https://github.com/imazen/codec-corpus";
const CRATE_VERSION: &str = env!("CARGO_PKG_VERSION");

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Errors that can occur when using the corpus.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// Network unavailable and dataset not in cache.
    NetworkUnavailable { dataset: String },
    /// Requested path does not exist after successful download.
    PathNotFound { path: String },
    /// Filesystem error (permissions, disk full, etc.).
    Io(std::io::Error),
    /// No cache directory could be determined.
    NoCacheDir,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NetworkUnavailable { dataset } => {
                write!(f, "network unavailable and dataset '{dataset}' not cached")
            }
            Error::PathNotFound { path } => {
                write!(f, "path not found: '{path}'")
            }
            Error::Io(e) => write!(f, "I/O error: {e}"),
            Error::NoCacheDir => write!(f, "could not determine cache directory"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

// ---------------------------------------------------------------------------
// Corpus
// ---------------------------------------------------------------------------

/// Handle for accessing cached test-image datasets.
///
/// Create with [`Corpus::new()`] (default cache location) or
/// [`Corpus::with_cache_root()`] (explicit path). Then call [`Corpus::get()`]
/// to obtain the local path to a dataset — downloading it if necessary.
pub struct Corpus {
    root: PathBuf,
}

impl Corpus {
    /// Initialize with default cache location.
    ///
    /// Resolves cache root via `CODEC_CORPUS_CACHE` env var, then
    /// `dirs::cache_dir()`. Performs no I/O beyond directory creation.
    pub fn new() -> Result<Self, Error> {
        let base = if let Ok(val) = std::env::var("CODEC_CORPUS_CACHE") {
            PathBuf::from(val)
        } else {
            dirs::cache_dir().ok_or(Error::NoCacheDir)?
        };
        Self::init(base)
    }

    /// Initialize with explicit cache root. Overrides the environment variable.
    ///
    /// Files will live at `{path}/codec-corpus/v{major}/`.
    pub fn with_cache_root(path: impl Into<PathBuf>) -> Result<Self, Error> {
        Self::init(path.into())
    }

    /// Get the local path to a corpus subdirectory, downloading if needed.
    ///
    /// `path` is any path into the repository (e.g. `"webp-conformance"`,
    /// `"webp-conformance/valid"`, `"clic2025/training"`). The top-level
    /// folder is the download unit — requesting any path under it fetches
    /// the entire folder recursively.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let corpus = codec_corpus::Corpus::new()?;
    /// let webp = corpus.get("webp-conformance")?;
    /// let valid = corpus.get("webp-conformance/valid")?;
    /// let training = corpus.get("clic2025/training")?;
    /// # Ok::<(), codec_corpus::Error>(())
    /// ```
    pub fn get(&self, path: &str) -> Result<PathBuf, Error> {
        let top = top_level_folder(path);
        let full_path = self.root.join(path);

        // Fast path: version matches and directory exists
        if self.version_matches() && full_path.exists() {
            return Ok(full_path);
        }

        // Slow path: download the top-level folder
        self.ensure_downloaded(top)?;

        if full_path.exists() {
            Ok(full_path)
        } else {
            Err(Error::PathNotFound {
                path: path.to_string(),
            })
        }
    }

    /// Check if a path is already cached locally, without downloading.
    pub fn is_cached(&self, path: &str) -> bool {
        self.version_matches() && self.root.join(path).exists()
    }

    /// List datasets currently cached on disk.
    ///
    /// Returns directory names under the cache root, excluding internal
    /// files (`.version`, `.lock`, `.tmp-*`).
    pub fn list_cached(&self) -> Vec<String> {
        let mut datasets = Vec::new();
        let Ok(entries) = std::fs::read_dir(&self.root) else {
            return datasets;
        };
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with('.') {
                continue;
            }
            if entry.path().is_dir() {
                datasets.push(name_str.into_owned());
            }
        }
        datasets.sort();
        datasets
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    fn init(base: PathBuf) -> Result<Self, Error> {
        let major = CRATE_VERSION
            .split('.')
            .next()
            .unwrap_or("0");
        let root = base.join("codec-corpus").join(format!("v{major}"));
        std::fs::create_dir_all(&root).map_err(Error::Io)?;
        Ok(Self { root })
    }

    fn version_matches(&self) -> bool {
        let version_file = self.root.join(".version");
        std::fs::read_to_string(&version_file)
            .map(|v| v.trim() == CRATE_VERSION)
            .unwrap_or(false)
    }

    /// Download the top-level folder that contains `folder`.
    fn ensure_downloaded(&self, folder: &str) -> Result<(), Error> {
        let lock_path = self.root.join(".lock");
        let lock_file = std::fs::File::create(&lock_path).map_err(Error::Io)?;
        let mut lock = fd_lock::RwLock::new(lock_file);
        let _guard = lock.write().map_err(Error::Io)?;

        // Re-check after acquiring lock — another process may have finished
        if self.version_matches() && self.root.join(folder).is_dir() {
            cleanup_old_temps(&self.root);
            return Ok(());
        }

        // If version mismatch, we need to re-download
        let need_version_reset = !self.version_matches();

        if need_version_reset {
            self.clear_datasets();
        }

        // Git sparse-checks out just the folder; HTTP downloads the
        // root-folder tarball (which may include sibling paths).
        let download_result = download::try_git_sparse_checkout(
            &self.root,
            folder,
            CRATE_VERSION,
            REPO_URL,
        )
        .or_else(|_| download::try_http_download(&self.root, folder, CRATE_VERSION));

        cleanup_old_temps(&self.root);
        download_result?;

        write_version_file(&self.root, CRATE_VERSION)?;
        Ok(())
    }

    fn clear_datasets(&self) {
        if let Ok(entries) = std::fs::read_dir(&self.root) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                // Keep .lock and .tmp-* entries (temps cleaned separately)
                if name_str == ".lock" || name_str.starts_with(".tmp-") {
                    continue;
                }
                let path = entry.path();
                if path.is_dir() {
                    let _ = std::fs::remove_dir_all(&path);
                } else {
                    let _ = std::fs::remove_file(&path);
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Free helpers
// ---------------------------------------------------------------------------

/// Extract the first path component (the root folder / download unit).
fn top_level_folder(path: &str) -> &str {
    path.split('/').next().unwrap_or(path)
}

/// Atomically write the version file (write to temp, then rename).
fn write_version_file(root: &std::path::Path, version: &str) -> Result<(), Error> {
    let version_file = root.join(".version");
    let tmp = root.join(".version.tmp");
    std::fs::write(&tmp, version).map_err(Error::Io)?;
    std::fs::rename(&tmp, &version_file).map_err(Error::Io)?;
    Ok(())
}

/// Remove `.tmp-*` entries older than 1 hour (orphaned from crashed runs).
fn cleanup_old_temps(root: &Path) {
    let one_hour = Duration::from_secs(3600);
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };

    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if !name_str.starts_with(".tmp-") {
            continue;
        }
        let Ok(meta) = entry.metadata() else {
            continue;
        };
        let age = meta
            .modified()
            .ok()
            .and_then(|t| t.elapsed().ok())
            .unwrap_or_default();
        if age > one_hour {
            let path = entry.path();
            if path.is_dir() {
                let _ = std::fs::remove_dir_all(&path);
            } else {
                let _ = std::fs::remove_file(&path);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_top_level_folder() {
        assert_eq!(top_level_folder("webp-conformance"), "webp-conformance");
        assert_eq!(top_level_folder("webp-conformance/valid"), "webp-conformance");
        assert_eq!(top_level_folder("clic2025/training/subdir"), "clic2025");
    }

    #[test]
    fn test_list_cached_empty() {
        let tmp = std::env::temp_dir().join("codec-corpus-test-list-cached");
        let _ = std::fs::remove_dir_all(&tmp);
        let corpus = Corpus::with_cache_root(&tmp).unwrap();
        assert!(corpus.list_cached().is_empty());
        let _ = std::fs::remove_dir_all(tmp);
    }

    #[test]
    fn test_list_cached_with_dirs() {
        let tmp = std::env::temp_dir().join("codec-corpus-test-list-cached2");
        let _ = std::fs::remove_dir_all(&tmp);
        let corpus = Corpus::with_cache_root(&tmp).unwrap();
        // Create fake dataset dirs
        std::fs::create_dir_all(corpus.root.join("alpha")).unwrap();
        std::fs::create_dir_all(corpus.root.join("beta")).unwrap();
        // Hidden dirs should be excluded
        std::fs::create_dir_all(corpus.root.join(".tmp-123")).unwrap();
        let cached = corpus.list_cached();
        assert_eq!(cached, vec!["alpha", "beta"]);
        let _ = std::fs::remove_dir_all(tmp);
    }

    #[test]
    fn test_unknown_dataset_downloads() {
        // With no hardcoded list, any name is accepted (will fail at download)
        let tmp = std::env::temp_dir().join("codec-corpus-test-any-name");
        let _ = std::fs::remove_dir_all(&tmp);
        let corpus = Corpus::with_cache_root(&tmp).unwrap();
        let result = corpus.get("nonexistent-dataset");
        // Should fail with NetworkUnavailable, not UnknownDataset
        assert!(matches!(result, Err(Error::NetworkUnavailable { .. })));
        let _ = std::fs::remove_dir_all(tmp);
    }

    #[test]
    fn test_is_cached_empty() {
        let tmp = std::env::temp_dir().join("codec-corpus-test-cached");
        let _ = std::fs::remove_dir_all(&tmp);
        let corpus = Corpus::with_cache_root(&tmp).unwrap();
        assert!(!corpus.is_cached("webp-conformance"));
        let _ = std::fs::remove_dir_all(tmp);
    }

    #[test]
    fn test_version_matches() {
        let tmp = std::env::temp_dir().join("codec-corpus-test-version");
        let _ = std::fs::remove_dir_all(&tmp);
        let corpus = Corpus::with_cache_root(&tmp).unwrap();
        assert!(!corpus.version_matches());

        write_version_file(&corpus.root, CRATE_VERSION).unwrap();
        assert!(corpus.version_matches());

        write_version_file(&corpus.root, "0.0.0-fake").unwrap();
        assert!(!corpus.version_matches());

        let _ = std::fs::remove_dir_all(tmp);
    }
}
