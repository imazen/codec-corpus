# Changelog

## [Unreleased]

### Added
- **`R2Corpus`** (#2, phase 1 — anonymous pull): path-keyed sync of a public
  R2/HTTP prefix into the cache. Fetches the auto-generated `<prefix>.list`
  (`ListIndex`: `{path: {size, sha256, in_bundle}}` + optional `BundleInfo`),
  diffs the local cache by size + SHA-256, takes the `.tar`/`.tar.gz`/`.tar.zst`
  bundle when ≥40 % of its files are missing (`PullMode::Auto`; also
  `ForceBundle` / `NoBundle` / `Offline`), fetches the rest individually, verifies
  every file before it lands, and prunes unlisted files. Same `curl`/`wget`/
  `powershell` + system `tar` shell-outs as `Corpus`; SHA-256 is implemented
  in-crate (FIPS 180-4 vectors tested) so no new dependencies. `ListIndex::from_dir`
  builds the index a push tool uploads. `DEFAULT_R2_BASE_URL` points at the
  public `codec-corpus.r2.imazen.org` bucket.
- `Error::ChecksumMismatch` and `Error::ListParse` (additive; `Error` is
  `#[non_exhaustive]`).
- New sibling workspace crate [`corruption-corpus`](corruption-corpus/README.md): the deterministic structural-corruption generator for zensim negative-tail validation (#7), landed on `main` from PR #8. It is a separate package — `codec-corpus` itself gains no new dependencies or API.

### Fixed
- `libjpeg-turbo-fuzz` now resolves to `libjpeg-turbo/seed-corpora` `afl-testcases`
  (bmp/gif/jpeg/targa seed trees). It previously pointed at
  `libjpeg-turbo/fuzz` `seed_corpus`, a directory that repo never contained, so
  every `get("libjpeg-turbo-fuzz")` returned `PathNotFound`.
- Doc examples and the `adhoc_github_repo` integration test no longer reference
  `niclas-aspect/jxl-rs`, a repository that does not exist (GitHub 404).

## [1.1.0] - 2026-04-14

### Added
- **Third-party corpus registry**: `oss-fuzz/*`, `go-fuzz-corpus/*`, `libjpeg-turbo-fuzz`, `image-rs/*` prefixes resolve to external sources automatically (e95fd49)
- **Ad-hoc download methods**: `github_repo()`, `zip_url()`, `tar_url()`, `local_path()` for arbitrary sources (e95fd49)
- **ManifestCorpus**: R2 blob fetching via JSONL manifest files (7b430b7)
- **wasm32-wasip1 support**: `fd-lock` gated behind `cfg(not(wasm32))`; crate compiles and runs on WASM (4e57be6)
- **`Error::DownloadUnsupported`**: new variant returned on WASM instead of misleading `NetworkUnavailable` (fbaed8a)
- **wasm CI**: compile checks (scalar + SIMD128), clippy, and unit tests via wasmtime (4b973cc)
- **WASM usage docs**: README section documenting native-prefetch + wasmtime-preopen pattern (445c50d)
- `with_max_age()` for configurable third-party cache staleness (default 7 days)
- `serde` and `serde_json` dependencies for manifest parsing

### Fixed
- `fd-lock` compile failure on `wasm32-wasip1` (4e57be6)

## [1.0.3] - 2026-03-04

### Added
- BMP conformance dataset table in README (47bbcb8)

## [1.0.2] - 2026-02-22

### Added
- Initial third-party source support groundwork

## [1.0.1] - 2026-02-14

### Fixed
- Cache directory resolution on Windows

## [1.0.0] - 2026-02-13

### Added
- Initial release: `Corpus::new()`, `get()`, `is_cached()`, `list_cached()`
- Git sparse checkout + HTTP tarball download with fallback
- `CODEC_CORPUS_CACHE` env var override
- Cross-process file locking via `fd-lock`
