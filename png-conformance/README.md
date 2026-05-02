# PNG conformance / decoder regression suite

11 PNG files that exposed bugs in the
[zenflate](https://github.com/imazen/zenflate) DEFLATE decoder and the
[zenpng](https://github.com/imazen/zenpng) PNG decoder. Each file is
preserved here as a regression seed; the corresponding bugs are
fixed but the inputs remain valuable for any decoder under
development.

Added in commit [7fd72a5](https://github.com/imazen/codec-corpus/commit/7fd72a5).

## Tier 1 — decompressor crash regressions (8 files, ~33 KB total)

One sample per `(color_type, interlace)` PNG combination. These
previously triggered:

- zenflate fastloop preload underrun
- zenflate uncompressed-block underflow
- zenflate streaming parse drift
- zenflate offset-decode refill bug

Files include `14b47384-7042-11e5-801d-804da7b4cbe6.png`,
`badadler.png`, `Disable_auto_recalculation_26.png`, and several
`wm_upload_wikimedia_org_*` samples (see Tier 2 list for the
wikimedia hash mapping).

## Tier 2 — decoder edge cases (3 files, ~1.7 MB)

Larger PNGs that previously failed `zenpng` decode due to filter or
decompressor issues:

| File | Size | Dims | Format |
|---|---|---|---|
| `wm_upload_wikimedia_org_45634e241d7821a3.png` | 350 KB | 1105×658 | RGBA |
| `wm_upload_wikimedia_org_c8a458b0cef3d942.png` | 619 KB | 1920×1920 | RGBA |
| `wm_upload_wikimedia_org_a23d1e831e128dff.png` | 753 KB | 3508×2480 | RGB |

## Provenance & license

- **`wm_upload_wikimedia_org_*.png`** — sourced from Wikimedia
  Commons (the filename hash matches the upstream `upload.wikimedia.org`
  CDN path). Wikimedia Commons content is generally
  freely-licensed (varies by file: CC0, CC BY, CC BY-SA, public
  domain). Verify the original file page on commons.wikimedia.org
  for the exact license of each. Used here as decoder regression
  inputs — fair use for that purpose.
- **`14b47384-...`, `badadler.png`, `Disable_auto_recalculation_26.png`**
  — origin uncertain; UUID/screenshot-style names suggest these
  were collected during decoder bug triage. Verify upstream before
  redistribution.

## Recommended use

✅ Decoder regression testing, bug repro material, codec research.

⚠ **Do NOT redistribute individual files commercially without
verifying upstream license.** These are kept as decoder bug-trigger
seeds, not as a redistributable image collection.

If you can identify the precise upstream and license for any file
here, please open a PR.
