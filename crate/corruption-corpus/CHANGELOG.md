# Changelog

## [Unreleased]

### Added
- **Initial crate**, landed on `main` as a workspace member of the
  `codec-corpus` repo (from PR #8), kept separate from the corpus *fetcher* so
  the generator has no download/registry code and the fetcher has no `image`
  dependency. Tracks imazen/codec-corpus#7.
- Structural-corruption distortion corpus: ten deterministic (SplitMix64-seeded,
  dependency-free) distortion families parameterized by region size + severity,
  plus a `ManifestEntry` schema and the `catalog()` / `manifest_for_reference()`
  sweep generators. Pure RGB-buffer math; stays in the default build and
  compiles to WASM.
- **`driver` feature** (optional `image` dep): the `driver` module builds the
  score-ready quad `(reference, corruption, q20-anchor, q10-anchor)` for the
  metric gate `score(corruption) < score(q20)`, plus the `corruption_corpus`
  example that emits images + `_MANIFEST.json` on demand (no corrupted bytes
  committed).
