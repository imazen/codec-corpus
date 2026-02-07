# codec-corpus crate specification

## Overview

`codec-corpus` is a tiny Rust crate (~1 KB on crates.io) that provides a runtime API for downloading, caching, and accessing test image datasets from the `imazen/codec-corpus` GitHub repository.

No data ships with the crate. Datasets are fetched lazily on first access and cached locally. Nothing is downloaded until a specific dataset is requested.

## Distribution

**crates.io**: `codec-corpus = "1.0.0"` — just the API, metadata, and download logic.

**GitHub tags**: Each crate version corresponds to a GitHub tag with the same version number. Tag `v1.0.0` contains the data files that crate version `1.0.0` expects.

**GitHub release assets**: Per-dataset tarballs attached to each release:

```
Release v1.0.0:
  webp-conformance.tar.gz    (1.3 MB)
  clic2025.tar.gz            (219 MB)
  cid22.tar.gz               (94 MB)
  gb82.tar.gz                (9.6 MB)
  ...
  checksums.sha256
```

A crate bugfix that doesn't change data still gets a new tag with the same data re-uploaded. The tag is the single source of truth for "what files belong to this crate version."

## Dependencies

Required:

- `dirs` — cross-platform cache directory resolution (tiny)
- `tar_light` — extract `.tar.gz` archives (lightweight, v0.1.9+)
- `fd-lock` — file-based locking for concurrent safety (tiny)

No heavy dependencies. No `gix`, `serde`, `toml`, `ureq`, or `reqwest`.

External tools used at runtime (not Rust dependencies):

- `git` — primary download method (shell command via `std::process::Command`)
- `curl` / `wget` / `powershell Invoke-WebRequest` — HTTP fallback for tarball download

## Cache layout

```
{cache_root}/codec-corpus/v{major}/
  .version                    # plain text: exact semver, e.g. "1.0.0"
  .lock                       # fd-lock file for concurrent access
  webp-conformance/
    valid/
    invalid/
    non-conformant/
    sources/
    generate_corpus.py
    README.md
  clic2025/
    training/
    final-test/
  cid22/
  gb82/
  ...
```

### Cache root resolution

Checked in order:

1. `CODEC_CORPUS_CACHE` environment variable (explicit override)
2. `dirs::cache_dir()` which resolves to:
   - Linux: `~/.cache/`
   - macOS: `~/Library/Caches/`
   - Windows: `%LOCALAPPDATA%`

### Major version isolation

Cache is stored under `v{major}/`. This means:

- A project using codec-corpus v1.x and a project using v2.x can coexist on the same system without interference.
- Different users on a shared system can use different major versions.
- Upgrading from v1 to v2 does not delete the v1 cache.

### Version checking within a major version

The `.version` file contains a single line: the exact semver string (e.g. `1.0.0`).

Any change in the crate's semver triggers a re-download of the entire `v{major}/` cache:

| Cached | Crate | Result |
|--------|-------|--------|
| 1.0.0 | 1.0.0 | Use cache |
| 1.0.0 | 1.0.1 | Re-download |
| 1.0.0 | 1.1.0 | Re-download |
| 1.0.0 | 2.0.0 | Different folder (`v2/`), download fresh |
| missing | 1.0.0 | Download |

This ensures correctness: even a patch release may add, remove, or modify test files.

## Public API

```rust
pub struct Corpus {
    root: PathBuf,
    is_local: bool,
}

impl Corpus {
    /// Initialize with default cache location.
    /// Resolves cache root but performs no I/O beyond directory creation.
    pub fn new() -> Result<Self, Error>;

    /// Initialize with explicit cache root.
    /// Files will live at `{path}/codec-corpus/v{major}/`.
    pub fn with_cache_root(path: impl Into<PathBuf>) -> Result<Self, Error>;

    /// Use an existing local directory directly. No downloads, no version checks.
    /// For offline use, CI pre-population, or pointing at a local git checkout.
    pub fn from_local(path: impl Into<PathBuf>) -> Self;

    /// Get the path to a dataset or subdirectory within a dataset.
    ///
    /// If the dataset is not cached (or cache is stale), downloads it first.
    /// The `path` argument is split on the first `/` into dataset name and
    /// optional subdirectory.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let corpus = codec_corpus::Corpus::new()?;
    ///
    /// // Get dataset root
    /// let webp = corpus.get("webp-conformance")?;
    /// // Returns: .../v1/webp-conformance/
    ///
    /// // Get subdirectory within dataset
    /// let valid = corpus.get("webp-conformance/valid")?;
    /// // Returns: .../v1/webp-conformance/valid/
    ///
    /// // Deeper paths work too
    /// let training = corpus.get("clic2025/training")?;
    /// // Returns: .../v1/clic2025/training/
    /// # Ok::<(), codec_corpus::Error>(())
    /// ```
    pub fn get(&self, path: &str) -> Result<PathBuf, Error>;

    /// Check if a dataset is already cached locally, without downloading.
    /// Only checks that the directory exists and the version matches.
    pub fn is_cached(&self, dataset: &str) -> bool;

    /// List all available datasets. Returns embedded metadata, no I/O.
    pub fn list() -> &'static [DatasetInfo];
}
```

### Dataset metadata

```rust
pub struct DatasetInfo {
    /// Dataset name, used as argument to `Corpus::get()`.
    pub name: &'static str,
    /// Approximate size in megabytes.
    pub size_mb: f32,
    /// Human-readable description.
    pub description: &'static str,
    /// SHA-256 checksum of the `.tar.gz` release asset.
    pub sha256: &'static str,
}
```

### Error types

```rust
#[non_exhaustive]
pub enum Error {
    /// Network unavailable and dataset not in cache.
    NetworkUnavailable { dataset: String },
    /// Dataset name not found in embedded metadata.
    UnknownDataset { name: String },
    /// Downloaded file failed SHA-256 verification.
    ChecksumMismatch {
        dataset: String,
        expected: String,
        actual: String,
    },
    /// Requested path does not exist after successful download.
    PathNotFound { path: String },
    /// Filesystem error (permissions, disk full, etc.).
    Io(std::io::Error),
    /// No cache directory could be determined.
    NoCacheDir,
}
```

## Embedded metadata

All metadata is hardcoded as constants in the crate source. No config files, no parsing.

```rust
const CORPUS_COMMIT: &str = "993584e";
const REPO_URL: &str = "https://github.com/imazen/codec-corpus";

const DATASETS: &[DatasetInfo] = &[
    DatasetInfo {
        name: "webp-conformance",
        size_mb: 1.3,
        description: "RFC 6386 WebP conformance test files (225 files)",
        sha256: "...",
    },
    DatasetInfo {
        name: "clic2025",
        size_mb: 219.0,
        description: "High-res photographic images for codec evaluation",
        sha256: "...",
    },
    DatasetInfo {
        name: "cid22",
        size_mb: 94.0,
        description: "250 diverse images, 512x512",
        sha256: "...",
    },
    // ...
];
```

The crate version is obtained from `env!("CARGO_PKG_VERSION")` at compile time. The GitHub tag for downloads is derived as `v{CARGO_PKG_VERSION}`.

## Download strategy

When `corpus.get("dataset/subdir")` is called and the cache is missing or stale:

### Step 1: Parse the request

Split `path` on the first `/`:
- `"webp-conformance"` → dataset=`webp-conformance`, subdir=`None`
- `"webp-conformance/valid"` → dataset=`webp-conformance`, subdir=`Some("valid")`
- `"clic2025/training"` → dataset=`clic2025`, subdir=`Some("training")`

Validate that `dataset` exists in `DATASETS`. Return `Error::UnknownDataset` if not.

### Step 2: Check cache validity

```
version_file = {root}/.version
if version_file exists AND contents.trim() == CRATE_VERSION:
    if {root}/{dataset}/ exists:
        goto step 6 (return path)
    else:
        need to download this specific dataset
else:
    need to re-download (version mismatch or first run)
```

### Step 3: Acquire lock

Open `{root}/.lock` with `fd-lock` (exclusive lock). After acquiring, re-check the version file — another process may have completed the download while we waited.

### Step 4: Download

Try methods in order until one succeeds:

#### 4a: Shell `git` sparse checkout

```bash
temp_dir={root}/.tmp-{pid}-{timestamp}/

git clone --depth=1 --filter=blob:none --sparse \
  --branch "v{CRATE_VERSION}" \
  "https://github.com/imazen/codec-corpus.git" \
  "{temp_dir}"

cd "{temp_dir}"
git sparse-checkout set "{dataset}/"
```

On success:
- Move `{temp_dir}/{dataset}/` → `{root}/{dataset}/`
- Remove `{temp_dir}/`

On failure (git not installed, clone fails, timeout): fall through to 4b.

#### 4b: HTTP tarball download

Construct URL:
```
https://github.com/imazen/codec-corpus/releases/download/v{CRATE_VERSION}/{dataset}.tar.gz
```

Download to `{root}/.tmp-{pid}-{timestamp}.tar.gz` using the first available tool:

**Linux/macOS:**
```bash
curl -fSL -o "{temp_file}" "{url}"
```

If `curl` not found:
```bash
wget -q -O "{temp_file}" "{url}"
```

**Windows:**
```powershell
powershell -Command "Invoke-WebRequest -Uri '{url}' -OutFile '{temp_file}'"
```

If `curl` is available on Windows (common on modern Windows), try it first before PowerShell.

**Tool detection**: Use `std::process::Command::new("curl").arg("--version")` (or equivalent) and check exit code. Cache the result for the session.

After download:
- Compute SHA-256 of the downloaded file.
- Compare against `DatasetInfo.sha256`.
- On mismatch: delete temp file, return `Error::ChecksumMismatch`.
- On match: extract with `tar_light::unpack()` to `{root}/.tmp-{pid}-{timestamp}/`.
- Move `{root}/.tmp-{pid}-{timestamp}/{dataset}/` → `{root}/{dataset}/`.
- Clean up temp files.

If all HTTP tools fail: return `Error::NetworkUnavailable`.

### Step 5: Write version file

After successful download, write `{root}/.version` with the crate version string.

Also clean up orphaned `.tmp-*` entries older than 1 hour (from crashed previous runs).

Release the lock.

### Step 6: Return path

```
full_path = {root}/{dataset}/{subdir}  (or just {root}/{dataset}/ if no subdir)

if full_path does not exist:
    return Error::PathNotFound

return Ok(full_path)
```

## Atomic download safety

- All downloads target `.tmp-{pid}-{timestamp}/` or `.tmp-{pid}-{timestamp}.tar.gz`.
- Only after successful download, checksum verification, and extraction does the final directory appear via rename/move.
- If the process crashes mid-download, orphaned `.tmp-*` entries are harmless and cleaned up on the next successful run.
- `fd-lock` prevents concurrent processes from downloading the same dataset simultaneously. After acquiring the lock, the version is re-checked to handle the case where another process completed the download while we waited.

## Concurrent access

Multiple test processes (e.g. `cargo test -j4`) may call `Corpus::get()` simultaneously:

1. First process acquires `.lock`, starts downloading.
2. Other processes block on `.lock`.
3. First process finishes download, writes `.version`, releases `.lock`.
4. Other processes acquire `.lock`, re-check `.version`, find it matches, skip download.

This is safe because:
- Only one process downloads at a time.
- The version file is written atomically (write to temp, rename).
- The dataset directory appears atomically (rename from `.tmp-*`).

## `from_local()` behavior

`Corpus::from_local(path)` bypasses all download logic, version checking, and locking.

The provided path is used directly as the corpus root. `corpus.get("webp-conformance/valid")` simply returns `{path}/webp-conformance/valid/` if it exists, or `Error::PathNotFound` if not.

This is useful for:
- CI environments where the corpus is pre-cloned.
- Developer machines with a local checkout: `Corpus::from_local("~/work/codec-eval/codec-corpus")`.
- Offline testing.
- Environments where network access is restricted.

## SHA-256 verification

Checksums are computed on the `.tar.gz` file after download, before extraction.

The SHA-256 implementation uses Rust's standard approach (a small embedded implementation or a lightweight crate like `sha2`). No OpenSSL dependency.

On checksum mismatch:
1. Delete the downloaded temp file.
2. Return `Error::ChecksumMismatch` with expected and actual hashes.
3. Do NOT retry automatically (avoids infinite loops on bad releases).

The caller can retry if desired.

## Platform support

| Platform | Cache dir | git | HTTP fallback |
|----------|-----------|-----|---------------|
| Linux | `~/.cache/codec-corpus/` | `git` | `curl`, `wget` |
| macOS | `~/Library/Caches/codec-corpus/` | `git` | `curl`, `wget` |
| Windows | `%LOCALAPPDATA%\codec-corpus\` | `git` | `curl`, `powershell Invoke-WebRequest` |

All file operations use `std::fs` and `std::path` for cross-platform compatibility. No hardcoded path separators.

## Usage examples

### Basic test integration

```rust
// zenwebp/Cargo.toml
// [dev-dependencies]
// codec-corpus = "1"

// zenwebp/tests/webp_conformance.rs
use codec_corpus::Corpus;

#[test]
#[ignore]
fn test_webp_valid() {
    let corpus = match Corpus::new() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Skipping conformance test: {e}");
            return;
        }
    };

    let valid_dir = match corpus.get("webp-conformance/valid") {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Skipping: {e}");
            return;
        }
    };

    for entry in std::fs::read_dir(valid_dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().map_or(false, |e| e == "webp") {
            let data = std::fs::read(&path).unwrap();
            // decode and validate...
        }
    }
}
```

### Custom cache location

```rust
let corpus = Corpus::with_cache_root("/mnt/fast-storage")?;
let clic = corpus.get("clic2025/training")?;
```

Or via environment variable:
```bash
CODEC_CORPUS_CACHE=/mnt/fast-storage cargo test -- --ignored
```

### Local checkout (no downloads)

```rust
let corpus = Corpus::from_local("/home/user/work/codec-eval/codec-corpus");
let webp = corpus.get("webp-conformance/valid")?;
```

### CI integration

No special CI setup required. `cargo test` handles everything:

```yaml
conformance:
  name: Conformance Tests
  runs-on: ubuntu-latest
  if: github.ref == 'refs/heads/main'
  continue-on-error: true
  steps:
    - uses: actions/checkout@v6
    - uses: dtolnay/rust-toolchain@stable
    - uses: Swatinem/rust-cache@v2

    # Cache the corpus across CI runs
    - uses: actions/cache@v4
      with:
        path: ~/.cache/codec-corpus
        key: corpus-v${{ hashFiles('Cargo.lock') }}
        restore-keys: corpus-v

    # Just run tests - codec-corpus handles downloading
    - run: cargo test --release -- --ignored
```

### Listing available datasets

```rust
for ds in Corpus::list() {
    println!("{}: {} ({:.1} MB)", ds.name, ds.description, ds.size_mb);
}
// Output:
// webp-conformance: RFC 6386 WebP conformance test files (225 files) (1.3 MB)
// clic2025: High-res photographic images for codec evaluation (219.0 MB)
// cid22: 250 diverse images, 512x512 (94.0 MB)
// ...
```

## Releasing a new version

### Data change (new test files, updated datasets)

1. Commit changes to `imazen/codec-corpus` repository.
2. Update the crate source:
   - Bump version in `Cargo.toml` (e.g. `1.0.0` → `1.1.0`)
   - Update `CORPUS_COMMIT` constant
   - Update SHA-256 checksums for affected datasets
   - Update `DatasetInfo` entries if datasets were added/removed
3. Create GitHub tag `v1.1.0` on the corpus repository.
4. Create GitHub release `v1.1.0` with per-dataset `.tar.gz` assets.
5. Publish crate: `cargo publish`.

### Code-only change (bug fix in download logic, API change)

1. Fix the bug in the crate source.
2. Bump version in `Cargo.toml` (e.g. `1.0.0` → `1.0.1`).
3. Create GitHub tag `v1.0.1` on the corpus repository (same data as v1.0.0).
4. Create GitHub release `v1.0.1` with same per-dataset `.tar.gz` assets.
5. Publish crate: `cargo publish`.

The tag and release are required even for code-only changes because the download URL includes the version number.

### Creating release assets

```bash
# From the codec-corpus repository root
for dataset in webp-conformance clic2025 cid22 gb82 gb82-sc \
               jpeg-conformance jxl pngsuite qoi-benchmark \
               image-rs imageflow mozjpeg zune; do
    tar czf "${dataset}.tar.gz" "${dataset}/"
    sha256sum "${dataset}.tar.gz" >> checksums.sha256
done

# Upload to GitHub release
gh release create "v1.0.0" *.tar.gz checksums.sha256
```

## File structure of the crate

```
codec-corpus/           (the crate, NOT the data repo)
  Cargo.toml
  src/
    lib.rs              (~400 lines)
    download.rs         (~200 lines: git, curl, wget, powershell)
    checksum.rs         (~50 lines: SHA-256 verification)
  README.md
  LICENSE
  SPEC.md              (this file)
```

## Design decisions

### Why runtime download, not build.rs?

`build.rs` runs on every `cargo build`, adds latency even when cached, is hard to debug, and can't be skipped. Runtime download only runs when a test actually needs corpus data.

### Why shell `git` instead of `gix`?

`gix` (gitoxide) compiles to 20+ MB. Shell `git` is already installed on most developer and CI machines. If git is unavailable, the tarball fallback works everywhere.

### Why shell `curl`/`wget`/`powershell` instead of `ureq`/`reqwest`?

Zero additional Rust dependencies for HTTP. These tools are ubiquitous:
- `curl` is pre-installed on macOS, most Linux, and modern Windows.
- `wget` is available on most Linux.
- `powershell` is always available on Windows.
- `curl.exe` ships with Windows 10+ (build 17063+).

### Why `tar_light`?

Lightweight pure-Rust tar extraction. No C dependencies, no OpenSSL. Supports `.tar.gz` which is the release asset format.

### Why per-dataset tarballs instead of one large archive?

Projects typically need 1-2 datasets, not all 12+. Per-dataset tarballs mean zenwebp downloads 1.3 MB (webp-conformance) instead of 990 MB (everything).

### Why crate version = GitHub tag?

Simplicity. One version number governs both the API and the data. `Cargo.lock` pins the exact version, which determines the exact data files. Reproducible by construction.

The cost is uploading duplicate data for code-only releases. This is acceptable because releases are infrequent and GitHub has no meaningful size limits on release assets.

### Why re-download on any semver change?

A patch release (v1.0.0 → v1.0.1) might add new test files. Trusting stale data risks false test passes. The download is cached, so re-downloading is a one-time cost per version bump.

### Why major version subfolders?

Different projects on the same machine may depend on different major versions of codec-corpus. Without subfolders, upgrading one project would break another. `v1/` and `v2/` can coexist.
