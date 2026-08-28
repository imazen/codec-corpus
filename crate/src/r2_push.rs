//! `R2Corpus::push`: the authenticated, maintainer-side half of
//! imazen/codec-corpus#2 — publish a local directory as a corpus prefix.
//!
//! Like pull, nothing is compiled in: objects are uploaded by shelling out to
//! the `aws` CLI (`aws s3 cp … --endpoint-url <R2 S3 endpoint>`), bundles are
//! built with the system `tar` (+ `zstd`), and the existing remote `.list` is
//! read over plain anonymous HTTP from the public base URL. Credentials are
//! never embedded in the crate; they come from a [`PushTarget`] (env vars, or
//! the file `r2-corpus login` writes).
//!
//! ## Push algorithm
//!
//! 1. Hash the local directory into a fresh [`ListIndex`].
//! 2. Fetch the remote `.list` (if the prefix exists). Files whose size and
//!    SHA-256 match keep their `in_bundle` flag; anything new or changed is a
//!    delta to upload. Files no longer present locally are dropped from the
//!    list (their objects are left behind, unlisted — pull never sees them).
//! 3. If [`Rebundle::Always`], or [`Rebundle::Auto`] and more than
//!    `max_deltas` files are outside the bundle, build one archive of every
//!    file and upload it; every file is then `in_bundle`.
//! 4. Upload the deltas (several at a time), then the bundle, then the
//!    `.list` last — a reader can never see a list that names an object
//!    which is not yet there.
//! 5. Register the prefix as a child in every ancestor's `.list`
//!    (`fuzz/zentiff.list` gains `fuzz_decode`, `fuzz.list` gains `zentiff`,
//!    …) so `pull("fuzz/")` finds it.
//!
//! [`PushOptions::dry_run`] runs steps 1–3 and reports what 4–5 would do.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde::{Deserialize, Serialize};

use crate::Error;
use crate::r2::{
    BundleInfo, Fetcher, ListIndex, R2Corpus, bundle_url, join_rel, list_url, normalize_prefix,
    run_parallel, temp_path,
};
use crate::sha256;

/// Where pushes go and how they authenticate.
///
/// The S3 endpoint + bucket receive the uploads; `base_url` is the *public*
/// read URL the pushed prefix is served from (used to fetch the existing
/// `.list`, and what pull callers use). Credentials are optional: when
/// `None`, the `aws` CLI's own configuration (`AWS_*` env vars, profiles) is
/// used unchanged.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PushTarget {
    /// S3 API endpoint, e.g. `https://<account-id>.r2.cloudflarestorage.com`.
    pub endpoint_url: String,
    /// Bucket name, e.g. `codec-corpus`.
    pub bucket: String,
    /// Public read base URL of the bucket (default
    /// [`DEFAULT_R2_BASE_URL`](crate::DEFAULT_R2_BASE_URL)).
    pub base_url: String,
    /// S3 access key id (an R2 API token's key); `None` = leave it to `aws`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub access_key_id: Option<String>,
    /// S3 secret access key; `None` = leave it to `aws`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secret_access_key: Option<String>,
}

/// Env vars [`PushTarget::from_env`] reads.
pub const ENV_ENDPOINT: &str = "CODEC_CORPUS_R2_ENDPOINT";
/// Bucket name env var.
pub const ENV_BUCKET: &str = "CODEC_CORPUS_R2_BUCKET";
/// Public base URL env var.
pub const ENV_BASE_URL: &str = "CODEC_CORPUS_R2_BASE_URL";

impl PushTarget {
    /// Build a target from `CODEC_CORPUS_R2_ENDPOINT`, `CODEC_CORPUS_R2_BUCKET`,
    /// `CODEC_CORPUS_R2_BASE_URL` (optional) and `AWS_ACCESS_KEY_ID` /
    /// `AWS_SECRET_ACCESS_KEY` or `R2_ACCESS_KEY_ID` / `R2_SECRET_ACCESS_KEY`
    /// (optional). Missing endpoint or bucket is an error.
    pub fn from_env() -> Result<Self, Error> {
        let var = |k: &str| std::env::var(k).ok().filter(|v| !v.trim().is_empty());
        let endpoint_url = var(ENV_ENDPOINT).ok_or_else(|| {
            Error::ListParse(format!("{ENV_ENDPOINT} is not set (R2 S3 endpoint URL)"))
        })?;
        let bucket = var(ENV_BUCKET)
            .ok_or_else(|| Error::ListParse(format!("{ENV_BUCKET} is not set (bucket name)")))?;
        Ok(Self {
            endpoint_url,
            bucket,
            base_url: var(ENV_BASE_URL).unwrap_or_else(|| crate::DEFAULT_R2_BASE_URL.to_string()),
            access_key_id: var("AWS_ACCESS_KEY_ID").or_else(|| var("R2_ACCESS_KEY_ID")),
            secret_access_key: var("AWS_SECRET_ACCESS_KEY").or_else(|| var("R2_SECRET_ACCESS_KEY")),
        })
    }

    /// Default location of the saved target:
    /// `{config_dir}/codec-corpus/r2-push.json`.
    pub fn config_path() -> Result<PathBuf, Error> {
        if let Ok(p) = std::env::var("CODEC_CORPUS_R2_CONFIG") {
            return Ok(PathBuf::from(p));
        }
        Ok(dirs::config_dir()
            .ok_or(Error::NoCacheDir)?
            .join("codec-corpus")
            .join("r2-push.json"))
    }

    /// Load the saved target ([`PushTarget::config_path`]), then let any env
    /// vars from [`PushTarget::from_env`] override individual fields. Errors
    /// if neither source yields an endpoint and bucket.
    pub fn load() -> Result<Self, Error> {
        let from_file = Self::config_path()
            .ok()
            .and_then(|p| Self::load_from(&p).ok());
        let from_env = Self::from_env();
        match (from_file, from_env) {
            (_, Ok(env)) => Ok(env),
            (Some(mut file), Err(_)) => {
                let var = |k: &str| std::env::var(k).ok().filter(|v| !v.trim().is_empty());
                if let Some(v) = var(ENV_BASE_URL) {
                    file.base_url = v;
                }
                if let Some(v) = var("AWS_ACCESS_KEY_ID").or_else(|| var("R2_ACCESS_KEY_ID")) {
                    file.access_key_id = Some(v);
                }
                if let Some(v) =
                    var("AWS_SECRET_ACCESS_KEY").or_else(|| var("R2_SECRET_ACCESS_KEY"))
                {
                    file.secret_access_key = Some(v);
                }
                Ok(file)
            }
            (None, Err(e)) => Err(Error::ListParse(format!(
                "no push target: run `r2-corpus login` or set env vars ({e})"
            ))),
        }
    }

    /// Read a target from a JSON file.
    pub fn load_from(path: &Path) -> Result<Self, Error> {
        let text = std::fs::read_to_string(path)?;
        serde_json::from_str(&text).map_err(|e| Error::ListParse(e.to_string()))
    }

    /// Write the target (including any credentials) to `path`, creating
    /// parent directories; mode `0600` on Unix.
    pub fn save_to(&self, path: &Path) -> Result<(), Error> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self).expect("PushTarget is serializable");
        std::fs::write(path, json)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
        }
        Ok(())
    }

    /// [`PushTarget::save_to`] at [`PushTarget::config_path`].
    pub fn save(&self) -> Result<PathBuf, Error> {
        let p = Self::config_path()?;
        self.save_to(&p)?;
        Ok(p)
    }

    /// The `aws s3 cp` command line that uploads `path` as `key`, without
    /// running it. Credentials go in the process environment, never on the
    /// command line.
    pub fn aws_cp_args(&self, path: &Path, key: &str, content_type: Option<&str>) -> Vec<String> {
        let mut args = vec![
            "s3".to_string(),
            "cp".to_string(),
            path.to_string_lossy().into_owned(),
            format!("s3://{}/{}", self.bucket, key.trim_start_matches('/')),
            "--endpoint-url".to_string(),
            self.endpoint_url.clone(),
            "--only-show-errors".to_string(),
        ];
        if let Some(ct) = content_type {
            args.push("--content-type".to_string());
            args.push(ct.to_string());
        }
        args
    }

    /// Upload one file with the `aws` CLI.
    #[cfg(not(target_arch = "wasm32"))]
    fn aws_upload(&self, path: &Path, key: &str) -> Result<(), Error> {
        let content_type = if key.ends_with(".list") {
            Some("application/json")
        } else {
            None
        };
        let mut cmd = Command::new("aws");
        cmd.args(self.aws_cp_args(path, key, content_type))
            .env("AWS_EC2_METADATA_DISABLED", "true")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped());
        if std::env::var_os("AWS_DEFAULT_REGION").is_none() {
            cmd.env("AWS_DEFAULT_REGION", "auto");
        }
        if let Some(k) = &self.access_key_id {
            cmd.env("AWS_ACCESS_KEY_ID", k);
        }
        if let Some(s) = &self.secret_access_key {
            cmd.env("AWS_SECRET_ACCESS_KEY", s);
        }
        let output = cmd.output().map_err(|e| Error::DownloadFailed {
            url: format!("s3://{}/{key}", self.bucket),
            reason: format!("could not run `aws` (is the AWS CLI installed?): {e}"),
        })?;
        if output.status.success() {
            Ok(())
        } else {
            Err(Error::DownloadFailed {
                url: format!("s3://{}/{key}", self.bucket),
                reason: format!(
                    "aws s3 cp failed: {}",
                    String::from_utf8_lossy(&output.stderr).trim()
                ),
            })
        }
    }
}

/// When a push regenerates the prefix's bundle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Rebundle {
    /// Never build a bundle; deltas only (the existing bundle, if any, stays).
    Never,
    /// Rebuild when more than `max_deltas` files are outside the bundle. A
    /// first push of a large prefix therefore bundles immediately.
    Auto {
        /// Delta count above which the bundle is regenerated.
        max_deltas: usize,
    },
    /// Always rebuild the bundle from every file.
    Always,
}

impl Default for Rebundle {
    fn default() -> Self {
        Rebundle::Auto { max_deltas: 100 }
    }
}

/// Archive format for a bundle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum BundleFormat {
    /// `.tar.zst` via `tar --zstd`, else `tar` + the `zstd` binary; falls
    /// back to `.tar.gz` when neither is available.
    #[default]
    TarZst,
    /// `.tar.gz` via `tar -czf`.
    TarGz,
    /// Plain `.tar`.
    Tar,
}

impl BundleFormat {
    fn extension(self) -> &'static str {
        match self {
            BundleFormat::TarZst => ".tar.zst",
            BundleFormat::TarGz => ".tar.gz",
            BundleFormat::Tar => ".tar",
        }
    }
}

/// Tunables for [`R2Corpus::push`].
#[derive(Debug, Clone)]
pub struct PushOptions {
    rebundle: Rebundle,
    bundle_format: BundleFormat,
    dry_run: bool,
    register_parents: bool,
    parallelism: usize,
}

impl Default for PushOptions {
    fn default() -> Self {
        Self {
            rebundle: Rebundle::default(),
            bundle_format: BundleFormat::default(),
            dry_run: false,
            register_parents: true,
            parallelism: 4,
        }
    }
}

impl PushOptions {
    /// Bundle policy (default `Rebundle::Auto { max_deltas: 100 }`).
    pub fn rebundle(mut self, policy: Rebundle) -> Self {
        self.rebundle = policy;
        self
    }

    /// Archive format for a regenerated bundle (default [`BundleFormat::TarZst`]).
    pub fn bundle_format(mut self, format: BundleFormat) -> Self {
        self.bundle_format = format;
        self
    }

    /// Compute and report what would be uploaded, without uploading.
    pub fn dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }

    /// Register the prefix as a child in every ancestor's `.list` (default
    /// `true`).
    pub fn register_parents(mut self, register: bool) -> Self {
        self.register_parents = register;
        self
    }

    /// Concurrent uploads (default 4; `0`/`1` = sequential).
    pub fn parallelism(mut self, n: usize) -> Self {
        self.parallelism = n.max(1);
        self
    }
}

/// What a push did (or, for a dry run, would do).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PushReport {
    /// The normalized prefix.
    pub prefix: String,
    /// Relative paths uploaded as individual objects (new or changed).
    pub uploaded: Vec<String>,
    /// Files whose remote copy already matched.
    pub unchanged: usize,
    /// Relative paths that were listed remotely but are gone locally.
    pub removed: Vec<String>,
    /// The bundle built by this push, if any.
    pub bundled: Option<BundleInfo>,
    /// Ancestor prefixes whose `.list` gained a child entry.
    pub parents_updated: Vec<String>,
    /// Whether the `.list` itself was (or would be) uploaded.
    pub list_uploaded: bool,
    /// The `.list` the remote now matches (or would).
    pub list: ListIndex,
    /// `true` when nothing was uploaded because this was a dry run.
    pub dry_run: bool,
}

/// Something that uploads the file at `path` as bucket `key`.
pub(crate) type Uploader<'a> = &'a (dyn Fn(&Path, &str) -> Result<(), Error> + Sync);

impl R2Corpus {
    /// Publish `local_dir` as `prefix` on `target`. See the module docs for
    /// the algorithm. Returns what was uploaded.
    ///
    /// Requires the `aws` CLI on `PATH` (and `tar`, plus `zstd` for
    /// `.tar.zst` bundles). Not available on `wasm32`.
    pub fn push(
        local_dir: &Path,
        prefix: &str,
        target: &PushTarget,
        options: PushOptions,
    ) -> Result<PushReport, Error> {
        #[cfg(target_arch = "wasm32")]
        {
            let _ = (local_dir, target, options);
            Err(Error::DownloadUnsupported {
                dataset: prefix.to_string(),
            })
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let upload = |path: &Path, key: &str| target.aws_upload(path, key);
            push_with_io(
                local_dir,
                prefix,
                &target.base_url,
                &options,
                &crate::download::download_file,
                &upload,
            )
        }
    }

    /// Compare `local_dir` against the published `prefix` without uploading
    /// anything: a [`PushOptions::dry_run`] push against the public
    /// `base_url` (no credentials needed).
    pub fn diff(
        local_dir: &Path,
        base_url: &str,
        prefix: &str,
        options: PushOptions,
    ) -> Result<PushReport, Error> {
        let options = options.dry_run(true);
        let no_upload = |_: &Path, _: &str| -> Result<(), Error> {
            Err(Error::DownloadUnsupported {
                dataset: "dry run".to_string(),
            })
        };
        #[cfg(target_arch = "wasm32")]
        {
            let no_fetch = |_: &str, _: &Path| -> Result<(), Error> {
                Err(Error::DownloadUnsupported {
                    dataset: prefix.to_string(),
                })
            };
            push_with_io(local_dir, prefix, base_url, &options, &no_fetch, &no_upload)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            push_with_io(
                local_dir,
                prefix,
                base_url,
                &options,
                &crate::download::download_file,
                &no_upload,
            )
        }
    }

    /// Fetch and parse the published `.list` of `prefix` from `base_url`
    /// (anonymous; no cache involved).
    pub fn fetch_list(base_url: &str, prefix: &str) -> Result<ListIndex, Error> {
        #[cfg(target_arch = "wasm32")]
        {
            let _ = base_url;
            Err(Error::DownloadUnsupported {
                dataset: prefix.to_string(),
            })
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let prefix = normalize_prefix(prefix)?;
            let scratch = std::env::temp_dir();
            fetch_remote_list(base_url, &prefix, &scratch, &crate::download::download_file)?.ok_or(
                Error::NetworkUnavailable {
                    dataset: prefix.clone(),
                },
            )
        }
    }
}

/// Fetch `<prefix>.list` into a temp file and parse it. `Ok(None)` when the
/// fetch fails (no such prefix yet, or no network).
fn fetch_remote_list(
    base_url: &str,
    prefix: &str,
    scratch: &Path,
    fetch: Fetcher<'_>,
) -> Result<Option<ListIndex>, Error> {
    let tmp = temp_path(scratch, "push-list");
    let fetched = fetch(&list_url(base_url, prefix), &tmp);
    let result = match fetched {
        Ok(()) => {
            let text = std::fs::read_to_string(&tmp)?;
            ListIndex::parse(&text).map(Some)
        }
        Err(_) => Ok(None),
    };
    let _ = std::fs::remove_file(&tmp);
    result
}

pub(crate) fn push_with_io(
    local_dir: &Path,
    prefix: &str,
    base_url: &str,
    options: &PushOptions,
    fetch: Fetcher<'_>,
    upload: Uploader<'_>,
) -> Result<PushReport, Error> {
    let prefix = normalize_prefix(prefix)?;
    if !local_dir.is_dir() {
        return Err(Error::LocalPathNotFound {
            path: local_dir.display().to_string(),
        });
    }

    // 1. Local state.
    let mut list = ListIndex::from_dir(local_dir, &prefix)?;

    // 2. Remote state + diff.
    // Temp files live in `local_dir` itself (dot-prefixed, never listed):
    // no reliance on a system temp dir.
    let remote = fetch_remote_list(base_url, &prefix, local_dir, fetch)?;
    let remote_files: BTreeMap<String, crate::FileEntry> =
        remote.as_ref().map(|r| r.files.clone()).unwrap_or_default();
    let mut uploaded = Vec::new();
    let mut unchanged = 0usize;
    for (rel, entry) in list.files.iter_mut() {
        match remote_files.get(rel) {
            Some(r) if r.size == entry.size && r.sha256 == entry.sha256 => {
                entry.in_bundle = r.in_bundle;
                unchanged += 1;
            }
            _ => uploaded.push(rel.clone()),
        }
    }
    let removed: Vec<String> = remote_files
        .keys()
        .filter(|k| !list.files.contains_key(*k))
        .cloned()
        .collect();
    if let Some(r) = &remote {
        list.bundle = r.bundle.clone();
        list.children = r.children.clone();
    }

    // 3. Bundle decision.
    let deltas = list.files.values().filter(|e| !e.in_bundle).count();
    let rebundle = match options.rebundle {
        Rebundle::Never => false,
        Rebundle::Always => !list.files.is_empty(),
        Rebundle::Auto { max_deltas } => deltas > max_deltas,
    };

    let changed = !uploaded.is_empty()
        || !removed.is_empty()
        || rebundle
        || remote.is_none()
        || remote.as_ref().is_some_and(|r| r.files != list.files);

    let mut report = PushReport {
        prefix: prefix.clone(),
        uploaded: uploaded.clone(),
        unchanged,
        removed,
        bundled: None,
        parents_updated: Vec::new(),
        list_uploaded: changed,
        list: list.clone(),
        dry_run: options.dry_run,
    };

    if rebundle {
        let key = format!(
            "{}{}",
            prefix.trim_end_matches('/'),
            options.bundle_format.extension()
        );
        if options.dry_run {
            // Report the intent with the size/hash left unknown.
            let info = BundleInfo {
                key,
                size: 0,
                sha256: String::new(),
                file_count: list.files.len() as u64,
                uncompressed_size: list.total_size(),
            };
            report.list.bundle = Some(info.clone());
            report.bundled = Some(info);
        }
    }
    if options.register_parents {
        report.parents_updated = ancestors_to_register(&prefix, base_url, local_dir, fetch)?
            .into_iter()
            .map(|(p, _)| p)
            .collect();
    }
    if options.dry_run {
        for e in report.list.files.values_mut() {
            if rebundle {
                e.in_bundle = true;
            }
        }
        return Ok(report);
    }

    // 4. Uploads: deltas, bundle, then the list.
    let upload_one = |rel: &String| -> Result<(), Error> {
        let path = join_rel(local_dir, rel);
        upload(&path, &format!("{prefix}{rel}"))
    };
    run_parallel(&uploaded, options.parallelism, upload_one)?;

    if rebundle {
        let (archive, key) = build_bundle(local_dir, &prefix, &list, options.bundle_format)?;
        let size = std::fs::metadata(&archive)?.len();
        let sha = sha256::sha256_file(&archive)?;
        let up = upload(&archive, &key);
        let _ = std::fs::remove_file(&archive);
        up?;
        let info = BundleInfo {
            key,
            size,
            sha256: sha,
            file_count: list.files.len() as u64,
            uncompressed_size: list.total_size(),
        };
        for e in list.files.values_mut() {
            e.in_bundle = true;
        }
        list.bundle = Some(info.clone());
        report.bundled = Some(info);
    }

    if changed {
        list.validate()?;
        upload_json(
            &list,
            &format!("{}.list", prefix.trim_end_matches('/')),
            local_dir,
            upload,
        )?;
    }
    report.list = list;

    // 5. Ancestors.
    if options.register_parents {
        for (parent_prefix, mut parent_list) in
            ancestors_to_register(&prefix, base_url, local_dir, fetch)?
        {
            let child = prefix
                .trim_end_matches('/')
                .strip_prefix(parent_prefix.as_str())
                .unwrap_or_default()
                .split('/')
                .next()
                .unwrap_or_default()
                .to_string();
            parent_list.add_child(&child)?;
            upload_json(
                &parent_list,
                &format!("{}.list", parent_prefix.trim_end_matches('/')),
                local_dir,
                upload,
            )?;
        }
    }
    Ok(report)
}

/// Every ancestor prefix whose `.list` does not yet name the next component
/// of `prefix`, paired with that list (an empty node when none exists).
/// Ordered nearest-ancestor first.
fn ancestors_to_register(
    prefix: &str,
    base_url: &str,
    scratch: &Path,
    fetch: Fetcher<'_>,
) -> Result<Vec<(String, ListIndex)>, Error> {
    let comps: Vec<&str> = prefix.trim_end_matches('/').split('/').collect();
    let mut out = Vec::new();
    for depth in (1..comps.len()).rev() {
        let parent = format!("{}/", comps[..depth].join("/"));
        let child = comps[depth];
        let list = match fetch_remote_list(base_url, &parent, scratch, fetch)? {
            Some(l) => l,
            None => ListIndex::empty(&parent)?,
        };
        if list.children.iter().any(|c| c == child) {
            continue;
        }
        out.push((parent, list));
    }
    Ok(out)
}

fn upload_json(
    list: &ListIndex,
    key: &str,
    scratch: &Path,
    upload: Uploader<'_>,
) -> Result<(), Error> {
    let tmp = temp_path(scratch, "push-json");
    std::fs::write(&tmp, list.to_json())?;
    let r = upload(&tmp, key);
    let _ = std::fs::remove_file(&tmp);
    r
}

/// Archive every listed file (paths relative to `local_dir`) with the system
/// `tar`. Returns the archive path and its bucket key; the format may
/// degrade (`.tar.zst` → `.tar.gz`) when the tools for the requested one are
/// missing.
fn build_bundle(
    local_dir: &Path,
    prefix: &str,
    list: &ListIndex,
    format: BundleFormat,
) -> Result<(PathBuf, String), Error> {
    let scratch = local_dir;
    let manifest = temp_path(scratch, "push-members");
    let members = list.files.keys().cloned().collect::<Vec<_>>().join("\n");
    std::fs::write(&manifest, format!("{members}\n"))?;

    let run_tar = |args: &[&str], out: &Path| -> bool {
        Command::new("tar")
            .args(args)
            .arg(out)
            .arg("-C")
            .arg(local_dir)
            .arg("-T")
            .arg(&manifest)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    };
    let stem = prefix.trim_end_matches('/');
    let mut result: Option<(PathBuf, String)> = None;
    let mut attempts: Vec<BundleFormat> = match format {
        BundleFormat::TarZst => vec![BundleFormat::TarZst, BundleFormat::TarGz, BundleFormat::Tar],
        BundleFormat::TarGz => vec![BundleFormat::TarGz, BundleFormat::Tar],
        BundleFormat::Tar => vec![BundleFormat::Tar],
    };
    attempts.reverse();
    while let Some(fmt) = attempts.pop() {
        let out = temp_path(scratch, "push-bundle").with_extension(match fmt {
            BundleFormat::TarZst => "tar.zst",
            BundleFormat::TarGz => "tar.gz",
            BundleFormat::Tar => "tar",
        });
        let ok = match fmt {
            BundleFormat::TarZst => {
                run_tar(&["--zstd", "-cf"], &out) || {
                    let plain = out.with_extension("");
                    let tarred = run_tar(&["-cf"], &plain);
                    let zipped = tarred
                        && Command::new("zstd")
                            .args(["-q", "-f", "-o"])
                            .arg(&out)
                            .arg(&plain)
                            .stdin(Stdio::null())
                            .stdout(Stdio::null())
                            .stderr(Stdio::null())
                            .status()
                            .map(|s| s.success())
                            .unwrap_or(false);
                    let _ = std::fs::remove_file(&plain);
                    zipped
                }
            }
            BundleFormat::TarGz => run_tar(&["-czf"], &out),
            BundleFormat::Tar => run_tar(&["-cf"], &out),
        };
        if ok && out.is_file() {
            result = Some((out, format!("{stem}{}", fmt.extension())));
            break;
        }
        let _ = std::fs::remove_file(&out);
    }
    let _ = std::fs::remove_file(&manifest);
    result.ok_or_else(|| Error::DownloadFailed {
        url: bundle_url("", &format!("{stem}{}", format.extension())),
        reason: "could not build the bundle (is `tar` installed?)".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aws_cp_args_shape() {
        let t = PushTarget {
            endpoint_url: "https://acct.r2.cloudflarestorage.com".to_string(),
            bucket: "codec-corpus".to_string(),
            base_url: "https://codec-corpus.r2.imazen.org".to_string(),
            access_key_id: Some("AK".to_string()),
            secret_access_key: Some("SK".to_string()),
        };
        let args = t.aws_cp_args(
            Path::new("/tmp/x.list"),
            "/fuzz/a.list",
            Some("application/json"),
        );
        assert_eq!(
            args,
            [
                "s3",
                "cp",
                "/tmp/x.list",
                "s3://codec-corpus/fuzz/a.list",
                "--endpoint-url",
                "https://acct.r2.cloudflarestorage.com",
                "--only-show-errors",
                "--content-type",
                "application/json",
            ]
        );
        assert!(
            !args.iter().any(|a| a.contains("AK") || a.contains("SK")),
            "credentials never appear on the command line"
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    mod sync {
        use super::*;
        use crate::r2::{PullMode, PullOptions};
        use std::sync::Mutex;

        const BASE: &str = "https://fake.example";
        const PREFIX: &str = "fuzz/demo/seeds/";

        /// A fake bucket on disk: the uploader writes objects into it and the
        /// fetcher reads them back over the public-URL layout.
        struct Bucket {
            root: PathBuf,
            store: PathBuf,
            local: PathBuf,
            cache: PathBuf,
            uploads: Mutex<Vec<String>>,
        }

        impl Bucket {
            fn new(name: &str) -> Self {
                let root = std::env::temp_dir().join(format!("codec-corpus-push-test-{name}"));
                let _ = std::fs::remove_dir_all(&root);
                let b = Self {
                    store: root.join("bucket"),
                    local: root.join("local"),
                    cache: root.join("cache"),
                    root,
                    uploads: Mutex::new(Vec::new()),
                };
                std::fs::create_dir_all(&b.store).unwrap();
                std::fs::create_dir_all(&b.local).unwrap();
                b
            }

            fn fetcher(&self) -> impl Fn(&str, &Path) -> Result<(), Error> + Sync + '_ {
                move |url: &str, dest: &Path| {
                    let rel = url
                        .strip_prefix(&format!("{BASE}/"))
                        .expect("url under BASE");
                    let src = join_rel(&self.store, rel);
                    if !src.is_file() {
                        return Err(Error::NetworkUnavailable {
                            dataset: rel.to_string(),
                        });
                    }
                    std::fs::copy(&src, dest)?;
                    Ok(())
                }
            }

            fn uploader(&self) -> impl Fn(&Path, &str) -> Result<(), Error> + Sync + '_ {
                move |path: &Path, key: &str| {
                    self.uploads.lock().unwrap().push(key.to_string());
                    let dest = join_rel(&self.store, key);
                    std::fs::create_dir_all(dest.parent().unwrap())?;
                    std::fs::copy(path, &dest)?;
                    Ok(())
                }
            }

            fn write_local(&self, rel: &str, bytes: &[u8]) {
                let p = join_rel(&self.local, rel);
                std::fs::create_dir_all(p.parent().unwrap()).unwrap();
                std::fs::write(p, bytes).unwrap();
            }

            fn push(&self, options: PushOptions) -> Result<PushReport, Error> {
                let f = self.fetcher();
                let u = self.uploader();
                push_with_io(&self.local, PREFIX, BASE, &options, &f, &u)
            }

            fn uploads(&self) -> Vec<String> {
                self.uploads.lock().unwrap().clone()
            }

            fn reset(&self) {
                self.uploads.lock().unwrap().clear();
            }

            fn stored_list(&self, prefix: &str) -> ListIndex {
                let p = join_rel(
                    &self.store,
                    &format!("{}.list", prefix.trim_end_matches('/')),
                );
                ListIndex::parse(&std::fs::read_to_string(p).unwrap()).unwrap()
            }

            /// Pull the pushed prefix back through the real pull path.
            fn pull(&self, mode: PullMode) -> Result<R2Corpus, Error> {
                let f = self.fetcher();
                R2Corpus::pull_with_fetcher(
                    BASE,
                    PREFIX,
                    PullOptions::default().cache_root(&self.cache).mode(mode),
                    &f,
                )
            }

            fn cleanup(self) {
                let _ = std::fs::remove_dir_all(&self.root);
            }
        }

        #[test]
        fn ancestors_of_a_prefix() {
            let no_remote = |_: &str, _: &Path| -> Result<(), Error> {
                Err(Error::NetworkUnavailable {
                    dataset: String::new(),
                })
            };
            let scratch = std::env::temp_dir();
            let anc = ancestors_to_register(
                "fuzz/zentiff/fuzz_decode/",
                "https://x",
                &scratch,
                &no_remote,
            )
            .unwrap();
            let prefixes: Vec<&str> = anc.iter().map(|(p, _)| p.as_str()).collect();
            assert_eq!(prefixes, ["fuzz/zentiff/", "fuzz/"]);
            assert!(
                anc.iter()
                    .all(|(_, l)| l.is_empty() && l.children.is_empty())
            );
            assert!(
                ancestors_to_register("top/", "https://x", &scratch, &no_remote)
                    .unwrap()
                    .is_empty()
            );
        }

        #[test]
        fn first_push_uploads_everything_registers_ancestors_and_pulls_back() {
            let b = Bucket::new("first");
            b.write_local("a.bin", b"alpha");
            b.write_local("sub/b.bin", b"beta beta");
            b.write_local(".hidden", b"never listed");

            let report = b.push(PushOptions::default()).unwrap();
            assert_eq!(report.prefix, PREFIX);
            assert_eq!(report.uploaded, ["a.bin", "sub/b.bin"]);
            assert_eq!(report.unchanged, 0);
            assert!(report.removed.is_empty());
            assert!(report.bundled.is_none(), "2 deltas < max_deltas: no bundle");
            assert!(report.list_uploaded);
            assert!(!report.dry_run);
            assert_eq!(report.parents_updated, ["fuzz/demo/", "fuzz/"]);

            let mut ups = b.uploads();
            ups.sort();
            assert_eq!(
                ups,
                [
                    "fuzz.list",
                    "fuzz/demo.list",
                    "fuzz/demo/seeds.list",
                    "fuzz/demo/seeds/a.bin",
                    "fuzz/demo/seeds/sub/b.bin",
                ]
            );
            // The list went up after the objects it names.
            let raw = b.uploads();
            let list_pos = raw
                .iter()
                .position(|k| k == "fuzz/demo/seeds.list")
                .unwrap();
            let obj_pos = raw
                .iter()
                .position(|k| k == "fuzz/demo/seeds/sub/b.bin")
                .unwrap();
            assert!(obj_pos < list_pos);

            let stored = b.stored_list(PREFIX);
            assert_eq!(stored.files, report.list.files);
            assert_eq!(stored.len(), 2);
            assert!(!stored.files.contains_key(".hidden"));
            assert_eq!(b.stored_list("fuzz/demo/").children, ["seeds"]);
            assert_eq!(b.stored_list("fuzz/").children, ["demo"]);

            // Round trip through the real pull.
            let c = b.pull(PullMode::Auto).unwrap();
            assert_eq!(
                std::fs::read(c.file_path("sub/b.bin")).unwrap(),
                b"beta beta"
            );
            let tree = {
                let f = b.fetcher();
                R2Corpus::pull_with_fetcher(
                    BASE,
                    "fuzz/",
                    PullOptions::default().cache_root(&b.cache),
                    &f,
                )
                .unwrap()
            };
            assert_eq!(
                tree.files().map(|(k, _)| k).collect::<Vec<_>>(),
                ["demo/seeds/a.bin", "demo/seeds/sub/b.bin"]
            );

            // Pushing again with no changes uploads nothing at all.
            b.reset();
            let again = b.push(PushOptions::default()).unwrap();
            assert!(again.uploaded.is_empty());
            assert_eq!(again.unchanged, 2);
            assert!(!again.list_uploaded);
            assert!(again.parents_updated.is_empty());
            assert!(b.uploads().is_empty(), "{:?}", b.uploads());
            b.cleanup();
        }

        #[test]
        fn incremental_push_uploads_deltas_and_drops_removed_files() {
            let b = Bucket::new("incremental");
            for i in 0..5 {
                b.write_local(&format!("s{i}.bin"), format!("seed {i}").as_bytes());
            }
            b.push(PushOptions::default().rebundle(Rebundle::Always))
                .unwrap();
            let bundled = b.stored_list(PREFIX);
            assert!(bundled.bundle.is_some());
            assert!(bundled.files.values().all(|e| e.in_bundle));

            b.write_local("s1.bin", b"seed 1 changed");
            b.write_local("s5.bin", b"seed 5 new");
            std::fs::remove_file(join_rel(&b.local, "s3.bin")).unwrap();
            b.reset();
            let report = b.push(PushOptions::default()).unwrap();
            assert_eq!(report.uploaded, ["s1.bin", "s5.bin"]);
            assert_eq!(report.removed, ["s3.bin"]);
            assert_eq!(report.unchanged, 3);
            assert!(report.bundled.is_none(), "2 deltas: bundle kept as is");
            let mut ups = b.uploads();
            ups.sort();
            assert_eq!(
                ups,
                [
                    "fuzz/demo/seeds.list",
                    "fuzz/demo/seeds/s1.bin",
                    "fuzz/demo/seeds/s5.bin"
                ]
            );
            let stored = b.stored_list(PREFIX);
            assert_eq!(stored.bundle, bundled.bundle, "old bundle info carried");
            assert!(!stored.files.contains_key("s3.bin"));
            assert!(stored.files["s0.bin"].in_bundle);
            assert!(!stored.files["s1.bin"].in_bundle, "changed file is a delta");
            assert!(!stored.files["s5.bin"].in_bundle);

            // Pull: the (stale) bundle supplies s0/s2/s4, deltas the rest, and
            // the removed member is not placed.
            let c = b.pull(PullMode::ForceBundle).unwrap();
            assert_eq!(
                std::fs::read(c.file_path("s1.bin")).unwrap(),
                b"seed 1 changed"
            );
            assert_eq!(std::fs::read(c.file_path("s4.bin")).unwrap(), b"seed 4");
            assert!(!c.file_path("s3.bin").exists());
            b.cleanup();
        }

        #[test]
        fn auto_rebundle_threshold_and_bundle_round_trip() {
            let b = Bucket::new("auto-bundle");
            for i in 0..8 {
                b.write_local(
                    &format!("d{}/s{i}.bin", i % 2),
                    format!("seed {i} {}", "y".repeat(i * 10)).as_bytes(),
                );
            }
            let report = b
                .push(PushOptions::default().rebundle(Rebundle::Auto { max_deltas: 3 }))
                .unwrap();
            let info = report.bundled.clone().expect("8 deltas > 3 → bundle");
            assert!(info.key.starts_with("fuzz/demo/seeds.tar"), "{}", info.key);
            assert_eq!(info.file_count, 8);
            assert_eq!(info.uncompressed_size, report.list.total_size());
            assert!(info.size > 0);
            let stored_bundle = join_rel(&b.store, &info.key);
            assert_eq!(sha256::sha256_file(&stored_bundle).unwrap(), info.sha256);
            assert!(report.list.files.values().all(|e| e.in_bundle));

            // Cold pull must take the bundle and never fetch a member singly.
            // Members are only reachable via the bundle: delete the objects.
            for rel in report.list.files.keys() {
                std::fs::remove_file(join_rel(&b.store, &format!("{PREFIX}{rel}"))).unwrap();
            }
            let c = b.pull(PullMode::Auto).unwrap();
            for (rel, entry) in c.files() {
                let bytes = std::fs::read(c.file_path(rel)).unwrap();
                assert_eq!(sha256::sha256_hex(&bytes), entry.sha256, "{rel}");
            }
            b.cleanup();
        }

        #[test]
        fn dry_run_uploads_nothing_but_reports_the_plan() {
            let b = Bucket::new("dry");
            b.write_local("a.bin", b"a");
            b.write_local("b.bin", b"b");
            let report = b
                .push(
                    PushOptions::default()
                        .dry_run(true)
                        .rebundle(Rebundle::Always),
                )
                .unwrap();
            assert!(report.dry_run);
            assert_eq!(report.uploaded, ["a.bin", "b.bin"]);
            assert!(report.list_uploaded);
            assert_eq!(report.parents_updated, ["fuzz/demo/", "fuzz/"]);
            let planned = report.bundled.expect("plan reports the bundle");
            assert_eq!(planned.key, "fuzz/demo/seeds.tar.zst");
            assert_eq!(planned.file_count, 2);
            assert!(report.list.files.values().all(|e| e.in_bundle));
            assert!(b.uploads().is_empty());
            assert!(!join_rel(&b.store, "fuzz/demo/seeds.list").exists());

            // R2Corpus::diff is the same dry run without an uploader.
            let f = b.fetcher();
            let no_upload = |_: &Path, _: &str| -> Result<(), Error> { panic!("must not upload") };
            let d = push_with_io(
                &b.local,
                PREFIX,
                BASE,
                &PushOptions::default().dry_run(true),
                &f,
                &no_upload,
            )
            .unwrap();
            assert_eq!(d.uploaded, ["a.bin", "b.bin"]);
            drop(f);
            b.cleanup();
        }

        #[test]
        fn push_rejects_missing_local_dir_and_bad_prefix() {
            let b = Bucket::new("bad");
            let f = b.fetcher();
            let u = b.uploader();
            let err = push_with_io(
                &b.root.join("nope"),
                PREFIX,
                BASE,
                &PushOptions::default(),
                &f,
                &u,
            )
            .unwrap_err();
            assert!(matches!(err, Error::LocalPathNotFound { .. }), "{err}");
            let err =
                push_with_io(&b.local, "../x/", BASE, &PushOptions::default(), &f, &u).unwrap_err();
            assert!(matches!(err, Error::ListParse(_)), "{err}");
            assert!(b.uploads().is_empty());
            drop((f, u));
            b.cleanup();
        }

        #[test]
        fn push_target_save_load_roundtrip() {
            let dir = std::env::temp_dir().join("codec-corpus-push-target-test");
            let _ = std::fs::remove_dir_all(&dir);
            let path = dir.join("nested").join("r2-push.json");
            let t = PushTarget {
                endpoint_url: "https://acct.r2.cloudflarestorage.com".to_string(),
                bucket: "codec-corpus".to_string(),
                base_url: crate::DEFAULT_R2_BASE_URL.to_string(),
                access_key_id: Some("AK".to_string()),
                secret_access_key: Some("SK".to_string()),
            };
            t.save_to(&path).unwrap();
            let back = PushTarget::load_from(&path).unwrap();
            assert_eq!(back, t);
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
                assert_eq!(mode, 0o600);
            }
            let _ = std::fs::remove_dir_all(&dir);
        }
    }
}
