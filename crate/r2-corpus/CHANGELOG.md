# Changelog

## [Unreleased]

### Added
- **Initial crate** (#2): the `r2-corpus` binary — `pull <prefix> [--into DIR]`
  (anonymous subtree pull; prints the cache path, optionally mirrors listed
  files into a project dir without deleting unlisted ones), `push <prefix>
  --local DIR [--rebundle|--no-bundle|--max-deltas N] [--format zst|gz|tar]
  [--dry-run]`, `list <prefix> [--json]`, `diff <prefix> --local DIR`,
  `login --endpoint URL --bucket NAME` (keys via flags or prompted on stdin;
  saved with `codec_corpus::PushTarget::save`), and `sync [--config
  corpus-sync.toml] [--push] [--dry-run]` driven by a `[corpus]` +
  `[[sync]]` TOML file. Hand-rolled `std` argument parsing; `toml` is the only
  extra dependency. An integration test runs every command against a
  `file://` bucket through the real download chain (push only as a dry run).
