# Changelog

## [Unreleased]

### Changed
- Extracted the structural-corruption generator — which briefly lived here as the `corruptions` module + optional `driver` feature (never shipped in a tagged codec-corpus release) — into a new sibling workspace crate, [`corruption-corpus`](corruption-corpus/README.md), decoupling the corpus *fetcher* from the corruption *generator*. codec-corpus no longer depends on the `image` crate; the `driver` feature and the `corruption_corpus` example now live in `corruption-corpus`.

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
