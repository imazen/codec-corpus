# imazen-26 — storage map

What exists, where the published copies live, and which artifacts are canonical.

**Canonical source of truth: this repository.** `imazen-26/` holds the per-folder
`MANIFEST.tsv` provenance files (in git), the aggregate `CORPUS-MANIFEST.tsv`, and the
split manifests under `imazen-26/manifests/`. Published image bytes are served from the
public R2 base (`https://codec-corpus.r2.imazen.org` — see [`ACCESS.md`](ACCESS.md)).
Anything not reachable from this repo plus `ACCESS.md` is a working copy, not a
distribution surface.

## Published surfaces (public HTTPS, no credentials)

| Layer | Location | Contents |
|---|---|---|
| Raw corpus | `…/imazen-26-unprocessed/<manifest path>` | the corpus images + manifests (ACCESS.md §1) |
| PNG derivatives | `…/imazen-26-png-v3/<manifest path minus extension>.sdr.png` (+ `.hdr.png` for the gain-map images) | one codec-test-ready PNG render per corpus image (ACCESS.md §2) |
| Multi-scale renditions + codec-encoded sweeps | `…/picker-sweep-2026-06-22/` | ACCESS.md §5–6 |

Known drift (recorded 2026-08-22): the R2 prefixes predate the final curation pass and
hold more objects than the 2,160-row canonical manifest (earlier mirror passes included
working material). A reconciliation pass to make each prefix exactly mirror the manifest
is queued. Until then, **`CORPUS-MANIFEST.tsv` is the membership oracle** — an object on
R2 with no manifest row is not part of the corpus.

## Metadata policy (published files)

Camera-sourced files carry **whitelist EXIF only**: Make/Model/lens identification,
exposure parameters (FNumber, ExposureTime, ISO, FocalLength incl. 35mm-equivalent,
ApertureValue, ShutterSpeedValue, ExposureProgram, MeteringMode, ExposureCompensation,
WhiteBalance, Flash), capture dates, Orientation, ColorSpace, and the ICC profile.
No location tags, no serial numbers, no device or asset UUIDs — in any tag group, in
any format (jpg/heic/dng/png). Enforcement is a whitelist rewrite
(`exiftool -all= -tagsFromFile @ <keep-list> -icc_profile`), so unknown vendor tags fail
closed rather than silently surviving. Decoded pixel data — including HEIC auxiliary and
JPEG MPF gain maps — is hash-verified unchanged by the rewrite.

Verified 2026-08-22: 0/319 camera-folder files carry location or device-identifying
metadata, in this tree and in the published R2 copies (spot sha256 match between the
two).

Non-camera categories (museum, government-document, Internet Archive, AI-generated,
screenshot sets) retain the attribution-relevant metadata their licenses expect.

## Split manifests

`imazen-26/manifests/` carries the canonical train/validate/test split (deterministic,
by the last digit of the image id — see `manifests/README.md`). Derived datasets must
inherit an image's bucket from these manifests; do not invent per-dataset splits.

## Working copies

Operator-side working copies, staging pools, and pre-publication archives are indexed
in a private operations repo, not here. If a layer isn't listed above, it isn't
published.
