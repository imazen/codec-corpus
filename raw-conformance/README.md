# RAW/DNG Conformance Test Suite

Camera RAW and DNG format test files for decoder conformance testing. Files are stored with **Git LFS** due to their size (10-50 MB each).

## Setup

Requires [Git LFS](https://git-lfs.github.com/) installed:

```bash
git lfs install
git lfs pull  # downloads the actual RAW files
```

If Git LFS is not installed, the RAW files will appear as small pointer files.

## Source: raw.pixls.us

The primary source of camera RAW samples is the [raw.pixls.us](https://raw.pixls.us/) project, which provides CC0-licensed sample files from hundreds of camera models.

To fetch the full raw.pixls.us dataset (for curation):

```bash
git clone https://raw.pixls.us/data.lfs.git /tmp/raw-pixls-data
```

## Directory Structure

```
raw-conformance/
  valid/
    bayer/          # Standard Bayer CFA DNG and proprietary RAW
    xtrans/         # X-Trans CFA (Fuji)
    linear/         # Linear DNG (demosaiced)
    compressed/     # Lossy DNG, JXL-compressed DNG
    mobile/         # Apple ProRAW, Android DNG
    proprietary/    # CR2, NEF, ARW, ORF, RW2, CR3, RAF
  invalid/          # Truncated, corrupt headers, bad IFDs
  edge-cases/       # Unusual bit depths, tiled, multi-page
  fetch.sh          # Script to populate from raw.pixls.us
  README.md
```

## Coverage Targets

### Bayer CFA (standard RAW)
- Canon CR2 (Canon 350D or similar)
- Nikon NEF (Nikon D40 or similar)
- Sony ARW (Sony NEX-3 or similar)
- Standard DNG (various converters)

### X-Trans CFA
- Fuji RAF (Fuji X-T1 or similar)
- X-Trans DNG (converted)

### Linear DNG
- Demosaiced linear DNG (scene-referred)

### Compressed DNG
- Lossy DNG (compression type 34892)
- JXL-compressed DNG (compression type 52546)

### Mobile
- Apple ProRAW / APPLEDNG (DNG 1.6, LJPEG predictor 7, LinearRaw)
- Apple AMPF (JPEG + HDR gain map, NOT actual raw)
- Samsung Galaxy DNG (Android)

### Proprietary RAW
One representative file per major camera family:
- Canon CR2, CR3 (CRAW)
- Nikon NEF
- Sony ARW
- Olympus ORF
- Panasonic RW2
- Fuji RAF

### Invalid
- Truncated TIFF IFD
- Bad strip/tile offsets
- Impossible CFA pattern descriptor
- Corrupt TIFF header (bad byte order marker)
- Zero-dimension IFD entries

### Edge Cases
- Tiled DNG
- Multi-page DNG
- Unusual bit depths (12-bit packed, 14-bit, 16-bit)
- Dual-pixel RAW

## License

- **raw.pixls.us samples**: [CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/) — Public domain
- **Local samples** (Apple ProRAW, Android DNG): Personal test files, not redistributed

## File Size

Expected total: 100-300 MB (Git LFS tracked).
Individual files: 10-50 MB each.
