//! Built-in registry of known third-party corpus sources.
//!
//! Maps path prefixes to download strategies so `Corpus::get()` can resolve
//! third-party datasets transparently.

/// A download strategy for a third-party source.
#[derive(Debug, Clone)]
pub(crate) enum SourceKind {
    /// Download a ZIP from a URL template, extract to cache.
    ///
    /// The URL is stored verbatim (already expanded).
    Zip { url: String },

    /// Git sparse checkout of a subfolder from a GitHub repo.
    GitHubSubfolder {
        /// GitHub repo in `owner/repo` format.
        repo: &'static str,
        /// Path within the repo to sparse-checkout.
        repo_path: &'static str,
        /// Branch to clone (None = default branch).
        branch: Option<&'static str>,
    },
}

/// A resolved third-party source entry.
#[derive(Debug, Clone)]
pub(crate) struct ResolvedSource {
    /// Cache key — used as the directory name under `third-party/`.
    pub cache_key: String,
    /// How to download it.
    pub kind: SourceKind,
}

/// Try to resolve a path prefix against the built-in registry.
///
/// Returns `Some(ResolvedSource)` if the path matches a known third-party
/// prefix, `None` otherwise (meaning it should fall through to the default
/// imazen/codec-corpus download).
pub(crate) fn resolve(path: &str) -> Option<ResolvedSource> {
    // OSS-Fuzz corpora: oss-fuzz/{project}
    if let Some(project) = path.strip_prefix("oss-fuzz/") {
        let project = project.split('/').next().unwrap_or(project);
        return resolve_oss_fuzz(project);
    }

    // go-fuzz-corpus: go-fuzz-corpus/{format}
    if let Some(rest) = path.strip_prefix("go-fuzz-corpus/") {
        let format = rest.split('/').next().unwrap_or(rest);
        return Some(ResolvedSource {
            cache_key: format!("go-fuzz-corpus__{format}"),
            kind: SourceKind::GitHubSubfolder {
                repo: "dvyukov/go-fuzz-corpus",
                repo_path: leak_string(format!("{format}/corpus")),
                branch: None,
            },
        });
    }

    // libjpeg-turbo fuzz seed corpus. The seeds live in
    // libjpeg-turbo/seed-corpora (afl-testcases/{bmp,gif,gif_im,jpeg,
    // jpeg_turbo,targa}); OSS-Fuzz's projects/libjpeg-turbo/Dockerfile zips
    // that tree into the fuzzer seed corpora. The libjpeg-turbo/fuzz repo only
    // holds branches.txt — it never carried a seed_corpus directory.
    if path == "libjpeg-turbo-fuzz" || path.starts_with("libjpeg-turbo-fuzz/") {
        return Some(ResolvedSource {
            cache_key: "libjpeg-turbo-fuzz".to_string(),
            kind: SourceKind::GitHubSubfolder {
                repo: "libjpeg-turbo/seed-corpora",
                repo_path: "afl-testcases",
                branch: Some("main"),
            },
        });
    }

    // image-rs test images: image-rs/{subpath}
    // Sparse-checks out the first path component (e.g., "tests" from
    // "tests/images/jpg"). All requests under the same first component
    // share one clone. Subpath navigation happens in third_party_subpath.
    if let Some(rest) = path.strip_prefix("image-rs/") {
        let checkout_dir = rest.split('/').next().unwrap_or(rest);
        return Some(ResolvedSource {
            cache_key: format!("image-rs__{checkout_dir}"),
            kind: SourceKind::GitHubSubfolder {
                repo: "image-rs/image",
                repo_path: leak_string(checkout_dir.to_string()),
                branch: Some("main"),
            },
        });
    }

    None
}

/// Resolve an OSS-Fuzz project name to the specific fuzzer corpus URL.
///
/// OSS-Fuzz backup corpora live at:
/// `https://storage.googleapis.com/{project}-backup.clusterfuzz-external.appspot.com/corpus/libFuzzer/{project}_{fuzzer}/public.zip`
fn resolve_oss_fuzz(project: &str) -> Option<ResolvedSource> {
    let fuzzer = match project {
        "libjpeg-turbo" => "cjpeg_fuzzer",
        "libpng" => "read_fuzzer",
        "libjxl" => "djxl_fuzzer",
        _ => return None,
    };

    let url = format!(
        "https://storage.googleapis.com/{project}-backup.clusterfuzz-external.appspot.com\
         /corpus/libFuzzer/{project}_{fuzzer}/public.zip"
    );

    Some(ResolvedSource {
        cache_key: format!("oss-fuzz__{project}"),
        kind: SourceKind::Zip { url },
    })
}

/// Leak a String to get a `&'static str`.
///
/// Used for constructing repo_path values at runtime. The leaked memory is
/// tiny (a few path strings per process lifetime) and lives until exit.
fn leak_string(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oss_fuzz_libjpeg_turbo() {
        let src = resolve("oss-fuzz/libjpeg-turbo").unwrap();
        assert_eq!(src.cache_key, "oss-fuzz__libjpeg-turbo");
        match &src.kind {
            SourceKind::Zip { url } => {
                assert!(url.contains("libjpeg-turbo-backup"));
                assert!(url.contains("cjpeg_fuzzer"));
                assert!(url.ends_with("public.zip"));
            }
            other => panic!("expected Zip, got {other:?}"),
        }
    }

    #[test]
    fn oss_fuzz_libpng() {
        let src = resolve("oss-fuzz/libpng").unwrap();
        assert_eq!(src.cache_key, "oss-fuzz__libpng");
        match &src.kind {
            SourceKind::Zip { url } => {
                assert!(url.contains("libpng-backup"));
                assert!(url.contains("read_fuzzer"));
            }
            other => panic!("expected Zip, got {other:?}"),
        }
    }

    #[test]
    fn oss_fuzz_libjxl() {
        let src = resolve("oss-fuzz/libjxl").unwrap();
        assert_eq!(src.cache_key, "oss-fuzz__libjxl");
        match &src.kind {
            SourceKind::Zip { url } => {
                assert!(url.contains("libjxl-backup"));
                assert!(url.contains("djxl_fuzzer"));
            }
            other => panic!("expected Zip, got {other:?}"),
        }
    }

    #[test]
    fn oss_fuzz_unknown_project() {
        assert!(resolve("oss-fuzz/unknown-project").is_none());
    }

    #[test]
    fn go_fuzz_corpus_gif() {
        let src = resolve("go-fuzz-corpus/gif").unwrap();
        assert_eq!(src.cache_key, "go-fuzz-corpus__gif");
        match &src.kind {
            SourceKind::GitHubSubfolder {
                repo, repo_path, ..
            } => {
                assert_eq!(*repo, "dvyukov/go-fuzz-corpus");
                assert_eq!(*repo_path, "gif/corpus");
            }
            other => panic!("expected GitHubSubfolder, got {other:?}"),
        }
    }

    #[test]
    fn go_fuzz_corpus_png() {
        let src = resolve("go-fuzz-corpus/png").unwrap();
        assert_eq!(src.cache_key, "go-fuzz-corpus__png");
    }

    #[test]
    fn libjpeg_turbo_fuzz() {
        let src = resolve("libjpeg-turbo-fuzz").unwrap();
        assert_eq!(src.cache_key, "libjpeg-turbo-fuzz");
        match &src.kind {
            SourceKind::GitHubSubfolder {
                repo,
                repo_path,
                branch,
            } => {
                assert_eq!(*repo, "libjpeg-turbo/seed-corpora");
                assert_eq!(*repo_path, "afl-testcases");
                assert_eq!(*branch, Some("main"));
            }
            other => panic!("expected GitHubSubfolder, got {other:?}"),
        }
    }

    #[test]
    fn libjpeg_turbo_fuzz_subpath() {
        let src = resolve("libjpeg-turbo-fuzz/some-file.jpg").unwrap();
        assert_eq!(src.cache_key, "libjpeg-turbo-fuzz");
    }

    #[test]
    fn image_rs_tests_images() {
        let src = resolve("image-rs/tests/images").unwrap();
        // Cache key is based on the checkout unit (first component), not full path
        assert_eq!(src.cache_key, "image-rs__tests");
        match &src.kind {
            SourceKind::GitHubSubfolder {
                repo, repo_path, ..
            } => {
                assert_eq!(*repo, "image-rs/image");
                assert_eq!(*repo_path, "tests");
            }
            other => panic!("expected GitHubSubfolder, got {other:?}"),
        }
    }

    #[test]
    fn image_rs_subpath_shares_cache() {
        // Different subpaths under the same checkout dir share one cache entry
        let a = resolve("image-rs/tests/images").unwrap();
        let b = resolve("image-rs/tests/images/jpg/progressive").unwrap();
        assert_eq!(a.cache_key, b.cache_key);
        assert_eq!(a.cache_key, "image-rs__tests");
    }

    #[test]
    fn unrecognized_prefix_returns_none() {
        assert!(resolve("jpeg-conformance").is_none());
        assert!(resolve("pngsuite").is_none());
        assert!(resolve("webp-conformance/valid").is_none());
    }
}
