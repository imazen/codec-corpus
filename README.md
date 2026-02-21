# Codec Corpus

A curated collection of reference images for codec quality calibration, compression benchmarking, and format conformance testing. Maintained by [Imazen](https://github.com/imazen).

Total repo size: ~577 MB committed, plus ~1 GB available via download scripts.

## Quick Start

```bash
# Clone everything (~600 MB)
git clone https://github.com/imazen/codec-corpus.git

# Or clone just one dataset using sparse checkout
git clone --depth 1 --filter=blob:none --sparse \
  https://github.com/imazen/codec-corpus.git
cd codec-corpus
git sparse-checkout set clic2025

# Add more datasets later
git sparse-checkout add CID22 gb82-sc qoi-benchmark
```

---

## Datasets at a Glance

### Quality Calibration & Compression Research

| Dataset | Images | Size | Resolution | License | Best For |
|---------|--------|------|------------|---------|----------|
| [CLIC 2025](#clic-2025) | 62 | 219 MB | ~2048px long edge | Unsplash | High-res lossy quality calibration |
| [CID22](#cid22) | 250 | 94 MB | 512×512 | CC BY-SA 4.0 | Perceptual quality training, diverse content |
| [KADID-10k](#kadid-10k) | 81 | 25 MB | 512×384 | Pixabay | IQA research reference images |
| [GB82](#gb82) | 25 | 9.6 MB | 576×576 | CC0 | Compact photographic benchmarking |
| [GB82-SC](#gb82-sc) | 10 | 2.9 MB | Various (640–2940px) | CC0 | Screen content & screenshot compression |
| [QOI Benchmark](#qoi-benchmark-suite) | 15+ | 39 MB+ | Various (1313×2874–8008) | CC0/PD/Mixed | Web screenshots, icons, textures |

### Format Conformance & Edge Cases

| Dataset | Files | Size | License | Best For |
|---------|-------|------|---------|----------|
| [JPEG Conformance](#jpeg-conformance) | 277 | 76 MB | MIT/IJG+BSD/Various | JPEG decoder conformance & robustness |
| [JXL](#jxl) | 188 | 88 MB | BSD-3-Clause | JPEG XL decoder conformance |
| [PNGSuite](#pngsuite) | 178 | 720 KB | Freeware | PNG decoder conformance |
| [WebP Conformance](#webp-conformance) | 230 | 1.3 MB | Various | WebP decoder conformance |
| [image-rs](#image-rs) | 127 | 4.5 MB | MIT | Multi-format edge cases |
| [zune-image](#zune-image) | 3,434 | 33 MB | MIT/Apache-2.0/Zlib | Fuzz testing, decoder robustness |
| [mozjpeg](#mozjpeg) | 16 | 1.2 MB | IJG + BSD | JPEG codec reference files |
| [imageflow](#imageflow) | 51 | 7.8 MB | Various | Orientation, format conversion edge cases |

---

## Quality Calibration & Compression Research

### CLIC 2025

**Challenge on Learned Image Compression 2025** — High-resolution photographic images curated for compression quality research. This is the most relevant modern benchmark for lossy codec calibration, featuring large, diverse photographs at resolutions representative of modern camera output and web delivery.

| Folder | Images | Size | Purpose |
|--------|--------|------|---------|
| `clic2025/training/` | 32 | 103 MB | Tune and calibrate encoders against these |
| `clic2025/final-test/` | 30 | 116 MB | Holdout set — final evaluation only |

Note: The CLIC challenge calls these "validation" and "test" respectively. We renamed "validation" → "training" because in codec development, "validation" typically implies a holdout set, which is the opposite of the intended use. See [`clic2025/README.md`](clic2025/README.md) for the original naming and download links.

- **Resolution**: ~2048px on long edge (varies from 878px to 2048px on short edge)
- **Format**: Lossless PNG, 8-bit sRGB
- **Source**: https://clic2025.compression.cc/
- **License**: [Unsplash License](https://unsplash.com/license) — Free for any use, no attribution required, cannot be sold unmodified or used to build a competing service.

---

### CID22

**Cloudinary Image Dataset 2022** — 250 diverse images selected by Cloudinary for perceptual quality research. This dataset spans a wider variety of content types than most photographic benchmarks: portraits, landscapes, text, graphics, medical imagery, scientific plots, and more. The 512px size makes it fast to process while remaining large enough for meaningful perceptual quality evaluation.

For compression benchmarking, CID22 is one of the best choices available: it was specifically designed for this purpose, its diversity avoids the over-fitting that plagues small homogeneous corpora, and the training/validation split enables principled evaluation.

| Folder | Images | Purpose |
|--------|--------|---------|
| `CID22/CID22-512/validation/` | 41 | Held out for validation |
| `CID22/CID22-512/training/` | 209 | Model training and calibration |

- **Resolution**: 512×512, 8-bit sRGB
- **Source**: https://github.com/Cloudinary/CID22
- **License**: [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/) — Free for any use, attribution required. Derivative works must use the same license.

---

### KADID-10k

**Konstanz Artificially Distorted Image quality Database** — 81 pristine reference images used for image quality assessment (IQA) research. Created at the University of Konstanz, this dataset is widely used for training and evaluating perceptual quality metrics. Only the pristine reference images are included here; the 10,125 distorted variants are not.

| Folder | Images | Size |
|--------|--------|------|
| `kadid10k/` | 81 | 25 MB |

- **Resolution**: 512×384, 8-bit sRGB
- **Format**: Lossless PNG
- **Source**: https://database.mmsp-kn.de/kadid-10k-database.html
- **License**: [Pixabay License](https://pixabay.com/service/license/) — Free for commercial and non-commercial use, no attribution required.
- **Citation**: H. Lin, V. Hosu and D. Saupe, "KADID-10k: A Large-scale Artificially Distorted IQA Database," 2019 Eleventh International Conference on Quality of Multimedia Experience (QoMEX), Berlin, Germany, 2019, pp. 1-3, doi: [10.1109/QoMEX.2019.8743252](https://doi.org/10.1109/QoMEX.2019.8743252).

---

### GB82

**GB82 Image Dataset** by Gianni Rosato — A compact, purpose-built CC0 dataset for image compression algorithm development. The 25 images are selected to be challenging: fine facial details, low-contrast sky gradients, digital noise, fine textures. Optimizing for weak metrics like PSNR should not yield visually compelling results on most images.

| Folder | Images | Size |
|--------|--------|------|
| `gb82/` | 25 | 9.6 MB |

Image categories:
- 3 portraits (2 human, 1 dog)
- 6 landscapes
- 8 closeups of inanimate objects or scenes
- 4 low-light shots
- 3 realistically rendered graphics
- 1 non-photographic image

- **Resolution**: 576×576, 8-bit sRGB, D65 white point
- **Source**: https://github.com/gianni-rosato/gb82-image-set
- **License**: [CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/) — Public domain, no restrictions.

---

### GB82-SC

**GB82 Screen Content Dataset** by Gianni Rosato — Screenshots and screen content images focusing on text, UI elements, and graphics from five platforms. Screen content compresses very differently from photographic content: sharp edges, flat color regions, anti-aliased text, and high-contrast UI elements stress different codec paths. This dataset fills an important gap that photographic benchmarks miss entirely.

| File | Resolution | Content |
|------|------------|---------|
| `codec_wiki.png` | 2560×1664 | Wikipedia article with text and diagrams |
| `gmessages.png` | 1440×3088 | Google Messages (Android) |
| `graph.png` | 796×481 | Data visualization / chart |
| `gui.png` | 1356×1132 | Desktop GUI elements |
| `imac_dark.png` | 2940×1912 | macOS desktop (dark mode) |
| `imac_g3.png` | 2940×1912 | macOS desktop (light, iMac G3 style) |
| `imessage.png` | 1206×2622 | iMessage conversation (iOS) |
| `terminal.png` | 1646×1062 | Terminal / command line |
| `windows95.png` | 640×480 | Windows 95 desktop |
| `windows.png` | 2560×1392 | Modern Windows desktop |

- **Resolution**: Various (640×480 to 2940×1912), 8-bit sRGB
- **Source**: https://github.com/gianni-rosato/gb82-image-set
- **License**: [CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/) — Public domain, no restrictions.

---

### QOI Benchmark Suite

Subsets from the [QOI Benchmark Suite](https://qoiformat.org/benchmark/) by Dominic Szablewski. The `screenshot_web` subset is committed directly to the repo; all other subsets can be fetched with the included download scripts.

**Committed:**

| Subset | Files | Size | License | Content |
|--------|-------|------|---------|---------|
| `qoi-benchmark/screenshot_web/` | 14 | 39 MB | CC0 1.0 | Full-page web screenshots (amazon, apple, cnn, wikipedia, reddit, etc.) |

**Available via download scripts:**

| Subset | Files | Size | License | Content |
|--------|-------|------|---------|---------|
| `icon_512` | 214 | 12 MB | Public Domain | Tango Icon Library at 512px |
| `icon_64` | 214 | 1.3 MB | Public Domain | Tango Icon Library at 64px |
| `screenshot_game` | 619 | 256 MB | CC BY-SA 3.0 | Game screenshots from Wikimedia Commons |
| `textures_pk` | 1004 | 44 MB | — | Texture pack |
| `textures_pk01` | 115 | 19 MB | — | Texture pack 01 |
| `textures_pk02` | 237 | 99 MB | — | Texture pack 02 |
| `textures_plants` | 61 | 50 MB | — | Plant textures |
| `textures_photo` | 21 | 37 MB | — | Photographic textures |
| `photo_kodak` | 25 | 15 MB | Unrestricted | Kodak suite<sup>1</sup> |
| `photo_tecnick` | 101 | 228 MB | — | Tecnick photographic set |
| `photo_wikipedia` | 50 | 85 MB | — | Wikipedia photographs |
| `pngimg` | 189 | 220 MB | CC BY-NC 4.0 | PNG images (non-commercial only) |

```bash
# Download all subsets (~1.1 GB tarball)
cd qoi-benchmark && ./download.sh

# Download specific subsets
./download.sh icon_512 icon_64

# List available subsets
./download.sh --list
```

```powershell
# Windows
cd qoi-benchmark
.\download.ps1
.\download.ps1 -Subsets icon_512,icon_64
.\download.ps1 -List
```

- **Source**: https://qoiformat.org/benchmark/
- **License**: Per-subset (see table above). `screenshot_web` is CC0. `icon_*` subsets are Public Domain (Tango Icon Library). `screenshot_game` is CC BY-SA 3.0 (Wikimedia Commons). `pngimg` is CC BY-NC 4.0 (non-commercial only). Other subsets have unspecified licensing in the archive.

---

## Format Conformance & Edge Cases

### JPEG Conformance

**JPEG Decoder Conformance Test Suite** — Files organized by expected decoder behavior, designed for systematic testing of JPEG decoders.

| Folder | Files | Expected Behavior |
|--------|-------|-------------------|
| `jpeg-conformance/valid/` | 41 | MUST decode correctly |
| `jpeg-conformance/invalid/` | 116 | MUST reject gracefully |
| `jpeg-conformance/non-conformant/` | 40 | MAY reject or recover |
| `jpeg-conformance/crash-repro/` | 78 | Crash reproducers from upstream decoders |

**valid/** — Reference JPEG images including camera samples from 12 manufacturers, restart intervals, CMYK/YCCK color spaces, and various sampling configurations.

**invalid/** — Crash tests and malformed files from imagetestsuite and fuzzing. Decoders must not crash or hang on these.

**non-conformant/** — Files that violate the JPEG spec but are common in the wild:
- `truncated/` — Files cut at various stream positions
- `extraneous-data/` — Extra bytes in unexpected locations
- `marker-quirks/` — Unusual marker sequences (e.g., multiple 0xFF before EOI)
- `metadata-quirks/` — ICC profile chunk issues (ordering, duplicates, missing chunks)
- `progressive-quirks/` — Progressive scan encoding edge cases

Each non-conformant file has a companion `.txt` file explaining the defect and expected strict vs. lenient decoder behavior.

- **Sources**: See [jpeg-conformance/SOURCES.md](jpeg-conformance/SOURCES.md) for full per-file attribution
- **License**: MIT / IJG+BSD / Various (per source)

---

### JXL

**JPEG XL Test Suite** — Comprehensive coverage of JPEG XL decoder features and conformance.

| Folder | Files | Size | Purpose |
|--------|-------|------|---------|
| `jxl/conformance/` | 39 | 6.2 MB | Official libjxl conformance tests |
| `jxl/features/` | 128 | 81 MB | Feature-specific test images |
| `jxl/edge-cases/` | 13 | 332 KB | Minimal and boundary-condition tests |

Feature coverage:
- **Encoding**: Lossless (modular), lossy (VarDCT), progressive
- **Color**: 8/12/16/32-bit depth, sRGB, linear, grayscale, CMYK
- **HDR**: PQ and HLG transfer functions
- **Animation**: Multi-frame, variable timing, splines
- **Alpha**: Premultiplied, non-premultiplied, blend modes
- **Features**: EXIF orientation, patches, ICC profiles, JPEG reconstruction

- **Source**: https://github.com/libjxl/libjxl, https://github.com/libjxl/conformance
- **License**: [BSD-3-Clause](https://opensource.org/licenses/BSD-3-Clause)

---

### PNGSuite

**Official PNG Conformance Test Suite** by Willem van Schaik — Covers all PNG features for decoder conformance testing.

| Folder | Files | Size |
|--------|-------|------|
| `pngsuite/` | 176 | 720 KB |

Coverage:
- Bit depths: 1, 2, 4, 8, 16
- Color types: grayscale, RGB, palette, grayscale+alpha, RGBA
- Interlacing (Adam7), transparency (tRNS), gamma correction
- Ancillary chunks: sRGB, iCCP, tEXt, sBIT, pHYs, etc.
- Corrupted files for error handling

- **Source**: http://www.schaik.com/pngsuite/
- **License**: Freeware — free to use, copy, modify, and distribute.

---

### image-rs

**Rust image library test images** — Multi-format edge cases and malformed files used by the [image-rs](https://github.com/image-rs/image) crate.

| Folder | Files | Content |
|--------|-------|---------|
| `image-rs/test-images/bmp/` | 60 | BMP format variants and malformed files |
| `image-rs/test-images/gif/` | 11 | GIF animation edge cases |
| `image-rs/test-images/ico/` | 7 | Icon format tests |
| `image-rs/test-images/jpg/` | 7 | JPEG metadata, progressive encoding |
| `image-rs/test-images/png/` | 22 | 16-bit, APNG, transparency |
| `image-rs/test-images/tiff/` | 10 | TIFF compression, predictors |
| `image-rs/test-images/webp/` | 9 | WebP lossless/lossy variants |

- **Source**: https://github.com/image-rs/image
- **License**: [MIT](https://opensource.org/licenses/MIT)

---

### zune-image

**zune-image test suite** — Fuzz corpus and decoder robustness tests from the [zune-image](https://github.com/etemesi254/zune-image) project.

| Folder | Files | Purpose |
|--------|-------|---------|
| `zune/test-images/jpeg/` | 30 | JPEG edge cases (CMYK, progressive, subsampling) |
| `zune/fuzz-corpus/jpeg/` | 1,836 | Minimal JPEG fuzz inputs |
| `zune/fuzz-corpus/png/` | 837 | Minimal PNG fuzz inputs |
| `zune/fuzz-corpus/inflate/` | 726 | DEFLATE/inflate edge cases |

The fuzz corpus files are minimal inputs designed to exercise specific code paths and edge cases in decoders. They are not meaningful images — they exist to catch crashes, hangs, and memory safety issues.

- **Source**: https://github.com/etemesi254/zune-image
- **License**: MIT OR Apache-2.0 OR Zlib (triple-licensed)

---

### mozjpeg

**Mozilla JPEG encoder test images** — Reference files for JPEG codec testing, from the mozjpeg project.

| File | Purpose |
|------|---------|
| `testorig.ppm` | Source image (PPM format) |
| `testorig.jpg` | Baseline JPEG reference |
| `testimgari.jpg` | Arithmetic-coded JPEG |
| `testimgint.jpg` | Progressive JPEG |
| `testorig12.jpg` | 12-bit JPEG |
| `shira_bird8.bmp`, `monkey16.ppm` | Additional source images |
| `test.scan`, `test1.scan` | Custom scan scripts |
| `test1.icc`, `test3.icc` | ICC color profiles |

- **Source**: https://github.com/mozilla/mozjpeg
- **License**: IJG License + Modified BSD License

---

### imageflow

**Imageflow test inputs** — Images used by [imageflow](https://github.com/imazen/imageflow) for testing format conversion, orientation handling, and edge cases.

| Folder | Files | Content |
|--------|-------|---------|
| `imageflow/test_inputs/` | 29 | WebP, JPEG, PNG, GIF test images |
| `imageflow/test_inputs/orientation/` | 16 | EXIF orientation test set (all 8 orientations × landscape/portrait) |

Includes: corrupt JPEG, color profile edge cases, transparency, gradients, whitespace handling, high-resolution (5760×4320) test image.

- **Source**: https://github.com/imazen/imageflow
- **License**: Various (per image)

---

## Directory Structure

```
codec-corpus/
├── clic2025/                    # CLIC 2025 (Unsplash License)
│   ├── LICENSE
│   ├── README.md                # Original naming & download links
│   ├── training/                # 32 high-res images (CLIC calls this "validation")
│   └── final-test/              # 30 high-res images (CLIC calls this "test")
├── CID22/                       # Cloudinary CID22 (CC BY-SA 4.0)
│   ├── LICENSE
│   └── CID22-512/
│       ├── validation/          # 41 images
│       └── training/            # 209 images
├── gb82/                        # GB82 photographic (CC0)
│   ├── LICENSE
│   └── *.png                    # 25 images
├── gb82-sc/                     # GB82 screen content (CC0)
│   ├── LICENSE
│   └── *.png                    # 10 screenshots
├── qoi-benchmark/               # QOI Benchmark Suite (CC0/PD/Mixed)
│   ├── README.md
│   ├── download.sh              # Fetch additional subsets (bash)
│   ├── download.ps1             # Fetch additional subsets (PowerShell)
│   └── screenshot_web/          # 14 web page screenshots (CC0)
├── kadid10k/                    # KADID-10k reference images (Pixabay License)
│   ├── LICENSE
│   └── *.png                    # 81 pristine images for IQA research
├── jxl/                         # JPEG XL tests (BSD-3-Clause)
│   ├── LICENSE
│   ├── conformance/             # 39 conformance tests
│   ├── features/                # 128 feature tests
│   └── edge-cases/              # 13 boundary tests
├── jpeg-conformance/            # JPEG conformance (MIT/IJG+BSD/Various)
│   ├── README.md
│   ├── SOURCES.md               # Per-file attribution
│   ├── valid/                   # 41 files — MUST decode
│   ├── invalid/                 # 116 files — MUST reject
│   ├── non-conformant/          # 40 files — MAY reject (with .txt descriptions)
│   └── crash-repro/             # 78 crash reproducers from upstream decoders
├── pngsuite/                    # PNG conformance (Freeware)
│   ├── LICENSE
│   └── *.png                    # 176 images
├── image-rs/                    # image-rs tests (MIT)
│   ├── LICENSE-MIT
│   └── test-images/
│       ├── bmp/, gif/, ico/
│       ├── jpg/, png/
│       ├── tiff/, webp/
├── zune/                        # zune-image tests (MIT/Apache-2.0/Zlib)
│   ├── LICENSE-MIT, LICENSE-APACHE, LICENSE-ZLIB
│   ├── test-images/jpeg/
│   └── fuzz-corpus/
│       ├── jpeg/, png/, inflate/
├── mozjpeg/                     # mozjpeg tests (IJG + BSD)
│   ├── LICENSE
│   └── *.ppm, *.jpg, *.bmp, *.icc
└── imageflow/                   # imageflow tests (Various)
    └── test_inputs/
        ├── orientation/         # 16 EXIF orientation tests
        └── *.jpg, *.png, *.webp, *.gif
```

---

## License Summary

Every dataset includes its own license file in its directory.

| Dataset | License | Commercial Use | Attribution Required | ShareAlike |
|---------|---------|:--------------:|:--------------------:|:----------:|
| CLIC 2025 | Unsplash License | Yes | No | No |
| CID22 | CC BY-SA 4.0 | Yes | **Yes** | **Yes** |
| GB82 | CC0 1.0 | Yes | No | No |
| GB82-SC | CC0 1.0 | Yes | No | No |
| QOI `screenshot_web` | CC0 1.0 | Yes | No | No |
| QOI `icon_*` | Public Domain | Yes | No | No |
| QOI `screenshot_game` | CC BY-SA 3.0 | Yes | **Yes** | **Yes** |
| QOI `pngimg` | CC BY-NC 4.0 | **No** | **Yes** | No |
| KADID-10k | Pixabay License | Yes | No | No |
| JXL | BSD-3-Clause | Yes | No | No |
| PNGSuite | Freeware | Yes | No | No |
| image-rs | MIT | Yes | No | No |
| zune-image | MIT/Apache-2.0/Zlib | Yes | No | No |
| mozjpeg | IJG + BSD | Yes | No | No |
| JPEG Conformance | MIT/IJG+BSD/Various | Yes | Varies | No |
| imageflow | Various | Yes | Varies | No |

---

## Choosing a Dataset

**For lossy codec quality calibration:**
Use [CLIC 2025](#clic-2025) (high-res, modern photos) and [CID22](#cid22) (diverse content, training/validation split). These are the most representative and methodologically sound choices for modern codec evaluation.

**For compact benchmarking:**
Use [GB82](#gb82) (25 challenging photos, CC0, 576×576). Fast to process, explicitly designed to resist metric gaming.

**For screen content / non-photographic images:**
Use [GB82-SC](#gb82-sc) and [QOI Benchmark `screenshot_web`](#qoi-benchmark-suite). Screenshots, UI elements, text, and graphics compress very differently from photos — testing both content types is essential for any codec deployed on the web.

**For decoder conformance:**
Use the format-specific test suites: [JPEG Conformance](#jpeg-conformance), [JXL](#jxl), [PNGSuite](#pngsuite). For fuzz/robustness testing, use [zune-image](#zune-image).

---

### A note on Kodak

The Kodak Lossless True Color Image Suite (24 images, 768×512) is not included in this corpus. It was the de facto image compression benchmark from the 1990s through the 2010s, but decades of codec tuning against the same 24 images have made Kodak scores nearly meaningless — many codecs are specifically optimized for it. The images are also too small for modern evaluation, too homogeneous (pastoral outdoor scenes circa 1990), and offer no train/test split. Use [CID22](#cid22) or [CLIC 2025](#clic-2025) instead.

---

## Color Space Normalization

All benchmark PNG images in this corpus use the standard `sRGB` PNG chunk to signal sRGB color space. As of February 2025, the following corrections were applied:

- **clic2025-1024/**, **clic2025/training/**, **imageflow/test_inputs/frymire.png**: Stripped `gAMA`+`cHRM` chunks (which encoded sRGB gamma and primaries the old way) and replaced with a proper `sRGB` chunk.
- **CID22/**: Stripped embedded `iCCP` profiles (which were sRGB IEC61966-2.1 profiles under generic names like "icc", ~2.6 KB each) and replaced with a proper `sRGB` chunk.
- **CID22/CID22-512/training/2387532.png**: Had `gAMA`-only (no chromaticities); renamed to `2387532-srgb.png` after normalization.
- **imageflow/test_inputs/gradients.png**: Stripped `iCCP` (Photoshop-named sRGB profile) and replaced with `sRGB` chunk.

No pixel data was changed — only PNG chunk metadata was modified. This eliminates inconsistent color space signaling that can cause subtle differences in color-managed decoders and metric calculations.

Datasets **not** modified: `gb82/`, `gb82-sc/`, `kadid10k/`, `qoi-benchmark/`, `clic2025/final-test/`, `webp-conformance/` (already clean — either proper `sRGB` chunks or no color metadata). Test/conformance directories (`pngsuite/`, `zune/`, `image-rs/`) were intentionally left untouched.

---

## Contributing

To suggest additional datasets, please open an issue with:
- Source URL
- License information
- Description of what the dataset tests or what content type it represents
