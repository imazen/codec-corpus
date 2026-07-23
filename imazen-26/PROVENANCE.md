# Provenance summary — imazen-26/

Per-folder `PROVENANCE.md` files map every file to its source URL. This top-level
summary records the URL-reverification campaign run 2026-05-28.

## Re-verification results (2026-05-28)

100 URLs were HEAD-verified across `office-documents/`, `nasa/`, `noaa/`, `national-park-service/`,
and `internet-archive-scans/` (via the source JP2/TIFF archive.org items).

| Folder | URLs | HEAD OK (first pass) | After fixes | Byte-identical re-download |
|---|---|---|---|---|
| `office-documents/` | 27 | 24/27 | 27/27 | 22 match + 5 deltas* |
| `nasa/` | 8 | 8/8 | 8/8 | 7 match (1 src-path mismatch) |
| `noaa/` | 15 | 15/15 | 15/15 | 12 match (3 src-path mismatch) |
| `national-park-service/` | 40 | 36/40 | 40/40 | 40 match |
| `internet-archive-scans/` (source items) | 10 | 9/10 (transient 500) | 10/10 | — (JP2 zips not redownloaded; large) |
| **Total** | **100** | **92/100** | **100/100** | **81 byte-match + 5 deltas + 14 mismatched paths** |

*The 5 deltas were verified by content: **4 USPTO patents** are byte-equivalent at the document level (page count, text, file size all identical) — only USPTO's PDF-generation embeds a per-fetch ObjectID, so sha256 differs but content is identical. **1 real content change**: IRS Form 1099-MISC was refreshed from a 1-page transitional version to the full 6-page 2026 form (both PD, both legitimate; the fresh version replaces the original in this corpus).

## Initial-pass URL fixes (8)

- **4 Federal Register PDFs** had been originally fetched via the FederalRegister.gov API (which returns the `pdf_url` field). The govinfo.gov direct URL pattern is `https://www.govinfo.gov/content/pkg/FR-{YYYY-MM-DD}/pdf/{docnum}.pdf` (all four landed under `FR-2026-05-28/pdf/2026-{NNNNN}.pdf`).
- **4 NPS brochures** were at non-standard paths discovered by re-scraping each park's `planyourvisit/maps.htm`:
  - Gates of the Arctic (`gaarmap1.pdf`) → `nps.gov/carto/hfc/carto/media/` (NPS Cartography hub)
  - Grand Canyon (`sr-pocket-map.pdf`) → `nps.gov/grca/learn/news/upload/`
  - Katmai (`KATM_Park-Map_for_web.pdf`) → `nps.gov/katm/learn/photosmultimedia/upload/`
  - Muir Woods (`map-muwo-trail-2019-small.pdf`) → `nps.gov/goga/planyourvisit/upload/` (served by sibling park Golden Gate NRA)

## See also

- `office-documents/PROVENANCE.md` — IRS / USPTO / Federal Register / NASA NTRS / Census / VA / FEC per-file URLs
- `nasa/PROVENANCE.md` — Artemis I+II press kits + reference guides + Europa Clipper + Psyche + Sentinel-6B + NTRS Artemis Status
- `noaa/PROVENANCE.md` — 14 NHC Tropical Cyclone Reports + NHC 2024 Verification Report
- `national-park-service/PROVENANCE.md` — 40 NPS brochures + maps + site bulletins across 29 parks (verified URLs inline)
- `internet-archive-scans/PROVENANCE.md` — 6 archive.org items (Haeckel × 2, Hokusai, Owen Jones, Redouté, Shin-Bijutsukai) + Trouvelot individual TIFFs

All source PDFs / JP2 zips / TIFFs are retained at `/mnt/v/output/projectgutenberg-corpus/`
for any future re-rasterisation. The freshly-downloaded copies (used for SHA verification)
are at `/mnt/v/output/projectgutenberg-corpus/redownload/`.
