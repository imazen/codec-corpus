# Codec Corpus

A curated collection of reference images for codec quality calibration, compression benchmarking, and format conformance testing.

## Quick Start

```bash
# Clone everything (~280 MB)
git clone https://github.com/imazen/codec-corpus.git

# Or clone just one dataset
git clone --depth 1 --filter=blob:none --sparse \
  https://github.com/imazen/codec-corpus.git
cd codec-corpus
git sparse-checkout set kodak

# Add more datasets later
git sparse-checkout add pngsuite clic2025
```

---

## Datasets

### Quality Calibration & Compression Research

| Dataset | Images | Size | Best For |
|---------|--------|------|----------|
| [CLIC 2025](#clic-2025) | 62 | 219 MB | High-res quality calibration |
| [CID22](#cid22) | 250 | 15 MB | Perceptual quality training |
| [Kodak](#kodak) | 24 | 15 MB | Classic compression benchmarks |

### Format Conformance & Edge Cases

| Dataset | Files | Size | Best For |
|---------|-------|------|----------|
| [JXL](#jxl) | 81 | 7.7 MB | JPEG XL decoder conformance |
| [PNGSuite](#pngsuite) | 176 | 60 KB | PNG decoder conformance |
| [image-rs](#image-rs) | 127 | 4.2 MB | Multi-format edge cases |
| [zune](#zune-image) | 3,431 | 20 MB | Fuzz testing, decoder robustness |
| [mozjpeg](#mozjpeg) | 16 | 1.1 MB | JPEG codec testing |

---

## CLIC 2025

**Challenge on Learned Image Compression 2025** — High-resolution images for compression quality research.

| Folder | Images | Size | Purpose |
|--------|--------|------|---------|
| `validation/` | 32 | 103 MB | Tune and calibrate against these |
| `final-test/` | 30 | 116 MB | Holdout set — final evaluation only |

- **Source**: https://clic2025.compression.cc/
- **License**: [Unsplash License](https://unsplash.com/license) — Free for any use, no attribution required
- **Resolution**: High-resolution PNG images

---

## CID22

**Compression Image Dataset 2022** by Cloudinary — Diverse images for perceptual quality research.

| Folder | Images | Purpose |
|--------|--------|---------|
| `CID22-512/validation/` | 41 | Held out for validation |
| `CID22-512/training/` | 209 | Model training and calibration |

- **Source**: https://github.com/Cloudinary/CID22
- **License**: [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/)
- **Resolution**: Resized to 512px on longest edge

---

## Kodak

**Kodak Lossless True Color Image Suite** — The classic benchmark for image compression research since the 1990s.

| Folder | Images | Resolution |
|--------|--------|------------|
| `kodak/` | 24 | 768×512 |

- **Source**: http://r0k.us/graphics/kodak/
- **License**: Unrestricted usage (released by Eastman Kodak Company)
- **Format**: Lossless PNG

---

## JXL

**JPEG XL Conformance Test Suite** — Comprehensive coverage of JPEG XL decoder features.

| Folder | Files | Purpose |
|--------|-------|---------|
| `jxl/conformance/` | 40 | Official conformance tests |
| `jxl/features/` | 26 | Feature-specific tests |
| `jxl/photographic/` | 4 | Real-world photographs |
| `jxl/edge-cases/` | 13 | Minimal and boundary tests |

Covers:
- Encoding: Lossless (modular), Lossy (VarDCT), progressive
- Color: 8/12/16/32-bit, sRGB, linear, grayscale, CMYK
- HDR: PQ and HLG transfer functions
- Animation: Multi-frame, variable timing, splines
- Alpha: Premultiplied, non-premultiplied, blendmodes
- Features: Orientation, patches, ICC profiles, JPEG reconstruction

- **Source**: https://github.com/libjxl/libjxl, https://github.com/libjxl/conformance
- **License**: BSD-3-Clause

---

## PNGSuite

**Official PNG Conformance Test Suite** by Willem van Schaik — Tests all PNG features.

| Folder | Images | Purpose |
|--------|--------|---------|
| `pngsuite/` | 176 | Full PNG feature coverage |

Covers:
- Bit depths: 1, 2, 4, 8, 16
- Color types: grayscale, RGB, palette, alpha
- Interlacing, transparency, gamma correction
- Corrupted files for error handling

- **Source**: http://www.schaik.com/pngsuite/
- **License**: Freeware — free to use, copy, modify, distribute

---

## image-rs

**Rust image library test images** — Format edge cases and malformed files.

| Folder | Files | Purpose |
|--------|-------|---------|
| `test-images/bmp/` | 59 | BMP format variants + malformed files |
| `test-images/gif/` | 11 | GIF animation edge cases |
| `test-images/ico/` | 7 | Icon format tests |
| `test-images/jpg/` | 6 | JPEG metadata, progressive encoding |
| `test-images/png/` | 22 | 16bpc, APNG, transparency |
| `test-images/tiff/` | 10 | TIFF compression, predictors |
| `test-images/webp/` | 9 | WebP lossless/lossy variants |

- **Source**: https://github.com/image-rs/image
- **License**: [MIT License](https://opensource.org/licenses/MIT)

---

## zune-image

**zune-image test suite** — Fuzz corpus and JPEG edge cases for decoder robustness.

| Folder | Files | Purpose |
|--------|-------|---------|
| `test-images/jpeg/` | 31 | JPEG edge cases (CMYK, progressive, sampling) |
| `test-images/jpeg/benchmarks/` | 8 | 7680×4320 decoder speed benchmarks |
| `fuzz-corpus/jpeg/` | 1,836 | Minimal JPEG fuzz test cases |
| `fuzz-corpus/png/` | ~800 | Minimal PNG fuzz test cases |
| `fuzz-corpus/inflate/` | ~700 | Compression edge cases |

- **Source**: https://github.com/etemesi254/zune-image
- **License**: MIT OR Apache-2.0 OR Zlib (triple-licensed)

---

## mozjpeg

**Mozilla JPEG encoder test images** — Reference files for JPEG codec testing.

| File | Purpose |
|------|---------|
| `testorig.ppm` | Source image (PPM format) |
| `testorig.jpg` | Baseline JPEG |
| `testimgari.jpg` | Arithmetic coding |
| `testimgint.jpg` | Progressive JPEG |
| `testorig12.jpg` | 12-bit JPEG |
| `*.bmp` | BMP test images |
| `*.icc` | ICC color profiles |

- **Source**: https://github.com/mozilla/mozjpeg
- **License**: IJG License + Modified BSD License

---

## Directory Structure

```
codec-corpus/
├── clic2025/
│   ├── LICENSE
│   ├── validation/          # 32 high-res images (tune against)
│   └── final-test/          # 30 high-res images (holdout)
├── CID22/
│   ├── LICENSE
│   └── CID22-512/
│       ├── validation/      # 41 images
│       └── training/        # 209 images
├── jxl/
│   ├── LICENSE
│   ├── README.md
│   ├── conformance/         # 40 official conformance tests
│   ├── features/            # 26 feature-specific tests
│   ├── photographic/        # 4 real-world photographs
│   └── edge-cases/          # 13 minimal/boundary tests
├── kodak/
│   ├── LICENSE
│   └── *.png                # 24 classic test images
├── pngsuite/
│   ├── LICENSE
│   └── *.png                # 176 PNG conformance images
├── image-rs/
│   ├── LICENSE-MIT
│   └── test-images/
│       ├── bmp/
│       ├── gif/
│       ├── ico/
│       ├── jpg/
│       ├── png/
│       ├── tiff/
│       └── webp/
├── zune/
│   ├── LICENSE-MIT
│   ├── LICENSE-APACHE
│   ├── LICENSE-ZLIB
│   ├── test-images/jpeg/
│   └── fuzz-corpus/
│       ├── jpeg/
│       ├── png/
│       └── inflate/
└── mozjpeg/
    ├── LICENSE
    └── *.ppm, *.jpg, *.bmp, *.icc
```

---

## License Summary

| Dataset | License | Commercial Use | Attribution |
|---------|---------|----------------|-------------|
| CLIC 2025 | Unsplash | Yes | No |
| CID22 | CC BY-SA 4.0 | Yes | Required |
| JXL | BSD-3-Clause | Yes | No |
| Kodak | Unrestricted | Yes | No |
| PNGSuite | Freeware | Yes | No |
| image-rs | MIT | Yes | No |
| zune | MIT/Apache-2.0/Zlib | Yes | No |
| mozjpeg | IJG + BSD | Yes | No |

---

## Usage

This corpus is used by [imageflow](https://github.com/imazen/imageflow) for codec quality calibration testing.

**See also**: [Codec Comparison Guide](https://github.com/imazen/codec-comparison) — How to compare image codecs fairly, quality metrics, and scientific methodology.

## Contributing

To suggest additional datasets, please open an issue with:
- Source URL
- License information
- Description of what the dataset tests
