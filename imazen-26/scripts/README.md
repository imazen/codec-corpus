# imazen-26 corpus — scripts

## Naming convention (all folders)

```
<NNNN>_<category>_<descriptor>[_<keyinfo>...]_<WxH>.<ext>
```

- `NNNN` — 4-digit unique id, allocated from each folder's block (folder is
  `NNNN-<name>`). Per-image unique.
- `category` — short corpus category (general, people, aic, met, nps, epa, noaa,
  scans-patents, scans-illustrations, scans-text, plots, mobile-screenshots,
  web-screenshots, clipart, illustrations, products).
- `descriptor` — description + attribution (AI 3-word slug for photos/screenshots;
  source title for museums/docs; photographer-id for unsplash; etc.).
- `keyinfo` — page/variant/camera/iso etc. f-stops use `f1p8` (no dot).
- `WxH` — actual pixel dimensions. All paths lowercase.

## Manifests

- `<folder>/MANIFEST.tsv` — **source of truth**, one row per image with provenance
  (`original_filename`, plus camera/artist/contributor/pd_basis/url/sha256/… where
  applicable) and license.
- `../CORPUS-MANIFEST.tsv` — unified aggregate; regenerate with
  `python3 scripts/build_corpus_manifest.py`.
- See `../CORPUS-MANIFEST.README.md` for column docs and license caveats.

## Tools here

- `build_corpus_manifest.py` — regenerate the unified manifest from per-folder ones.
- `fetch_art.py` — fetch CC0/PD artworks (Met / Art Institute of Chicago).

## History

The one-off scripts that built and renamed the corpus (crop, patent extract,
gpt-5-nano description, per-folder renames, manifest consolidation) plus the old
synth-pipeline scripts were archived 2026-06-09 to
`/mnt/v/output/codec-corpus/imazen-26-archive-2026-06-09/` (with `museum-art/`).
The state they produced is fully captured in the per-folder manifests.
