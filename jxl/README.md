# JPEG XL Test Suite

Comprehensive test images for JPEG XL decoder conformance and quality testing.

## Sources

- **libjxl**: https://github.com/libjxl/libjxl (BSD-3-Clause)
- **JPEG XL Conformance**: https://github.com/libjxl/conformance (BSD-3-Clause)
- **jxl-rs**: https://github.com/libjxl/jxl-rs (BSD-3-Clause)

## Directory Structure

```
jxl/
├── conformance/     # Official JPEG XL conformance test images
├── features/        # Feature-specific tests (HDR, orientation, etc.)
├── photographic/    # Real-world photographs
└── edge-cases/      # Minimal and edge case tests
```

## Conformance Test Images (40 files)

Official JPEG XL conformance test suite covering:

| Category | Files | Features |
|----------|-------|----------|
| Animation | 5 | Multi-frame, splines, variable timing |
| Alpha | 8 | Premultiplied, non-premultiplied, blendmodes |
| HDR | 2 | 32-bit float, PQ transfer function |
| Color spaces | 5 | Grayscale, CMYK, opsin inverse |
| Photographic | 6 | Lossy, lossless, progressive |
| Technical | 14 | Patches, upsampling, noise, delta palette |

### Key Conformance Files

| File | Features |
|------|----------|
| `animation_icos4d.jxl` | 50-frame animation, alpha, lossy |
| `animation_spline.jxl` | Animated splines, lossless, alpha |
| `lossless_pfm.jxl` | 32-bit float lossless, linear color |
| `cmyk_layers.jxl` | CMYK color space, extra channels |
| `progressive.jxl` | Progressive encoding, alpha |
| `alpha_premultiplied.jxl` | Premultiplied alpha, high precision |

## Feature Test Images (26 files)

### HDR & Color Science
| File | Features |
|------|----------|
| `hdr_pq_test.jxl` | PQ (Perceptual Quantizer) HDR |
| `hdr_hlg_test.jxl` | HLG (Hybrid Log-Gamma) HDR |
| `with_icc.jxl` | Embedded ICC color profile |

### Orientation (EXIF)
| File | Transform |
|------|-----------|
| `orientation1_identity.jxl` | No transform |
| `orientation2_flip_horizontal.jxl` | Horizontal flip |
| `orientation3_rotate_180.jxl` | 180° rotation |
| `orientation4_flip_vertical.jxl` | Vertical flip |
| `orientation5_transpose.jxl` | Transpose |
| `orientation6_rotate_90_cw.jxl` | 90° clockwise |
| `orientation7_anti_transpose.jxl` | Anti-transpose |
| `orientation8_rotate_90_ccw.jxl` | 90° counter-clockwise |

### Encoding Methods
| File | Features |
|------|----------|
| `green_queen_modular_e3.jxl` | Modular (lossless-capable) |
| `green_queen_vardct_e3.jxl` | VarDCT (lossy) |
| `grayscale_patches_modular.jxl` | Modular grayscale |
| `grayscale_patches_var_dct.jxl` | VarDCT grayscale |

### Special Features
| File | Features |
|------|----------|
| `splines.jxl` | Spline primitives |
| `multiple_layers_noise_spline.jxl` | Multi-layer with noise + spline |
| `extra_channels.jxl` | Extra channel support |
| `squeeze_alpha.jxl` | Alpha squeezing |
| `3x3_jpeg_recompression.jxl` | JPEG reconstruction |
| `has_permutation_with_container.jxl` | Container with permutation |

## Photographic Images (4 files)

Real-world photographs for quality evaluation:

| File | Size | Description |
|------|------|-------------|
| `zoltan_tasi_unsplash.jxl` | 411 KB | Landscape (Unsplash license) |
| `candle.jxl` | 154 KB | ILM OpenEXR test image |
| `dice.jxl` | 46 KB | Geometric objects |
| `efb.jxl` | 20 KB | Extended frame buffer |

## Edge Case Images (13 files)

Minimal and boundary condition tests:

| File | Features |
|------|----------|
| `basic.jxl` | 65 bytes, minimal valid JXL |
| `3x3_srgb_lossless.jxl` | 3×3 pixels, lossless |
| `3x3_srgb_lossy.jxl` | 3×3 pixels, lossy |
| `tree_max_property_20.jxl` | Tree property edge case |
| `squeeze_edge.jxl` | Squeeze edge conditions |
| `squeeze_empty_residual.jxl` | Empty residual handling |
| `patch_y_out_of_bounds.jxl` | Patch boundary test |
| `oddsize_ups.jxl` | Odd dimension upsampling |
| `multiple_lf_420.jxl` | Multiple LF with 4:2:0 |
| `cropped_traffic_light.jxl` | Cropped image handling |
| `large_header.jxl` | Large container metadata |
| `has_permutation.jxl` | Permutation support |

## Feature Coverage Matrix

| Feature | Conformance | Features | Edge |
|---------|:-----------:|:--------:|:----:|
| Lossless | ✓ | ✓ | ✓ |
| Lossy (VarDCT) | ✓ | ✓ | ✓ |
| Animation | ✓ | ✓ | |
| Alpha channel | ✓ | ✓ | |
| HDR (PQ/HLG) | ✓ | ✓ | |
| 32-bit float | ✓ | | |
| Grayscale | ✓ | ✓ | |
| CMYK | ✓ | | |
| ICC profiles | | ✓ | |
| Orientation | | ✓ | |
| Splines | ✓ | ✓ | |
| Patches | ✓ | | ✓ |
| Progressive | ✓ | | |
| Container format | | ✓ | ✓ |
| JPEG reconstruction | | ✓ | |

## Usage for Parity Testing

These images are designed for exact parity testing between JXL decoder implementations.
Reference outputs should be generated using libjxl (the reference implementation).

```bash
# Generate reference output with libjxl
djxl input.jxl reference_output.ppm

# Compare decoder output with reference
# Exact byte-match required for lossless
# Conformance threshold for lossy (documented per-image)
```

## Total Statistics

- **Files**: 83 JXL images
- **Size**: ~7.2 MB total
- **Categories**: 4 (conformance, features, photographic, edge-cases)
- **Animation frames**: 77+ frames across 5 animated images
- **Bit depths**: 8-bit, 12-bit, 16-bit, 32-bit float
- **Color spaces**: sRGB, linear, grayscale, CMYK, opsin
