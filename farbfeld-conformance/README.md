# Farbfeld Conformance Test Suite

Test corpus for [farbfeld](https://tools.suckless.org/farbfeld/), the suckless image format. Farbfeld stores uncompressed RGBA pixels at 16 bits per channel in big-endian byte order, preceded by an 8-byte magic string and two big-endian u32 dimension fields.

## Format Summary

| Field | Bytes | Encoding |
|-------|-------|----------|
| Magic | 8 | ASCII `farbfeld` |
| Width | 4 | Big-endian u32 |
| Height | 4 | Big-endian u32 |
| Pixels | width * height * 8 | R, G, B, A each big-endian u16 |

Total file size: `16 + width * height * 8` bytes.

Spec: <https://tools.suckless.org/farbfeld/>

## Regenerating

```
python3 generate.py
```

The script is idempotent -- running it again overwrites files with identical content.

## License

Generated test data. Public domain / CC0.

## File Index

### valid/

| File | Dimensions | Description |
|------|-----------|-------------|
| `1x1_black.ff` | 1x1 | Single black pixel (0,0,0,65535) |
| `1x1_white.ff` | 1x1 | Single white pixel (65535,65535,65535,65535) |
| `1x1_red.ff` | 1x1 | Single red pixel (65535,0,0,65535) |
| `1x1_transparent.ff` | 1x1 | Fully transparent pixel (0,0,0,0) |
| `4x4_solid_blue.ff` | 4x4 | Solid blue |
| `4x4_gradient.ff` | 4x4 | Horizontal gradient, black to white |
| `8x8_checkerboard.ff` | 8x8 | Alternating black/white pixels |
| `4x4_semitransparent.ff` | 4x4 | All pixels at 50% alpha (32768) |
| `2x3_colors.ff` | 2x3 | Six colors: R, G, B, C, M, Y |
| `16x16_rgb_ramp.ff` | 16x16 | Smooth RGB ramp across 256 pixels |
| `100x1_wide.ff` | 100x1 | Wide single-row gradient |
| `1x100_tall.ff` | 1x100 | Tall single-column gradient |

### invalid/

| File | Description |
|------|-------------|
| `bad_magic.ff` | Magic bytes are "farbfool" instead of "farbfeld" |
| `empty.ff` | Zero bytes |
| `header_only.ff` | Valid 16-byte header (4x4) but no pixel data |
| `truncated_header.ff` | Only 10 bytes -- header cut short |
| `truncated_pixels.ff` | Valid 4x4 header but only half the expected pixel data |
| `extra_data.ff` | Valid 1x1 image with 100 extra trailing bytes |
| `zero_width.ff` | Width = 0, height = 4 |
| `zero_height.ff` | Width = 4, height = 0 |
| `zero_both.ff` | Width = 0, height = 0 |

### edge-cases/

| File | Dimensions | Description |
|------|-----------|-------------|
| `max_dimension_1d.ff` | 1000x1 | Large single-row image (~8 KB) |
| `large_square.ff` | 64x64 | Gradient square (~32 KB) |
| `single_channel_illusion.ff` | 4x4 | Grayscale: R=G=B for all pixels |
