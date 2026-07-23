# imazen-26 — storage map (where every stage lives)

Traced 2026-06-25 from the build scripts in
`/mnt/v/output/codec-corpus/imazen-26-archive-2026-06-09/scripts/`. This is the
canonical "where is the data" index for the imazen-26 corpus.

**Key fact:** the corpus *images* are **not in git** (0 of 2,567 image files tracked;
only manifests/READMEs/2 helper scripts — 11 files — are committed). The images live on
local disk + `/mnt/v` + a partial tower mirror. Locations below are the source of truth.

## Pipeline stages

| # | Stage | Location | What's there |
|---|---|---|---|
| 1 | **Upstream raw export** (originals, metadata intact) | `./imazen26/` *(no hyphen, local, 108 MB)* | UUID-named pool: 60 HEIC + 1 JPG, EXIF intact. An upstream ingest stash (UUID names ⇒ likely an iOS/Photos asset export). **Not referenced by any build script.** 52/60 HEIC + the JPG made it into the corpus; 8 are unused sibling burst-frames (see `/mnt/v/output/codec-corpus/imazen-compare/`). |
| 2 | **Originals, corpus-renamed, pre-strip backup** (metadata intact) | `/mnt/v/output/codec-corpus/lilith-photos-backup-2026-06-09/` | The lilith camera photos (folders 1000/1200/1400/1600) renamed to corpus names but **before** PII strip: 216 jpg + 90 heic + 10 png + 3 dng. Full EXIF + gain maps. The metadata-intact reference. Created 15:33, 2 min before the strip. |
| 3 | **Metadata-privacy-stripped** (the live corpus) | `./imazen-26/` *(local, 6.6 GB, 2,567 images)* — **also mirrored to R2**, see "R2 mirror" section below | Strip ran **in-place here** via `strip_pii_with_gainmap_diff.py`. |
| 4 | **PNG derivatives** (codec-test-ready) — **current** | `/mnt/v/output/imazen-26-png-v3/` + tower mirror `/mnt/tower/output/imazen-26-png-v3/` *(16 GB)* | 2,639 PNGs = **2,563 `.sdr.png`** (one SDR render per corpus image) + **76 `.hdr.png`** (HDR render for gain-map images). Name = `<corpus-basename>.sdr.png` / `.hdr.png`. |
| 4b | PNG derivatives — superseded | `/mnt/v/output/imazen-26-png/` (v1, 06-10), `imazen-26-png-v2/` (06-12) | Same 2,639 count, kept as history. Logs: `imazen-26-png-convert*.log`, `-v2-convert.log`, `-largejpeg.log`, `-retry.log`. |
| 5 | **HDR + multi-scale training derivatives** | `/mnt/v/output/imazen-26-hdr-2026-06-14/` (16 GB, 2,639) ; `…/imazen-26-hdr-grid-2026-06-14/` (7.8 GB, 1,140) | HDR-aware render pass + a multi-scale grid (`.scale1200x1600.hdr.png`, `.scale120x160.hdr.png` …) for dense size-sweep feature/picker training. |
| 6 | **Feature / ML layer** | `/mnt/v/output/imazen-26-features/` (local, 2.7 GB working set) + tower `/mnt/tower/output/imazen-26-features/` (246 MB, 22 canonical files) | `imazen26_features_2026-06-{13,22}.parquet`, `imazen26_train_features…`, `imazen26_hdr_features…`, `imazen26_hdr_grid_features…`, representative selections `imazen26_representatives_K{300,500,1000}_2026-06-14.tsv`, `imazen26_train_variants…tsv`, `imazen26_manifest.tsv`. |
| 7 | **Non-PD quarantine** | `./imazen-26-not-pd/` *(local, 1.2 GB)* | Non-PD images moved out by `08_quarantine_nonpd.py` (CC-BY-SA/CC-BY/PSF/… web-screenshot + auto_screen crops, + `maybe/`, `nope/`). Log: `imazen-26-not-pd/WHAT-WAS-MOVED.md`. Restore = `mv …-not-pd/<x> imazen-26/<x>`. |
| 8 | **Synthetic sets** | `./imazen-26-synth/` (1.1 GB) + `./imazen-26-synth-500/` (116 MB, 500-subset) | Generated charts/line patterns (7000-lilith-plots lineage). |
| 9 | **Build pipeline + history (archived)** | `/mnt/v/output/codec-corpus/imazen-26-archive-2026-06-09/scripts/` | All 23 numbered build scripts + PII strip + geocode/landmark + gpt-5-nano describe + manifest builders + caches + `photo_manifest.tsv`/`photo_slugs.tsv`. Plus `museum-art/` staging. |
| 9b | Doc/scan + screenshot inputs | docs/scans: `/mnt/v/output/projectgutenberg-corpus/` (+ `redownload/`) ; screenshots: `/mnt/v/input/imazen-26-screenshots-2026-05-28/` (654 MB) | Source PDFs/JP2/TIFFs the PNG doc/scan pages were rasterized from; raw screenshot captures for the 8000/8100 sets. |
| 10 | **Manifests** (source of truth, **in git**) | `imazen-26/<folder>/MANIFEST.tsv` + `imazen-26/CORPUS-MANIFEST.tsv` | Per-image provenance: `original_filename`, `sha256`, camera/iso/fnum, license, `pd_basis`, `url`. |

Full duplicate of the live corpus: `./imazen-26 - Copy (2)/` (6.6 GB, complete mirror).
`./imazen-26 - Copy/` (454 MB, earlier pre-rename stage) and `…- Copy (3)/` (333 MB,
partial) are older snapshots. `imazen-26.lnk` is just a Windows shortcut to `imazen-26/`.

## R2 mirror (found 2026-07-22 — corrects "Backup-coverage gaps" below)

The live corpus (stage 3) **is** mirrored to R2, under a different name than any local
folder, which is why an earlier trace of this map (2026-06-25) missed it:

| What | Location |
|---|---|
| Raw corpus mirror | `s3://codec-corpus/imazen-26-unprocessed/` — 2,688 objects, 6.99 GB, uploaded 2026-06-09 (same day as the PII strip). Full 21-category structure + README.md/CORPUS-MANIFEST.tsv/per-folder MANIFEST.tsv. Public HTTPS: `https://codec-corpus.r2.imazen.org/imazen-26-unprocessed/<key>` (verified 200). |
| Codec-encoded derivatives | `s3://codec-corpus/picker-sweep-2026-06-22/` — confirmed via its `imazen26_manifest.tsv` (indexes the PNG-v3 render paths). ~178 GB total: `renditions/` (multi-scale PNGs, 1,482 objs, 1.73 GB), `datagen-2026-06-23/` (zenavif/zenjpeg/zenpng/zenwebp codec-encoded quality sweeps, 1,549 objs, 119 GB), `datagen-2026-06-23-hdr/` + `datagen-2026-07-03-hdr-hq/` (zenjxl HDR passes, 9.2 + 10.9 GB), `runs/` (named benchmark runs — avif-dense*, cpusmoke, dgcpu-*, 490 objs, 37.1 GB). Same public-HTTPS pattern as above. |
| Probable but unconfirmed | `s3://codec-corpus/clean-picker-corpus-2026-06-26/` (4,497 objs, 1.1 GB — numeric IDs overlap imazen-26's range, no manifest found tying it explicitly to imazen-26) and `s3://codec-corpus/synthetic-v2/` (hash-named, too large to fully enumerate in that pass, no linking manifest found). |
| Not found in R2 | The stage-6 feature/ML parquet files (`imazen26_features_*.parquet` etc.) — checked `zentrain` bucket's top-level `features/`, `datasets/`, `canonical/`, `eval-corpora/` prefixes (one level deep only, not exhaustive — that bucket is large) with no hits. Tower mirror (`/mnt/tower/output/imazen-26-features/`) is still the known-good copy for those. |

Credentials + bucket-listing method: `~/work/claudehints/topics/r2-credentials.md`.

## The privacy strip (stage 3) — history and current state

**2026-07-23 — full re-strip, superseding everything below this line.** The
original strip (history preserved further down) only targeted a fixed field
list and missed vendor-specific tags. A follow-up sweep (`exiftool -a -G1 -u`
across all 319 files in `{1000,1200,1400,1600}`, cross-referencing every tag
group present) found: **live GPS on 241 files** (all locations, not just
non-Colorado — the earlier "Colorado-only by design" scoping, below, is now
superseded: all GPS is removed, everywhere), a real device serial number
(`GoPro:CameraSerialNumber`, 3 files, GoPro's proprietary tag group — the
original field list only covered the standard EXIF `SerialNumber` tags, not
vendor maker-note equivalents) and `Apple:MediaGroupUUID` (4 files, burst/
live-photo grouping IDs — not personally identifying alone, but an unremoved
device-fingerprint field the original strip's stated intent already covered).
Checked and ruled out as a concern: `XMP-mwg-rs:RegionType` (present but
`Focus` — autofocus point metadata, not Photos.app person/face tagging; no
name field exists anywhere in the corpus).

Re-run as a **whitelist** rewrite (`exiftool -all= -tagsFromFile @ <keep-list>
-icc_profile -m -overwrite_original`) rather than the original's blacklist —
strips everything not explicitly kept, so unknown/future proprietary tags
fail closed instead of silently surviving. Keep-list: Make, Model, LensMake,
LensModel, UniqueCameraModel, FNumber, ExposureTime, ISO, FocalLength(+35mm),
ApertureValue, ShutterSpeedValue, DateTimeOriginal/CreateDate/ModifyDate/
SubSecTimeOriginal, Orientation, ColorSpace, WhiteBalance, Flash,
ExposureProgram, MeteringMode, ExposureCompensation, LensInfo, ICC profile.

Verified before and after, all formats present (jpg/heic/dng/png): decoded
pixel data byte-identical (PIL/pillow-heif hash compare, 20-file random
sample against untouched R2 originals) plus an explicit raw-strip
byte-range compare for DNG (PIL can't decode Samsung linear-DNG; extracted
the exact `StripByteCounts` region via `dd` and `sha256sum`-matched it —
same length, same bytes, offset shifted only because the metadata block
size changed). Post-strip: 0/319 files have GPS, GoPro serial, or Apple
UUID; Make/Model present on 308/319 (the rest never had camera EXIF to
begin with — screenshots etc.); ICC profile present on 274/319.
Re-uploaded to R2 (`imazen-26-unprocessed/{1000,1200,1400,1600}-lilith-*/`),
overwriting the GPS-carrying versions; verified live on the public domain.

**Not touched by this pass:** the other 17 categories (Unsplash/Met/AIC/
Internet Archive/government-doc/AI-gen/screenshot) — confirmed zero GPS
there in the original scan, and some may carry license-relevant attribution
metadata (CC-BY-SA credit, etc.) that a blanket whitelist-strip could wrongly
remove; out of scope for a same-day GPS fix. The `bytes` column in
`CORPUS-MANIFEST.tsv` / per-folder `MANIFEST.tsv` is now slightly stale
(files shrank a little) — not regenerated in this pass, cosmetic only.
`imazen-26 - Copy (2)/` and other local duplicate directories were **not**
touched (still carry the original GPS-including EXIF) — not part of the
published surface, but flagging in case they get used as a source later.

### Original strip, as first implemented (superseded above, kept for history)

`strip_pii_with_gainmap_diff.py` runs `exiftool -overwrite_original` in-place on
`imazen-26/{1000,1200,1400,1600}`, diffing against the stage-2 backup:

- **Removed** (device fingerprinting): `SerialNumber`, `LensSerialNumber`,
  `BodySerialNumber`, `InternalSerialNumber`, `Software`, `HostComputer`,
  `ImageUniqueID`, `MediaGroupUUID`, `DocumentID`, `InstanceID`,
  `OriginalDocumentID`, `ImageDescription`. (Turned out incomplete — see above.)
- **Kept**: camera model, ISO, f-number, exposure, dates, all attribution.
- **Verified byte-identical** after strip: gain maps (43 HEIC aux + 36 JPEG MPF) and
  primary pixels. This is *selective field removal*, not a full EXIF wipe — which is why
  corpus files are only ~46–370 bytes smaller than their originals.
- **GPS scope, as originally verified 2026-07-22** (superseded 2026-07-23 — all GPS is
  now stripped, see above): raw GPS was stripped **only from Colorado-tagged content**
  (48/48 `*colorado*`-named files checked had zero GPS tags — home-state location, the
  actually sensitive signal). GPS from every other location was retained — 238 files
  carried live coordinates across dozens of one-off travel destinations (Iceland, Japan,
  Barcelona, Seattle, Mexico, France, Costa Rica, Hawaii, …), none repeating often enough
  to suggest a second home. This was confirmed as a deliberate, scoped privacy design
  (hide where you live, keep travel/landmark metadata) rather than a bug — the decision
  was then revisited and all GPS is now removed regardless of location.
  Location is additionally preserved as landmark names baked into filenames by
  `add_gps_locations.py` + `add_landmarks.py` (geocode / nominatim / overpass caches in
  the archive) — this runs regardless of the GPS-stripping scope above.

## Backup-coverage gaps (flagged 2026-06-25; **corrected 2026-07-22**, see "R2 mirror" above)

Only **png-v3** and the **canonical features** are mirrored to tower. ~~The live corpus
(stage 3)~~, the pre-strip originals backup (stage 2), the HDR sets (5), synth (8), and
not-pd (7) are **single-copy on local disk / `/mnt/v`** ~~— no tower or R2 mirror~~.

**Correction:** the live corpus (stage 3) *is* mirrored to R2 (`imazen-26-unprocessed/`,
uploaded 2026-06-09) — it was live before this doc's own 2026-06-25 trace date and simply
wasn't found under a matching name. The pre-strip backup (2), HDR sets (5), synth (8), and
not-pd (7) remain unverified in R2 as of 2026-07-22 (not found under an obvious name in a
non-exhaustive pass — could still exist under a different prefix).

## `PROVENANCE.md` folder names (resolved 2026-07-22, previously mis-flagged as stale)

The top-level `PROVENANCE.md` describes folders (`office-documents/`, `nasa/`, `noaa/`,
`national-park-service/`, `internet-archive-scans/`) that don't match the numbered
1000–9226 categories above — this was previously (wrongly) flagged here as staleness.
It's actually the `imazen-26/nope/` staging structure (`nope/office-documents/`,
`nope/nasa/`, …): confirmed by reading `nope/office-documents/PROVENANCE.md` directly,
whose content ("Each PNG is rasterised at 300 dpi from a source PDF downloaded from a US
federal agency…") matches the top-level summary's office-documents row exactly. `nope/`
is the pre-numbering candidate pool the corpus was curated from (README calls it
"rejected candidates", though some of its content clearly informed what got promoted into
the numbered categories, e.g. NOAA/NPS material). The top-level doc's "See also" links use
shorthand names, not exact paths — e.g. "national-park-service/PROVENANCE.md" really means
`5000-national-park-service-brochures/PROVENANCE.md`. Per-folder `PROVENANCE.md` files
under the numbered categories and `CORPUS-MANIFEST.tsv` remain the reliable per-image
source; `nope/` itself is excluded from the corpus per the top-level README.
