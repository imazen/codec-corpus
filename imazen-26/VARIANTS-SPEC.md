# imazen-26 — variant generation spec (v1, 2026-08-22)

Variants (scaled, cropped, re-encoded, sampled derivatives) of imazen-26 have so far
been produced by at least four ad-hoc pipelines with three naming grammars, mixed
resampling kernels, and no content-hash provenance chain (`train_renditions_2026-06-14`
= Mitchell **+ sharpen**; the clean-picker cropscale set = `o_<id>.png.scale<W>x<H>.png`;
the HDR grid = `<basename>.scale<W>x<H>.hdr.png`; the synth sets = their own scheme).
This spec makes every FUTURE variant deterministic, attributable, and
split-inheriting. Existing sets stay valid as historical inputs — do not regenerate
them retroactively; regenerate under this spec the next time a wave needs variants.

## 1. Naming grammar

```
<id>__<op>[.<op>…].<ext>
```

- `<id>` = the origin's 4-digit corpus id, **always the leading token**. Split bucket
  is therefore derivable mechanically: `split_of(int(name[:4]))` (last-digit rule,
  `manifests/README.md`). The descriptor stem is dropped from variant names (the
  manifest carries the join back to the full origin name).
- `__` (double underscore) separates id from ops; single underscores stay reserved for
  the origin naming scheme.
- Op tokens, applied left to right:
  - `s<W>x<H>` — full-frame resample to exactly W×H (same aspect as source; no pad, no
    letterbox, no upscale).
  - `s<W>x<H>l3` — same, Lanczos3 (eval sets only; kernel must be visible in the name).
  - `c<X>+<Y>+<W>x<H>` — crop rect (origin-pixel offset X,Y and size W×H) recorded
    exactly; only spec anchors (§4) may be used for dataset production.
  - codec/quality tokens for encoded variants (e.g. `q85`, `d1p0`, `e7`) — the full
    knob tuple still belongs in the dataset's cell identity, not the filename.

## 2. Resampling rules

- **Downscale kernel: Mitchell–Netravali (B=C=1/3), computed in linear light.** No
  baked-in sharpening — the 2026-06-14 renditions' `+sharpen` is a recorded deviation,
  not a precedent. Lanczos3 is permitted for sharpness-sensitive eval sets only and
  must appear in the op token.
- **HDR sources resample in linear light before PQ re-encode** (the png-v2→v3 lesson);
  SDR PNGs are 8-bit, HDR PNGs 16-bit PQ with correct cICP.
- **No upscaling, ever** (sweep discipline: synthetic upscales mislead every
  sharpness-conditioned model).

## 3. Size ladder

One canonical log-spaced ladder; experiments pick a contiguous subrange, never invent
new rungs:

```
32 40 48 64 80 96 128 160 192 256 320 384 512 640 768 1024 1280 1536 2048 3072 4096
```

(applied to the long edge, capped at source size; short edge follows aspect).

## 4. Crop anchors

Deterministic anchors only: `center`, `tl`, `br`, at 25% and 50% of the source
min-dimension. The materialized rect is recorded in the op token per §1. Ad-hoc crops
are for exploration only and never ship in a dataset.

## 5. Manifest (mandatory)

A variant set without a manifest is not a dataset. Every set ships `variants.tsv`:

```
origin_id  origin_sha256  op_chain  kernel  colorspace_path  generator  generator_commit  out_path  out_sha256  width  height  split
```

`split` is inherited from the origin via the canonical rule — populated at generation
time so consumers never re-derive it wrong.

## 6. Storage

Content-addressed alongside name-addressed: bulk bytes land at
`variants/<sha256>.<ext>` on the object store, and the manifest maps names → hashes
(matching the zenfleet job system's blob convention). Name-addressed copies are
optional conveniences; the hash is the identity.

## 7. Generator

One committed tool per pipeline generation — ad-hoc ImageMagick/PIL invocations are
banned for dataset production. Until a dedicated tool lands in the zen workspace, the
generator + its commit hash MUST be recorded in `variants.tsv` for every row.
