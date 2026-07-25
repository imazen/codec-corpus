# Changelog

## [Unreleased]

### Added
- **Initial extraction** from `codec-corpus` — this crate was the
  `codec_corpus::corruptions` module through codec-corpus 1.1.x, now split into
  its own workspace member so the structural-corruption *generator* is decoupled
  from the corpus *fetcher*. The module was self-contained (it depended only on
  `image`, `serde`, and its own `prng`), so the split is a clean move; no logic
  changed.
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
