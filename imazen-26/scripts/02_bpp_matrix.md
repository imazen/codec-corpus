# Step 2 — per-codec bpp matrix (for clustering)

The bpp feature used by `03_select_500.py` is produced by encoding each synth PNG
**once per zen codec at a fixed default quality** and recording `bpp = bytes*8/pixels`.
The encoded bytes are **discarded** — only the byte count is kept. (No distorted
images are saved here; this set is clean references only.)

## Recipe (what was run 2026-05-27)
Encoder = **zencodecs** (`~/work/zen/zencodecs`), one fixed quality per codec:
- jpeg — q75, 4:2:0, progressive
- webp — lossy q75
- avif — q75, rav1e **speed 10** (fast, to keep runtime sane)
- jxl  — distance 1.0  *(SKIPPED 2026-05-27: `zenjxl` won't compile against the local
  `jxl-encoder` checkout — `jxl_encoder::AnimationFrame::new` API drift at
  `zenjxl/src/codec.rs:1051`. Fix the constructor + rebuild to fill `bpp_jxl`.)*

All 10,734 images encoded in ~88 s (jpeg/webp/avif), 0 load failures, parallelized with rayon.

## How to reproduce the harness
There is no standing CLI that encodes "once per zen codec". Build a tiny binary in a
**sibling worktree** of a workspace that already depends on zencodecs (e.g. `codec-eval`,
whose `crates/codec-iter` has jpeg+avif plumbing wired), iterate the image dir, encode per
codec via `zencodecs`, and emit `bpp.csv` (`filename,bpp_jpeg,bpp_webp,bpp_jxl,bpp_avif`).
Output landed at `/mnt/v/output/zensim/synth500/bpp.csv`.

Content features (the other clustering axis) come from
`zenanalyze` `cargo run --release --example extract_features_for_picker`
→ `/mnt/v/output/zensim/synth500/features.tsv` (`feat_*` columns).
