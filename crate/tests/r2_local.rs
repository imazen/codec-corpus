//! End-to-end `R2Corpus` test through the real download chain, with no
//! network: a local directory is served via `file://` URLs (which `curl`
//! handles), so the actual `curl` shell-out, SHA-256 verification, bundle
//! download, and system-`tar` extraction all run for real.
//!
//! Not `#[ignore]`d: it needs `curl` and `tar`, which every CI runner this
//! crate targets ships, and which the crate already requires at runtime. If
//! either is missing this fails loudly rather than skipping.

#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;

use codec_corpus::{
    BundleInfo, Error, FileEntry, LIST_VERSION, ListIndex, PullMode, PullOptions, R2Corpus,
};

const PREFIX: &str = "fuzz/demo/seeds/";

fn sha256_hex_via_list(path: &Path) -> String {
    // Reuse the crate's own hasher through the public API: index a one-file
    // dir and read the hash back. Keeps the test free of a second SHA impl.
    let dir = path.parent().unwrap();
    let list = ListIndex::from_dir(dir, "x").unwrap();
    list.files[path.file_name().unwrap().to_str().unwrap()]
        .sha256
        .clone()
}

fn file_url(root: &Path) -> String {
    let s = root.display().to_string().replace('\\', "/");
    format!("file:///{}", s.trim_start_matches('/'))
}

struct Remote {
    root: PathBuf,
    cache: PathBuf,
}

impl Remote {
    fn new(name: &str) -> Self {
        let base = std::env::temp_dir().join(format!("codec-corpus-r2-local-{name}"));
        let _ = std::fs::remove_dir_all(&base);
        let root = base.join("remote");
        let cache = base.join("cache");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::create_dir_all(&cache).unwrap();
        Self { root, cache }
    }

    fn prefix_dir(&self) -> PathBuf {
        self.root.join("fuzz").join("demo").join("seeds")
    }

    fn write_list(&self, list: &ListIndex) {
        std::fs::write(self.root.join("fuzz/demo/seeds.list"), list.to_json()).unwrap();
    }

    fn options(&self) -> PullOptions {
        PullOptions::default().cache_root(&self.cache)
    }

    fn base_url(&self) -> String {
        file_url(&self.root)
    }

    fn cleanup(self) {
        let _ = std::fs::remove_dir_all(self.root.parent().unwrap());
    }
}

#[test]
fn pull_over_file_urls_with_real_curl_and_tar() {
    let remote = Remote::new("e2e");
    let objects = remote.prefix_dir();
    std::fs::create_dir_all(objects.join("nested")).unwrap();

    // Ten bundled seeds + one delta object.
    let mut members = Vec::new();
    for i in 0..10 {
        let name = format!("seed{i}.bin");
        let bytes: Vec<u8> = (0..(100 + i * 37)).map(|k| (k * 7 + i) as u8).collect();
        std::fs::write(objects.join(&name), &bytes).unwrap();
        members.push(name);
    }
    std::fs::write(objects.join("nested").join("delta.bin"), b"delta object").unwrap();

    // Bundle the ten seeds with the system tar (member paths relative to the
    // prefix), then remove their individual objects so we can prove they
    // came from the bundle.
    let bundle_path = remote.root.join("fuzz/demo/seeds.tar");
    let status = Command::new("tar")
        .arg("-cf")
        .arg(&bundle_path)
        .arg("-C")
        .arg(&objects)
        .args(&members)
        .status()
        .expect("tar must be installed");
    assert!(status.success(), "tar -cf failed");

    // Build the list from the full local dir (the push-side half), then mark
    // the bundled members and attach the bundle info.
    let mut list = ListIndex::from_dir(&objects, PREFIX).unwrap();
    assert_eq!(list.version, LIST_VERSION);
    assert_eq!(list.files.len(), 11);
    for m in &members {
        list.files.get_mut(m).unwrap().in_bundle = true;
    }
    let bundle_bytes = std::fs::metadata(&bundle_path).unwrap().len();
    list.bundle = Some(BundleInfo {
        key: "fuzz/demo/seeds.tar".to_string(),
        size: bundle_bytes,
        sha256: sha256_hex_via_list(&bundle_path),
        file_count: members.len() as u64,
        uncompressed_size: members.iter().map(|m| list.files[m].size).sum(),
    });
    remote.write_list(&list);
    for m in &members {
        std::fs::remove_file(objects.join(m)).unwrap();
    }

    // Cold pull: bundle + delta.
    let corpus = R2Corpus::pull_with_options(&remote.base_url(), PREFIX, remote.options())
        .expect("cold pull over file:// should succeed");
    assert!(corpus.path().starts_with(&remote.cache));
    assert_eq!(corpus.list().len(), 11);
    for (rel, entry) in corpus.files() {
        let p = corpus.file_path(rel);
        assert!(p.is_file(), "{rel} missing");
        assert_eq!(std::fs::metadata(&p).unwrap().len(), entry.size, "{rel}");
    }
    assert_eq!(
        std::fs::read(corpus.file_path("nested/delta.bin")).unwrap(),
        b"delta object"
    );

    // Warm pull is a no-op that still succeeds; offline mode also succeeds.
    R2Corpus::pull_with_options(&remote.base_url(), PREFIX, remote.options()).unwrap();
    let offline = R2Corpus::pull_with_options(
        &remote.base_url(),
        PREFIX,
        remote.options().mode(PullMode::Offline),
    )
    .unwrap();
    assert_eq!(offline.path(), corpus.path());

    // A list that lies about a hash is rejected with ChecksumMismatch.
    let mut lying = list.clone();
    lying.files.insert(
        "nested/delta.bin".to_string(),
        FileEntry {
            size: 12,
            sha256: "00".repeat(32),
            in_bundle: false,
        },
    );
    remote.write_list(&lying);
    let err =
        R2Corpus::pull_with_options(&remote.base_url(), PREFIX, remote.options()).unwrap_err();
    assert!(
        matches!(err, Error::ChecksumMismatch { ref path, .. } if path == "nested/delta.bin"),
        "{err}"
    );

    remote.cleanup();
}

#[test]
fn pull_of_missing_prefix_is_network_unavailable() {
    let remote = Remote::new("missing");
    let err = R2Corpus::pull_with_options(&remote.base_url(), "does/not/exist/", remote.options())
        .unwrap_err();
    assert!(matches!(err, Error::NetworkUnavailable { .. }), "{err}");
    remote.cleanup();
}
