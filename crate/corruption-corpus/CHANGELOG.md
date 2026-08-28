# Changelog

## [Unreleased]

### Fixed
- The `references` unit tests embed `imazen-26/manifests/train.tsv` at compile
  time instead of reading it from a host-absolute path, so they pass under the
  wasm32-wasip1 CI job (`wasmtime --dir .` preopens only the package root).
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
- **Real-bug gold-standard members** (#7): `real_bugs::RealBugId` — all 41
  rows of the issue's mined-bug table (zenjpeg 10, zenwebp 11, zengif 2,
  zenpng 3, zenavif 4, heic 5, imageflow 6), each carrying repo, commit/issue
  reference, summary, pixel-defect pattern and family analogue
  (`RealBugInfo`). `Family::RealBug(id)` applies each as a deterministic
  synthetic pixel-pattern repro; `real_bug_catalog()` / `full_catalog()`
  expose them and `manifest_for_reference()` now emits them with
  `source: real-bug:<repo>#<ref>` after the synthetic sweep. The example
  includes them by default (`--no-real-bugs`, `--real-bugs-only`). Tests pin
  the row count per repo, that every repro changes pixels on textured content
  (no identity "defects", #9), determinism, the off-by-one bugs staying
  within ±1, several exact patterns, and tiny-image safety. These are pattern
  repros, not recovered buggy-decoder outputs (that needs pre-fix checkouts
  of the sibling repos).
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
