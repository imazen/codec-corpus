# CLIC 2025

**Challenge on Learned Image Compression 2025** — High-resolution photographic images for compression quality research.

## Folders

| Folder | Images | Size | Purpose |
|--------|--------|------|---------|
| `training/` | 32 | 103 MB | Tune and calibrate encoders against these images |
| `final-test/` | 30 | 116 MB | Holdout set — use for final evaluation only |

## Original Naming

The CLIC 2025 challenge uses different names for these sets. Our folder names differ from the originals to better reflect their purpose in codec development:

| Our Name | Original CLIC Name | Original Filename |
|----------|--------------------|--------------------|
| `training/` | "Validation set" | `clic2025_image_validation.zip` |
| `final-test/` | "Test set" | `clic2025_image_test.zip` |

In the CLIC challenge context, "validation" means the set participants use during development (to tune and iterate on), and "test" means the final holdout evaluation. We renamed "validation" to "training" because in codec development, "validation" typically implies a holdout set, which is the opposite of its intended use here.

## Details

- **Resolution**: ~2048px on longest edge (varies)
- **Format**: Lossless PNG, 8-bit sRGB
- **Source**: https://clic2025.compression.cc/
- **Downloads**:
  - Validation (our `training/`): https://d152a8jkvz9wzs.cloudfront.net/data/clic2025_image_validation.zip
  - Test (our `final-test/`): https://storage.googleapis.com/clic_datasets/clic2025_image_test.zip
- **License**: [Unsplash License](https://unsplash.com/license) — Free for any use, no attribution required.
