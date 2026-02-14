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

| Path | Size | Description | License |
|------|------|-------------|---------|
| `clic2025/training` | 103 MB | 32 high-res photos for encoder tuning (~2048px) | Unsplash |
| `clic2025/final-test` | 116 MB | 30 holdout images — final evaluation only | Unsplash |
| `CID22/CID22-512/training` | — | 209 diverse 512×512 images (Cloudinary) | CC BY-SA 4.0 |
| `CID22/CID22-512/validation` | — | 41 holdout images | CC BY-SA 4.0 |
| `kadid10k` | 25 MB | 81 pristine IQA reference images, 512×384 | Pixabay |
| `gb82` | 9.6 MB | 25 challenging CC0 photos, 576×576 | CC0 |
| `gb82-sc` | 2.9 MB | 10 screenshots and screen content | CC0 |
| `qoi-benchmark/screenshot_web` | 39 MB | 14 full-page web screenshots | CC0 |

### Format conformance

| Path | Size | Description | License |
|------|------|-------------|---------|
| `jpeg-conformance/valid` | — | 41 JPEG files that MUST decode correctly | MIT/IJG+BSD |
| `jpeg-conformance/invalid` | — | 116 files that MUST be rejected gracefully | MIT/IJG+BSD |
| `jpeg-conformance/non-conformant` | — | 20 spec-violating files common in the wild | MIT/IJG+BSD |
| `jxl/conformance` | 6.2 MB | Official libjxl conformance tests | BSD-3-Clause |
| `jxl/features` | 81 MB | JPEG XL feature coverage (HDR, animation, etc.) | BSD-3-Clause |
| `pngsuite` | 720 KB | 176 PNG conformance tests (all color types, depths) | Freeware |
| `webp-conformance/valid` | — | WebP files that MUST decode correctly | Various |
| `webp-conformance/invalid` | — | WebP files that MUST be rejected | Various |

### Decoder robustness

| Path | Size | Description | License |
|------|------|-------------|---------|
| `image-rs/test-images` | 4.5 MB | Multi-format edge cases (BMP, GIF, JPEG, PNG, TIFF, WebP) | MIT |
| `zune/test-images/jpeg` | — | JPEG edge cases (CMYK, progressive, subsampling) | MIT/Apache-2.0/Zlib |
| `zune/fuzz-corpus/jpeg` | — | 1,836 minimal JPEG fuzz inputs | MIT/Apache-2.0/Zlib |
| `zune/fuzz-corpus/png` | — | 837 minimal PNG fuzz inputs | MIT/Apache-2.0/Zlib |
| `mozjpeg` | 1.2 MB | MozJPEG encoder reference files | IJG + BSD |
| `imageflow/test_inputs` | 7.8 MB | Orientation, format conversion edge cases | Various |

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
