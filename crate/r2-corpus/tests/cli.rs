//! End-to-end run of the `r2-corpus` binary against a `file://` "bucket":
//! the real `curl`/`wget` chain and the real `.list` layout, no network.
//!
//! `push` without `--dry-run` needs the `aws` CLI and a writable bucket, so
//! it is exercised only as a dry run here; the push algorithm itself is
//! unit-tested in the library against an on-disk fake bucket.

#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;

use codec_corpus::ListIndex;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_r2-corpus"))
}

fn file_url(root: &Path) -> String {
    let s = root.to_string_lossy().replace('\\', "/");
    format!("file:///{}", s.trim_start_matches('/'))
}

struct Remote {
    root: PathBuf,
    bucket: PathBuf,
    cache: PathBuf,
}

impl Remote {
    fn new(name: &str) -> Self {
        let root = std::env::temp_dir().join(format!("r2-corpus-cli-test-{name}"));
        let _ = std::fs::remove_dir_all(&root);
        let r = Self {
            bucket: root.join("bucket"),
            cache: root.join("cache"),
            root,
        };
        std::fs::create_dir_all(&r.bucket).unwrap();
        r
    }

    fn put(&self, key: &str, bytes: &[u8]) {
        let p = key.split('/').fold(self.bucket.clone(), |d, c| d.join(c));
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, bytes).unwrap();
    }

    /// Publish `dir` as `prefix`: objects + a `.list` built by the library.
    fn publish(&self, dir: &Path, prefix: &str) -> ListIndex {
        let list = ListIndex::from_dir(dir, prefix).unwrap();
        for rel in list.files.keys() {
            let src = rel.split('/').fold(dir.to_path_buf(), |d, c| d.join(c));
            self.put(&format!("{prefix}{rel}"), &std::fs::read(src).unwrap());
        }
        self.put(
            &format!("{}.list", prefix.trim_end_matches('/')),
            list.to_json().as_bytes(),
        );
        list
    }

    fn cmd(&self) -> Command {
        let mut c = bin();
        c.env("CODEC_CORPUS_CACHE", &self.cache)
            .env("CODEC_CORPUS_R2_BASE_URL", file_url(&self.bucket))
            .env("CODEC_CORPUS_R2_CONFIG", self.root.join("r2-push.json"))
            .env_remove("CODEC_CORPUS_R2_ENDPOINT")
            .env_remove("CODEC_CORPUS_R2_BUCKET");
        c
    }

    fn cleanup(self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn run(cmd: &mut Command) -> (bool, String, String) {
    let out = cmd.output().expect("spawn r2-corpus");
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

#[test]
fn pull_list_diff_login_and_sync_end_to_end() {
    let remote = Remote::new("e2e");
    let src = remote.root.join("src");
    std::fs::create_dir_all(src.join("sub")).unwrap();
    std::fs::write(src.join("one.bin"), b"one").unwrap();
    std::fs::write(src.join("sub").join("two.bin"), b"two two").unwrap();
    let prefix = "fuzz/demo/seeds/";
    let list = remote.publish(&src, prefix);
    // A parent node so subtree pulls work.
    let mut parent = ListIndex::empty("fuzz/demo/").unwrap();
    parent.add_child("seeds").unwrap();
    remote.put("fuzz/demo.list", parent.to_json().as_bytes());

    // list
    let (ok, out, err) = run(remote.cmd().args(["list", prefix]));
    assert!(ok, "list failed: {err}");
    assert!(out.contains("2 files"), "{out}");
    assert!(out.contains("sub/two.bin"), "{out}");
    let (ok, out, _) = run(remote.cmd().args(["list", prefix, "--json"]));
    assert!(ok);
    assert_eq!(ListIndex::parse(&out).unwrap().files, list.files);

    // pull --into
    let into = remote.root.join("mirror");
    let (ok, out, err) = run(remote.cmd().args(["pull", prefix, "--into"]).arg(&into));
    assert!(ok, "pull failed: {err}");
    let cache_dir = PathBuf::from(out.trim());
    assert!(cache_dir.starts_with(&remote.cache), "{out}");
    assert!(cache_dir.join("one.bin").is_file());
    assert_eq!(
        std::fs::read(into.join("sub").join("two.bin")).unwrap(),
        b"two two"
    );
    assert!(err.contains("mirrored into"), "{err}");

    // Subtree pull from the parent node.
    let (ok, out, err) = run(remote.cmd().args(["pull", "fuzz/demo/"]));
    assert!(ok, "subtree pull failed: {err}");
    assert!(err.contains("2 files"), "{err}");
    assert!(
        PathBuf::from(out.trim())
            .join("seeds")
            .join("one.bin")
            .is_file()
    );

    // diff: local has one changed + one new + one removed file.
    let local = remote.root.join("local");
    std::fs::create_dir_all(local.join("sub")).unwrap();
    std::fs::write(local.join("one.bin"), b"one changed").unwrap();
    std::fs::write(local.join("three.bin"), b"three").unwrap();
    let (ok, out, err) = run(remote.cmd().args(["diff", prefix, "--local"]).arg(&local));
    assert!(ok, "diff failed: {err}");
    assert!(
        out.contains("would upload 2 object(s), 0 unchanged, 1 removed"),
        "{out}"
    );
    assert!(
        out.contains("+ one.bin") && out.contains("+ three.bin"),
        "{out}"
    );
    assert!(out.contains("- sub/two.bin"), "{out}");
    // Nothing was written to the bucket by a diff.
    assert_eq!(
        std::fs::read(remote.bucket.join("fuzz/demo/seeds/one.bin")).unwrap(),
        b"one"
    );

    // push --dry-run needs no credentials and uploads nothing either.
    let (ok, out, err) = run(remote
        .cmd()
        .args(["push", prefix, "--local"])
        .arg(&local)
        .args(["--dry-run", "--rebundle"]));
    assert!(ok, "push --dry-run failed: {err}");
    assert!(out.contains("would upload 2 object(s)"), "{out}");
    assert!(out.contains("bundle fuzz/demo/seeds.tar.zst"), "{out}");
    assert!(!remote.bucket.join("fuzz/demo/seeds.tar.zst").exists());

    // push without a target fails cleanly (no `aws` invocation, exit 1).
    let (ok, _, err) = run(remote.cmd().args(["push", prefix, "--local"]).arg(&local));
    assert!(!ok);
    assert!(err.contains("no push target"), "{err}");

    // login stores the target (keys on the command line here; prompted otherwise).
    let (ok, out, err) = run(remote.cmd().args([
        "login",
        "--endpoint",
        "https://acct.r2.cloudflarestorage.com",
        "--bucket",
        "codec-corpus",
        "--access-key-id",
        "AK",
        "--secret-access-key",
        "SK",
    ]));
    assert!(ok, "login failed: {err}");
    assert!(out.contains("saved push target"), "{out}");
    let saved = codec_corpus::PushTarget::load_from(&remote.root.join("r2-push.json")).unwrap();
    assert_eq!(saved.bucket, "codec-corpus");
    assert_eq!(saved.access_key_id.as_deref(), Some("AK"));
    assert_eq!(saved.base_url, file_url(&remote.bucket));

    // login with keys on stdin.
    let mut c = remote.cmd();
    c.args([
        "login",
        "--endpoint",
        "https://acct.r2.cloudflarestorage.com",
        "--bucket",
        "b2",
    ])
    .stdin(std::process::Stdio::piped())
    .stdout(std::process::Stdio::piped())
    .stderr(std::process::Stdio::piped());
    let mut child = c.spawn().unwrap();
    {
        use std::io::Write;
        child
            .stdin
            .take()
            .unwrap()
            .write_all(b"AK2\nSK2\n")
            .unwrap();
    }
    let out = child.wait_with_output().unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let saved = codec_corpus::PushTarget::load_from(&remote.root.join("r2-push.json")).unwrap();
    assert_eq!(saved.bucket, "b2");
    assert_eq!(saved.secret_access_key.as_deref(), Some("SK2"));

    // sync from a corpus-sync.toml (pull direction), relative paths resolved
    // against the config's directory + local_dir.
    let project = remote.root.join("project");
    std::fs::create_dir_all(&project).unwrap();
    std::fs::write(
        project.join("corpus-sync.toml"),
        format!(
            "[corpus]\nbase_url = \"{}\"\nlocal_dir = \"fuzz/corpus\"\n\n[[sync]]\nprefix = \"{prefix}\"\nlocal = \"seeds\"\n",
            file_url(&remote.bucket)
        ),
    )
    .unwrap();
    let (ok, out, err) = run(remote
        .cmd()
        .args(["sync", "--config"])
        .arg(project.join("corpus-sync.toml")));
    assert!(ok, "sync failed: {err}");
    assert!(out.contains("2 files written"), "{out}");
    assert_eq!(
        std::fs::read(project.join("fuzz/corpus/seeds/sub/two.bin")).unwrap(),
        b"two two"
    );
    // sync --push --dry-run reports without credentials.
    let (ok, out, err) = run(remote
        .cmd()
        .args(["sync", "--push", "--dry-run", "--config"])
        .arg(project.join("corpus-sync.toml")));
    assert!(ok, "sync --push --dry-run failed: {err}");
    assert!(
        out.contains("would upload 0 object(s), 2 unchanged"),
        "{out}"
    );

    // Usage errors exit non-zero with a message.
    let (ok, _, err) = run(remote.cmd().args(["frobnicate"]));
    assert!(!ok && err.contains("unknown command"));
    let (ok, _, err) = run(remote.cmd().args(["pull"]));
    assert!(!ok && err.contains("missing <prefix>"), "{err}");
    let (ok, _, err) = run(remote.cmd().args(["sync", "--config", "/nonexistent.toml"]));
    assert!(!ok && err.contains("cannot read"), "{err}");
    remote.cleanup();
}
