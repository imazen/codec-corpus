# Extracted page images — 6000-lilith-scans-public-patents/

> **Renamed 2026-06-08** to the corpus convention
> `NNNN_patents_<inventor>-<patent>-<variant>_pNNN_<WxH>.<ext>` (numbers 6000–6112,
> per page) by `../scripts/11_rename_patents.py`. Subfolder structure kept for now;
> full before/after map in `RENAME-HISTORY.tsv` (use it to build manifests). The
> `page_NNN.*` names below describe the original extraction layout.

`<pdf-stem>/page_NNN.{jpg,png}` — one folder per PDF, one file per page, extracted
2026-06-08 by `../scripts/10_extract_patents_verbatim.py` (pikepdf 10.3).

Each page is a single full-page 300 DPI scan, extracted directly from the PDF (not
rendered), so the output is the source image at exact native resolution:

| Variant | Pages | Output | Dims | Fidelity |
|---|---:|---|---|---|
| `*_1bitOriginal` | 39 | `page_NNN.png` | 2320×3408 | **true 1-bit PNG** (bit-depth 1), lossless decode of the CCITT G4 bits |
| `*_PrintRescanColor300` | 37 | `page_NNN.jpg` | 2479×3230 | **verbatim JPEG** — original embedded bytes, no re-encode |
| `*_PrintRescan{Gray,Grayscale}300` | 37 | `page_NNN.jpg` | 2479×3230 | **verbatim JPEG** — original embedded bytes, no re-encode |

113 files, 83 MB. Verified: every JPEG is byte-identical to the embedded PDF
stream (decoder-independent); every 1-bit PNG is bit-depth 1 at native dims and
pixel-identical to a hayro reference render.

## Why extraction, not rendering

These PDFs are pure single-image-per-page scans (one full-page image per page, no
rotation, no `/Decode` inversion, no ICC/masks/palette — checked), so the embedded
image *is* the page. Extraction is strictly more faithful than rasterizing:

- **JPEG rescans** keep their exact original bytes (a render decodes + resamples
  them ~1px off native: 2478×3229 vs the true 2479×3230).
- **1-bit originals** stay genuinely 1-bit — the point of the bilevel variant for
  codec testing. A render flattens them to 8-bit grayscale (only 0/255, but 8 bpc).

## Provenance / supersedes

- The earlier hayro renders (`scripts/09_render_patents_hayro.py`, 113 PNGs) were
  moved to `/mnt/v/output/codec-corpus/patent-hayro-renders-2026-06-08/` and remain
  regenerable from that script. Rendering is the right tool only if a page composites
  vector/text/multiple images — none of these do.
- Sources are public-domain US patents: Lynn Conway US5046022, Martha Jones US77494,
  Yvonne Brill US3807657.
