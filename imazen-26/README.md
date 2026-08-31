> **SUPERSEDED — this is a stale copy.** The imazen-26 corpus moved to
> **https://github.com/imazen/imazen-26** on 2026-08-23 and the canonical
> manifests, docs, and variant-set registry live there. This copy has since
> diverged; do not rely on it.
>
> In particular, the render-URL pattern below is **wrong**: render names cannot
> be derived from the corpus filename. The corpus name ends in a `_WxH` token
> from the *stored* dimensions, while the render pass applied EXIF rotation and
> named its output by the *rotated* dimensions — so for 196 of 2,160 images the
> two numbers are transposed and an extension swap 404s. Take URLs from the
> `png_v3_sdr_url` / `png_v3_hdr_url` columns of the canonical split manifests,
> or from `variant-sets/png-v3-index.tsv` in the canonical repo.

# imazen-26 image corpus

A test corpus for image codecs and processing — **2,160 images, ~5.9 GB** across 21
categories. Deliberately diverse: real photographs, stock photography, artwork
reproductions, born-digital government documents, document/manuscript scans, synthetic
charts and line patterns, UI screenshots, and AI-generated graphics. It spans the hard
cases that codecs trip on — photographic noise, hard-edged vector graphics, bilevel
scan texture, dense text, and large flat regions.

Every file follows one naming convention (lowercase throughout):

```
<NNNN>_<category>_<descriptor>[_<keyinfo>...]_<WxH>.<ext>
```

`NNNN` is a unique id from the folder's block; `descriptor` carries the description and
attribution; `keyinfo` holds page/variant/camera/iso (f-stops written `f1p8`, no dot).

## Folders

| Folder | n | Size | Format | Dims (MP) | Source | License | Contents |
|---|--:|--:|---|---|---|---|---|
| `1000-lilith-photos-general` | 72 | 0.26 GB | jpg/heic | 6.9–12.2 | lilith (own) | PD-own | Everyday photographs (phones: Note9…S25U, iPhones), mixed subjects |
| `1200-lilith-interiors` | 49 | 0.20 GB | jpg/heic/png | 3.0–12.2 | lilith (own) | PD-own | Interior & architecture photos |
| `1400-lilith-nature` | 157 | 0.75 GB | heic/jpg/png/dng | 1.6–124 | lilith (own) | PD-own | Landscape/nature; incl. HEIC + 3 DNG raws + panoramas |
| `1600-lilith-food` | 41 | 0.13 GB | jpg/heic | 7.4–24.5 | lilith (own) | PD-own | Food & plated dishes |
| `2000-unsplash-people` | 28 | 0.08 GB | jpg | 6.0–32.7 | Unsplash | Unsplash-License | Stock portraits / people |
| `2200-unsplash-renders` | 13 | 0.04 GB | jpg | 5.8–99 | Unsplash | Unsplash-License | Abstract / 3D renders |
| `2400-unsplash-textures` | 10 | 0.09 GB | jpg | 5.6–102 | Unsplash | Unsplash-License | Surfaces / textures |
| `3000-art-institute-of-chicago-photos` | 15 | 0.04 GB | jpg | 1.8–20.9 | Art Institute of Chicago | CC0 | Open-access artwork reproductions (paintings, prints) |
| `3300-met-museum-photos` | 24 | 0.06 GB | jpg | 0.2–16 | The Met | CC0 | Open-access artwork reproductions |
| `5000-national-park-service-brochures` | 59 | 0.56 GB | png | 0.5–55.6 | NPS | PD-USGov | Park brochures/maps (born-digital PDF→PNG), color + grayscale variants |
| `5200-epa-climate-impact-2021-report` | 25 | 0.03 GB | png | 11.4 | EPA | PD-USGov | Report pages — text, tables, charts |
| `5300-noaa-hurricane-documents` | 44 | 0.06 GB | png | 8.4 | NOAA/NHC | PD-USGov | Hurricane report pages — text, maps, charts |
| `6000-lilith-scans-public-patents` | 113 | 0.09 GB | png/jpg | 7.9–8.0 | USPTO | PD | 3 US patents × {1-bit CCITT original, color rescan, gray rescan} — **bilevel + scan texture** |
| `6600-ia-scans-manuscript-illustrations` | 36 | 1.04 GB | png | 6.0–37 | Internet Archive | PD | PD plate scans (Haeckel, Hokusai, Owen Jones, Redouté, Shin-Bijutsukai, Trouvelot) |
| `6800-ia-scans-manuscript-text` | 36 | 0.32 GB | png | 6.0–30 | Internet Archive | PD | PD manuscript text-page scans (same works) |
| `7000-lilith-plots` | 126 | 0.02 GB | png | 1.0 | lilith (generated) | PD-own | Synthetic charts + line/polygon test patterns (hard-edge graphics) |
| `8000-lilith-mobile-screenshots` | 32 | 0.04 GB | png/jpg | 2.3–5.6 | various web/apps | **screenshot-unverified** | Phone UI/web screenshots — third-party content |
| `8100-lilith-web-screenshots` | 370 | 0.27 GB | png | 0.2–5.2 | various web | PD (PD-filtered) | Website viewport crops at multiple resolutions/dpr |
| `9000-lilith-ai-clipart` | 86 | 0.04 GB | png | 1.0–1.6 | lilith (AI-gen) | PD-own | AI-generated clipart (flat / transparent) |
| `9094-lilith-ai-illustrations` | 75 | 0.26 GB | png | 1.6 | lilith (AI-gen) | PD-own | AI-generated illustrations |
| `9226-lilith-ai-products` | 749 | 1.53 GB | png/jpg | 1.0–17 | lilith (AI-gen) | PD-own | AI-generated product images, by product-category subfolders |

## Licensing status

| License | Files | Notes |
|---|--:|---|
| PD-own | 1,360 | lilith's own photos, generated plots, AI images — released to public domain |
| PD | 533 | US patents + Internet-Archive scans of PD works; per-file `pd_basis` in manifests |
| PD-USGov | 128 | NPS/EPA/NOAA federal works (17 USC §105) |
| Unsplash-License | 51 | Free to use; attribution appreciated (photographer in filename). Unsplash terms apply |
| CC0 | 39 | Met + Art Institute open-access (public-domain dedication) |
| Mailing-list-PD / PD-tool | 17 | within the web-screenshot set |
| **screenshot-unverified** | 32 | mobile screenshots of third-party sites — **not cleared for redistribution** |

**~2,128 of 2,160 files are public-domain or PD-own** (freely usable). The 51
Unsplash images follow the Unsplash License; the 32 mobile screenshots are unverified
and should be excluded from any redistribution until cleared.

Caveats: `license` is **folder-level best-effort, not per-file legal clearance**.
Artwork "photos" are reproductions of PD works (CC0 on the reproduction). Document
and screenshot renders may embed third-party logos/photos whose rights differ from the
page. For redistribution, verify the 8000 set and spot-check embedded media.

## Manifests

- `<folder>/MANIFEST.tsv` — per-folder, one row per image, with full provenance
  (`original_filename`, camera/artist/contributor/`pd_basis`/`url`/`sha256` where
  applicable) and license. **Source of truth.**
- `CORPUS-MANIFEST.tsv` — unified aggregate (regenerate via
  `python3 scripts/build_corpus_manifest.py`). Column docs in `CORPUS-MANIFEST.README.md`.

Excluded from the corpus: `nope/` (rejected candidates) and the build-history archive
at `/mnt/v/output/codec-corpus/imazen-26-archive-2026-06-09/`.
