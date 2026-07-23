# imazen-26 — public access index

Everything below is served over plain HTTPS from a public R2 bucket — no
credentials, no presigned URLs. Base URL for every item on this page:

```
https://codec-corpus.r2.imazen.org/<key>
```

See [`STORAGE-MAP.md`](STORAGE-MAP.md) for the full local/tower/R2 provenance
of every stage; this page is the "just give me a URL" index.

## Privacy scope — read before use

Home-state (Colorado) GPS is stripped from every photo; GPS from travel/vacation
locations elsewhere is **intentionally retained** as landmark metadata (verified
2026-07-22: 0/48 Colorado-tagged files carry GPS, vs. 238 files across dozens of
one-off travel destinations that do — see `STORAGE-MAP.md` for the full check).
This is a deliberate scope, not a redaction failure — but it means the raw
originals below are **not** GPS-free in general. The PNG derivatives (below) do
not carry EXIF/GPS at all (4/4 samples checked, PNG conversion drops it).

## 1. Raw corpus (originals, privacy-scoped per above)

`s3://codec-corpus/imazen-26-unprocessed/` — 2,688 objects, 6.99 GB. Full
21-category structure, original jpg/heic/png/dng formats, home-state GPS
stripped as described above.

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
