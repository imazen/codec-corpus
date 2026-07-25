# corruption-corpus

Deterministic **structural-corruption** image generator: a held-out
falsification set for a perceptual metric's *negative tail*.

Structural corruptions are the localized defects no honest encoder ever
produces — a channel swap, a dropped block, an off-by-one edge — as opposed to
the uniform softening of honest lossy compression. A faithful metric must rank
such a corruption *below* an honestly-lossy encode of the same reference. That
is the gate every entry asserts:

```text
score(reference, corruption) < score(reference, honest_lq_anchor)   // JPEG q20 / q10
```

This crate was extracted from the `codec_corpus::corruptions` module (it lived
inside [`codec-corpus`](https://github.com/imazen/codec-corpus) through the 1.1
line) so the corruption *generator* is decoupled from the corpus *fetcher*.

## The ten families

Each family is a deterministic, in-place mutation of an `Rgb8` buffer,
parameterized by region size (whole-image → 1/4 → 1/16 → 64×64 → 16×16 → 8×8)
and severity (opaque → 50% → 20% opacity):

1. **Channel** — invert / swap (RGB↔BGR, R↔G, …) / zero a plane in a rectangle.
2. **Block** — zero / gray / garbage / copy-wrong / repeat-neighbor (dropped MCUs).
3. **Edge** — k-px border, 1px interior shift, duplicated row (partial-MCU / off-by-one).
4. **Noise** — salt-and-pepper, single-bit flips.
5. **Tone** — wrong sRGB gamma, local contrast boost, brightness offset, in a block.
6. **Overlay** — low-opacity rect / line / glyph (render leak / watermark bleed).
7. **Chroma boundary** — wrong-phase chroma upsample at 8px block edges.
8. **Aliasing** — nearest-neighbor downscale→upscale (resampler bug).
9. **Geometric** — 1px shift / flip / 90° rotate of a region.
10. **Composite** — premultiplied-as-straight alpha, wrong background color.

## Design

- **Deterministic** — every corruption is seeded from `(ref_id, seed, params)`
  via a dependency-free [SplitMix64](https://prng.di.unimi.it/splitmix64.c).
  No `rand`, no OS entropy: the same inputs produce the same bytes on every
  platform and every run.
- **Pure RGB-buffer math** — the generators take an `Rgb8` and mutate it in
  place; no image decoding is needed to *apply* a corruption, so the default
  build has no image-codec dependency and compiles to WASM.
- **Nothing large is committed** — the corpus is reproduced on demand from
  `(ref_id, seed, params)` plus whatever reference set you point it at.

## Usage

```rust
use corruption_corpus::{Rgb8, Family, ChannelOp, CorruptionParams, Region, Severity};

// A flat gray 64x64 reference.
let mut img = Rgb8::filled(64, 64, [128, 128, 128]);
let params = CorruptionParams {
    family: Family::Channel(ChannelOp::SwapRb),
    region: Region::Fraction(4), // 1/4 of the image
    severity: Severity::Opaque,
};
params.apply(&mut img, /* seed */ 1);
// img now has a region with R and B swapped.
```

`catalog()` expands every family × region × severity; `manifest_for_reference()`
builds the `ManifestEntry` list (with per-entry seeds) for one reference.

## The `driver` feature

Behind the optional `driver` feature (which pulls in the `image` crate), the
`driver` module loads a reference image and emits the score-ready quad
`(reference, corruption, q20-anchor, q10-anchor)` — the two honest low-quality
JPEG anchors are encoded then decoded back to pixels so the gate scores pixels,
not bitstreams.

The `corruption_corpus` example writes those quads plus a `_MANIFEST.json` to an
output dir:

```bash
cargo run -p corruption-corpus --example corruption_corpus --features driver -- \
    --ref ../gb82-sc/imac_g3_strip.png --ref-id gb82-sc/imac_g3_strip \
    --class screen --out ./corruption-out
```

Real-bug reproductions of historical decoder/renderer defects are tracked
separately; the manifest's `source` field reserves a `real_bug` slot for them.

## License

Apache-2.0.
