# Internet Archive scans — public-domain illustrated plates + text

72 high-resolution pages curated from public-domain illustrated works on the Internet
Archive: **36 illustration plates** (`illustrations/`) and a balanced **36 text pages**
(`text/`). Every page is lossless PNG decoded from the archival JP2 (Trouvelot from the
original TIFF). Resolution 6–37 MP. Per-file dimensions, sha256, source identifier, and
the exact PD basis are in `MANIFEST.tsv`.

The two halves give a codec corpus both content classes that compress very differently:
continuous-tone / colour illustration vs. sharp high-frequency typeset text.

## Public-domain status — verified

PD validated against the archive.org metadata API (`possible-copyright-status`,
`licenseurl`, publication date) cross-checked against author death + 70 yr. Full audit +
raw metadata snapshots: `../projectgutenberg/validation/PD-VALIDATION.md`.
**All sources are public domain *worldwide*** — US-only items (Nielsen, Séguy) were
excluded so the set is clean in any jurisdiction.

| Source | archive.org id | used for | PD basis |
|---|---|---|---|
| Haeckel, *Kunstformen* — plates ed. | `KunstformenDerNaturErnstHaeckel` | illus | pub 1899; d.1919 |
| Haeckel, *Kunstformen* — German text ed. | `kunstformenderna00haec` | text | BHL Public domain; 1904; d.1919 |
| Hokusai, *Denshin kaishu Manga* | `denshinkaishuhov1kats` | both | Smithsonian PD; 1828; d.1849 |
| Owen Jones, *Grammar of Ornament* | `gri_33125008700086` | both | Getty NOT_IN_COPYRIGHT; 1856; d.1874 |
| Redouté, *Les Roses* | `lesroses1821pjre` | both | pub 1821; d.1840 |
| Furuya Kōrin, *Shin-Bijutsukai* | `shinbijutsukai278800` | both | pub 1902; d.1910 |
| Trouvelot, *Astronomical Drawings* | `TrouvelotAstronomicalDrawings` | illus | pub 1882; d.1895 |

## illustrations/ (36)

- **8 Haeckel** — cephalopods, green radiolaria, red starfish, pitcher plants, red algae (colour);
  trilobites, moths, radiolaria spheres (monochrome line/tone).
- **4 Hokusai** — figure studies, insects/creatures, boats-and-waves, bridge landscape (line woodblock).
- **6 Owen Jones** — Egyptian columns, multicolour border bands, Moorish arches, Moorish tile grid,
  Renaissance figurative panels, Persian/Indian medallion (flat colour + hard-edge pattern).
- **5 Redouté** — rose portraits (stipple engraving).
- **5 Shin-Bijutsukai** — chrysanthemum spirals, flame-flower, geometric lattice, flowing-lines, cherry blossoms (Art Nouveau design).
- **8 Trouvelot** — sun spots, total solar eclipse, aurora, Mars, Jupiter, Saturn, 1881 comet, Orion nebula (continuous-tone pastel).

## text/ (36) — four scripts, varied typography

- **6 German** (Haeckel) — typeset title page, *Vorwort* preface, systematic table, per-plate descriptions.
- **11 English** (Owen Jones) — typeset title page, preface body, list-of-plates (tabular), descriptive essays.
- **15 French** (Redouté) — facing-plate botanical descriptions, each with a centred *Rosa* binomial heading.
- **3 Japanese woodblock** (Hokusai) + **1 Japanese letterpress** (Shin-Bijutsukai colophon).

Filenames: illustrations `<source>_<page>_<tag>.png`; text `text_<source>-<lang>_<page>_<tag>.png`.

## Why this mix (codec corpus rationale)

Illustrations span fine stipple/line engraving (Redouté, Hokusai, B&W Haeckel → high-frequency
detail), flat colour + hard edges (Owen Jones, Shin-Bijutsukai → palette/banding), and
continuous-tone gradients (Trouvelot, colour Haeckel). Text adds the sharp bilevel-like
typeset class across Latin (dense essays, sparse titles, tabular lists), Fraktur-era German,
and Japanese woodblock/letterpress — each a distinct edge/frequency profile.

## Regeneration

Sources staged at `/mnt/v/output/projectgutenberg-corpus/` (zips/TIFFs, extracts, contact
sheets, colourfulness/text-score TSVs). Illustrations were picked from labeled contact-sheet
montages; the two large books' plates were auto-located by std-of-saturation ranking. Text
pages were located by visual scan of contiguous-range contact sheets (global thresholds
mis-classify cream-paper text), then confirmed on a labeled montage before extraction.

## Note

Scans retain authentic archival margins (mount edges, the NYPL colour-bar/ruler on some
Trouvelot plates, library seals on the Japanese pages). Crop downstream if a clean frame is needed.
