# APNG Conformance Test Suite

Test files for validating APNG (Animated PNG) decoders and encoders.

APNG extends PNG with three chunk types:
- **`acTL`** (animation control) -- `num_frames`, `num_plays`
- **`fcTL`** (frame control) -- sequence number, dimensions, offsets, timing, dispose/blend ops
- **`fdAT`** (frame data) -- sequence number + compressed image data (same format as IDAT)

The first frame may use standard `IDAT` chunks (if `fcTL` precedes `IDAT`) or `IDAT` may serve as a static fallback for non-APNG decoders (if no `fcTL` precedes `IDAT`).

Spec: <https://wiki.mozilla.org/APNG_Specification>

## License

Generated test data. Public domain / CC0.

## Generation

All files are generated from scratch using Python stdlib (`struct`, `zlib`). No external tools required.

```bash
python3 generate.py
```

## Valid Files (`valid/`)

| File | Dimensions | Frames | Color Type | Features Tested | Expected Behavior |
|------|-----------|--------|------------|----------------|-------------------|
| `2frame_simple.png` | 8x8 | 2 | RGBA | Basic animation | Alternates red/blue, 500ms per frame |
| `3frame_rgb.png` | 4x4 | 3 | RGBA | Multi-frame cycle | Cycles R, G, B solid colors, 333ms each |
| `10frame_gradient.png` | 8x8 | 10 | RGBA | Many frames, gradient | Fades from black to white, 100ms per frame |
| `dispose_none.png` | 8x8 | 3 | RGBA | `dispose_op=0` (NONE) | Each frame replaces canvas; previous buffer persists underneath |
| `dispose_background.png` | 8x8 | 3 | RGBA | `dispose_op=1` (BACKGROUND) | Frame region cleared to transparent black after display; frame 1 is full-canvas, frames 2-3 are 4x4 sub-regions |
| `dispose_previous.png` | 8x8 | 3 | RGBA | `dispose_op=2` (PREVIOUS) | Frame region restored to pre-frame state after display; first frame's PREVIOUS treated as NONE per spec |
| `blend_source.png` | 8x8 | 3 | RGBA | `blend_op=0` (SOURCE) | Semi-transparent frames fully replace canvas pixels (no compositing) |
| `blend_over.png` | 8x8 | 3 | RGBA | `blend_op=1` (OVER) | Opaque red base, then semi-transparent green/blue alpha-composited over |
| `offset_frames.png` | 8x8 | 5 | RGBA | Sub-frame offsets | Gray canvas, then 2x2 colored patches at four corners via `x_offset`/`y_offset` |
| `loop_infinite.png` | 4x4 | 2 | RGBA | `num_plays=0` | Loops forever (red/blue alternation) |
| `loop_once.png` | 4x4 | 2 | RGBA | `num_plays=1` | Plays once then stops on last frame |
| `loop_3times.png` | 4x4 | 2 | RGBA | `num_plays=3` | Plays 3 times then stops |
| `fast_animation.png` | 4x4 | 3 | RGBA | 10ms delay | R/G/B at 10ms per frame (100 fps) |
| `slow_animation.png` | 4x4 | 3 | RGBA | 2000ms delay | R/G/B at 2 seconds per frame |
| `variable_delay.png` | 4x4 | 3 | RGBA | Mixed delays | Frame delays: 100ms, 500ms, 1000ms |
| `rgba_8bit.png` | 4x4 | 2 | RGBA (type 6) | RGBA color type | Semi-transparent red/blue, 8-bit depth |
| `rgb_8bit.png` | 4x4 | 2 | RGB (type 2) | RGB color type | Red/blue, no alpha channel |
| `gray_8bit.png` | 4x4 | 2 | Gray (type 0) | Grayscale | Dark gray / light gray animation |
| `palette_8bit.png` | 4x4 | 3 | Palette (type 3) | Indexed color + PLTE | Cycles palette indices 0/1/2 (red/green/blue) |
| `default_is_first.png` | 4x4 | 2 | RGBA | `fcTL` before `IDAT` | Default image IS part of animation (fcTL precedes IDAT) |
| `default_separate.png` | 4x4 | 2 | RGBA | `IDAT` without `fcTL` | Default image is static fallback (gray); animation is red/blue via fdAT only |
| `single_frame.png` | 4x4 | 1 | RGBA | Single-frame APNG | Valid APNG with `acTL` declaring 1 frame |

## Invalid Files (`invalid/`)

| File | Violation | Expected Behavior |
|------|-----------|-------------------|
| `missing_actl.png` | Has `fcTL`/`fdAT` but no `acTL` chunk | Decoder should treat as static PNG (ignore APNG chunks) or reject |
| `bad_sequence.png` | Duplicate sequence numbers (all chunks use seq 0) | Decoder should reject -- sequence numbers must be 0, 1, 2, ... |
| `frame_out_of_bounds.png` | `fcTL` has `x_offset + width > canvas_width` (3+3 > 4) | Decoder should reject -- frame must fit within canvas |
| `zero_delay_den.png` | `delay_den = 0` | Per spec, treat as 100 (so delay = 50/100 = 500ms). Some decoders may reject. |
| `truncated_fdat.png` | `fdAT` compressed data is truncated (half the bytes removed) | Decoder should error on decompression failure |
| `no_fdat.png` | `acTL` claims 3 frames but only 1 frame (IDAT) exists | Decoder should error on missing frames or show only the first frame |

## Edge Cases (`edge/`)

| File | Dimensions | Frames | Feature | Notes |
|------|-----------|--------|---------|-------|
| `zero_delay.png` | 4x4 | 3 | `delay_num=0` | Render as fast as possible. Some decoders clamp to a minimum. |
| `many_frames.png` | 4x4 | 50 | High frame count | Stress test: 50 frames cycling through hues, 50ms each |
| `1x1_animated.png` | 1x1 | 3 | Minimum dimensions | 1x1 pixel canvas, 3 frames (R/G/B) |

## Chunk Structure Reference

A conformant APNG file has this structure:

```
PNG signature
IHDR
[PLTE]          (for palette images)
acTL            (animation control, before IDAT)
[fcTL]          (if default image is part of animation, before IDAT)
IDAT [IDAT...] (default image)
fcTL            (frame 2 control)
fdAT [fdAT...] (frame 2 data)
fcTL            (frame 3 control)
fdAT [fdAT...] (frame 3 data)
...
IEND
```

Sequence numbers are shared between `fcTL` and `fdAT` chunks and must increment from 0 without gaps.
