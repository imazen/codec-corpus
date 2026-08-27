# Changelog

## [Unreleased]

### Fixed
- `block_repeat_neighbor` was an exact identity for a whole-image region (the
  "above" neighbor wrapped onto the pixel's own row) and a no-op inside flat
  regions. It now copies the adjacent same-size block (left, else right, else
  above, else below) and, for a whole-image region, repeats the first 8-px
  column band of each row (a decoder stuck re-emitting one MCU). (#9)
- `chroma_boundary` only sampled the 1-px horizontal neighbor's chroma, an
  identity wherever chroma varies slowly — measured at zero changed pixels for
  every localized region on real content. It now acts on both row and column
  block-boundary bands and takes chroma from one full block (8 px) across the
  boundary, keeping luma. (#9)
- New `CorruptionParams::is_identity_on()` and `manifest_for_image()` drop
  entries that change zero pixels of a given reference (e.g. any chroma defect
  on achromatic content), so a `_MANIFEST.json` never lists an un-catchable
  "defect". The `corruption_corpus` example uses it and reports the count. (#9)

### Added
- `references` module: selects the ≥ 5 content classes × ≥ 10 references
  sweep set from the public imazen-26 corpus manifests checked into this repo
  (`content_class_for_category` folds the 21 imazen-26 categories into the
  five `ContentClass`es; `parse_imazen26_manifest` + `select_per_class` pick a
  deterministic, category-balanced set). The `corruption_corpus` example
  gained `--refs-tsv` / `--per-class` / `--refs-dir` to download and sweep
  them. (#7)
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
