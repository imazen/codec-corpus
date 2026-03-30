# HEIC/HEIF Conformance Test Files

Test files for HEIC/HEIF (ISO/IEC 23008-12) decoder and parser validation.

## Directory Structure

```
heic-conformance/
  valid/
    nokia-conformance/   63 files, 21 MB  - Nokia HEIF conformance candidates
    dsoprea-exif/         4 files, 6.5 MB - HEIC files with EXIF metadata
    libheif-testdata/    90 files, 1.2 MB - libheif uncompressed/compressed variants
  invalid/                8 files         - Corrupt/malformed files for error handling
  edge-cases/             5 files         - Minimal/unusual but structurally interesting files
```

Total: 170 files, ~29 MB

## Sources

### nokia-conformance/ (NO LICENSE - use with caution)

- **Source:** https://github.com/nokiatech/heif_conformance
- **License:** NO LICENSE FILE. An open issue (https://github.com/nokiatech/heif_conformance/issues/3) asking about the license has been unanswered since 2019. The related Nokia HEIF *code* repository uses a non-commercial evaluation/testing/academic research license. These conformance files may fall under similar terms but this is unconfirmed.
- **Risk:** Cannot redistribute or use commercially without clarification from Nokia.
- **Content:** 63 .heic files (C001-C053, MIAF001-007, multilayer001-005) covering:
  - Single image items (C002)
  - Multiple image items with shared/different decoder configs (C003, C004)
  - Thumbnails (C005, C012)
  - Alpha auxiliary images (C006, C052)
  - Grid derived images 1x1 through 3x2 (C007, C022-C025)
  - Identity transforms with rotation/crop (C008, C013, C014, C039)
  - Hidden images (C009)
  - Alternative groups (C010, C011)
  - Overlay derived images with various offsets/fill (C015-C021)
  - Image sequences: all-intra, inter-predicted (C026-C028)
  - Edit lists: repeat, pause, loop (C029-C031, C036-C038)
  - Sequence with thumbnails (C032)
  - EXIF metadata (C034)
  - Mirror transform (C042)
  - Predictively coded items (C043, C044)
  - Burst sequences (C045, C046)
  - Time-synchronized capture groups (C047, C048)
  - Audio association (C049)
  - Album collections with user descriptions (C050)
  - Creation/modification timestamps (C051)
  - Stereo entity groups (C053)
  - MIAF profiles: Basic, Advanced, Extended, Progressive (MIAF001-007)
  - Multi-layer: base+enhanced quality, multi-view stereo, AVC base layer (multilayer001-005)
- **Descriptions:** See `conformance_file_descriptions.xlsx` in the directory.

### dsoprea-exif/ (MIT)

- **Source:** https://github.com/dsoprea/heic-exif-samples
- **License:** MIT
- **Content:** 4 HEIC files with EXIF metadata from real devices. Useful for testing EXIF extraction from HEIF containers. Sizes range from 41 KB to 2.9 MB.

### libheif-testdata/ (LGPL-3.0 library, MIT examples)

- **Source:** https://github.com/strukturag/libheif (`tests/data/` and `examples/`)
- **License:** Library test data under LGPL-3.0-or-later; example files under MIT.
- **Content:** 90 files covering:
  - `example.heic` - Standard HEVC-compressed HEIF image
  - `example.avif` - AVIF sample
  - `lightning_mini.heif` - Small compressed HEIF
  - 7 compressed variants (brotli, deflate, zlib, tiled)
  - 2 AVIF files (with alpha, with metadata)
  - ~78 uncompressed HEIF variants covering:
    - Color spaces: RGB, ABGR, YUV (420/422), YVU, VUY, monochrome
    - Bit depths: 8-bit, 16-bit, 5-6-5, 7-bit sub-byte
    - Storage: component-interleaved, pixel-interleaved, row-interleaved, tile-interleaved
    - Tiling: tiled and non-tiled variants
    - Row alignment: with and without tile row alignment
    - Padding channels: RGxB (with unused channel)

## invalid/

Hand-crafted corrupt files for testing error handling:

| File | Description |
|------|-------------|
| `zero_length.heic` | 0 bytes |
| `truncated_100bytes.heic` | First 100 bytes of C001.heic (truncated mid-ftyp) |
| `truncated_1k.heic` | First 1024 bytes of C001.heic (truncated mid-box) |
| `bad_ftyp_magic.heic` | Recognizable ftyp header followed by random data |
| `corrupt_after_ftyp.heic` | Valid ftyp box from C001 followed by 1024 bytes of random data |
| `wrong_format.heic` | A JPEG file renamed to .heic |
| `bitflip_c002.heic` | C002.heic with 20 random byte-flips after the ftyp box |
| `oversized_box.heic` | ftyp box claiming 1 GB length with only 36 bytes of data |

## edge-cases/

Structurally interesting minimal files:

| File | Description |
|------|-------------|
| `minimal_ftyp_only.heic` | Valid ftyp box with `heic` brand, no other boxes |
| `double_ftyp.heic` | Two consecutive ftyp boxes (invalid per spec) |
| `many_brands.heic` | ftyp box with 50+ compatible brands |
| `heix_brand.heic` | ftyp with `heix` major brand (HEVC with extensions) |
| `avif_brand.heif` | ftyp with `avif` major brand (AVIF in HEIF container) |

## Additional Sources (Not Downloaded)

These sources were identified but not included for various reasons:

- **GPAC ComplianceWarden** (https://github.com/gpac/ComplianceWarden): BSD-3 licensed compliance checker. Test vectors are described in x86 assembly (NASM), not as actual media files. Useful for understanding spec requirements but not for binary test data.
- **heic.digital samples** (https://heic.digital/samples/): iPhone/Android HEIC photos. No explicit license for redistribution.
- **MPEG HEIF conformance tests**: Requires MPEG membership to access.
- **ISO/IEC 23008-12 test vectors**: Not freely available.
