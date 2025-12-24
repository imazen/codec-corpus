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
| PNG | `libpng` or `png` crate | Use EXPAND transformations |
| JPEG | `libjpeg-turbo` | Standard baseline |
| WebP | `libwebp` | Official reference |
| GIF | `giflib` | Frame timing may vary |

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

## See Also

- [PNGSuite Documentation](http://www.schaik.com/pngsuite/)
- [zune-image Repository](https://github.com/etemesi254/zune-image)
- [image-rs Repository](https://github.com/image-rs/image)
