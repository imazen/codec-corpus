use std::path::{Path, PathBuf};

use crate::Error;
use crate::download;
use crate::manifest::{self, ExpectedBehavior, ManifestEntry};

pub const DEFAULT_MANIFEST_URL: &str =
    "https://pub-7c5c57fd3e0842f0b147946928891d40.r2.dev/manifest.jsonl";
pub const DEFAULT_BLOB_BASE_URL: &str = "https://pub-7c5c57fd3e0842f0b147946928891d40.r2.dev";
pub const ENV_MANIFEST: &str = "CODEC_CORPUS_MANIFEST";
pub const ENV_BASE_URL: &str = "CODEC_CORPUS_BASE_URL";

/// A fetched blob: its manifest entry and the local path on disk.
#[derive(Debug)]
pub struct FetchedBlob {
    pub entry: ManifestEntry,
    pub path: PathBuf,
}

/// Handle for accessing content-addressable blobs via a JSONL manifest.
///
/// Unlike [`crate::Corpus`] (which fetches curated datasets from GitHub),
/// `ManifestCorpus` fetches individual blobs from a Cloudflare R2 bucket
/// based on a JSONL manifest of 100k+ images.
pub struct ManifestCorpus {
    cache_dir: PathBuf,
    base_url: String,
    entries: Vec<ManifestEntry>,
}

impl ManifestCorpus {
    /// Load from the default R2 manifest URL, or `CODEC_CORPUS_MANIFEST` env
    /// var if set. Base URL is `CODEC_CORPUS_BASE_URL` or the default R2 bucket.
    pub fn from_env() -> Result<Self, Error> {
        let manifest_url =
            std::env::var(ENV_MANIFEST).unwrap_or_else(|_| DEFAULT_MANIFEST_URL.to_string());
        let base_url =
            std::env::var(ENV_BASE_URL).unwrap_or_else(|_| DEFAULT_BLOB_BASE_URL.to_string());
        let cache_dir = Self::resolve_cache_dir()?;
        Self::download_and_parse(&manifest_url, &base_url, cache_dir)
    }

    /// Download and parse a manifest from `manifest_url`, using the default
    /// blob base URL and default cache directory.
    pub fn from_url(manifest_url: &str) -> Result<Self, Error> {
        let cache_dir = Self::resolve_cache_dir()?;
        Self::download_and_parse(manifest_url, DEFAULT_BLOB_BASE_URL, cache_dir)
    }

    /// Download and parse a manifest, using an explicit cache root.
    ///
    /// Blobs and manifest are stored under `{cache_root}/codec-corpus/manifests/`.
    pub fn with_cache_root(
        cache_root: impl Into<PathBuf>,
        manifest_url: &str,
    ) -> Result<Self, Error> {
        let cache_dir = cache_root.into().join("codec-corpus").join("manifests");
        let base_url =
            std::env::var(ENV_BASE_URL).unwrap_or_else(|_| DEFAULT_BLOB_BASE_URL.to_string());
        Self::download_and_parse(manifest_url, &base_url, cache_dir)
    }

    /// Load a manifest from a local file.
    pub fn from_file(path: &Path) -> Result<Self, Error> {
        let base_url =
            std::env::var(ENV_BASE_URL).unwrap_or_else(|_| DEFAULT_BLOB_BASE_URL.to_string());
        let text = std::fs::read_to_string(path)?;
        let entries = manifest::parse_manifest(&text);
        if entries.is_empty() {
            return Err(Error::ManifestParse(
                "manifest contained no valid entries".into(),
            ));
        }
        let cache_dir = Self::resolve_cache_dir()?;
        Ok(Self {
            cache_dir,
            base_url,
            entries,
        })
    }

    /// Number of entries in the manifest.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the manifest is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// All manifest entries.
    pub fn entries(&self) -> &[ManifestEntry] {
        &self.entries
    }

    /// Create a filter over this corpus's entries.
    pub fn filter(&self) -> Filter<'_> {
        Filter {
            corpus: self,
            format: None,
            source: None,
            source_label: None,
            expected_behavior: None,
            max_file_size: None,
        }
    }

    /// Fetch a single blob to the local cache. Returns the local path.
    ///
    /// If the blob is already cached, returns immediately.
    pub fn fetch_one(&self, entry: &ManifestEntry) -> Result<PathBuf, Error> {
        let local = self.blob_path(&entry.sha256);
        if local.exists() {
            return Ok(local);
        }

        let url = self.blob_url(&entry.sha256);
        if let Some(parent) = local.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Download to a temp file, then rename for atomicity.
        let tmp = local.with_extension("tmp");
        let result = download::download_file(&url, &tmp);
        if result.is_err() {
            let _ = std::fs::remove_file(&tmp);
            return Err(Error::BlobFetch {
                sha256: entry.sha256.clone(),
            });
        }

        std::fs::rename(&tmp, &local)?;
        Ok(local)
    }

    /// Check if a blob is already in the local cache.
    pub fn is_cached(&self, sha256: &str) -> bool {
        self.blob_path(sha256).exists()
    }

    // -----------------------------------------------------------------------
    // Private
    // -----------------------------------------------------------------------

    fn download_and_parse(
        manifest_url: &str,
        base_url: &str,
        cache_dir: PathBuf,
    ) -> Result<Self, Error> {
        let manifest_path = cache_dir.join("manifest.jsonl");

        // Download manifest if not cached.
        if !manifest_path.exists() {
            std::fs::create_dir_all(&cache_dir)?;
            let tmp = manifest_path.with_extension("tmp");
            let result = download::download_file(manifest_url, &tmp);
            if result.is_err() {
                let _ = std::fs::remove_file(&tmp);
                return Err(Error::ManifestParse("failed to download manifest".into()));
            }
            std::fs::rename(&tmp, &manifest_path)?;
        }

        let text = std::fs::read_to_string(&manifest_path)?;
        let entries = manifest::parse_manifest(&text);
        if entries.is_empty() {
            // Remove bad cached manifest so next attempt re-downloads.
            let _ = std::fs::remove_file(&manifest_path);
            return Err(Error::ManifestParse(
                "manifest contained no valid entries".into(),
            ));
        }

        Ok(Self {
            cache_dir,
            base_url: base_url.to_string(),
            entries,
        })
    }

    fn resolve_cache_dir() -> Result<PathBuf, Error> {
        let base = if let Ok(val) = std::env::var("CODEC_CORPUS_CACHE") {
            PathBuf::from(val)
        } else {
            dirs::cache_dir().ok_or(Error::NoCacheDir)?
        };
        Ok(base.join("codec-corpus").join("manifests"))
    }

    fn blob_path(&self, sha256: &str) -> PathBuf {
        let (a, rest) = sha256.split_at(2.min(sha256.len()));
        let (b, _) = rest.split_at(2.min(rest.len()));
        self.cache_dir.join("blobs").join(a).join(b).join(sha256)
    }

    fn blob_url(&self, sha256: &str) -> String {
        let (a, rest) = sha256.split_at(2.min(sha256.len()));
        let (b, _) = rest.split_at(2.min(rest.len()));
        format!("{}/blobs/{a}/{b}/{sha256}", self.base_url)
    }
}

/// Builder for filtering manifest entries by various criteria.
pub struct Filter<'a> {
    corpus: &'a ManifestCorpus,
    format: Option<String>,
    source: Option<String>,
    source_label: Option<String>,
    expected_behavior: Option<ExpectedBehavior>,
    max_file_size: Option<u64>,
}

impl<'a> Filter<'a> {
    /// Only include entries with this format (e.g. "png", "webp").
    pub fn format(mut self, fmt: &str) -> Self {
        self.format = Some(fmt.to_string());
        self
    }

    /// Only include entries from this source (e.g. "github-issues", "built").
    pub fn source(mut self, src: &str) -> Self {
        self.source = Some(src.to_string());
        self
    }

    /// Only include entries with this source label (e.g. "corpus/png-8").
    pub fn source_label(mut self, label: &str) -> Self {
        self.source_label = Some(label.to_string());
        self
    }

    /// Only include entries with this expected behavior.
    pub fn expected_behavior(mut self, eb: ExpectedBehavior) -> Self {
        self.expected_behavior = Some(eb);
        self
    }

    /// Only include entries up to this file size in bytes.
    pub fn max_file_size(mut self, max: u64) -> Self {
        self.max_file_size = Some(max);
        self
    }

    /// Return matching entries.
    pub fn entries(&self) -> Vec<&'a ManifestEntry> {
        self.corpus
            .entries
            .iter()
            .filter(|e| self.matches(e))
            .collect()
    }

    /// Count of matching entries.
    pub fn count(&self) -> usize {
        self.corpus
            .entries
            .iter()
            .filter(|e| self.matches(e))
            .count()
    }

    /// Fetch all matching blobs to the local cache.
    pub fn fetch(&self) -> Result<Vec<FetchedBlob>, Error> {
        let matching = self.entries();
        let mut results = Vec::with_capacity(matching.len());
        for entry in matching {
            let path = self.corpus.fetch_one(entry)?;
            results.push(FetchedBlob {
                entry: entry.clone(),
                path,
            });
        }
        Ok(results)
    }

    fn matches(&self, entry: &ManifestEntry) -> bool {
        if let Some(ref fmt) = self.format {
            if entry.format.as_deref() != Some(fmt.as_str()) {
                return false;
            }
        }
        if let Some(ref src) = self.source {
            if entry.source != *src {
                return false;
            }
        }
        if let Some(ref label) = self.source_label {
            if entry.source_label != *label {
                return false;
            }
        }
        if let Some(eb) = self.expected_behavior {
            if entry.expected_behavior != eb {
                return false;
            }
        }
        if let Some(max) = self.max_file_size {
            if entry.file_size > max {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_entries() -> Vec<ManifestEntry> {
        let jsonl = r#"{"sha256":"aaa1","format":"png","file_size":100,"source":"built","source_label":"corpus/png-8","expected_behavior":"must_not_crash","confidence":0.0}
{"sha256":"bbb2","format":"webp","file_size":5000,"source":"internet","source_label":"scraping/webp","expected_behavior":"should_decode","confidence":0.9}
{"sha256":"ccc3","format":null,"file_size":50,"source":"github-issues","source_label":"github-issues","expected_behavior":"unknown","confidence":0.0}"#;
        manifest::parse_manifest(jsonl)
    }

    #[test]
    fn blob_path_layout() {
        let corpus = ManifestCorpus {
            cache_dir: PathBuf::from("/cache/manifests"),
            base_url: DEFAULT_BLOB_BASE_URL.to_string(),
            entries: vec![],
        };
        let p = corpus.blob_path("abcdef0123456789");
        assert_eq!(
            p,
            PathBuf::from("/cache/manifests/blobs/ab/cd/abcdef0123456789")
        );
    }

    #[test]
    fn blob_url_layout() {
        let corpus = ManifestCorpus {
            cache_dir: PathBuf::from("/cache"),
            base_url: "https://example.com".to_string(),
            entries: vec![],
        };
        let url = corpus.blob_url("abcdef0123456789");
        assert_eq!(url, "https://example.com/blobs/ab/cd/abcdef0123456789");
    }

    #[test]
    fn filter_by_format() {
        let entries = sample_entries();
        let corpus = ManifestCorpus {
            cache_dir: PathBuf::from("/cache"),
            base_url: DEFAULT_BLOB_BASE_URL.to_string(),
            entries,
        };
        let png = corpus.filter().format("png").entries();
        assert_eq!(png.len(), 1);
        assert_eq!(png[0].sha256, "aaa1");
    }

    #[test]
    fn filter_by_source() {
        let entries = sample_entries();
        let corpus = ManifestCorpus {
            cache_dir: PathBuf::from("/cache"),
            base_url: DEFAULT_BLOB_BASE_URL.to_string(),
            entries,
        };
        assert_eq!(corpus.filter().source("built").count(), 1);
        assert_eq!(corpus.filter().source("internet").count(), 1);
        assert_eq!(corpus.filter().source("github-issues").count(), 1);
    }

    #[test]
    fn filter_by_max_size() {
        let entries = sample_entries();
        let corpus = ManifestCorpus {
            cache_dir: PathBuf::from("/cache"),
            base_url: DEFAULT_BLOB_BASE_URL.to_string(),
            entries,
        };
        assert_eq!(corpus.filter().max_file_size(100).count(), 2);
        assert_eq!(corpus.filter().max_file_size(49).count(), 0);
    }

    #[test]
    fn filter_combined() {
        let entries = sample_entries();
        let corpus = ManifestCorpus {
            cache_dir: PathBuf::from("/cache"),
            base_url: DEFAULT_BLOB_BASE_URL.to_string(),
            entries,
        };
        let result = corpus
            .filter()
            .source("built")
            .format("png")
            .max_file_size(200)
            .entries();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].sha256, "aaa1");
    }

    #[test]
    fn filter_no_match() {
        let entries = sample_entries();
        let corpus = ManifestCorpus {
            cache_dir: PathBuf::from("/cache"),
            base_url: DEFAULT_BLOB_BASE_URL.to_string(),
            entries,
        };
        assert_eq!(corpus.filter().format("bmp").count(), 0);
    }
}
