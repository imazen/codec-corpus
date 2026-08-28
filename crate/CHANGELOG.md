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
- `R2Corpus` pull (#2): individual objects are fetched concurrently
  (`PullOptions::parallelism`, default `DEFAULT_PARALLELISM` = 8, one
  `curl`/`wget` process each; first failure stops the rest, verified files
  stay, temp files are cleaned up). Hierarchical prefixes: a `.list` may name
  `children` (validated single-component sub-prefixes; `ListIndex::add_child`,
  `ListIndex::empty`, `ListIndex::validate`), and a pull recurses into them —
  `pull("fuzz/")` fetches everything below — merging child files into
  `R2Corpus::list()` as `child/rel`. `PullOptions::recursive(false)` pulls a
  node's own files only. Offline mode walks the cached tree the same way.
- **`R2Corpus::push`** (#2, authenticated push): publishes a local directory
  as a prefix — hashes it, diffs against the remote `.list`, uploads only
  new/changed objects (`PushOptions::parallelism`, default 4) via an `aws s3
  cp --endpoint-url` shell-out (no compiled-in S3 client; credentials go in
  the subprocess environment, never on the command line), regenerates the
  bundle with the system `tar` per `Rebundle::{Auto{max_deltas},Always,Never}`
  (`BundleFormat::TarZst` degrading to `.tar.gz`/`.tar` when the tools are
  missing), uploads the `.list` last, and registers the prefix as a child in
  every ancestor `.list`. `PushTarget` (`from_env` / `load` / `save` — the
  `r2-corpus login` file, 0600) names the endpoint, bucket, public base URL
  and optional keys. `PushOptions::dry_run` and the credential-free
  `R2Corpus::diff` report the plan (`PushReport`) without uploading;
  `R2Corpus::fetch_list` reads a published `.list`. Tests drive the whole
  algorithm against an on-disk fake bucket (first push + ancestor
  registration + pull-back through the real pull path, incremental deltas
  with a stale bundle, auto-rebundle threshold with a real `tar` round trip,
  dry run, bad inputs); the live `aws` upload path is exercised only by
  `aws_cp_args` unit tests.
- New sibling workspace crate [`r2-corpus`](r2-corpus/README.md) (#2): the
  CLI (`pull` / `push` / `list` / `diff` / `login` / `sync` +
  `corpus-sync.toml`). Separate package — `codec-corpus` itself gains no
  dependency.
- New sibling workspace crate [`corruption-corpus`](corruption-corpus/README.md): the deterministic structural-corruption generator for zensim negative-tail validation (#7), landed on `main` from PR #8. It is a separate package — `codec-corpus` itself gains no new dependencies or API.

### Fixed
- `libjpeg-turbo-fuzz` now resolves to `libjpeg-turbo/seed-corpora` `afl-testcases`
  (bmp/gif/jpeg/targa seed trees). It previously pointed at
  `libjpeg-turbo/fuzz` `seed_corpus`, a directory that repo never contained, so
  every `get("libjpeg-turbo-fuzz")` returned `PathNotFound`. On Windows the
  entry now returns `Error::DownloadUnsupported` up front: the upstream
  AFL-style file names contain `:`, which NTFS cannot store, so the checkout
  cannot succeed there (previously a misleading `NetworkUnavailable`).
- Doc examples and the `adhoc_github_repo` integration test no longer reference
  `niclas-aspect/jxl-rs`, a repository that does not exist (GitHub 404).
- `sha256::tests::file_matches_in_memory` no longer aborts the wasm32-wasip1
  unit-test run (`std::env::temp_dir()` panics on WASI); it writes under
  `target/` there, which CI preopens.

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
