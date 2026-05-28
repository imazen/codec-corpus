# imazen-26 synth pipeline

Scripts that derive synthetic **reference** content from the imazen-26 raw source images
and curate a clustered subset. **These produce clean references only — no distortions.**

| Step | Script | Output |
|---|---|---|
| 1 | `01_generate_synth.py` | `../imazen-26-synth/` — downscales (Lanczos, longest-edge 2048→128, **downscale-only**) + native crops (center 512²/256², random 512², **no resampling**), monotonic names + `_manifest.csv` + sRGB chunks. ~10.7k PNGs from 1,163 sources. |
| 2 | `02_bpp_matrix.md` | per-codec bpp (`/mnt/v/output/zensim/synth500/bpp.csv`) via zencodecs single-shot encode; bytes discarded. |
| — | (zenanalyze `extract_features_for_picker`) | `features.tsv` — `feat_*` content vectors. |
| 3 | `03_select_500.py` | `../imazen-26-synth-500/` — K-means(K=500) on z-scored [log per-codec bpp ⊕ content feats], centroid-nearest pick, content-balanced, ≤4/source. `selection_manifest.csv`. |
| 4 | `04_rename_subset.py` | renames subset → `NNNN-provenance-category-subcategory.png`. |
| 5 | `05_regen_graphics_aa.py` | regenerates the curated `lilith/generated-graphics/` **line-art** with anti-aliasing → `lilith/generated-graphics-aa/`. Reproduces each curated PNG byte-exact from the seed in its filename (replaying the driver's 512→1024 RNG sequence), then re-renders the 1024² with `ScaledDraw` supersampling (SS=4). `--validate` asserts byte-exactness vs the originals before generating. Charts are owned by step 6. |
| 6 | `06_quickchart_gallery.py` | quickchart.io gallery: every chart type the public endpoint renders (bar/hbar/line/area/stacked/radar/pie/doughnut/polarArea/scatter/bubble/radialGauge/gauge/progressBar/sparkline/boxplot/violin v2; candlestick/ohlc/sankey/funnel v4) across 8 themes, with real-world data triple-checked for validity (`--check-only` runs the asserts). 1024² at devicePixelRatio=2, Lanczos-down. Heatmap via matplotlib (quickchart's public endpoint doesn't register matrix/treemap/graph). |

## Anti-aliased regeneration (steps 5–6)

`generated-graphics-aa/` mirrors the curated taxonomy:
- `polygons/ lines/ line-patterns/` + the grid-pattern part of `grids/` — AA line-art (byte-faithful geometry, seeds preserved, same counts).
- `charts/` — quickchart gallery (20 types × rotating themes + a pie shown in all 8 themes).
- `grids/` — AA grid-pattern line-art + one matplotlib heatmap (9×9 multiplication table).

Folder = f(type, kind): `chart/heatmap→grids`, other charts→`charts`; line-art `polygons→polygons`, `tiling→line-patterns`, `grid-pattern→grids`, `concentric|voronoi-ish|stars-burst→lines`.

Naming: `gen-<chart|line>-<kindslug>__<idx>_s<seed>_1024sq.png` (line-art) /
`gen-chart-<type>__<NN>_<theme>_1024sq.qc.png` (gallery) / `.mpl.png` (matplotlib).

Reproduction note: the original driver reuses one `sub_rng` across sizes `[512,1024]`, so a `*_1024sq.png` is drawn from the RNG state **after** the 512² render — render 512 first (discard) to advance the RNG, then 1024. Line-art is byte-exact this way; matplotlib charts are not (global rcParams style leakage across the original chart sequence), which is why charts moved to quickchart.

**Naming:** `NNNN-<provenance>-<category>-<subcategory>.png`
- provenance = source collection (gen, gengfx, unsplashppl, unsplashtex, skitter, openclipart, flickr, loc, root…)
- category = content-cluster → semantic (photo / illust / texture / detail / flat / graphic / lineart), from feat centroids
- subcategory = `dsN` (downscale longest-edge N) / `cropcN` / `croprN`

Requires: `vips`/`vipsthumbnail`, python3, ImageMagick (`identify`); zencodecs + zenanalyze for steps 2/feat.

See `ZENSIM_CORPUS_PLAN.md` for how this feeds (and what's missing for) zensim metric training.
