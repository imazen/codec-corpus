# Codec Corpus

Reference image corpus for codec quality calibration testing.

## Structure

```
CID22/
  LICENSE           # CC BY-SA 4.0
  CID22-512/
    validation/     # 41 images - held out for validation
    training/       # 209 images - used for calibration

zune/
  LICENSE-MIT       # MIT OR Apache-2.0 OR Zlib
  LICENSE-APACHE
  LICENSE-ZLIB
  test-images/
    jpeg/           # 31 JPEG edge case images + benchmarks
  fuzz-corpus/
    jpeg/           # 1836 minimal fuzz test cases
    png/            # PNG fuzz test cases
    inflate/        # Compression fuzz test cases

image-rs/
  LICENSE-MIT       # MIT
  test-images/
    bmp/            # BMP format variants + malformed files
    gif/            # GIF animation tests
    ico/            # Icon format tests
    jpg/            # JPEG metadata/progressive tests
    png/            # PNG 16bpc, APNG, transparency (PNGsuite subset)
    tiff/           # TIFF compression/predictor tests
    webp/           # WebP lossless/lossy variants

pngsuite/
  LICENSE           # Freeware (Willem van Schaik)
  *.png             # 176 PNG conformance test images

kodak/
  LICENSE           # Unrestricted (Eastman Kodak)
  *.png             # 24 classic test images (768x512)

mozjpeg/
  LICENSE           # IJG + BSD
  *.ppm, *.bmp, *.jpg, *.icc  # JPEG codec test files

clic2025/
  LICENSE           # Unsplash License
  *.png             # 32 high-resolution validation images
```

## CID22

A subset of the CID22 (Compression Image Dataset 2022) resized to 512px on the longest edge.

- **Source**: https://github.com/Cloudinary/CID22
- **License**: CC BY-SA 4.0
- **CID22-512/validation/**: Images from the official CID22 validation set
- **CID22-512/training/**: Remaining images for model training/calibration

## zune-image

Test images and fuzz corpus from the zune-image project for decoder robustness testing.

- **Source**: https://github.com/etemesi254/zune-image
- **License**: MIT OR Apache-2.0 OR Zlib
- **test-images/jpeg/**: JPEG edge cases (CMYK, progressive, unusual sampling factors, large dimensions)
- **fuzz-corpus/**: Minimal test cases for crash/hang detection

## image-rs

Test images from the Rust image library for format conformance testing.

- **Source**: https://github.com/image-rs/image
- **License**: MIT
- **test-images/**: Format edge cases, bit depths, compression modes, malformed files

## PNGSuite

The official PNG conformance test suite by Willem van Schaik.

- **Source**: http://www.schaik.com/pngsuite/
- **License**: Freeware (free to use, copy, modify, distribute)
- **Files**: 176 test images covering all PNG features (bit depths, color types, interlacing, transparency, gamma, etc.)

## Kodak

The classic Kodak Lossless True Color Image Suite, widely used as a standard benchmark for image compression research.

- **Source**: http://r0k.us/graphics/kodak/
- **License**: Unrestricted usage (released by Eastman Kodak Company)
- **Files**: 24 uncompressed 768x512 PNG images

## mozjpeg

Test images from the mozjpeg project for JPEG encoder/decoder testing.

- **Source**: https://github.com/mozilla/mozjpeg
- **License**: IJG License + Modified BSD License
- **Files**: PPM, BMP, JPG test images + ICC color profiles

## CLIC 2025

Validation images from the Challenge on Learned Image Compression (CLIC) 2025.

- **Source**: https://clic2025.compression.cc/
- **License**: Unsplash License (free for any use, no attribution required)
- **Files**: 32 high-resolution PNG images (~103 MB)

## Usage

This corpus is fetched during optional Rust tests in [imageflow](https://github.com/imazen/imageflow) for codec quality calibration.

```bash
# Clone everything
git clone https://github.com/imazen/codec-corpus.git
```

### Download a single dataset

Use sparse checkout to download only the dataset you need:

```bash
git clone --depth 1 --filter=blob:none --sparse \
  https://github.com/imazen/codec-corpus.git
cd codec-corpus
git sparse-checkout set kodak        # or: pngsuite, CID22, mozjpeg, etc.
```

To add more datasets later:
```bash
git sparse-checkout add pngsuite
```
