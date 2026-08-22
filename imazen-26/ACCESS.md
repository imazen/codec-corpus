# imazen-26 — public access index

Everything below is served over plain HTTPS from a public R2 bucket — no
credentials, no presigned URLs. Base URL for every item on this page:

```
https://codec-corpus.r2.imazen.org/<key>
```

See [`STORAGE-MAP.md`](STORAGE-MAP.md) for the canonical-vs-derived layer map and
the metadata policy; this page is the "just give me a URL" index.

## Metadata policy — read before use

The published files in the four camera-photo categories (`1000/1200/1400/1600`)
carry **whitelist EXIF only**: camera make/model, lens, exposure parameters,
capture dates, orientation, color space, and the ICC profile. No location tags,
no serial numbers, no device or asset UUIDs — in any tag group, in any format
(jpg/heic/png/dng); the whitelist rewrite fails closed on unknown vendor tags
(policy + verification method: `STORAGE-MAP.md`). Decoded pixel data, including
gain maps, is hash-verified unchanged by the rewrite. The other 17 categories
retain the attribution-relevant metadata their licenses expect, and the PNG-v3
derivatives (below) carry no EXIF at all (PNG conversion drops it).

## 1. Raw corpus (originals, metadata policy per above)

`s3://codec-corpus/imazen-26-unprocessed/` — 2,688 objects, 6.99 GB. Full
21-category structure, original jpg/heic/png/dng formats.

```
https://codec-corpus.r2.imazen.org/imazen-26-unprocessed/<category>/<filename>
https://codec-corpus.r2.imazen.org/imazen-26-unprocessed/README.md
https://codec-corpus.r2.imazen.org/imazen-26-unprocessed/CORPUS-MANIFEST.tsv
```

Includes the patent scan source PDFs (3 US patents × {1-bit original, color
rescan, gray rescan}, 82 MB) alongside their rasterized page images:

```
https://codec-corpus.r2.imazen.org/imazen-26-unprocessed/6000-lilith-scans-public-patents/pdfs/<filename>.pdf
```

## 2. PNG derivatives (codec-test-ready, SDR + HDR renders)

`s3://codec-corpus/imazen-26-png-v3/` — 2,639 objects, 16 GB. One SDR render
per corpus image (`.sdr.png`) plus an HDR render for the 76 gain-map images
(`.hdr.png`). **No EXIF/GPS** — this is the layer to use if you don't need the
raw originals.

```
https://codec-corpus.r2.imazen.org/imazen-26-png-v3/<category>/<basename>.sdr.png
https://codec-corpus.r2.imazen.org/imazen-26-png-v3/<category>/<basename>.hdr.png   (gain-map images only)
```

## 3. Train / val manifests

[`manifests/train.tsv`](manifests/train.tsv) (1,910 rows) and
[`manifests/val.tsv`](manifests/val.tsv) (323 rows) — one row per PNG-v3
file (sdr + hdr variants each get a row), with a direct public URL per row.
Columns: `id, split, content_class, variant, relative_path, url`.

**No `test` split exists.** The only split manifest found (`picker-sweep-2026-06-22/imazen26_manifest.tsv`,
the source these are built from) has just `train`/`val`. These two manifests
cover 2,157 of the corpus's 2,567 images — the subset that manifest tracks,
not the full corpus (see `STORAGE-MAP.md` if you need the full-corpus list,
`CORPUS-MANIFEST.tsv`).

## 4. Representative subsets

Four K-sized representative selections (diversity-sampled clusters), each row
carrying a `crop_label` (crop/tile scheme, e.g. `c25_bl`) alongside the direct
PNG-v3 URL:

- [`manifests/imazen26_representatives_K300_2026-06-14.tsv`](manifests/imazen26_representatives_K300_2026-06-14.tsv) — 300 rows
- [`manifests/imazen26_representatives_K500_2026-06-14.tsv`](manifests/imazen26_representatives_K500_2026-06-14.tsv) — 500 rows
- [`manifests/imazen26_representatives_K1000_2026-06-14.tsv`](manifests/imazen26_representatives_K1000_2026-06-14.tsv) — 1,000 rows
- [`manifests/imazen26_representatives_K500_even_2026-06-18.tsv`](manifests/imazen26_representatives_K500_even_2026-06-18.tsv) — 500 rows, even-balanced variant

Columns: `url, crop_label, content_class, cluster_id, cluster_size`.

## 5. Cropscale set

`s3://codec-corpus/clean-picker-corpus-2026-06-26/` — 4,497 objects, 1.1 GB.
Multi-scale PNG crops (`o_<id>.png.scale<W>x<H>.png`).

```
https://codec-corpus.r2.imazen.org/clean-picker-corpus-2026-06-26/<filename>
```

*Provenance note: numeric IDs overlap imazen-26's range and this set is used
alongside imazen-26 in practice, but no manifest ties it explicitly to imazen-26
by content hash — flagging for anyone who needs a hard provenance chain.*

## 6. Codec-encoded derivatives (quality sweeps, benchmark runs)

~178 GB of codec-encoded outputs (zenavif/zenjpeg/zenpng/zenwebp quality
sweeps, HDR zenjxl passes, named benchmark runs) at
`s3://codec-corpus/picker-sweep-2026-06-22/` — same public-URL pattern as
above. Full breakdown (sizes, sub-prefixes, what's confirmed vs. not) is in
[`STORAGE-MAP.md`](STORAGE-MAP.md#r2-mirror-found-2026-07-22--corrects-backup-coverage-gaps-below).
