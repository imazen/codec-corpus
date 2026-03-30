# Expected Results for Codec Corpus

This document specifies the expected behavior when decoding test files in this corpus. Files are categorized by expected outcome: **valid** (should decode), **error** (should fail gracefully), or **edge-case** (decoder-dependent).

## PNGSuite Conformance Tests

### Valid Files (Should Decode Successfully)

All files in `pngsuite/` without the `x` prefix are valid PNG files that should decode correctly. The file naming convention is:

```
[test][param][interlace][colortype][bitdepth].png
```

Where:
- **test**: Test category (bas=basic, f=filter, g=gamma, etc.)
- **param**: Parameter value within category
- **interlace**: `n` (non-interlaced) or `i` (interlaced/Adam7)
- **colortype**: `0g` (grayscale), `2c` (RGB), `3p` (palette), `4a` (gray+alpha), `6a` (RGBA)
- **bitdepth**: `01`, `02`, `04`, `08`, or `16`

### Invalid Files (Should Return Error)

Files with the `x` prefix are intentionally corrupted and MUST return an error. Decoders should NOT crash, panic, or hang.

| File | Corruption | Expected Error Type |
|------|------------|---------------------|
| `xc1n0g08.png` | Invalid color type (1) | `InvalidColorType` or `FormatError` |
| `xc9n2c08.png` | Invalid color type (9) | `InvalidColorType` or `FormatError` |
| `xcrn0g04.png` | CR bytes inserted (ASCII transfer corruption) | `InvalidData` or `ChecksumError` |
| `xcsn0g01.png` | Incorrect IDAT checksum | `ChecksumError` or `CrcError` |
| `xd0n2c08.png` | Invalid bit depth (0) | `InvalidBitDepth` or `FormatError` |
| `xd3n2c08.png` | Invalid bit depth (3) | `InvalidBitDepth` or `FormatError` |
| `xd9n2c08.png` | Invalid bit depth (9) | `InvalidBitDepth` or `FormatError` |
| `xdtn0g01.png` | Missing IDAT chunk | `MissingChunk` or `UnexpectedEof` |
| `xhdn0g08.png` | Incorrect IHDR checksum | `ChecksumError` or `CrcError` |
| `xlfn0g04.png` | LF bytes inserted (ASCII transfer corruption) | `InvalidData` or `ChecksumError` |
| `xs1n0g01.png` | Invalid dimensions (signature?) | `InvalidSignature` or `FormatError` |
| `xs2n0g01.png` | Invalid dimensions (signature?) | `InvalidSignature` or `FormatError` |
| `xs4n0g01.png` | Invalid dimensions (signature?) | `InvalidSignature` or `FormatError` |
| `xs7n0g01.png` | Invalid dimensions (signature?) | `InvalidSignature` or `FormatError` |

**Reference**: http://www.schaik.com/pngsuite/

---

## Zune Fuzz Corpus

### Purpose

Files in `zune/fuzz-corpus/` are minimized test cases from fuzzing that previously caused crashes or hangs. All files in this directory:

1. **MUST NOT crash** the decoder
2. **MUST NOT hang** indefinitely
3. **MAY return an error** (most will)
4. **MAY decode successfully** (some are valid edge cases)

### Directory Structure

| Directory | Format | Count | Purpose |
|-----------|--------|-------|---------|
| `fuzz-corpus/jpeg/` | JPEG | ~1,836 | Decoder robustness |
| `fuzz-corpus/png/` | PNG | ~800 | Decoder robustness |
| `fuzz-corpus/inflate/` | DEFLATE | ~700 | Decompression edge cases |

### Expected Behavior

For each file in the fuzz corpus:

```
decode(file) -> Result<Image, Error>
```

Both `Ok` and `Err` are acceptable outcomes. The test passes if:
- No panic/crash
- No infinite loop (timeout after reasonable duration)
- Memory usage stays bounded

### Known Error Categories

These error types are expected from fuzz corpus files:

**JPEG:**
- `UnexpectedEof` - Truncated file
- `InvalidMarker` - Unknown or misplaced marker
- `InvalidHuffman` - Corrupted Huffman table
- `InvalidQuantization` - Corrupted quantization table
- `BadDimensions` - Zero or excessive dimensions
- `BadSOS` - Corrupted scan header

**PNG:**
- `InvalidSignature` - Bad magic bytes
- `ChecksumError` - CRC mismatch
- `InvalidChunk` - Unknown critical chunk
- `DecompressionError` - DEFLATE failure
- `BadDimensions` - Zero or excessive dimensions

**DEFLATE:**
- `InvalidBlockType` - Unknown compression block
- `DistanceTooFar` - Invalid back-reference
- `InvalidCode` - Bad Huffman code
- `UnexpectedEof` - Truncated stream

---

## Image-rs Test Images

### Valid Files

Files in `image-rs/test-images/` are generally valid unless in a `bugfixes/` or similar subdirectory.

### Edge Cases by Format

**BMP (`test-images/bmp/`):**
- `*.bad_bmp` - Intentionally malformed, should error
- Files with large dimensions may trigger limits

**GIF (`test-images/gif/`):**
- Animation edge cases
- May have unusual frame counts

**PNG (`test-images/png/`):**
- `16bpc/` - 16-bit per channel (may not be supported by all decoders)
- `apng/` - Animated PNG (requires APNG support)
- `bugfixes/` - Regression test cases

**WebP (`test-images/webp/`):**
- Lossless and lossy variants
- Extended format features

---

## Reference Implementation Behavior

When verifying decoder correctness, compare against these reference implementations:

| Format | Reference | Notes |
|--------|-----------|-------|
| AVIF | `libavif` / `dav1d` | Official AOM reference |
| HEIC | `libheif` | Decode-only reference |
| JPEG | `libjpeg-turbo` | Standard baseline |
| JXL | `libjxl` | Official reference |
| PNG | `libpng` or `png` crate | Use EXPAND transformations |
| APNG | `libpng` (1.6.3+) | APNG patch or built-in |
| GIF | `giflib` | Frame timing may vary |
| WebP | `libwebp` | Official reference |
| TIFF | `libtiff` | Official reference |
| BMP | `image-rs` / system | Various implementations |
| PNM/PAM | `pnmtopng` / Netpbm | Netpbm reference tools |
| Farbfeld | `2ff` / `ff2png` | suckless reference tools |
| UltraHDR | `libultrahdr` | Google reference |

### Cross-Reference Testing

```rust
// Example: PNG conformance test
fn test_png_conformance(path: &Path) {
    let reference = decode_with_libpng(path);
    let our_result = decode_with_our_decoder(path);

    match (reference, our_result) {
        (Ok(ref_img), Ok(our_img)) => assert_pixels_match(ref_img, our_img),
        (Err(_), Err(_)) => (), // Both error - acceptable
        (Ok(_), Err(e)) => panic!("We errored but reference succeeded: {}", e),
        (Err(_), Ok(_)) => (), // We're more permissive - log warning
    }
}
```

---

## Error Severity Levels

| Level | Behavior | Acceptable? |
|-------|----------|-------------|
| **Crash/Panic** | Process terminates | NEVER acceptable |
| **Hang** | Infinite loop | NEVER acceptable |
| **OOM** | Unbounded allocation | NEVER acceptable |
| **Error** | Returns Err variant | Always acceptable for invalid input |
| **Decode** | Returns valid image | Expected for valid input |

---

## Updating This Document

When adding new test files to the corpus:

1. Document the source of the file
2. Specify whether it should decode or error
3. If it should error, document the expected error type
4. Run the file through reference implementations to verify expected behavior

---

## Reference Image Verification Pattern

The image-rs project uses a CRC32-based reference image verification system that is worth adopting:

### Filename Format

```
{original_filename}.{crc32_hex}.png
```

Example: `basn2c08.png.7855b9bf.png`

- The decoded pixel data is hashed with CRC32
- This hash is embedded in the reference filename
- Tests can verify the decoded content matches regardless of PNG encoding differences

### For Animated Images

```
{original_filename}.anim_{frame_number}_{crc32_hex}.png
```

Example: `ball.png.anim_13_bf335902.png`

- Frame numbers are 1-based
- Each frame is verified independently

### Implementation

```rust
use crc32fast::Hasher as Crc32;

fn compute_image_crc(img: &DynamicImage) -> u32 {
    let mut hasher = Crc32::new();
    hasher.update(img.as_bytes());
    hasher.finalize()
}

fn verify_against_reference(test_img: &DynamicImage, expected_crc: u32) -> bool {
    compute_image_crc(test_img) == expected_crc
}
```

This approach allows:
- Byte-exact verification of decoded output
- Storage-efficient reference files (PNG compressed)
- Easy detection of decoder regressions
- Filename encodes expected result

---

## AVIF Conformance Tests

### Valid Files (Should Decode Successfully)

All 106 files in `avif-conformance/valid/` are valid AVIF files sourced from AOM test vectors (av1-avif, libavif, link-u). They cover all AV1 profiles, chroma formats, bit depths 8/10/12, alpha, HDR, animation, grids, gain maps, and metadata.

### Invalid Files (Should Return Error)

Files in `avif-conformance/invalid/` (12 files) are intentionally corrupted:

| File | Corruption | Expected Error |
|------|------------|----------------|
| `empty.avif` | 0 bytes | `UnexpectedEof` |
| `not_avif.avif` | JPEG data renamed | `InvalidSignature` or `FormatError` |
| `truncated_header.avif` | First 100 bytes only | `UnexpectedEof` |
| `truncated_data.avif` | Pixel data cut to 50% | `InvalidData` or `IncompleteFrame` |
| `bad_ftyp.avif` | Corrupted ftyp box | `InvalidBox` or `FormatError` |
| `wrong_brand.avif` | Major brand set to 'mp41' | `UnsupportedBrand` or decode attempt |
| `corrupted_mdat.avif` | Valid ISOBMFF, corrupted AV1 | `DecodeError` |
| `zero_dimensions.avif` | ispe box zeroed to 0×0 | `BadDimensions` |

### Edge Cases (19 files)

Decoder-dependent: unusual configurations, odd dimensions, extreme parameters. Both `Ok` and `Err` are acceptable.

---

## HEIC/HEIF Conformance Tests

### Valid Files (Should Decode Successfully)

- `valid/nokia-conformance/` (63 files) — Nokia's comprehensive HEIF test set: grids, overlays, thumbnails, alpha, EXIF, stereo, sequences. ⚠️ No license.
- `valid/dsoprea-exif/` (4 files) — HEIC files with rich EXIF metadata. MIT license.
- `valid/libheif-testdata/` (90 files) — libheif test data covering various HEIF containers. LGPL-3.0.

### Invalid Files (8 files)

Truncated ISOBMFF containers, corrupted HEVC NALUs, empty files. Decoders MUST NOT crash.

### Edge Cases (5 files)

Unusual container structures. Both `Ok` and `Err` are acceptable.

---

## APNG Conformance Tests

### Valid Files (22 files)

All files in `apng-conformance/valid/` are generated APNG files covering:
- Basic animations (2, 3, 10 frames)
- Disposal: `APNG_DISPOSE_OP_NONE` (0), `_BACKGROUND` (1), `_PREVIOUS` (2)
- Blend: `APNG_BLEND_OP_SOURCE` (0), `_OVER` (1)
- Frame offsets, variable timing, looping (0/1/3)
- Color types: RGBA, RGB, grayscale, palette
- Default image handling: first-frame-as-default vs separate fallback

### Invalid Files (6 files)

| File | Defect | Expected Error |
|------|--------|----------------|
| `missing_actl.png` | fcTL/fdAT without acTL | Decode as static PNG or error |
| `bad_sequence.png` | Duplicate sequence numbers | `InvalidSequence` or `FormatError` |
| `frame_out_of_bounds.png` | Frame exceeds canvas | `BadDimensions` or `FormatError` |
| `zero_delay_den.png` | delay_den=0 | Treat as 100 (per spec) or error |
| `truncated_fdat.png` | Incomplete frame data | `UnexpectedEof` |
| `no_fdat.png` | acTL claims 3 frames, only 1 | `MissingFrameData` |

### Edge Cases (3 files)

Zero delay, 50 frames, 1×1 pixel canvas. Both `Ok` and `Err` are acceptable.

---

## GIF Conformance Tests

### Valid Files (28 files)

All files in `gif-conformance/valid/` are generated GIF89a files covering:
- Static images (solid, palette, 256-color, interlaced)
- Animations (2/3/10 frames, fade effects)
- Disposal methods: none (1), background (2), previous (3), unspecified (0)
- Transparency (background, per-frame)
- Timing (0-delay, 10ms, 1s, variable)
- Looping (infinite/once/3/no extension)
- Color tables (global, local, mixed)
- Canvas/frame geometry (offsets, overlapping)

### Invalid Files (7 files)

| File | Defect | Expected Error |
|------|--------|----------------|
| `bad_magic.gif` | "GIF90a" magic | `InvalidSignature` |
| `truncated_header.gif` | Only 4 bytes | `UnexpectedEof` |
| `truncated_lzw.gif` | LZW stream cut short | `InvalidData` |
| `empty.gif` | 0 bytes | `UnexpectedEof` |
| `no_trailer.gif` | Missing 0x3B trailer | May decode or error |
| `bad_lzw_code.gif` | Invalid LZW code | `InvalidData` |
| `zero_dimensions.gif` | Width or height = 0 | `BadDimensions` |

### Edge Cases (4 files)

GIF87a format, comment extension, plain text extension, large sparse palette. Decoder support varies.

---

## PNM/PAM Conformance Tests

### Valid Files (35 files)

All files in `pnm-conformance/valid/` subdivided by format:
- `pbm/` (10 files): P1 ASCII + P4 binary — checkerboard, 1×1, wide, tall
- `pgm/` (8 files): P2 ASCII + P5 binary — gradient 8/16-bit, maxval 1/100
- `ppm/` (11 files): P3 ASCII + P6 binary — color bars, primaries, 16-bit, comments
- `pam/` (6 files): P7 — GRAYSCALE, GRAYSCALE_ALPHA, RGB, RGB_ALPHA, 16-bit variants

### Invalid Files (10 files)

| File | Defect | Expected Error |
|------|--------|----------------|
| `bad_magic.ppm` | "P9" magic | `InvalidMagic` |
| `negative_width.pgm` | Width = -1 | `BadDimensions` |
| `zero_width.pgm` | Width = 0 | `BadDimensions` |
| `zero_height.ppm` | Height = 0 | `BadDimensions` |
| `missing_maxval.pgm` | P5 without maxval | `FormatError` |
| `truncated_data.ppm` | P6 with half the pixel data | `UnexpectedEof` |
| `maxval_zero.pgm` | Maxval = 0 | `InvalidMaxval` |
| `maxval_too_large.pgm` | Maxval = 70000 | `InvalidMaxval` |
| `bad_ascii_data.ppm` | "abc" in pixel data | `InvalidData` |
| `overflow_dimensions.pgm` | Width × height overflows u32 | `Overflow` |

### Edge Cases (6 files)

Extra whitespace, maxval=65535, single pixel, 1000-char comment, concatenated multi-image stream, CRLF line endings.

---

## Farbfeld Conformance Tests

### Valid Files (12 files)

All files in `farbfeld-conformance/valid/` are generated farbfeld images:
- Single pixels (black, white, red, transparent)
- Small tiles (4×4, 8×8)
- Gradients, checkerboards, color swatches
- Extreme aspect ratios (100×1, 1×100)

### Invalid Files (9 files)

| File | Defect | Expected Error |
|------|--------|----------------|
| `bad_magic.ff` | "farbfool" magic | `InvalidMagic` |
| `empty.ff` | 0 bytes | `UnexpectedEof` |
| `header_only.ff` | 4×4 header, no pixels | `UnexpectedEof` |
| `truncated_header.ff` | Only 10 bytes | `UnexpectedEof` |
| `truncated_pixels.ff` | Half the expected pixel data | `UnexpectedEof` |
| `extra_data.ff` | Valid 1×1 + 100 extra bytes | May decode (ignore trailing) or error |
| `zero_width.ff` | Width = 0 | `BadDimensions` |
| `zero_height.ff` | Height = 0 | `BadDimensions` |
| `zero_both.ff` | Width = 0, height = 0 | `BadDimensions` |

### Edge Cases (3 files)

Large single-row (1000×1), 64×64 square, grayscale-in-RGB (R=G=B).

---

## UltraHDR / Gain Map Conformance Tests

### Valid Files

- `valid/jpeg/` (43 files): UltraHDR JPEG-R files with embedded gain maps. Sources include Google libultrahdr benchmarks (Apache-2.0), Pixel phone samples (CC-BY-4.0), and Awesome-Gain-Maps collection (MIT).
- `valid/avif/` and `valid/jxl/`: Empty — no freely-licensed gain map samples found for these formats yet.

All valid JPEG files should decode as standard SDR JPEGs. Gain-map-aware decoders should additionally extract the gain map and produce HDR output.

### Invalid Files (5 files)

Corrupted XMP, missing gain map data, truncated MPF segments. Decoders MUST NOT crash; may fall back to SDR decode.

### Edge Cases (3 files)

Unusual gain map dimensions, edge-case transfer functions. Decoder-dependent behavior.

---

## RAW/DNG Conformance Tests

### Valid Files

Files in `raw-conformance/valid/` are CC0-licensed camera samples from raw.pixls.us, stored with Git LFS. Organized by sensor type:
- `bayer/` — Standard Bayer CFA: Canon CR2, Nikon NEF, Sony ARW, DNG
- `xtrans/` — X-Trans CFA: Fuji RAF
- `linear/` — Demosaiced linear DNG
- `compressed/` — Lossy DNG (type 34892), JXL-compressed DNG (type 52546)
- `mobile/` — Apple ProRAW (APPLEDNG), Android DNG
- `proprietary/` — ORF, RW2, CR3, and other vendor formats

Decoders should produce scene-referred linear RGB output. Exact pixel values depend on the demosaic algorithm and color pipeline, but dimensions and CFA pattern should match the file metadata.

### Invalid Files

| File | Defect | Expected Error |
|------|--------|----------------|
| `truncated_header.raw` | First 100 bytes only | `UnexpectedEof` |
| `truncated_data.raw` | Pixel data cut to 50% | `UnexpectedEof` or `IncompleteData` |
| `bad_byte_order.raw` | Corrupted TIFF byte order marker | `InvalidHeader` |
| `empty.raw` | 0 bytes | `UnexpectedEof` |
| `not_tiff.raw` | JPEG header (FF D8 FF E0) | `InvalidSignature` |

### Edge Cases

Tiled DNG, multi-page DNG, unusual bit depths (12-bit packed, 14-bit). Decoder-dependent behavior.

---

## See Also

- [APNG Specification](https://wiki.mozilla.org/APNG_Specification)
- [Farbfeld Specification](https://tools.suckless.org/farbfeld/)
- [Netpbm Format](https://netpbm.sourceforge.net/doc/)
- [PNGSuite Documentation](http://www.schaik.com/pngsuite/)
- [zune-image Repository](https://github.com/etemesi254/zune-image)
- [image-rs Repository](https://github.com/image-rs/image)
