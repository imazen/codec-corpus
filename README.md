# Codec Corpus

Reference image corpus for codec quality calibration testing.

## Structure

```
CID22/
  LICENSE           # CC BY-SA 4.0
  CID22-512/
    validation/     # 41 images - held out for validation
    training/       # 209 images - used for calibration
```

## CID22

A subset of the CID22 (Compression Image Dataset 2022) resized to 512px on the longest edge.

- **Source**: https://github.com/Cloudinary/CID22
- **License**: CC BY-SA 4.0
- **CID22-512/validation/**: Images from the official CID22 validation set
- **CID22-512/training/**: Remaining images for model training/calibration

## Planned Additions

- **PNGSuite**: Standard PNG test images

## Usage

This corpus is fetched during optional Rust tests in [imageflow](https://github.com/imazen/imageflow) for codec quality calibration.

```bash
# Clone for testing
git clone https://github.com/imazen/codec-corpus.git
```
