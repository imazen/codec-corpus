# DCT Overflow Test Patterns

Synthetic images that trigger mozilla/mozjpeg#453: 16-bit SIMD forward DCT overflow
when overshoot deringing is enabled.

## The Bug

Overshoot deringing pushes level-shifted sample values to ±158 (vs normal ±128).
The SIMD ISLOW forward DCT uses 16-bit packed arithmetic. After the row pass produces
intermediate values up to ±5056, the column pass final butterfly sums 8 identical
row outputs: `8 × 5056 = 40,448`, exceeding the signed 16-bit maximum of 32,767.

The wrapping causes catastrophic sign flips — entire 8×8 blocks have their brightness
inverted.

**Fix:** Use saturating add/sub (`paddsw`/`psubsw`) instead of wrapping (`paddw`/`psubw`)
in the final even-part butterfly of the column pass.

## Files

| File | Size | Triggers overflow? | Notes |
|------|------|--------------------|-------|
| `left_black_right_white.png` | 64×64 | Yes | Vertical split per 8×8 block |
| `left_white_right_black.png` | 64×64 | Yes | Inverted vertical split |
| `single_8x8_half.png` | 8×8 | Yes | Minimal reproducer |
| `top_black_bottom_white.png` | 64×64 | No | Horizontal split (row pass sees uniform rows) |
| `checkerboard_8x8.png` | 64×64 | No | Full black/white blocks (no intra-block edge) |

## Why Only Vertical Splits Trigger It

The DCT processes rows first, then columns. A vertical split within an 8×8 block
means each row sees `[0,0,0,0,255,255,255,255]` (or inverted), producing maximum
AC energy and intermediate values of ±5056 in the row pass. The column pass then
sums 8 identical row results.

A horizontal split means each row is either all-black or all-white (DC-only),
producing zero AC energy. The column pass intermediates stay small.

## Reference

- https://github.com/mozilla/mozjpeg/pull/453
- Affects: libjpeg-turbo ISLOW FDCT on all SIMD architectures (SSE2, AVX2, NEON, etc.)
- Quality range: Q1–Q57 (DC quantization value ≥ 14)
