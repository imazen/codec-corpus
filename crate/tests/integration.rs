//! Integration tests that exercise real downloads.
//!
//! All tests are `#[ignore]` — they require network access and are run
//! separately via `cargo test -- --ignored`.

use codec_corpus::Corpus;
use std::path::Path;

/// Helper: create a corpus with a per-test temp cache root.
fn corpus_in_tmp(name: &str) -> (Corpus, std::path::PathBuf) {
    let tmp = std::env::temp_dir().join(format!("codec-corpus-integ-{name}"));
    let _ = std::fs::remove_dir_all(&tmp);
    let c = Corpus::with_cache_root(&tmp).expect("failed to create corpus");
    (c, tmp)
}

fn cleanup(tmp: &Path) {
    let _ = std::fs::remove_dir_all(tmp);
}

// ── Download a small dataset ────────────────────────────────────────────

#[test]
#[ignore]
fn download_pngsuite() {
    let (corpus, tmp) = corpus_in_tmp("pngsuite");

    assert!(!corpus.is_cached("pngsuite"));

    let dir = corpus.get("pngsuite").expect("download failed");
    assert!(dir.is_dir(), "expected directory at {}", dir.display());

    // Should have some .png files
    let pngs: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|e| {
            e.path()
                .extension()
                .map_or(false, |ext| ext == "png")
        })
        .collect();
    assert!(!pngs.is_empty(), "expected png files in pngsuite");

    assert!(corpus.is_cached("pngsuite"));

    cleanup(&tmp);
}

// ── Subpath access ──────────────────────────────────────────────────────

#[test]
#[ignore]
fn download_webp_subdir() {
    let (corpus, tmp) = corpus_in_tmp("webp-subdir");

    // Request a subdirectory — should download the whole root folder
    let valid = corpus.get("webp-conformance/valid");
    match valid {
        Ok(dir) => {
            assert!(dir.is_dir());
            // The parent folder should also exist now
            assert!(corpus.is_cached("webp-conformance"));
        }
        Err(codec_corpus::Error::PathNotFound { .. }) => {
            // The subdir may not exist in the repo layout — that's fine,
            // the root folder should still have been downloaded.
            assert!(
                corpus.is_cached("webp-conformance"),
                "root folder should be cached even if subdir doesn't exist"
            );
        }
        Err(e) => panic!("unexpected error: {e}"),
    }

    cleanup(&tmp);
}

// ── Caching: second call is fast ────────────────────────────────────────

#[test]
#[ignore]
fn cached_access_is_fast() {
    let (corpus, tmp) = corpus_in_tmp("cache-speed");

    // First call downloads
    let _ = corpus.get("pngsuite").expect("download failed");

    // Second call should be instant (< 50ms)
    let start = std::time::Instant::now();
    let _ = corpus.get("pngsuite").expect("cached access failed");
    let elapsed = start.elapsed();
    assert!(
        elapsed.as_millis() < 50,
        "cached access took {}ms, expected <50ms",
        elapsed.as_millis()
    );

    cleanup(&tmp);
}

// ── list_cached reflects downloads ──────────────────────────────────────

#[test]
#[ignore]
fn list_cached_after_download() {
    let (corpus, tmp) = corpus_in_tmp("list-cached");

    assert!(corpus.list_cached().is_empty());

    let _ = corpus.get("pngsuite").expect("download failed");
    let cached = corpus.list_cached();
    assert!(
        cached.contains(&"pngsuite".to_string()),
        "expected pngsuite in cached list, got: {:?}",
        cached
    );

    cleanup(&tmp);
}

// ── Nonexistent path returns error, not panic ───────────────────────────

#[test]
#[ignore]
fn nonexistent_path_is_error() {
    let (corpus, tmp) = corpus_in_tmp("nonexistent");

    let result = corpus.get("this-folder-does-not-exist-in-the-repo");
    assert!(
        result.is_err(),
        "expected error for nonexistent folder"
    );

    cleanup(&tmp);
}

// ── Concurrent access doesn't corrupt ───────────────────────────────────

#[test]
#[ignore]
fn concurrent_downloads() {
    let tmp = std::env::temp_dir().join("codec-corpus-integ-concurrent");
    let _ = std::fs::remove_dir_all(&tmp);

    let handles: Vec<_> = (0..4)
        .map(|_| {
            let tmp = tmp.clone();
            std::thread::spawn(move || {
                let corpus =
                    Corpus::with_cache_root(&tmp).expect("failed to create corpus");
                corpus.get("pngsuite").expect("download failed");
            })
        })
        .collect();

    for h in handles {
        h.join().expect("thread panicked");
    }

    // Verify the result is intact
    let corpus = Corpus::with_cache_root(&tmp).expect("failed to create corpus");
    let dir = corpus.get("pngsuite").expect("final access failed");
    assert!(dir.is_dir());

    cleanup(&tmp);
}
