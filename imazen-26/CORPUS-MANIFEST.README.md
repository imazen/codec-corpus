# Manifests — imazen-26/

As of 2026-06-09, each content folder has ONE consolidated `MANIFEST.tsv`, and the
root has a unified `CORPUS-MANIFEST.tsv` aggregated from them. The old per-folder
`MANIFEST.json`/`MANIFEST.tsv`/`*.bak` and the separate `RENAME-HISTORY.tsv` files
were folded in and deleted.

## Per-folder `<folder>/MANIFEST.tsv`  (source of truth)

One row per image. Common columns first, then folder-specific provenance:

| column | meaning |
|---|---|
| `filename` | current name, relative to the folder (keeps any subdir, e.g. `color/…`) |
| `number` | 4-digit unique id |
| `category` | corpus category (general, aic, met, nps, scans-patents, …) |
| `descriptor` | description + attribution slug (category prefix stripped) |
| `width` `height` `format` `bytes` | from the actual file |
| `original_filename` | pre-rename name (provenance; = current for never-renamed folders) |
| `source` | origin (lilith, unsplash, art-institute-chicago, met, nps, epa, noaa, uspto, internet-archive, lilith-generated, lilith-ai, various-web) |
| `license` | see note below |

Folder-specific provenance columns (only where they apply):
- photos (1000–1600): `camera` `iso` `fnum`
- patents (6000): `inventor` `patent` `variant` `page`
- museums (3000/3300): `artist` `title` `date` `medium` `art_license` `image_url` `sha256`
- NPS/NOAA (5000/5300): `source_class` `source_description` `pd_basis` `source_url` `sha256`
- IA scans (6600/6800): `source_work` `ia_identifier` `page_index` `contributor` `pd_basis` `source_url` `sha256`
- web screenshots (8100): `url` `source_kind` + per-capture `capture_license`

## Unified `CORPUS-MANIFEST.tsv`

Aggregate of all per-folder manifests, common columns only (`path` = `folder/filename`).
Regenerate after any change with `scripts/23_swap_and_cleanup_manifests.py` (or the
inline regen) — the per-folder manifests are authoritative.

## License column — folder-level best-effort, NOT per-file legal clearance

- `PD` US patents + Internet-Archive scans of PD works (Haeckel etc.); see `pd_basis`.
- `PD-USGov` NPS / EPA / NOAA federal works.
- `PD-own` lilith's own photos, plots, AI-generated images.
- `CC0` Art Institute of Chicago + Met open-access artworks (also in `art_license`).
- `Unsplash-License` unsplash stock (photographer in `original_filename`).
- 8100 web screenshots carry the **per-capture** license harvested from the original
  crop manifest (`PD`/`PD-tool`/`Mailing-list-PD`/`PD-own`).
- `screenshot-unverified` 8000 mobile screenshots — third-party sites; unverified.

Excluded: `nope/` (reject pile), `museum-art/` (fetch tooling).

Build scripts: `scripts/22_consolidate_per_folder_manifests.py` (build),
`scripts/23_swap_and_cleanup_manifests.py` (swap/delete/regen).
