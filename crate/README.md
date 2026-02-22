# codec-corpus

Runtime access to the [imazen/codec-corpus](https://github.com/imazen/codec-corpus) test image collection. No data ships with the crate — datasets download on first use and cache locally.

```rust
let corpus = codec_corpus::Corpus::new()?;
let valid = corpus.get("webp-conformance/valid")?;
for entry in std::fs::read_dir(valid)? {
    let path = entry?.path();
    // decode, validate, benchmark...
}
```

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

### Custom cache location

```rust
let corpus = Corpus::with_cache_root("/mnt/fast-storage")?;
```

Or via environment variable:

```bash
CODEC_CORPUS_CACHE=/mnt/fast-storage cargo test -- --ignored
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

## Datasets

Any top-level folder in the [codec-corpus repo](https://github.com/imazen/codec-corpus) is a valid path. Pass any path into `get()` — the first component determines the download unit.

### Quality calibration

| Dataset | Download | Files | Description | License |
|---------|----------|-------|-------------|---------|
| `clic2025` | 218 MB | 64 | High-res photos for codec evaluation (~2048px) | Unsplash |
| `CID22` | 86 MB | 251 | Diverse 512×512 images (209 training + 41 validation) | CC BY-SA 4.0 |
| `kadid10k` | 25 MB | 82 | Pristine IQA reference images, 512×384 | Pixabay |
| `gb82` | 9.6 MB | 26 | Challenging CC0 photos, 576×576 | CC0 |
| `gb82-sc` | 3.0 MB | 11 | Screenshots and screen content | CC0 |
| `qoi-benchmark` | 39 MB | 18 | Full-page web screenshots | CC0 |

### Format conformance

| Dataset | Download | Files | Description | License |
|---------|----------|-------|-------------|---------|
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

  .version          # "1.0.0" — triggers re-download on version change
  .lock             # fd-lock for concurrent access
  pngsuite/
  jpeg-conformance/
  ...
```

Different major versions coexist (`v1/`, `v2/`). Any crate version change within a major version triggers a re-download to ensure correctness.

## CI integration

```yaml
- uses: actions/cache@v4
  with:
    path: ~/.cache/codec-corpus
    key: corpus-v1
- run: cargo test --release -- --ignored
```

No special setup. The crate handles downloading; the CI cache avoids re-downloading across runs.

## Dependencies

Two Rust crates, both small:

- `dirs` — cross-platform cache directory
- `fd-lock` — file locking for concurrent safety

Archive extraction uses the system `tar` command. No `reqwest`, `ureq`, `gix`, `serde`, or `toml`.

## License

The crate itself is Apache-2.0. Each dataset in the corpus has its own license — see the table above and the [full license summary](https://github.com/imazen/codec-corpus#license-summary).
