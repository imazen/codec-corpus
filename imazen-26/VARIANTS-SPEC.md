# imazen-26 — variant generation spec (v2, 2026-08-22)

Variants (scaled, cropped, re-encoded derivatives) of imazen-26 have been produced by
several ad-hoc pipelines with divergent naming, kernels, and no unified provenance.
This spec standardizes FUTURE variant sets. v2 replaces v1's fixed density mandate:
**density is chosen per task by measured selection procedures, not by one ladder** —
the procedures below are the ones that already produced representative, diverse sets
without prohibitive pixel counts (references at the bottom).

Existing sets stay valid historical inputs; do not regenerate them retroactively.

## 0. How density is chosen (the core of v2)

Feature extraction is cheap; encode-sweeping is ~100× costlier. So coverage is bought
in three measured moves, then the expensive axes are applied **only to the
selection** — never cluster away size/q axes, and never densify on unselected bulk.

**(a) Coverage floor — thumbnail everything.** Every in-split corpus image at
longest-side ≤128 px. Measured cost for the ~1,100-image train bucket: ~12.7 MP —
essentially free. No selection step may drop below this floor: every image is always
represented at least once.

**(b) Representative selection — when PER-SOURCE cost dominates** (per-image tooling,
metric eval, GPU-bound scoring): k-means over zenanalyze **content** features,
z-scored, with geometry/size features excluded (`pixel_count, log_pixels,
bitmap_bytes, min_dim, max_dim, aspect, block_misalign, log_padded, channel_count`)
— size is a densification axis applied *to* the reps, never a selection axis. Pick K
at the **measured knee subject to a class floor**: variance-explained flattens past
K≈300–500 on this corpus (0.87 @300 → 0.894 @500 → only 0.936 @1500), K=300 zeroes a
content class, **K=500 keeps every class ≥2 and retains outlier singletons** — the
standing default (≈23% of units). k-means measurably rebalances against modal bias
(the 34.7%-of-corpus homogeneous class fell to 21.8% of reps; small diverse classes
rose). **Cluster WITHIN the split bucket only** (z-score stats from that bucket): the
original K500 clustered over all ids and picked 202/414 hold-out sources — the
contamination the `_even` re-cluster existed to fix. Selection units are
`(image, crop)` pairs from the crop vocabulary in §3.

**(c) Budget-first FPS — when PER-PIXEL cost dominates** (encode sweeps, trained
models): set a **gigapixel budget B**, take the floor from (a), then spend the
remainder by farthest-point sampling in content-feature space over the
size-candidate pool. Emit a **priority-ordered manifest with `cumulative_gp` — any
budget is a prefix**, so density is retunable by truncation without regeneration.
Measured on the ~1,100-source train pool: coverage p95 falls steeply to ~1.5 GP
(≈1,480 renditions) and flattens after — the standing default; FPS discovers the
size mix itself (mostly thumbnails + a few hundred large full-detail renditions).
Downstream cost budgets as `B × n_q × n_configs`.

**Choosing:** (b) when you need N *sources*; (c) when you need pixels under a bill;
both when a rep set then gets a dense ladder. Record the method + parameters (K or
B, feature snapshot, seed) in the manifest (§5).

## 1. Task profiles

| profile | selection | sizes | q axis |
|---|---|---|---|
| **anchor / benchmark** (constants land in source) | all images, or class-stratified ≥50/class | the 4-tier floor: tiny/small/medium/large (sweep discipline) | full grid, step 5 |
| **trained model / picker** | floor (a) + FPS (c) to B GP (default 1.5 GP per ~1k sources) | FPS-chosen from the candidate grid (§2) | dense: step 5 for 0–70, step 2 for 70–100 |
| **metric eval / GPU-bound scoring** | k-means reps (b), K at knee + class floor | a contiguous ladder subrange per the question | as needed |
| **smoke / CI** | ~12–16 class-stratified | 2–3 rungs | 3–5 points |

Under-density and over-density are both failures: a benchmark below the 4-tier floor
mis-fits intercepts; a trained model on 4 buckets smooths over the gaps production
lives in; a metric-eval set at full FPS density burns GPU-hours nobody needed.

## 2. The candidate size grid (joinability, not a mandate)

All sets draw sizes from ONE canonical log-spaced longest-side grid so any two sets
join on exact dimensions:

```
32 40 48 64 80 96 128 160 192 256 320 384 512 640 768 896 1024 1280 1536 2048 3072 4096
```

capped at source size — **no upscaling, ever**. A set materializes only the rungs its
profile/selector chooses. The deployed picker default is the 64…1024 subrange
(11 rungs, deliberately dense in the starved small/medium band; tier boundaries
tiny<224 ≤small<448 ≤medium<840 <large — keep these for tier accounting). Aspect is
always preserved; the rung names the longest side.

## 3. Crop vocabulary

The canonical crop set is the measured 11-unit vocabulary per image:
**`full` + `c50 × {center,tl,tr,bl,br}` + `c25 × {center,tl,tr,bl,br}`** (fractions
of min-dimension). Measured at K=500: crops contribute real content diversity, not
redundancy (balanced 20–61 reps per crop label). Crops also bound pixel cost for
huge sources (a native-resolution window instead of a full-frame encode). Ad-hoc
crops never ship in a dataset. (Gap, still true: crop *window math* exists in
zenanalyze; a crop-PNG renderer needs building the first time a set materializes
crop files.)

## 4. Naming, kernels, colorspace

- **Grammar (matches the deployed convention):** dot-chained ops appended to the
  id-leading stem — `<id-stem>.scale<W>x<H>[.crop<x>.<y>.<w>.<h>]….<ext>` —
  identity = everything before the first `.scale`. The 4-digit corpus id leads the
  stem, so the split bucket is derivable mechanically from any variant name
  (`manifests/README.md`). v1's `__` grammar is withdrawn (never deployed).
- **Kernel + sharpening are recorded axes, chosen per purpose — never silent:**
  *production-emulating* SDR training renditions use Mitchell **+ sharpen**
  (the deliberate, deployed choice — it mirrors web resize pipelines);
  *metric-reference / analysis* sets use a plain kernel (Mitchell or Lanczos3) with
  no sharpening. One set never mixes. New sets must carry the kernel in
  `variants.tsv` (and in the op token when it deviates from the set's declared
  default).
- **Colorspace:** HDR sources resample in **linear light** before PQ re-encode
  (16-bit PNG, correct cICP) — mandatory, per the deployed HDR grid. SDR sets record
  their colorspace path (linear vs gamma) in the manifest; production-emulating sets
  may legitimately resize in gamma space because production does.

## 5. Manifest (mandatory)

A variant set without a manifest is not a dataset. Every set ships `variants.tsv`:

```
origin_id  origin_sha256  op_chain  kernel  sharpen  colorspace_path  selection_method
selection_param  rank  cumulative_gp  generator  generator_commit  out_path  out_sha256
width  height  split
```

`selection_method` ∈ {all, stratified, kmeans, fps, floor}; `selection_param` = K or
B; `rank`/`cumulative_gp` populated for FPS sets (prefix-truncatable). `split` is
inherited from the origin id at generation time. Bulk bytes are content-addressed
(`variants/<sha256>.<ext>`) with the manifest mapping names → hashes.

## 6. Generator

One committed tool per pipeline generation; ad-hoc ImageMagick/PIL invocations are
banned for dataset production. Until a unified tool lands, record generator +
commit hash per row.

## References (the measured basis)

- k-means ablation + K knee + class floor + rebalancing: zenanalyze
  `benchmarks/imazen26_cluster_ablation_2026-06-14.md` (+ `.py`; Rust:
  `zenpicker-train/src/bin/cluster_features.rs`).
- Budget-first FPS + thumbnail floor + prefix manifests + the 1.5 GP knee:
  zenanalyze `benchmarks/imazen26_budget_select_2026-06-14.md` (+ `.py`).
- Within-split clustering fix: zenmetrics `scripts/imazen26_recluster_even.py`.
- Deployed ladder + tiers + `.scale` grammar + provenance TSV: zenmetrics
  `scripts/picker/gen_dense_corpus.py` (clean-picker corpus, 414 sources × ≤11).
- Deployed renderers: zenanalyze `examples/render_imazen26_variants.rs` (SDR,
  Mitchell+sharpen), `examples/extract_hdr_size_grid.rs` (HDR, linear-light).
