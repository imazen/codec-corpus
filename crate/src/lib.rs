//! # codec-corpus
//!
//! Runtime API for downloading, caching, and accessing test image datasets
//! from the [`imazen/codec-corpus`](https://github.com/imazen/codec-corpus)
//! GitHub repository and third-party sources.
//!
//! No data ships with the crate. Datasets are fetched lazily on first access
//! and cached locally.
//!
//! ## imazen/codec-corpus (default)
//!
//! ```no_run
//! let corpus = codec_corpus::Corpus::new().unwrap();
//! let valid = corpus.get("webp-conformance/valid").unwrap();
//! ```
//!
//! ## Third-party sources (built-in registry)
//!
//! ```no_run
//! let corpus = codec_corpus::Corpus::new().unwrap();
//!
//! // OSS-Fuzz backup corpora (ZIP download)
//! let ossfuzz_jpeg = corpus.get("oss-fuzz/libjpeg-turbo").unwrap();
//! let ossfuzz_png = corpus.get("oss-fuzz/libpng").unwrap();
//! let ossfuzz_jxl = corpus.get("oss-fuzz/libjxl").unwrap();
//!
//! // dvyukov/go-fuzz-corpus subfolders
//! let gofuzz_gif = corpus.get("go-fuzz-corpus/gif").unwrap();
//!
//! // libjpeg-turbo fuzz seed corpus
//! let ljt = corpus.get("libjpeg-turbo-fuzz").unwrap();
//!
//! // image-rs/image test images
//! let imagers = corpus.get("image-rs/tests/images").unwrap();
//! ```
//!
//! ## Ad-hoc sources
//!
//! ```no_run
//! let corpus = codec_corpus::Corpus::new().unwrap();
//!
//! // Arbitrary GitHub repo subfolder
//! let path = corpus.github_repo("niclas-aspect/jxl-rs", "test-data", "main").unwrap();
//!
//! // Arbitrary ZIP URL
//! let path = corpus.zip_url("my-corpus", "https://example.com/corpus.zip").unwrap();
//!
//! // Arbitrary tarball URL
//! let path = corpus.tar_url("my-tarball", "https://example.com/corpus.tar.gz").unwrap();
//!
//! // Local directory (validates existence, no download)
//! let path = corpus.local_path("/home/user/my-images").unwrap();
//! ```

#![forbid(unsafe_code)]

mod download;
mod registry;

use std::path::{Path, PathBuf};
use std::time::Duration;

// ---------------------------------------------------------------------------
// Embedded metadata
// ---------------------------------------------------------------------------

#[cfg(not(target_arch = "wasm32"))]
const REPO_URL: &str = "https://github.com/imazen/codec-corpus";
const CRATE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default staleness threshold for third-party sources: 7 days.
const DEFAULT_MAX_AGE: Duration = Duration::from_secs(7 * 24 * 3600);

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
    /// HTTP download failed.
    DownloadFailed { url: String, reason: String },
    /// Local path does not exist.
    LocalPathNotFound { path: String },
    /// Downloads are not supported on this platform (e.g. wasm32).
    ///
    /// Returned by [`Corpus::get()`] when the dataset is not cached and the
    /// runtime cannot spawn subprocesses to fetch it. Pre-populate the cache
    /// on the host and point [`Corpus::with_cache_root()`] at the preopened
    /// path instead.
    DownloadUnsupported { dataset: String },
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
            Error::DownloadFailed { url, reason } => {
                write!(f, "download failed for '{url}': {reason}")
            }
            Error::LocalPathNotFound { path } => {
                write!(f, "local path does not exist: '{path}'")
            }
            Error::DownloadUnsupported { dataset } => {
                write!(
                    f,
                    "downloads are not supported on this platform; \
                     dataset '{dataset}' must be pre-cached on the host"
                )
            }
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
///
/// Third-party sources are resolved by prefix (see module docs) and cached
/// under `{cache}/codec-corpus/v{major}/third-party/{source-key}/`.
pub struct Corpus {
    root: PathBuf,
    /// Maximum age for third-party `.fetched` markers before re-download.
    max_age: Duration,
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

    /// Set the maximum age for third-party cached data before re-download.
    ///
    /// Default is 7 days. Applies only to third-party sources — imazen/codec-corpus
    /// datasets use the crate version for staleness.
    pub fn with_max_age(mut self, max_age: Duration) -> Self {
        self.max_age = max_age;
        self
    }

    /// Get the local path to a corpus subdirectory, downloading if needed.
    ///
    /// `path` can be:
    /// - An imazen/codec-corpus path (e.g. `"webp-conformance"`, `"clic2025/training"`)
    /// - A registered third-party prefix (e.g. `"oss-fuzz/libpng"`, `"go-fuzz-corpus/gif"`)
    ///
    /// For imazen/codec-corpus paths, the top-level folder is the download unit.
    /// For third-party sources, the entire source is fetched on first access.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let corpus = codec_corpus::Corpus::new()?;
    ///
    /// // imazen/codec-corpus
    /// let webp = corpus.get("webp-conformance")?;
    /// let valid = corpus.get("webp-conformance/valid")?;
    ///
    /// // Third-party
    /// let fuzz = corpus.get("oss-fuzz/libjpeg-turbo")?;
    /// let gofuzz = corpus.get("go-fuzz-corpus/gif")?;
    /// # Ok::<(), codec_corpus::Error>(())
    /// ```
    pub fn get(&self, path: &str) -> Result<PathBuf, Error> {
        // Check if this matches a registered third-party source
        if let Some(source) = registry::resolve(path) {
            return self.get_third_party(path, &source);
        }

        // Fall through to imazen/codec-corpus
        self.get_imazen(path)
    }

    /// Check if a path is already cached locally, without downloading.
    pub fn is_cached(&self, path: &str) -> bool {
        // Check third-party first
        if let Some(source) = registry::resolve(path) {
            let tp_dir = self.third_party_dir(&source.cache_key);
            return tp_dir.is_dir() && !self.is_stale(&tp_dir);
        }

        // imazen/codec-corpus
        self.version_matches() && self.root.join(path).exists()
    }

    /// List datasets currently cached on disk.
    ///
    /// Returns directory names under the cache root, excluding internal
    /// files (`.version`, `.lock`, `.tmp-*`). Third-party sources are
    /// listed with their `third-party/` prefix.
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
            if name_str == "third-party" {
                // Enumerate third-party subdirectories
                if let Ok(tp_entries) = std::fs::read_dir(entry.path()) {
                    for tp_entry in tp_entries.flatten() {
                        let tp_name = tp_entry.file_name();
                        let tp_str = tp_name.to_string_lossy();
                        if tp_entry.path().is_dir() && !tp_str.starts_with('.') {
                            datasets.push(format!("third-party/{tp_str}"));
                        }
                    }
                }
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
    // Ad-hoc source methods
    // -----------------------------------------------------------------------

    /// Download and cache a GitHub repo subfolder.
    ///
    /// `owner_repo` is in `"owner/repo"` format. `repo_path` is the path
    /// within the repo to check out. `branch` is the branch or tag to use
    /// (e.g. `"main"`).
    ///
    /// Returns the local path to the checked-out subfolder.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let corpus = codec_corpus::Corpus::new()?;
    /// let path = corpus.github_repo("niclas-aspect/jxl-rs", "test-data", "main")?;
    /// # Ok::<(), codec_corpus::Error>(())
    /// ```
    pub fn github_repo(
        &self,
        owner_repo: &str,
        repo_path: &str,
        branch: &str,
    ) -> Result<PathBuf, Error> {
        let cache_key = format!(
            "github__{}__{}",
            owner_repo.replace('/', "__"),
            repo_path.replace('/', "__")
        );
        let dest = self.third_party_dir(&cache_key);

        if dest.is_dir() && !self.is_stale(&dest) {
            return Ok(dest);
        }

        self.with_lock(|| {
            // Re-check after lock
            if dest.is_dir() && !self.is_stale(&dest) {
                return Ok(());
            }
            download::git_sparse_checkout_github(&dest, owner_repo, repo_path, Some(branch))?;
            write_fetched_marker(&dest)?;
            Ok(())
        })?;

        if dest.is_dir() {
            Ok(dest)
        } else {
            Err(Error::PathNotFound {
                path: format!("{owner_repo}/{repo_path}"),
            })
        }
    }

    /// Download and cache a ZIP file from a URL.
    ///
    /// `cache_key` is a short identifier used as the directory name under
    /// `third-party/`. The ZIP is validated (magic bytes) and extracted.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let corpus = codec_corpus::Corpus::new()?;
    /// let path = corpus.zip_url("custom-corpus", "https://example.com/corpus.zip")?;
    /// # Ok::<(), codec_corpus::Error>(())
    /// ```
    pub fn zip_url(&self, cache_key: &str, url: &str) -> Result<PathBuf, Error> {
        let sanitized = sanitize_cache_key(cache_key);
        let dest = self.third_party_dir(&sanitized);

        if dest.is_dir() && !self.is_stale(&dest) {
            return Ok(dest);
        }

        self.with_lock(|| {
            if dest.is_dir() && !self.is_stale(&dest) {
                return Ok(());
            }
            download::download_and_extract_zip(&dest, url, &sanitized)?;
            write_fetched_marker(&dest)?;
            Ok(())
        })?;

        if dest.is_dir() {
            Ok(dest)
        } else {
            Err(Error::PathNotFound {
                path: cache_key.to_string(),
            })
        }
    }

    /// Download and cache a tarball from a URL.
    ///
    /// `cache_key` is a short identifier used as the directory name under
    /// `third-party/`. Extraction uses the system `tar` command.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let corpus = codec_corpus::Corpus::new()?;
    /// let path = corpus.tar_url("my-corpus", "https://example.com/corpus.tar.gz")?;
    /// # Ok::<(), codec_corpus::Error>(())
    /// ```
    pub fn tar_url(&self, cache_key: &str, url: &str) -> Result<PathBuf, Error> {
        let sanitized = sanitize_cache_key(cache_key);
        let dest = self.third_party_dir(&sanitized);

        if dest.is_dir() && !self.is_stale(&dest) {
            return Ok(dest);
        }

        self.with_lock(|| {
            if dest.is_dir() && !self.is_stale(&dest) {
                return Ok(());
            }
            download::download_and_extract_tar(&dest, url, &sanitized)?;
            write_fetched_marker(&dest)?;
            Ok(())
        })?;

        if dest.is_dir() {
            Ok(dest)
        } else {
            Err(Error::PathNotFound {
                path: cache_key.to_string(),
            })
        }
    }

    /// Point at a local directory without downloading anything.
    ///
    /// Validates that the path exists and is a directory.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let corpus = codec_corpus::Corpus::new()?;
    /// let path = corpus.local_path("/home/user/my-images")?;
    /// # Ok::<(), codec_corpus::Error>(())
    /// ```
    pub fn local_path(&self, path: impl AsRef<Path>) -> Result<PathBuf, Error> {
        let p = path.as_ref();
        if p.is_dir() {
            Ok(p.to_path_buf())
        } else {
            Err(Error::LocalPathNotFound {
                path: p.display().to_string(),
            })
        }
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    fn init(base: PathBuf) -> Result<Self, Error> {
        let major = CRATE_VERSION.split('.').next().unwrap_or("0");
        let root = base.join("codec-corpus").join(format!("v{major}"));
        std::fs::create_dir_all(&root).map_err(Error::Io)?;
        Ok(Self {
            root,
            max_age: DEFAULT_MAX_AGE,
        })
    }

    fn version_matches(&self) -> bool {
        let version_file = self.root.join(".version");
        std::fs::read_to_string(&version_file)
            .map(|v| v.trim() == CRATE_VERSION)
            .unwrap_or(false)
    }

    /// Path to the third-party cache directory for a given key.
    fn third_party_dir(&self, cache_key: &str) -> PathBuf {
        self.root.join("third-party").join(cache_key)
    }

    /// Check if a third-party source's `.fetched` marker is older than `max_age`.
    fn is_stale(&self, dir: &Path) -> bool {
        let marker = dir.join(".fetched");
        let Ok(contents) = std::fs::read_to_string(&marker) else {
            return true;
        };
        let Ok(ts) = contents.trim().parse::<u64>() else {
            return true;
        };
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now.saturating_sub(ts) > self.max_age.as_secs()
    }

    /// Run `f` while holding the file lock.
    /// On wasm32 there is no concurrent-process risk, so the lock is skipped.
    fn with_lock(&self, f: impl FnOnce() -> Result<(), Error>) -> Result<(), Error> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let lock_path = self.root.join(".lock");
            let lock_file = std::fs::File::create(&lock_path).map_err(Error::Io)?;
            let mut lock = fd_lock::RwLock::new(lock_file);
            let _guard = lock.write().map_err(Error::Io)?;
            f()
        }
        #[cfg(target_arch = "wasm32")]
        {
            f()
        }
    }

    /// Download the top-level folder from imazen/codec-corpus.
    fn get_imazen(&self, path: &str) -> Result<PathBuf, Error> {
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

    /// Get a third-party source. `path` is the user-facing path (e.g.
    /// `"oss-fuzz/libjpeg-turbo"`), `source` is the resolved registry entry.
    fn get_third_party(
        &self,
        path: &str,
        source: &registry::ResolvedSource,
    ) -> Result<PathBuf, Error> {
        let dest = self.third_party_dir(&source.cache_key);

        // Fast path: cached and fresh
        if dest.is_dir() && !self.is_stale(&dest) {
            // The user might be requesting a subpath within the source
            let user_subpath = self.third_party_subpath(path, source);
            let full = if let Some(sub) = &user_subpath {
                dest.join(sub)
            } else {
                dest.clone()
            };
            if full.exists() {
                return Ok(full);
            }
            // Fall through to download if subpath doesn't exist
        }

        // Slow path: download
        self.with_lock(|| {
            if dest.is_dir() && !self.is_stale(&dest) {
                return Ok(());
            }
            std::fs::create_dir_all(dest.parent().unwrap_or(&self.root)).map_err(Error::Io)?;
            match &source.kind {
                registry::SourceKind::Zip { url } => {
                    download::download_and_extract_zip(&dest, url, &source.cache_key)?;
                }
                registry::SourceKind::GitHubSubfolder {
                    repo,
                    repo_path,
                    branch,
                } => {
                    download::git_sparse_checkout_github(&dest, repo, repo_path, *branch)?;
                }
            }
            write_fetched_marker(&dest)?;
            Ok(())
        })?;

        let user_subpath = self.third_party_subpath(path, source);
        let full = if let Some(sub) = &user_subpath {
            dest.join(sub)
        } else {
            dest.clone()
        };

        if full.exists() {
            Ok(full)
        } else if dest.is_dir() {
            // The source was downloaded but the specific subpath doesn't exist.
            // Return the root of the source.
            Ok(dest)
        } else {
            Err(Error::PathNotFound {
                path: path.to_string(),
            })
        }
    }

    /// Compute the subpath within a third-party source that the user is
    /// requesting. Returns `None` if the user path exactly matches the
    /// source root.
    fn third_party_subpath(
        &self,
        user_path: &str,
        _source: &registry::ResolvedSource,
    ) -> Option<String> {
        // The registry key might consume multiple path components.
        // We need to figure out what the user path looks like relative to the
        // source. This is prefix-dependent.

        // For "oss-fuzz/libjpeg-turbo" the source root IS the path.
        // For "go-fuzz-corpus/gif/some-file" the subpath is "some-file".
        // For "image-rs/tests/images/jpeg" the subpath is "jpeg".

        // Strategy: the registry resolved this path, so the cache_key maps to
        // the downloaded root. Any remaining path components after what the
        // registry consumed are the subpath.

        // We reconstruct the "prefix" by checking what the registry would
        // resolve for progressively shorter prefixes of user_path.
        // Simpler approach: strip known prefixes.

        if let Some(rest) = user_path.strip_prefix("oss-fuzz/") {
            let project = rest.split('/').next().unwrap_or(rest);
            let after_project = &rest[project.len()..];
            let after_project = after_project.strip_prefix('/').unwrap_or(after_project);
            if after_project.is_empty() {
                return None;
            }
            return Some(after_project.to_string());
        }

        if let Some(rest) = user_path.strip_prefix("go-fuzz-corpus/") {
            let format = rest.split('/').next().unwrap_or(rest);
            let after_format = &rest[format.len()..];
            let after_format = after_format.strip_prefix('/').unwrap_or(after_format);
            if after_format.is_empty() {
                return None;
            }
            return Some(after_format.to_string());
        }

        if user_path.starts_with("libjpeg-turbo-fuzz") {
            let after = user_path.strip_prefix("libjpeg-turbo-fuzz").unwrap_or("");
            let after = after.strip_prefix('/').unwrap_or(after);
            if after.is_empty() {
                return None;
            }
            return Some(after.to_string());
        }

        if let Some(rest) = user_path.strip_prefix("image-rs/") {
            // image-rs/tests/images -> cache_key is "image-rs__tests__images"
            // image-rs/tests/images/jpeg -> subpath is "jpeg"
            // The cache_key encodes the full repo path, so we need to figure
            // out how many components the registry consumed.
            // The registry consumes all of `rest` for the cache key. But user
            // might go deeper. The cache_key is "image-rs__{rest with / -> __}".
            // We need to find the boundary. The registry uses the first component
            // of rest as the sparse checkout target.
            let _first = rest.split('/').next().unwrap_or(rest);
            // Actually, the cache_key is built from ALL components of rest.
            // So image-rs/tests/images has cache_key image-rs__tests__images.
            // But image-rs/tests/images/jpeg would also match the prefix
            // "image-rs/" and get cache_key "image-rs__tests__images__jpeg".
            // These would be different sources. So there's no subpath for
            // image-rs — the entire path after "image-rs/" defines the source.
            return None;
        }

        None
    }

    /// Download the top-level folder that contains `folder`.
    ///
    /// On wasm32, subprocess-based downloads are not available. Returns
    /// [`Error::DownloadUnsupported`] immediately — callers should
    /// pre-populate the cache on the host instead.
    #[cfg(target_arch = "wasm32")]
    fn ensure_downloaded(&self, folder: &str) -> Result<(), Error> {
        Err(Error::DownloadUnsupported {
            dataset: folder.to_string(),
        })
    }

    /// Download the top-level folder that contains `folder`.
    #[cfg(not(target_arch = "wasm32"))]
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
        let download_result =
            download::try_git_sparse_checkout(&self.root, folder, CRATE_VERSION, REPO_URL)
                .or_else(|_| download::try_http_download(&self.root, folder, CRATE_VERSION));

        cleanup_old_temps(&self.root);
        download_result?;

        write_version_file(&self.root, CRATE_VERSION)?;
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn clear_datasets(&self) {
        if let Ok(entries) = std::fs::read_dir(&self.root) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                // Keep .lock and .tmp-* entries (temps cleaned separately)
                if name_str == ".lock" || name_str.starts_with(".tmp-") {
                    continue;
                }
                // Keep third-party dir — those aren't version-gated
                if name_str == "third-party" {
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
// Used in wasm tests but not in the library on wasm32.
#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
fn write_version_file(root: &std::path::Path, version: &str) -> Result<(), Error> {
    let version_file = root.join(".version");
    let tmp = root.join(".version.tmp");
    std::fs::write(&tmp, version).map_err(Error::Io)?;
    std::fs::rename(&tmp, &version_file).map_err(Error::Io)?;
    Ok(())
}

/// Write a `.fetched` marker with the current Unix timestamp (seconds).
fn write_fetched_marker(dir: &Path) -> Result<(), Error> {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let marker = dir.join(".fetched");
    std::fs::write(&marker, ts.to_string()).map_err(Error::Io)?;
    Ok(())
}

/// Sanitize a user-provided cache key for use as a directory name.
fn sanitize_cache_key(key: &str) -> String {
    key.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Remove `.tmp-*` entries older than 1 hour (orphaned from crashed runs).
#[cfg(not(target_arch = "wasm32"))]
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

    // Pure-logic tests that work on all platforms including wasm32.

    #[test]
    fn test_top_level_folder() {
        assert_eq!(top_level_folder("webp-conformance"), "webp-conformance");
        assert_eq!(
            top_level_folder("webp-conformance/valid"),
            "webp-conformance"
        );
        assert_eq!(top_level_folder("clic2025/training/subdir"), "clic2025");
    }

    #[test]
    fn test_sanitize_cache_key() {
        assert_eq!(sanitize_cache_key("simple-key"), "simple-key");
        assert_eq!(sanitize_cache_key("a/b/c"), "a_b_c");
        assert_eq!(sanitize_cache_key("my key!"), "my_key_");
        assert_eq!(sanitize_cache_key("foo.bar_baz-1"), "foo.bar_baz-1");
    }

    // Tests that use std::env::temp_dir() — not available on wasm32.
    #[cfg(not(target_arch = "wasm32"))]
    mod native {
        use super::*;

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
        fn test_list_cached_includes_third_party() {
            let tmp = std::env::temp_dir().join("codec-corpus-test-list-tp");
            let _ = std::fs::remove_dir_all(&tmp);
            let corpus = Corpus::with_cache_root(&tmp).unwrap();
            // Create fake dataset dirs
            std::fs::create_dir_all(corpus.root.join("alpha")).unwrap();
            std::fs::create_dir_all(corpus.root.join("third-party").join("oss-fuzz__libpng"))
                .unwrap();
            std::fs::create_dir_all(corpus.root.join("third-party").join("go-fuzz-corpus__gif"))
                .unwrap();
            let cached = corpus.list_cached();
            assert_eq!(
                cached,
                vec![
                    "alpha",
                    "third-party/go-fuzz-corpus__gif",
                    "third-party/oss-fuzz__libpng",
                ]
            );
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

        #[test]
        fn test_fetched_marker_staleness() {
            let tmp = std::env::temp_dir().join("codec-corpus-test-staleness");
            let _ = std::fs::remove_dir_all(&tmp);
            let corpus = Corpus::with_cache_root(&tmp)
                .unwrap()
                .with_max_age(Duration::from_secs(3600));
            let dir = corpus.root.join("third-party").join("test-stale");
            std::fs::create_dir_all(&dir).unwrap();

            // No marker = stale
            assert!(corpus.is_stale(&dir));

            // Fresh marker = not stale
            write_fetched_marker(&dir).unwrap();
            assert!(!corpus.is_stale(&dir));

            // Old marker = stale
            let old_ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                - 7200; // 2 hours ago
            std::fs::write(dir.join(".fetched"), old_ts.to_string()).unwrap();
            assert!(corpus.is_stale(&dir));

            let _ = std::fs::remove_dir_all(tmp);
        }

        #[test]
        fn test_local_path_exists() {
            let tmp = std::env::temp_dir().join("codec-corpus-test-local");
            let _ = std::fs::remove_dir_all(&tmp);
            std::fs::create_dir_all(&tmp).unwrap();
            let corpus = Corpus::with_cache_root(&tmp).unwrap();

            let result = corpus.local_path(&tmp);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), tmp);

            let _ = std::fs::remove_dir_all(&tmp);
        }

        #[test]
        fn test_local_path_not_found() {
            let tmp = std::env::temp_dir().join("codec-corpus-test-local-nf");
            let _ = std::fs::remove_dir_all(&tmp);
            let corpus = Corpus::with_cache_root(&tmp).unwrap();

            let result = corpus.local_path("/nonexistent/path/here");
            assert!(matches!(result, Err(Error::LocalPathNotFound { .. })));

            let _ = std::fs::remove_dir_all(tmp);
        }

        #[test]
        fn test_is_cached_third_party() {
            let tmp = std::env::temp_dir().join("codec-corpus-test-tp-cached");
            let _ = std::fs::remove_dir_all(&tmp);
            let corpus = Corpus::with_cache_root(&tmp).unwrap();

            // Not cached initially
            assert!(!corpus.is_cached("oss-fuzz/libpng"));

            // Create fake third-party cache with fresh marker
            let tp_dir = corpus.third_party_dir("oss-fuzz__libpng");
            std::fs::create_dir_all(&tp_dir).unwrap();
            write_fetched_marker(&tp_dir).unwrap();

            assert!(corpus.is_cached("oss-fuzz/libpng"));

            let _ = std::fs::remove_dir_all(tmp);
        }

        #[test]
        fn test_clear_datasets_preserves_third_party() {
            let tmp = std::env::temp_dir().join("codec-corpus-test-clear-tp");
            let _ = std::fs::remove_dir_all(&tmp);
            let corpus = Corpus::with_cache_root(&tmp).unwrap();

            // Create imazen dataset and third-party dataset
            std::fs::create_dir_all(corpus.root.join("pngsuite")).unwrap();
            let tp_dir = corpus.root.join("third-party").join("some-source");
            std::fs::create_dir_all(&tp_dir).unwrap();
            std::fs::write(tp_dir.join("data.bin"), b"hello").unwrap();

            corpus.clear_datasets();

            // imazen dataset should be gone
            assert!(!corpus.root.join("pngsuite").exists());
            // third-party should survive
            assert!(tp_dir.exists());
            assert!(tp_dir.join("data.bin").exists());

            let _ = std::fs::remove_dir_all(tmp);
        }

        #[test]
        fn test_third_party_subpath() {
            let tmp = std::env::temp_dir().join("codec-corpus-test-tp-sub");
            let _ = std::fs::remove_dir_all(&tmp);
            let corpus = Corpus::with_cache_root(&tmp).unwrap();

            let src = registry::ResolvedSource {
                cache_key: "oss-fuzz__libpng".to_string(),
                kind: registry::SourceKind::Zip { url: String::new() },
            };

            assert_eq!(corpus.third_party_subpath("oss-fuzz/libpng", &src), None);
            assert_eq!(
                corpus.third_party_subpath("oss-fuzz/libpng/some-file", &src),
                Some("some-file".to_string())
            );

            let src2 = registry::ResolvedSource {
                cache_key: "go-fuzz-corpus__gif".to_string(),
                kind: registry::SourceKind::Zip { url: String::new() },
            };

            assert_eq!(
                corpus.third_party_subpath("go-fuzz-corpus/gif", &src2),
                None
            );
            assert_eq!(
                corpus.third_party_subpath("go-fuzz-corpus/gif/sub/file.gif", &src2),
                Some("sub/file.gif".to_string())
            );

            let _ = std::fs::remove_dir_all(tmp);
        }

        #[test]
        fn test_with_max_age() {
            let tmp = std::env::temp_dir().join("codec-corpus-test-max-age");
            let _ = std::fs::remove_dir_all(&tmp);
            let corpus = Corpus::with_cache_root(&tmp)
                .unwrap()
                .with_max_age(Duration::from_secs(60));
            assert_eq!(corpus.max_age, Duration::from_secs(60));
            let _ = std::fs::remove_dir_all(tmp);
        }
    }

    // Wasm32-specific test: exercises the read-only cache path using a
    // pre-populated directory under the preopened cwd.
    #[cfg(target_arch = "wasm32")]
    mod wasm {
        use super::*;

        #[test]
        fn test_read_only_cache() {
            let cwd = std::env::current_dir().expect("cwd must be preopened via --dir");
            let tmp = cwd.join(".corpus-test-tmp");

            // Clean up any leftover from a prior run.
            let _ = std::fs::remove_dir_all(&tmp);

            // Build a fake cache layout that Corpus::with_cache_root expects:
            //   {tmp}/codec-corpus/v1/fake-dataset/
            //   {tmp}/codec-corpus/v1/.version  (matching crate version)
            let corpus = Corpus::with_cache_root(&tmp).unwrap();
            let dataset_dir = corpus.root.join("fake-dataset");
            std::fs::create_dir_all(&dataset_dir).unwrap();
            std::fs::write(dataset_dir.join("sample.bin"), b"hello wasm").unwrap();
            write_version_file(&corpus.root, CRATE_VERSION).unwrap();

            // Verify is_cached returns true for the fake dataset.
            assert!(
                corpus.is_cached("fake-dataset"),
                "expected fake-dataset to be cached"
            );

            // Verify list_cached includes it.
            let cached = corpus.list_cached();
            assert!(
                cached.contains(&"fake-dataset".to_string()),
                "expected list_cached to contain fake-dataset, got: {cached:?}"
            );

            // Verify get() returns the cached path without hitting the network.
            let path = corpus
                .get("fake-dataset")
                .expect("get() should succeed for a cached dataset on wasm");
            assert!(path.is_dir(), "expected returned path to be a directory");
            assert!(
                path.join("sample.bin").exists(),
                "expected sample.bin in the returned directory"
            );

            // Verify get() for an uncached dataset returns DownloadUnsupported.
            let err = corpus
                .get("uncached-dataset")
                .expect_err("get() for uncached dataset should fail on wasm");
            assert!(
                matches!(err, Error::DownloadUnsupported { .. }),
                "expected DownloadUnsupported, got: {err}"
            );

            // Clean up.
            let _ = std::fs::remove_dir_all(&tmp);
        }
    }
}
