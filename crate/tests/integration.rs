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
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "png"))
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
    assert!(result.is_err(), "expected error for nonexistent folder");

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
                let corpus = Corpus::with_cache_root(&tmp).expect("failed to create corpus");
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

// ═══════════════════════════════════════════════════════════════════════════
// Third-party source tests
// ═══════════════════════════════════════════════════════════════════════════

// ── go-fuzz-corpus (git sparse checkout) ───────────────────────────────

#[test]
#[ignore]
fn download_go_fuzz_corpus_gif() {
    let (corpus, tmp) = corpus_in_tmp("go-fuzz-gif");

    assert!(!corpus.is_cached("go-fuzz-corpus/gif"));

    let dir = corpus.get("go-fuzz-corpus/gif").expect("download failed");
    assert!(dir.is_dir(), "expected directory at {}", dir.display());

    // Should have some files (the go-fuzz gif corpus has many small files)
    let count = std::fs::read_dir(&dir).unwrap().count();
    assert!(
        count > 0,
        "expected files in go-fuzz-corpus/gif, got {count}"
    );

    assert!(corpus.is_cached("go-fuzz-corpus/gif"));

    // Second call should be fast (cached)
    let start = std::time::Instant::now();
    let _ = corpus
        .get("go-fuzz-corpus/gif")
        .expect("cached access failed");
    let elapsed = start.elapsed();
    assert!(
        elapsed.as_millis() < 50,
        "cached access took {}ms, expected <50ms",
        elapsed.as_millis()
    );

    cleanup(&tmp);
}

// ── libjpeg-turbo fuzz seed corpus (git sparse checkout) ───────────────
//
// Every file in the upstream tree has an AFL-style name containing ':',
// which NTFS rejects, so the checkout is impossible on Windows. There the
// crate reports `DownloadUnsupported` up front (asserted below, no network
// needed); elsewhere the real download is exercised.

#[cfg(windows)]
#[test]
fn libjpeg_turbo_fuzz_unsupported_on_windows() {
    let (corpus, tmp) = corpus_in_tmp("ljt-fuzz-win");

    let err = corpus
        .get("libjpeg-turbo-fuzz")
        .expect_err("libjpeg-turbo-fuzz must not be fetchable on Windows");
    assert!(
        matches!(err, codec_corpus::Error::DownloadUnsupported { ref dataset } if dataset == "libjpeg-turbo-fuzz"),
        "expected DownloadUnsupported, got {err:?}"
    );
    assert!(!corpus.is_cached("libjpeg-turbo-fuzz"));

    cleanup(&tmp);
}

#[cfg(not(windows))]
#[test]
#[ignore]
fn download_libjpeg_turbo_fuzz() {
    let (corpus, tmp) = corpus_in_tmp("ljt-fuzz");

    let dir = corpus.get("libjpeg-turbo-fuzz").expect("download failed");
    assert!(dir.is_dir());

    // Should have some JPEG seed files
    let files: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    assert!(
        !files.is_empty(),
        "expected files in libjpeg-turbo fuzz seed corpus"
    );

    cleanup(&tmp);
}

// ── Ad-hoc GitHub repo subfolder ───────────────────────────────────────

#[test]
#[ignore]
fn adhoc_github_repo() {
    let (corpus, tmp) = corpus_in_tmp("adhoc-github");

    // Fetch a small, nested subfolder of a repo we control (the third-party
    // repo used here previously returns GitHub 404, which made this test fail).
    // The nested `repo_path` also exercises cone-mode sparse checkout below
    // the top level.
    let dir = corpus
        .github_repo("imazen/codec-corpus", "farbfeld-conformance/valid", "main")
        .expect("download failed");
    assert!(dir.is_dir(), "expected directory at {}", dir.display());

    let count = std::fs::read_dir(&dir).unwrap().count();
    assert!(
        count > 0,
        "expected files in codec-corpus/farbfeld-conformance/valid, got {count}"
    );

    // list_cached should include it under third-party/
    let cached = corpus.list_cached();
    assert!(
        cached.iter().any(|s| s.starts_with("third-party/")),
        "expected third-party entry in cached list, got: {cached:?}"
    );

    cleanup(&tmp);
}

// ── Ad-hoc local path ──────────────────────────────────────────────────

#[test]
fn local_path_validation() {
    let tmp = std::env::temp_dir().join("codec-corpus-integ-local");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();

    let corpus = Corpus::with_cache_root(&tmp).expect("failed to create corpus");

    // Existing dir succeeds
    let result = corpus.local_path(&tmp);
    assert!(result.is_ok());

    // Non-existent dir fails
    let result = corpus.local_path("/this/does/not/exist");
    assert!(result.is_err());

    cleanup(&tmp);
}

// ── OSS-Fuzz corpus (ZIP download) ─────────────────────────────────────
// Note: OSS-Fuzz corpora can be very large (100s of MB). This test downloads
// a real corpus from GCS, so it's slow and bandwidth-intensive. Only enable
// when specifically testing ZIP download functionality.

#[test]
#[ignore]
fn download_oss_fuzz_libjpeg_turbo() {
    let (corpus, tmp) = corpus_in_tmp("oss-fuzz-ljt");

    let dir = corpus
        .get("oss-fuzz/libjpeg-turbo")
        .expect("download failed");
    assert!(dir.is_dir(), "expected directory at {}", dir.display());

    let count = std::fs::read_dir(&dir).unwrap().count();
    assert!(
        count > 0,
        "expected files in oss-fuzz/libjpeg-turbo, got {count}"
    );

    cleanup(&tmp);
}

// ── Third-party is_cached before download ──────────────────────────────

#[test]
fn third_party_not_cached_initially() {
    let (corpus, tmp) = corpus_in_tmp("tp-not-cached");

    assert!(!corpus.is_cached("oss-fuzz/libpng"));
    assert!(!corpus.is_cached("go-fuzz-corpus/gif"));
    assert!(!corpus.is_cached("libjpeg-turbo-fuzz"));
    assert!(!corpus.is_cached("image-rs/tests/images"));

    cleanup(&tmp);
}

// ── Ad-hoc ZIP with invalid URL returns DownloadFailed ─────────────────

#[test]
fn adhoc_zip_bad_url() {
    let (corpus, tmp) = corpus_in_tmp("bad-zip-url");

    let result = corpus.zip_url("bad-corpus", "https://httpbin.org/status/404");
    assert!(result.is_err(), "expected error for non-existent ZIP URL");

    cleanup(&tmp);
}

// ── Ad-hoc tar with invalid URL returns DownloadFailed ─────────────────

#[test]
fn adhoc_tar_bad_url() {
    let (corpus, tmp) = corpus_in_tmp("bad-tar-url");

    let result = corpus.tar_url("bad-corpus", "https://httpbin.org/status/404");
    assert!(result.is_err(), "expected error for non-existent tar URL");

    cleanup(&tmp);
}
