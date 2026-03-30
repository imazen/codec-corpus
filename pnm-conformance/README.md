# PNM/PAM Conformance Test Suite

Test files for validating PNM (Portable Any Map) and PAM (Portable Arbitrary Map) decoders and encoders.

**PNM** is a family of simple image formats from the Netpbm project:
- **PBM** (Portable Bitmap) — 1-bit black and white (P1 ASCII, P4 binary)
- **PGM** (Portable Graymap) — grayscale with configurable bit depth (P2 ASCII, P5 binary)
- **PPM** (Portable Pixmap) — RGB color with configurable bit depth (P3 ASCII, P6 binary)
- **PAM** (Portable Arbitrary Map) — generalized format supporting arbitrary channel counts and tuple types (P7, binary only)

All formats use a simple header followed by pixel data. ASCII variants store pixel values as decimal text; binary variants store raw bytes (big-endian for 16-bit).

**Specification:** <https://netpbm.sourceforge.net/doc/>

## License

Generated test data, public domain / CC0.

## Regenerating

```bash
python3 generate.py
```

The script is idempotent. Running it twice produces identical output.

## File Inventory

### valid/pbm/ — PBM (Portable Bitmap)

| File | Format | Description |
|------|--------|-------------|
| `checkerboard_8x8_ascii.pbm` | P1 | 8x8 checkerboard pattern, ASCII |
| `checkerboard_8x8_binary.pbm` | P4 | 8x8 checkerboard pattern, binary |
| `1x1_black_ascii.pbm` | P1 | Single black pixel (value 1), ASCII |
| `1x1_black_binary.pbm` | P4 | Single black pixel (value 1), binary |
| `1x1_white_ascii.pbm` | P1 | Single white pixel (value 0), ASCII |
| `1x1_white_binary.pbm` | P4 | Single white pixel (value 0), binary |
| `wide_100x1_ascii.pbm` | P1 | 100x1 alternating pixels, ASCII |
| `wide_100x1_binary.pbm` | P4 | 100x1 alternating pixels, binary |
| `tall_1x100_ascii.pbm` | P1 | 1x100 alternating pixels, ASCII |
| `tall_1x100_binary.pbm` | P4 | 1x100 alternating pixels, binary |

### valid/pgm/ — PGM (Portable Graymap)

| File | Format | Description |
|------|--------|-------------|
| `gradient_8x8_255_ascii.pgm` | P2 | 8x8 gradient, maxval 255, ASCII |
| `gradient_8x8_255_binary.pgm` | P5 | 8x8 gradient, maxval 255, binary |
| `gradient_8x8_65535_ascii.pgm` | P2 | 8x8 gradient, maxval 65535 (16-bit), ASCII |
| `gradient_8x8_65535_binary.pgm` | P5 | 8x8 gradient, maxval 65535 (16-bit), binary |
| `4x4_maxval1_ascii.pgm` | P2 | 4x4 checkerboard, maxval 1 (minimal), ASCII |
| `4x4_maxval1_binary.pgm` | P5 | 4x4 checkerboard, maxval 1 (minimal), binary |
| `4x4_maxval100_ascii.pgm` | P2 | 4x4 gradient, maxval 100 (non-power-of-two), ASCII |
| `4x4_maxval100_binary.pgm` | P5 | 4x4 gradient, maxval 100 (non-power-of-two), binary |

### valid/ppm/ — PPM (Portable Pixmap)

| File | Format | Description |
|------|--------|-------------|
| `colorbars_4x4_ascii.ppm` | P3 | 4x4 color bars (R/G/B/W columns), ASCII |
| `colorbars_4x4_binary.ppm` | P6 | 4x4 color bars (R/G/B/W columns), binary |
| `primaries_2x2_ascii.ppm` | P3 | 2x2 primary colors (R/G/B/black), ASCII |
| `primaries_2x2_binary.ppm` | P6 | 2x2 primary colors (R/G/B/black), binary |
| `colorbars_4x4_16bit_ascii.ppm` | P3 | 4x4 color bars, maxval 65535 (16-bit), ASCII |
| `colorbars_4x4_16bit_binary.ppm` | P6 | 4x4 color bars, maxval 65535 (16-bit), binary |
| `gradient_8x8_ascii.ppm` | P3 | 8x8 RGB gradient, ASCII |
| `gradient_8x8_binary.ppm` | P6 | 8x8 RGB gradient, binary |
| `comment_after_magic.ppm` | P3 | Comment line immediately after magic number |
| `comment_between_dims.ppm` | P3 | Comment line between width and height |
| `multiple_comments.ppm` | P3 | Multiple comment lines throughout header |

### valid/pam/ — PAM (Portable Arbitrary Map)

| File | Format | Description |
|------|--------|-------------|
| `grayscale_4x4.pam` | P7 | GRAYSCALE, 1 channel, maxval 255 |
| `grayscale_alpha_4x4.pam` | P7 | GRAYSCALE_ALPHA, 2 channels, maxval 255 |
| `rgb_4x4.pam` | P7 | RGB, 3 channels, maxval 255 |
| `rgb_alpha_4x4.pam` | P7 | RGB_ALPHA, 4 channels, maxval 255 |
| `grayscale_16bit_4x4.pam` | P7 | GRAYSCALE, 1 channel, maxval 65535 (16-bit) |
| `rgb_16bit_4x4.pam` | P7 | RGB, 3 channels, maxval 65535 (16-bit) |

### invalid/ — Files That Must Cause Decode Errors

| File | Description | Expected behavior |
|------|-------------|-------------------|
| `bad_magic.ppm` | Magic number "P9" (undefined) | Reject: unknown format |
| `negative_width.pgm` | Width = -1 | Reject: invalid dimensions |
| `zero_width.pgm` | Width = 0 | Reject: zero dimension |
| `zero_height.ppm` | Height = 0 | Reject: zero dimension |
| `missing_maxval.pgm` | P5 header without maxval line | Reject: malformed header |
| `truncated_data.ppm` | P6 4x4 with only half the pixel data | Reject: unexpected EOF |
| `maxval_zero.pgm` | Maxval = 0 | Reject: invalid maxval |
| `maxval_too_large.pgm` | Maxval = 70000 (exceeds 65535 limit) | Reject: maxval out of range |
| `bad_ascii_data.ppm` | P3 with "abc" instead of numbers | Reject: non-numeric pixel data |
| `overflow_dimensions.pgm` | 65536x65536 (overflows u32 multiply) | Reject: dimension overflow |

### edge-cases/ — Unusual But Technically Valid

| File | Description | Expected behavior |
|------|-------------|-------------------|
| `extra_whitespace.ppm` | P3 with tabs and multiple spaces | Decode normally (whitespace is whitespace) |
| `max_maxval.pgm` | P5 with maxval 65535 (maximum allowed) | Decode normally, 16-bit samples |
| `single_pixel_rgb.ppm` | 1x1 P6 image | Decode normally |
| `large_comment.pgm` | P5 with a 1000-character comment line | Decode normally (comment is ignored) |
| `concatenated.ppm` | Two valid P6 images concatenated | Decode at least the first image |
| `crlf_lineendings.ppm` | P3 with \r\n line endings | Decode normally (\r is whitespace) |
