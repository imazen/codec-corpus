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

## Planned Additions

- **PNGSuite**: Standard PNG test images

## Usage

This corpus is fetched during optional Rust tests in [imageflow](https://github.com/imazen/imageflow) for codec quality calibration.

```bash
# Clone for testing
git clone https://github.com/imazen/codec-corpus.git
```
