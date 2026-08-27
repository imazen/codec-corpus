# codec-corpus

Runtime access to the [imazen/codec-corpus](https://github.com/imazen/codec-corpus) test image collection and third-party fuzz/conformance corpora. No data ships with the crate — datasets download on first use and cache locally.

```rust
let corpus = codec_corpus::Corpus::new()?;
let valid = corpus.get("webp-conformance/valid")?;
for entry in std::fs::read_dir(valid)? {
    let path = entry?.path();
    // decode, validate, benchmark...
}
```

`get()` returns a `PathBuf` to **encoded** image bytes on disk — a directory of dataset files, or a single file when you pass a path to one. It does **no decoding**: `std::fs::read` the path (or walk the directory) to get the raw bytes, then hand them to your own decoder. On a cold cache, `get()` performs a **blocking subprocess download** (`git`, falling back to `curl`/`wget`/`powershell`), so first use needs network access and one of those binaries — gate it behind `#[ignore]` in tests.

## What it does

1. You call `corpus.get("some-folder/optional-subpath")`
2. If the folder isn't cached (or the cache is stale), it downloads via git sparse checkout or HTTP tarball
3. Returns the local path. Done.

Downloads use shell `git` (preferred) with fallback to `curl`/`wget`/`powershell`. No heavy HTTP crate dependencies.

## Install

```toml
[dev-dependencies]
codec-corpus = "1"
```

## Usage

### imazen/codec-corpus (default)

Any path without a recognized third-party prefix downloads from `imazen/codec-corpus`:

```rust
use codec_corpus::Corpus;

#[test]
#[ignore] // network access required
fn jpeg_conformance() {
    let corpus = Corpus::new().unwrap();
    let valid = corpus.get("jpeg-conformance/valid").unwrap();

    for entry in std::fs::read_dir(valid).unwrap() {
        let path = entry.unwrap().path();
        let data = std::fs::read(&path).unwrap();
        // test your decoder...
    }
}
```

### Third-party sources (built-in registry)

Prefixed paths resolve to known external sources automatically:

```rust
let corpus = codec_corpus::Corpus::new().unwrap();

// OSS-Fuzz backup corpora (downloaded as ZIP from GCS)
let ossfuzz_jpeg = corpus.get("oss-fuzz/libjpeg-turbo").unwrap();
let ossfuzz_png = corpus.get("oss-fuzz/libpng").unwrap();
let ossfuzz_jxl = corpus.get("oss-fuzz/libjxl").unwrap();

// dvyukov/go-fuzz-corpus subfolders (git sparse checkout)
let gofuzz_gif = corpus.get("go-fuzz-corpus/gif").unwrap();
let gofuzz_png = corpus.get("go-fuzz-corpus/png").unwrap();
let gofuzz_jpeg = corpus.get("go-fuzz-corpus/jpeg").unwrap();

// libjpeg-turbo fuzz seed corpus (git sparse checkout)
let ljt_fuzz = corpus.get("libjpeg-turbo-fuzz").unwrap();

// image-rs/image test images (git sparse checkout)
let imagers = corpus.get("image-rs/tests/images").unwrap();
```

Third-party sources are cached under `third-party/` in the cache root and re-fetched if the `.fetched` marker is older than 7 days (configurable via `with_max_age()`).

### Ad-hoc sources

Fetch from arbitrary locations without needing them in the built-in registry:

```rust
let corpus = codec_corpus::Corpus::new().unwrap();

// Arbitrary GitHub repo subfolder
let path = corpus.github_repo("niclas-aspect/jxl-rs", "test-data", "main").unwrap();

// Download and cache a ZIP URL
let path = corpus.zip_url("custom-corpus", "https://example.com/corpus.zip").unwrap();

// Download and cache a tarball URL
let path = corpus.tar_url("my-corpus", "https://example.com/corpus.tar.gz").unwrap();

// Point at a local directory (validates existence, no download)
let path = corpus.local_path("/home/user/my-images").unwrap();
```

### Public R2 prefixes (`R2Corpus`)

Continuously-growing corpora — fuzz seeds, CI-generated fixtures — are too large
and too churny for GitHub releases. They live in a **public-read R2 bucket**
(`https://codec-corpus.r2.imazen.org`), one auto-generated `.list` index per
prefix, and are pulled anonymously with the same `curl`/`wget`/`powershell`
chain as everything else. Path-keyed, not hash-keyed: you get a directory that
mirrors the remote prefix and walk it with `std::fs`.

```rust
use codec_corpus::{R2Corpus, DEFAULT_R2_BASE_URL};

let seeds = R2Corpus::pull(DEFAULT_R2_BASE_URL, "fuzz/zentiff/fuzz_decode/")?;
for entry in std::fs::read_dir(seeds.path())? {
    let bytes = std::fs::read(entry?.path())?;
    // feed your decoder / fuzzer...
}
```

How a pull works: fetch `<prefix>.list` (one small request) → diff the local
cache by size + SHA-256 → if the prefix has a frozen `.tar`/`.tar.gz`/`.tar.zst`
bundle and ≥40 % of its files are missing, fetch the bundle once and extract it
with the system `tar` → fetch the remaining objects individually → verify every
file against its listed hash before it lands → remove cached files that are no
longer listed. Every step is verified; a hash mismatch is `Error::ChecksumMismatch`
and nothing is placed.

```rust
use codec_corpus::{R2Corpus, PullOptions, PullMode};

// Cold start on a big prefix: always take the bundle.
let c = R2Corpus::pull_with_options(base, prefix, PullOptions::default().mode(PullMode::ForceBundle))?;

// CI / WASM: never touch the network, use what a previous pull cached.
let c = R2Corpus::pull_with_options(base, prefix, PullOptions::default().mode(PullMode::Offline))?;
```

Cache layout: `{cache}/codec-corpus/r2/{host}/{prefix…}/` with a `.list.json`
copy of the index the directory was last synced against. Not version-gated —
content is addressed by the hashes in the list.

**Publishing a prefix** (maintainers): `ListIndex::from_dir(local_dir, prefix)`
hashes a directory into the index; upload the files under `<prefix>` and the
index as `<prefix-without-trailing-slash>.list`. The upload itself is not in this
crate yet (tracked in [#2](https://github.com/imazen/codec-corpus/issues/2)).

### Custom cache location

```rust
let corpus = Corpus::with_cache_root("/mnt/fast-storage")?;
```

Or via environment variable:

```bash
CODEC_CORPUS_CACHE=/mnt/fast-storage cargo test -- --ignored
```

### Cache staleness

Third-party sources use a `.fetched` timestamp marker. By default, sources older than 7 days are re-fetched:

```rust
use std::time::Duration;
let corpus = codec_corpus::Corpus::new()?
    .with_max_age(Duration::from_secs(30 * 24 * 3600)); // 30 days
```

### Check what's cached

```rust
if corpus.is_cached("pngsuite") {
    println!("already downloaded");
}

for name in corpus.list_cached() {
    println!("cached: {name}");
}
```

## Built-in third-party registry

| Prefix | Source | Download method |
|--------|--------|-----------------|
| `oss-fuzz/libjpeg-turbo` | OSS-Fuzz backup corpus (cjpeg_fuzzer) | ZIP from GCS |
| `oss-fuzz/libpng` | OSS-Fuzz backup corpus (read_fuzzer) | ZIP from GCS |
| `oss-fuzz/libjxl` | OSS-Fuzz backup corpus (djxl_fuzzer) | ZIP from GCS |
| `go-fuzz-corpus/gif` | dvyukov/go-fuzz-corpus gif/corpus | git sparse checkout |
| `go-fuzz-corpus/png` | dvyukov/go-fuzz-corpus png/corpus | git sparse checkout |
| `go-fuzz-corpus/jpeg` | dvyukov/go-fuzz-corpus jpeg/corpus | git sparse checkout |
| `libjpeg-turbo-fuzz` | libjpeg-turbo/fuzz seed_corpus | git sparse checkout |
| `image-rs/{subpath}` | image-rs/image {subpath} | git sparse checkout |

## Datasets (imazen/codec-corpus)

Any top-level folder in the [codec-corpus repo](https://github.com/imazen/codec-corpus) is a valid path. Pass any path into `get()` — the first component determines the download unit.

### Quality calibration

| Dataset | Download | Files | Description | License |
|---------|----------|-------|-------------|---------|
| `clic2025` | 218 MB | 64 | High-res photos for codec evaluation (~2048px) | Unsplash |
| `CID22` | 86 MB | 251 | Diverse 512x512 images (209 training + 41 validation) | CC BY-SA 4.0 |
| `kadid10k` | 25 MB | 82 | Pristine IQA reference images, 512x384 | Pixabay |
| `gb82` | 9.6 MB | 26 | Challenging CC0 photos, 576x576 | CC0 |
| `gb82-sc` | 3.0 MB | 11 | Screenshots and screen content | CC0 |
| `qoi-benchmark` | 39 MB | 18 | Full-page web screenshots | CC0 |

### Format conformance

| Dataset | Download | Files | Description | License |
|---------|----------|-------|-------------|---------|
| `bmp-conformance` | 2.1 MB | 126 | BMP spec conformance (valid, invalid, non-conformant, crash repros) | MIT |
| `jpeg-conformance` | 76 MB | 277 | 41 valid, 116 invalid, 40 non-conformant, 78 crash repros | MIT/IJG+BSD |
| `jxl` | 88 MB | 188 | Conformance, features, edge cases, photographic | BSD-3-Clause |
| `png-conformance` | 1.7 MB | 11 | Real-world PNG edge cases (decompressor crashes, filter bugs) | Various |
| `pngsuite` | 720 KB | 178 | Official PNG conformance tests (all color types, depths) | Freeware |
| `webp-conformance` | 1.3 MB | 230 | 225 valid WebP files + sources | Various |

### Decoder robustness

| Dataset | Download | Files | Description | License |
|---------|----------|-------|-------------|---------|
| `zune` | 33 MB | 3,434 | Fuzz corpus (1,836 JPEG + 837 PNG) and test images | MIT/Apache-2.0/Zlib |
| `image-rs` | 4.5 MB | 127 | Multi-format edge cases (BMP, GIF, JPEG, PNG, TIFF, WebP) | MIT |
| `imageflow` | 7.8 MB | 51 | Orientation, format conversion edge cases | Various |
| `mozjpeg` | 1.2 MB | 16 | MozJPEG encoder reference files | IJG + BSD |

Full dataset descriptions and per-file attribution: [codec-corpus README](https://github.com/imazen/codec-corpus#readme).

## Cache layout

```
~/.cache/codec-corpus/v1/          # Linux
~/Library/Caches/codec-corpus/v1/  # macOS
%LOCALAPPDATA%\codec-corpus\v1\    # Windows

  .version          # "1.0.3" — triggers re-download on version change
  .lock             # fd-lock for concurrent access
  pngsuite/
  jpeg-conformance/
  third-party/                     # Third-party sources (not version-gated)
    oss-fuzz__libpng/
      .fetched      # Unix timestamp — re-fetch if >7 days old
      ...files...
    go-fuzz-corpus__gif/
    github__owner__repo__path/
    ...
```

Different major versions coexist (`v1/`, `v2/`). imazen/codec-corpus datasets re-download on any crate version change. Third-party sources use time-based staleness (default 7 days) and survive version changes.

## CI integration

```yaml
- uses: actions/cache@v4
  with:
    path: ~/.cache/codec-corpus
    key: corpus-v1
- run: cargo test --release -- --ignored
```

No special setup. The crate handles downloading; the CI cache avoids re-downloading across runs.

## Using codec-corpus from WASM

`codec-corpus` compiles for `wasm32-wasip1` (and `wasm32-unknown-unknown`). On WASM, downloads are not available since the crate relies on host subprocesses (`git`, `curl`). Instead, **prefetch on the host** and **preopen the cache** for the WASM runtime.

### 1. Prefetch on the host (native)

```bash
# Download the datasets you need into a local cache directory
CODEC_CORPUS_CACHE=$(pwd)/corpus cargo run --example prefetch -- cid22 webp-conformance
```

Or just run your native tests once — `Corpus::new()` populates `~/.cache/codec-corpus/` automatically.

### 2. Run WASM tests with a preopened cache

```bash
# Map the host cache directory into the WASM sandbox
CODEC_CORPUS_CACHE=/corpus \
  CARGO_TARGET_WASM32_WASIP1_RUNNER="wasmtime --dir $(pwd)/corpus::/corpus" \
  cargo test --target wasm32-wasip1
```

The `--dir host_path::guest_path` flag gives the WASM process read access to the host directory at the guest mount point.

### 3. In your WASM code

```rust
// Point at the preopened cache root
let corpus = codec_corpus::Corpus::with_cache_root("/corpus").unwrap();

// These work — they only read the filesystem
assert!(corpus.is_cached("webp-conformance"));
let datasets = corpus.list_cached();
let path = corpus.get("webp-conformance").unwrap();

// This returns Error::DownloadUnsupported — by design
let err = corpus.get("not-yet-cached").unwrap_err();
// "downloads are not supported on this platform; dataset 'not-yet-cached' must be pre-cached on the host"
```

Calling `get()` for an uncached dataset on WASM returns `Error::DownloadUnsupported` (not `NetworkUnavailable`). This is intentional — downloads must happen host-side.

## Structural-corruption corpus

The structural-corruption *generator* — a held-out falsification set for a
perceptual metric's negative tail (channel swaps, dropped blocks, off-by-one
edges, and seven more deterministic distortion families) — lives in the sibling
**[`corruption-corpus`](corruption-corpus/)** workspace crate, decoupled from
this fetcher. See its [README](corruption-corpus/README.md) for the family
catalog, the `score(corruption) < score(q20-anchor)` gate, and the
`corruption_corpus` generator example.

## Dependencies

Four Rust crates, all small:

- `dirs` — cross-platform cache directory
- `fd-lock` — file locking for concurrent safety (native only, not on WASM)
- `serde` + `serde_json` — manifest parsing for R2/JSONL corpus sources

Archive extraction uses system commands (`tar`, `unzip`, `powershell`). No `reqwest`, `ureq`, `gix`, or `toml`.

## License

The crate itself is Apache-2.0. Each dataset in the corpus has its own license — see the table above and the [full license summary](https://github.com/imazen/codec-corpus#license-summary).
