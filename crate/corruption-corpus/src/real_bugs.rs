//! Gold-standard members: reproductions of real, shipped wrong-pixel bugs
//! (imazen/codec-corpus#7, "MINED real historical decoder/renderer bugs").
//!
//! Every row of the issue's table is a [`RealBugId`]. Each one is a
//! deterministic *synthetic pixel-pattern repro* of the bug's documented
//! defect — the shape of the wrong pixels a fixed decoder/renderer used to
//! emit — applied to any reference image. None of them is a recovered
//! buggy-decoder output: that would need the pre-fix commit of each sibling
//! repo checked out and built, which this crate deliberately does not depend
//! on. The manifest marks them `source: real-bug:<repo>#<ref>` so evaluators
//! can separate them from the ten synthetic families.
//!
//! The repros are "true to the pattern", not to the bit: e.g. the XYB
//! false-detection bug is modelled by running an sRGB triple through the
//! XYB→RGB inverse, and the WebP dispose-stride bug by replaying a 3-byte
//! stride write over a 4-byte-stride canvas. Where the original bug lived in
//! an alpha plane the reference has none, so an alpha of 255 is assumed and
//! the defect is what the wrong alpha composites to.

use serde::{Deserialize, Serialize};

use super::{Rgb8, blend, prng::SplitMix64};

/// One documented real bug. Variant names encode `<repo><Bug>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RealBugId {
    // ---- zenjpeg -----------------------------------------------------------
    /// Progressive decoder truncated AC coefficients near restart markers:
    /// trailing MCUs decode DC-only (blocky, desaturated).
    ZenjpegProgressiveAcTruncation,
    /// Progressive MCU-padded storage stride mismatch (4:2:0, non-MCU-aligned
    /// width): one block of horizontal shift per block row, accumulating.
    ZenjpegProgressiveStrideMismatch,
    /// Progressive interleaved DC scan padding desynced the Huffman decoder:
    /// from some block on, every block decodes as its predecessor.
    ZenjpegProgressiveDcDesync,
    /// 4:2:0 chroma upsampling edge-replicated at the MCU *bottom* boundary:
    /// a horizontal chroma band at every 16-row MCU edge.
    ZenjpegChromaBottomEdgeReplication,
    /// XYB Full mode emitted BQuarter block ordering: every component read as
    /// the wrong plane (red → green).
    ZenjpegXybPlaneOrder,
    /// XYB linear-input path double-scaled: every MCU saturates, the image
    /// decodes to solid white.
    ZenjpegXybDoubleScaleWhite,
    /// XYB 4:2:0 DC category clamp mismatch: bitstream undecodable past a
    /// point, the rest of the image is the decoder's fill.
    ZenjpegXybDcClampBitstream,
    /// False XYB-ICC detection on cjpegli JPEGs: plain RGB run through the
    /// XYB→RGB inverse (completely wrong colors).
    ZenjpegFalseXybIcc,
    /// Grayscale Hi/Vi > 1 IDCT strip overflow: rows within each 16-row strip
    /// land in the wrong order (garbled grayscale).
    ZenjpegGrayscaleSamplingStripOverflow,
    /// h2v2 boundary fixup overflow / sharp-YUV y_offset sign error: chroma
    /// pushed low (green shift) along the right/bottom boundary.
    ZenjpegH2v2BoundaryOverflowGreenShift,
    // ---- zenwebp -----------------------------------------------------------
    /// Multi-pass probability signaling bug: 99.9 % of pixels garbage.
    ZenwebpMultipassProbabilityGarbage,
    /// Chroma error diffusion applied in encode but not in reconstruction:
    /// chroma quantized without the diffusion the encoder assumed.
    ZenwebpChromaErrorDiffusionMismatch,
    /// Missing U/V chroma-conversion rounding: U/V off by one nearly everywhere.
    ZenwebpUvRoundingOffByOne,
    /// sharp_yuv used a non-gamma-corrected BT.601 matrix: Cb/Cr ~12 low.
    ZenwebpSharpYuvMatrixGreenShift,
    /// Fused-scalar fancy upsampling collapsed to a single chroma sample:
    /// 2×2 chroma blockiness, no neighbour blend.
    ZenwebpFancyUpsamplingCollapsed,
    /// NEON `sse_8x8_chroma` stride bug: wrong chroma SSE → mode
    /// mis-selection, flat-chroma blocks.
    ZenwebpNeonChromaSseStride,
    /// Animation dispose-to-background used a 3-byte stride on an RGBA
    /// canvas: the cleared rect's writes land misaligned.
    ZenwebpAnimDisposeStrideCorruption,
    /// Lossless + alpha + padded stride: the alpha plane read with the
    /// unpadded stride, so padding (alpha 0) shears into the image.
    ZenwebpLosslessAlphaPaddedStride,
    /// Color-indexing transform decode bug: palette lookups off by one.
    ZenwebpColorIndexingDecode,
    /// Lossless color-cache update missing in the fast path: stale cache
    /// entries yield colors from earlier in the stream.
    ZenwebpColorCacheStale,
    /// Systematic quality drop at codec mode switches (q75→80, q87→90):
    /// half the image quantized much coarser than the other half.
    ZenwebpQualityCliffModeSwitch,
    // ---- zengif ------------------------------------------------------------
    /// Transparent pixels mapped to the nearest palette color (shared-palette
    /// mode): the transparent region renders dark gray.
    ZengifTransparentNearestPalette,
    /// `quantize_frame` skipped the alpha→transparent-index post-process:
    /// a transparent region renders opaque with the previous frame's content.
    ZengifTransparentIndexMissing,
    // ---- zenpng ------------------------------------------------------------
    /// U16→U8 depth reduction used `>> 8` truncation, not rounding.
    ZenpngU16ToU8Truncation,
    /// 16-bit decode truncated to u8 before f32: lost precision, banding.
    ZenpngSixteenBitPrecisionLoss,
    /// Adam7 buffer too small for wide images: the last passes never land,
    /// odd rows/columns repeat their neighbours.
    ZenpngAdam7BufferTooSmall,
    // ---- zenavif -----------------------------------------------------------
    /// `unpremultiply8` truncated `c*255/a` instead of rounding.
    ZenavifUnpremultiplyTruncation,
    /// `scale_from_u16` half-up rounding fought LSB replication: half of the
    /// values shift +1.
    ZenavifScaleFromU16Rounding,
    /// Bilinear chroma interpolation with negative weights at image edges.
    ZenavifBilinearChromaEdgeNegativeWeights,
    /// f32 PQ/HLG routed through `linear_to_srgb_u8`: HDR tone-clipped.
    ZenavifPqHlgThroughSrgb,
    // ---- heic --------------------------------------------------------------
    /// Chroma MC shift 4 instead of 6 (uni-pred): chroma scaled ×4.
    HeicChromaMcShift4,
    /// Chroma MC wrong intermediate precision (bi-pred): wrong chroma in
    /// B-slice blocks.
    HeicChromaBipredPrecision,
    /// Spurious rounding offset in the bi-pred MC vertical pass: luma off by
    /// a small amount in bi-pred blocks.
    HeicBipredRoundingOffset,
    /// Deblocking boundary strength wrong (inter) + SAO corruption: over-
    /// smoothed block edges and a wrong band offset.
    HeicDeblockSao,
    /// Tile-boundary CABAC/QP/MPM errors: wrong pixels along tile boundaries
    /// of a grid HEIC.
    HeicTileBoundary,
    // ---- imageflow ---------------------------------------------------------
    /// Transparent PNG → JPEG loses color (`bgcolor=transparent` matte):
    /// desaturated, low-contrast flatten.
    ImageflowTransparentToJpegMatte,
    /// "within" constraint rounding → 1px transparent border (99×33 content
    /// in a 100×33 canvas).
    ImageflowWithinConstraintTransparentBorder,
    /// Transparent PNG shows a black background on JPEG convert (composited
    /// onto black, not white).
    ImageflowTransparentPngBlackBackground,
    /// Swapped width/height in `enable_transparency` canvas creation: the
    /// buffer is read with the wrong stride.
    ImageflowSwappedCanvasDims,
    /// EXIF-orientated image cropped in the wrong region: the crop taken
    /// before orientation shows mirrored content.
    ImageflowExifOrientationCropRegion,
    /// WebP lossy output systematically desaturated.
    ImageflowWebpLossyDesaturated,
}

/// Static description of one real bug.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RealBugInfo {
    /// The bug.
    pub id: RealBugId,
    /// Repository the bug shipped in.
    pub repo: &'static str,
    /// Commit / issue / PR reference within that repository.
    pub reference: &'static str,
    /// One-line description of the bug.
    pub summary: &'static str,
    /// The pixel-defect pattern this repro models.
    pub pattern: &'static str,
    /// The synthetic family this bug is the real-world analogue of, if any.
    pub family_analog: Option<&'static str>,
    /// Whether the defect is egregious enough that a calibrated score is
    /// expected to go negative (vs merely ranking below the lq anchor).
    pub structure_destroying: bool,
}

macro_rules! table {
    ($( $id:ident => $repo:literal, $reference:literal, $summary:literal, $pattern:literal, $analog:expr, $destroy:literal; )*) => {
        /// Every real bug, in the issue table's order.
        pub const ALL: &[RealBugId] = &[ $( RealBugId::$id, )* ];

        impl RealBugId {
            /// Static description of this bug.
            pub fn info(self) -> RealBugInfo {
                match self {
                    $( RealBugId::$id => RealBugInfo {
                        id: RealBugId::$id,
                        repo: $repo,
                        reference: $reference,
                        summary: $summary,
                        pattern: $pattern,
                        family_analog: $analog,
                        structure_destroying: $destroy,
                    }, )*
                }
            }
        }
    };
}

table! {
    ZenjpegProgressiveAcTruncation => "zenjpeg", "08ef601",
        "Progressive decoder truncated AC coeffs near restart markers",
        "trailing MCUs decode DC-only: blocky, desaturated", Some("block"), false;
    ZenjpegProgressiveStrideMismatch => "zenjpeg", "29d6d81",
        "Progressive MCU-padded storage stride mismatch (4:2:0, non-MCU-aligned width)",
        "one block of horizontal shift per block row, accumulating to max_diff=255", Some("block"), true;
    ZenjpegProgressiveDcDesync => "zenjpeg", "759a4a7",
        "Progressive interleaved DC scan padding desynced the Huffman decoder",
        "every block after a point decodes as its predecessor", Some("block"), true;
    ZenjpegChromaBottomEdgeReplication => "zenjpeg", "bd0f8d7",
        "4:2:0 scanline chroma upsampling edge-replicated at the MCU bottom boundary",
        "horizontal chroma band at every 16-row MCU edge", Some("chroma_boundary"), false;
    ZenjpegXybPlaneOrder => "zenjpeg", "daf52508",
        "XYB Full mode emitted BQuarter MCU block ordering",
        "every component read as the wrong plane (red -> green)", Some("channel"), true;
    ZenjpegXybDoubleScaleWhite => "zenjpeg", "28658af6+9e2348fe",
        "XYB linear-input path double-scaled, Y~1600 saturates every MCU",
        "decodes to solid white", None, true;
    ZenjpegXybDcClampBitstream => "zenjpeg", "b0cafce",
        "XYB 4:2:0 DC category clamped to 11 but encoder wrote unclamped",
        "bitstream undecodable past a point; remaining rows are decoder fill", Some("block"), true;
    ZenjpegFalseXybIcc => "zenjpeg", "744d38a",
        "False XYB-ICC detection on cjpegli JPEGs (\"jxl \" CMM)",
        "plain RGB run through the XYB->RGB inverse: completely wrong colors", Some("tone"), true;
    ZenjpegGrayscaleSamplingStripOverflow => "zenjpeg", "4af35564",
        "grayscale Hi/Vi>1 + Bgr -> IDCT strip overflow",
        "rows within each 16-row strip written in the wrong order (garbled grayscale)", None, true;
    ZenjpegH2v2BoundaryOverflowGreenShift => "zenjpeg", "1c9631a8",
        "h2v2 boundary fixup overflow / sharp-YUV y_offset sign error",
        "Cb/Cr pushed low (green shift) along the right/bottom boundary", Some("chroma_boundary"), false;
    ZenwebpMultipassProbabilityGarbage => "zenwebp", "d58fc25",
        "Multi-pass probability signaling: encoder signals 0 updates, decoder expects ~125",
        "99.9% of pixels garbage", None, true;
    ZenwebpChromaErrorDiffusionMismatch => "zenwebp", "eaa7afe",
        "Chroma error-diffusion applied in encode but not in reconstruction",
        "chroma quantized without diffusion: subtle chroma banding", None, false;
    ZenwebpUvRoundingOffByOne => "zenwebp", "11465dd",
        "Missing U/V chroma-conversion rounding",
        "U/V off by one on ~99% of pixels", None, false;
    ZenwebpSharpYuvMatrixGreenShift => "zenwebp", "7a23ffa",
        "sharp_yuv used non-gamma-corrected BT.601 forward matrix",
        "systematic green shift (Cb/Cr ~12 low)", None, false;
    ZenwebpFancyUpsamplingCollapsed => "zenwebp", "c63d898",
        "fused-scalar fancy upsampling collapsed to a single chroma sample",
        "2x2 chroma blockiness, no neighbour blend", Some("chroma_boundary"), false;
    ZenwebpNeonChromaSseStride => "zenwebp", "167ad48",
        "NEON sse_8x8_chroma chroma-width stride bug",
        "wrong chroma SSE -> mode mis-selection: flat-chroma 8x8 blocks", Some("block"), false;
    ZenwebpAnimDisposeStrideCorruption => "zenwebp", "a8df324",
        "animation dispose-to-background used 3-byte stride on RGBA canvas",
        "misaligned writes: canvas corruption of the cleared rect", Some("block"), true;
    ZenwebpLosslessAlphaPaddedStride => "zenwebp", "8c29215",
        "lossless + alpha + padded-stride corruption (PR #18 / issue #10)",
        "alpha plane read with the unpadded stride: padding shears in as transparent", Some("composite"), true;
    ZenwebpColorIndexingDecode => "zenwebp", "cfd582c",
        "color-indexing transform decode bug (#28)",
        "palette lookups off by one: wrong indexed colors", None, false;
    ZenwebpColorCacheStale => "zenwebp", "7b2bbbc",
        "lossless color-cache update missing in fast path (#114)",
        "stale cache entries: scattered pixels take colors from earlier in the stream", Some("noise"), false;
    ZenwebpQualityCliffModeSwitch => "zenwebp", "zensim-MEMORY-known-issues",
        "systematic quality drops at q75->80 and q87->90 (mode switches)",
        "non-monotonic quality cliff: half the image quantized much coarser", None, false;
    ZengifTransparentNearestPalette => "zengif", "93b6da2",
        "transparent pixels mapped to nearest palette color (shared-palette mode)",
        "transparent region renders dark gray", Some("composite"), true;
    ZengifTransparentIndexMissing => "zengif", "0d9f031",
        "quantize_frame returned raw indices without alpha->transparent-index post-process",
        "transparent region renders opaque with the previous frame's content", Some("block"), true;
    ZenpngU16ToU8Truncation => "zenpng", "d88325c",
        "U16->U8 depth reduction used >>8 truncation not rounding",
        "per-channel off-by-up-to-1 on 16-bit sources", None, false;
    ZenpngSixteenBitPrecisionLoss => "zenpng", "838cad7",
        "16-bit PNG decode truncated to u8 before f32",
        "lost precision: banding on smooth content", Some("tone"), false;
    ZenpngAdam7BufferTooSmall => "zenpng", "3cbed35",
        "Adam7 interlaced decode buffer too small for wide images",
        "last passes never land: odd rows/columns repeat their neighbours", Some("aliasing"), false;
    ZenavifUnpremultiplyTruncation => "zenavif", "4509713",
        "unpremultiply8 truncated (c*255/a) instead of rounding",
        "off-by-1 on un-premultiplied pixels", None, false;
    ZenavifScaleFromU16Rounding => "zenavif", "42d06a7",
        "scale_from_u16 half-up rounding fought LSB replication",
        "50% of values shifted +1 on roundtrip", None, false;
    ZenavifBilinearChromaEdgeNegativeWeights => "zenavif", "cb366c5",
        "bilinear chroma interp negative weights at image edges",
        "broken edge chroma: overshoot along the 2px border", Some("edge"), false;
    ZenavifPqHlgThroughSrgb => "zenavif", "37d2073",
        "f32 PQ/HLG routed through linear_to_srgb_u8",
        "HDR signal destroyed (tone-clipped)", Some("tone"), true;
    HeicChromaMcShift4 => "heic", "c20861e",
        "chroma MC shift 4 instead of 6 (uni-pred)",
        "chroma scaled 4x: wildly wrong color", Some("channel"), true;
    HeicChromaBipredPrecision => "heic", "b24c111",
        "chroma MC wrong intermediate precision (bi-pred)",
        "wrong chroma in B-slice blocks", Some("block"), false;
    HeicBipredRoundingOffset => "heic", "9e9b1c5",
        "spurious rounding offset in bi-pred MC vertical pass",
        "luma off by a small amount in bi-pred blocks", Some("tone"), false;
    HeicDeblockSao => "heic", "251968b+754a029",
        "deblocking boundary strength wrong (inter) / SAO corruption",
        "over-smoothed 8px block edges + wrong SAO band offset", Some("chroma_boundary"), false;
    HeicTileBoundary => "heic", "dafa255+5d035ec",
        "tile-boundary CABAC/QP/MPM errors",
        "wrong pixels along the tile boundaries of a grid HEIC", Some("edge"), false;
    ImageflowTransparentToJpegMatte => "imageflow", "#669",
        "transparent PNG->JPEG loses color (bgcolor=transparent matte)",
        "desaturated / wrong matte on flatten", Some("composite"), false;
    ImageflowWithinConstraintTransparentBorder => "imageflow", "#656",
        "\"within\" constraint rounding -> 1px transparent border",
        "99x33 content in a 100x33 canvas: off-by-one edge", Some("edge"), false;
    ImageflowTransparentPngBlackBackground => "imageflow", "#158+#190+#175",
        "transparent PNG shows black background on JPEG convert",
        "wrong-bg composite (black not white)", Some("composite"), true;
    ImageflowSwappedCanvasDims => "imageflow", "0e7c0385",
        "swapped width/height in enable_transparency canvas creation",
        "misaligned transparent canvas: buffer read with the wrong stride", Some("geometric"), true;
    ImageflowExifOrientationCropRegion => "imageflow", "#553",
        "EXIF-orientated image cropped in wrong region",
        "crop applied pre-orientation: wrong (mirrored) content", Some("geometric"), true;
    ImageflowWebpLossyDesaturated => "imageflow", "#588",
        "WebPLossy desaturated color",
        "systematic desaturation", Some("tone"), false;
}

impl RealBugId {
    /// Every real bug, in the issue table's order.
    pub fn all() -> &'static [RealBugId] {
        ALL
    }

    /// `real-bug:<repo>#<ref>` — the manifest `source` reference.
    pub fn source_reference(self) -> String {
        let i = self.info();
        format!("real-bug:{}#{}", i.repo, i.reference)
    }

    /// Stable snake_case slug of the variant name.
    pub fn slug(self) -> String {
        let dbg = format!("{self:?}");
        let mut out = String::with_capacity(dbg.len() + 8);
        for (i, ch) in dbg.chars().enumerate() {
            if ch.is_uppercase() {
                if i != 0 {
                    out.push('_');
                }
                out.extend(ch.to_lowercase());
            } else {
                out.push(ch);
            }
        }
        out
    }

    /// Apply the repro to `img` in place at `opacity` (1.0 = the defect as
    /// shipped), deterministically from `rng`.
    pub(crate) fn apply(self, img: &mut Rgb8, opacity: f32, rng: &mut SplitMix64) {
        if img.width() == 0 || img.height() == 0 {
            return;
        }
        let orig = img.clone();
        let mut out = orig.clone();
        self.render(&orig, &mut out, rng);
        if opacity >= 1.0 {
            *img = out;
            return;
        }
        for y in 0..img.height() {
            for x in 0..img.width() {
                img.set(x, y, blend(orig.get(x, y), out.get(x, y), opacity));
            }
        }
    }

    fn render(self, src: &Rgb8, out: &mut Rgb8, rng: &mut SplitMix64) {
        let (w, h) = (src.width(), src.height());
        match self {
            RealBugId::ZenjpegProgressiveAcTruncation => {
                let (bw, bh) = (w.div_ceil(8), h.div_ceil(8));
                let total = (bw * bh) as u64;
                let first_bad = total * 2 / 3;
                for_each_block(w, h, 8, |bx, by, rect| {
                    if (by * bw + bx) as u64 >= first_bad {
                        let mean = block_mean(src, rect);
                        fill(out, rect, mean);
                    }
                });
            }
            RealBugId::ZenjpegProgressiveStrideMismatch => {
                for y in 0..h {
                    let shift = (y / 8) * 8 % w;
                    for x in 0..w {
                        out.set(x, y, src.get((x + w - shift) % w, y));
                    }
                }
            }
            RealBugId::ZenjpegProgressiveDcDesync => {
                let (bw, bh) = (w.div_ceil(8), h.div_ceil(8));
                let start = (bw * bh) / 3;
                for_each_block(w, h, 8, |bx, by, rect| {
                    let i = by * bw + bx;
                    if i == start {
                        fill(out, rect, [128, 128, 128]);
                    } else if i > start {
                        let (px, py) = ((i - 1) % bw * 8, (i - 1) / bw * 8);
                        for (x, y) in rect.pixels(w, h) {
                            let (sx, sy) =
                                ((px + x - rect.0).min(w - 1), (py + y - rect.1).min(h - 1));
                            out.set(x, y, src.get(sx, sy));
                        }
                    }
                });
            }
            RealBugId::ZenjpegChromaBottomEdgeReplication => {
                for y in 0..h {
                    if y % 16 < 14 {
                        continue;
                    }
                    let y0 = y - y % 16;
                    let sy = (y0 + 13).min(h - 1);
                    for x in 0..w {
                        out.set(x, y, with_chroma_of(src.get(x, y), src.get(x, sy)));
                    }
                }
            }
            RealBugId::ZenjpegXybPlaneOrder => map_pixels(src, out, |p| [p[1], p[2], p[0]]),
            RealBugId::ZenjpegXybDoubleScaleWhite => map_pixels(src, out, |_| [255, 255, 255]),
            RealBugId::ZenjpegXybDcClampBitstream => {
                let cut = (h as u64 * 2 / 5) as u32;
                for y in cut..h {
                    for x in 0..w {
                        out.set(x, y, [128, 128, 128]);
                    }
                }
            }
            RealBugId::ZenjpegFalseXybIcc => map_pixels(src, out, xyb_inverse_of_srgb),
            RealBugId::ZenjpegGrayscaleSamplingStripOverflow => {
                for y0 in (0..h).step_by(16) {
                    let hs = (h - y0).min(16);
                    for r in 0..hs {
                        let sr = (r * 2 + r * 2 / hs) % hs;
                        for x in 0..w {
                            let g = luma8(src.get(x, y0 + sr));
                            out.set(x, y0 + r, [g, g, g]);
                        }
                    }
                }
            }
            RealBugId::ZenjpegH2v2BoundaryOverflowGreenShift => {
                for y in 0..h {
                    for x in 0..w {
                        if x + 2 >= w || y + 2 >= h {
                            out.set(x, y, shift_chroma(src.get(x, y), -64.0, true));
                        }
                    }
                }
            }
            RealBugId::ZenwebpMultipassProbabilityGarbage => {
                for y in 0..h {
                    for x in 0..w {
                        if rng.below(1000) != 0 {
                            out.set(x, y, [rng.next_u8(), rng.next_u8(), rng.next_u8()]);
                        }
                    }
                }
            }
            RealBugId::ZenwebpChromaErrorDiffusionMismatch => map_pixels(src, out, |p| {
                let (yl, cb, cr) = rgb_to_ycbcr(p);
                ycbcr_to_rgb(yl, (cb / 4.0).floor() * 4.0, (cr / 4.0).floor() * 4.0)
            }),
            RealBugId::ZenwebpUvRoundingOffByOne => {
                map_pixels(src, out, |p| shift_chroma(p, -1.0, false))
            }
            RealBugId::ZenwebpSharpYuvMatrixGreenShift => {
                map_pixels(src, out, |p| shift_chroma(p, -12.0, false))
            }
            RealBugId::ZenwebpFancyUpsamplingCollapsed => {
                for y0 in (0..h).step_by(2) {
                    for x0 in (0..w).step_by(2) {
                        let rect = (x0, y0, 2.min(w - x0), 2.min(h - y0));
                        let (_, cb, cr) = mean_ycbcr(src, rect);
                        for (x, y) in rect.pixels(w, h) {
                            let (yl, _, _) = rgb_to_ycbcr(src.get(x, y));
                            out.set(x, y, ycbcr_to_rgb(yl, cb, cr));
                        }
                    }
                }
            }
            RealBugId::ZenwebpNeonChromaSseStride => {
                for_each_block(w, h, 8, |_, _, rect| {
                    if rng.below(10) < 3 {
                        let (_, cb, cr) = mean_ycbcr(src, rect);
                        for (x, y) in rect.pixels(w, h) {
                            let (yl, _, _) = rgb_to_ycbcr(src.get(x, y));
                            out.set(x, y, ycbcr_to_rgb(yl, cb, cr));
                        }
                    }
                });
            }
            RealBugId::ZenwebpAnimDisposeStrideCorruption => {
                // Replay a `w*3`-bytes-per-row clear with row stride `W*3`
                // over a canvas whose real pixel stride is 4 bytes.
                let (rw, rh) = ((w / 2).max(1), (h / 2).max(1));
                let (x0, y0) = ((w - rw) / 2, (h - rh) / 2);
                let (wu, hu) = (w as u64, h as u64);
                for r in 0..rh as u64 {
                    let start = y0 as u64 * wu * 4 + r * wu * 3 + x0 as u64 * 4;
                    for b in 0..rw as u64 * 3 {
                        let off = start + b;
                        let (px, ch) = (off / 4, (off % 4) as usize);
                        if px >= wu * hu || ch == 3 {
                            continue;
                        }
                        let (x, y) = ((px % wu) as u32, (px / wu) as u32);
                        let mut p = out.get(x, y);
                        p[ch] = 255;
                        out.set(x, y, p);
                    }
                }
            }
            RealBugId::ZenwebpLosslessAlphaPaddedStride => {
                let pad = 8u64;
                let (wu, stride) = (w as u64, w as u64 + pad);
                for y in 0..h {
                    for x in 0..w {
                        let i = y as u64 * wu + x as u64;
                        if i % stride >= wu {
                            out.set(x, y, [255, 255, 255]); // alpha 0 → background
                        }
                    }
                }
            }
            RealBugId::ZenwebpColorIndexingDecode => map_pixels(src, out, |p| {
                let idx = ((p[0] >> 6) << 4) | ((p[1] >> 6) << 2) | (p[2] >> 6);
                let wrong = (idx + 1) & 63;
                [
                    ((wrong >> 4) & 3) * 85,
                    ((wrong >> 2) & 3) * 85,
                    (wrong & 3) * 85,
                ]
            }),
            RealBugId::ZenwebpColorCacheStale => {
                let n = w as u64 * h as u64;
                for i in 0..n {
                    if rng.below(20) == 0 {
                        let back = 64 + rng.below(960);
                        let j = i.saturating_sub(back);
                        let (sx, sy) = ((j % w as u64) as u32, (j / w as u64) as u32);
                        let (x, y) = ((i % w as u64) as u32, (i / w as u64) as u32);
                        out.set(x, y, src.get(sx, sy));
                    }
                }
            }
            RealBugId::ZenwebpQualityCliffModeSwitch => {
                for_each_block(w, h, 8, |_, _, rect| {
                    let step = if rect.1 < h / 2 { 6.0 } else { 40.0 };
                    dct_quantize_block(src, out, rect, step);
                });
            }
            RealBugId::ZengifTransparentNearestPalette => {
                let rect = seeded_rect(w, h, 2, rng);
                fill(out, rect, [0x40, 0x40, 0x40]);
            }
            RealBugId::ZengifTransparentIndexMissing => {
                let rect = seeded_rect(w, h, 2, rng);
                for (x, y) in rect.pixels(w, h) {
                    out.set(x, y, src.get(x.saturating_sub(8), y.saturating_sub(8)));
                }
            }
            RealBugId::ZenpngU16ToU8Truncation => {
                for y in 0..h {
                    for x in 0..w {
                        let p = src.get(x, y);
                        let mut q = p;
                        for c in 0..3 {
                            // A 16-bit source value that rounds to p[c].
                            let v16 =
                                (p[c] as i32 * 257 + rng.below(257) as i32 - 128).clamp(0, 65535);
                            q[c] = (v16 >> 8) as u8;
                        }
                        out.set(x, y, q);
                    }
                }
            }
            RealBugId::ZenpngSixteenBitPrecisionLoss => {
                map_pixels(src, out, |p| [p[0] & !3, p[1] & !3, p[2] & !3])
            }
            RealBugId::ZenpngAdam7BufferTooSmall => {
                for y in 0..h {
                    for x in 0..w {
                        out.set(x, y, src.get(x & !1, y & !1));
                    }
                }
            }
            RealBugId::ZenavifUnpremultiplyTruncation => map_pixels(src, out, |p| {
                let a = 200u32;
                let mut q = p;
                for c in 0..3 {
                    let pm = (p[c] as u32 * a + 127) / 255;
                    q[c] = (pm * 255 / a).min(255) as u8;
                }
                q
            }),
            RealBugId::ZenavifScaleFromU16Rounding => map_pixels(src, out, |p| {
                let mut q = p;
                for c in 0..3 {
                    if p[c] & 1 == 1 {
                        q[c] = p[c].saturating_add(1);
                    }
                }
                q
            }),
            RealBugId::ZenavifBilinearChromaEdgeNegativeWeights => {
                for y in 0..h {
                    for x in 0..w {
                        let edge = x < 2 || y < 2 || x + 2 >= w || y + 2 >= h;
                        if !edge {
                            continue;
                        }
                        let ix = if x < 2 {
                            (x + 2).min(w - 1)
                        } else if x + 2 >= w {
                            x.saturating_sub(2)
                        } else {
                            x
                        };
                        let iy = if y < 2 {
                            (y + 2).min(h - 1)
                        } else if y + 2 >= h {
                            y.saturating_sub(2)
                        } else {
                            y
                        };
                        let (yl, cb, cr) = rgb_to_ycbcr(src.get(x, y));
                        let (_, icb, icr) = rgb_to_ycbcr(src.get(ix, iy));
                        out.set(
                            x,
                            y,
                            ycbcr_to_rgb(yl, cb + 2.0 * (cb - icb), cr + 2.0 * (cr - icr)),
                        );
                    }
                }
            }
            RealBugId::ZenavifPqHlgThroughSrgb => map_pixels(src, out, |p| {
                let mut q = p;
                for c in 0..3 {
                    // Treat the sRGB code as a PQ signal: EOTF → nits, scale
                    // so 203 nits is SDR white, then clip through sRGB.
                    let lin = pq_eotf_nits(p[c] as f32 / 255.0) / 203.0;
                    q[c] = (linear_to_srgb(lin.clamp(0.0, 1.0)) * 255.0).round() as u8;
                }
                q
            }),
            RealBugId::HeicChromaMcShift4 => map_pixels(src, out, |p| {
                let (yl, cb, cr) = rgb_to_ycbcr(p);
                ycbcr_to_rgb(yl, 128.0 + (cb - 128.0) * 4.0, 128.0 + (cr - 128.0) * 4.0)
            }),
            RealBugId::HeicChromaBipredPrecision => {
                for_each_block(w, h, 16, |_, _, rect| {
                    if rng.below(2) == 0 {
                        for (x, y) in rect.pixels(w, h) {
                            let (yl, cb, cr) = rgb_to_ycbcr(src.get(x, y));
                            out.set(
                                x,
                                y,
                                ycbcr_to_rgb(
                                    yl,
                                    128.0 + (cb - 128.0) * 0.5,
                                    128.0 + (cr - 128.0) * 0.5,
                                ),
                            );
                        }
                    }
                });
            }
            RealBugId::HeicBipredRoundingOffset => {
                for_each_block(w, h, 16, |_, _, rect| {
                    if rng.below(2) == 0 {
                        for (x, y) in rect.pixels(w, h) {
                            let (yl, cb, cr) = rgb_to_ycbcr(src.get(x, y));
                            out.set(x, y, ycbcr_to_rgb(yl + 2.0, cb, cr));
                        }
                    }
                });
            }
            RealBugId::HeicDeblockSao => {
                for y in 0..h {
                    for x in 0..w {
                        let mut p = src.get(x, y);
                        // Wrong-strength deblock: the two pixels either side
                        // of every 8px boundary become their mean.
                        if x % 8 == 7 && x + 1 < w {
                            p = mean2(src.get(x, y), src.get(x + 1, y));
                        } else if x % 8 == 0 && x > 0 {
                            p = mean2(src.get(x - 1, y), src.get(x, y));
                        }
                        if y % 8 == 7 && y + 1 < h {
                            p = mean2(p, src.get(x, y + 1));
                        } else if y % 8 == 0 && y > 0 {
                            p = mean2(src.get(x, y - 1), p);
                        }
                        // SAO band offset applied to the wrong band.
                        let (yl, cb, cr) = rgb_to_ycbcr(p);
                        if (64.0..96.0).contains(&yl) {
                            p = ycbcr_to_rgb(yl + 6.0, cb, cr);
                        }
                        out.set(x, y, p);
                    }
                }
            }
            RealBugId::HeicTileBoundary => {
                let tile = 64u32;
                for y in 0..h {
                    for x in 0..w {
                        let bx = x % tile < 8 && x >= tile;
                        let by = y % tile < 8 && y >= tile;
                        if !(bx || by) {
                            continue;
                        }
                        let sx = if bx { x - 1 - 2 * (x % tile) } else { x };
                        let sy = if by { y - 1 - 2 * (y % tile) } else { y };
                        let p = src.get(sx, sy);
                        let mut q = [0u8; 3];
                        for c in 0..3 {
                            q[c] = (128.0 + (p[c] as f32 - 128.0) * 1.3)
                                .round()
                                .clamp(0.0, 255.0) as u8;
                        }
                        out.set(x, y, q);
                    }
                }
            }
            RealBugId::ImageflowTransparentToJpegMatte => {
                map_pixels(src, out, |p| [p[0] / 2 + 64, p[1] / 2 + 64, p[2] / 2 + 64])
            }
            RealBugId::ImageflowWithinConstraintTransparentBorder => {
                for y in 0..h {
                    out.set(w - 1, y, [255, 255, 255]);
                }
            }
            RealBugId::ImageflowTransparentPngBlackBackground => {
                let rect = seeded_rect(w, h, 2, rng);
                fill(out, rect, [0, 0, 0]);
            }
            RealBugId::ImageflowSwappedCanvasDims => {
                let n = w as u64 * h as u64;
                for y in 0..h {
                    for x in 0..w {
                        let i = (y as u64 * h as u64 + x as u64) % n;
                        out.set(x, y, src.get((i % w as u64) as u32, (i / w as u64) as u32));
                    }
                }
            }
            RealBugId::ImageflowExifOrientationCropRegion => {
                let (rw, rh) = ((w / 2).max(1), (h / 2).max(1));
                let rect = ((w - rw) / 2, (h - rh) / 2, rw, rh);
                for (x, y) in rect.pixels(w, h) {
                    out.set(x, y, src.get(w - 1 - x, y));
                }
            }
            RealBugId::ImageflowWebpLossyDesaturated => map_pixels(src, out, |p| {
                let (yl, cb, cr) = rgb_to_ycbcr(p);
                ycbcr_to_rgb(yl, 128.0 + (cb - 128.0) * 0.8, 128.0 + (cr - 128.0) * 0.8)
            }),
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// `(x0, y0, w, h)`.
type R = (u32, u32, u32, u32);

trait RectPixels {
    fn pixels(&self, w: u32, h: u32) -> Vec<(u32, u32)>;
}

impl RectPixels for R {
    fn pixels(&self, w: u32, h: u32) -> Vec<(u32, u32)> {
        let (x0, y0, rw, rh) = *self;
        let mut v = Vec::with_capacity((rw * rh) as usize);
        for y in y0..(y0 + rh).min(h) {
            for x in x0..(x0 + rw).min(w) {
                v.push((x, y));
            }
        }
        v
    }
}

fn for_each_block(w: u32, h: u32, size: u32, mut f: impl FnMut(u32, u32, R)) {
    for by in 0..h.div_ceil(size) {
        for bx in 0..w.div_ceil(size) {
            let (x0, y0) = (bx * size, by * size);
            f(bx, by, (x0, y0, size.min(w - x0), size.min(h - y0)));
        }
    }
}

fn map_pixels(src: &Rgb8, out: &mut Rgb8, f: impl Fn([u8; 3]) -> [u8; 3]) {
    for y in 0..src.height() {
        for x in 0..src.width() {
            out.set(x, y, f(src.get(x, y)));
        }
    }
}

fn fill(out: &mut Rgb8, rect: R, rgb: [u8; 3]) {
    for (x, y) in rect.pixels(out.width(), out.height()) {
        out.set(x, y, rgb);
    }
}

fn block_mean(src: &Rgb8, rect: R) -> [u8; 3] {
    let px = rect.pixels(src.width(), src.height());
    let mut sum = [0u64; 3];
    for &(x, y) in &px {
        let p = src.get(x, y);
        for c in 0..3 {
            sum[c] += p[c] as u64;
        }
    }
    let n = px.len().max(1) as f64;
    [
        (sum[0] as f64 / n).round() as u8,
        (sum[1] as f64 / n).round() as u8,
        (sum[2] as f64 / n).round() as u8,
    ]
}

fn mean_ycbcr(src: &Rgb8, rect: R) -> (f32, f32, f32) {
    let px = rect.pixels(src.width(), src.height());
    let (mut sy, mut scb, mut scr) = (0f32, 0f32, 0f32);
    for &(x, y) in &px {
        let (yl, cb, cr) = rgb_to_ycbcr(src.get(x, y));
        sy += yl;
        scb += cb;
        scr += cr;
    }
    let n = px.len().max(1) as f32;
    (sy / n, scb / n, scr / n)
}

fn mean2(a: [u8; 3], b: [u8; 3]) -> [u8; 3] {
    [
        (a[0] as u16 + b[0] as u16).div_ceil(2) as u8,
        (a[1] as u16 + b[1] as u16).div_ceil(2) as u8,
        (a[2] as u16 + b[2] as u16).div_ceil(2) as u8,
    ]
}

/// A `1/n`-side rectangle at a seeded offset.
fn seeded_rect(w: u32, h: u32, n: u32, rng: &mut SplitMix64) -> R {
    let (rw, rh) = ((w / n).max(1), (h / n).max(1));
    let x0 = if w > rw {
        rng.below((w - rw + 1) as u64) as u32
    } else {
        0
    };
    let y0 = if h > rh {
        rng.below((h - rh + 1) as u64) as u32
    } else {
        0
    };
    (x0, y0, rw, rh)
}

fn luma8(p: [u8; 3]) -> u8 {
    rgb_to_ycbcr(p).0.round().clamp(0.0, 255.0) as u8
}

fn with_chroma_of(luma_src: [u8; 3], chroma_src: [u8; 3]) -> [u8; 3] {
    let (yl, _, _) = rgb_to_ycbcr(luma_src);
    let (_, cb, cr) = rgb_to_ycbcr(chroma_src);
    ycbcr_to_rgb(yl, cb, cr)
}

/// Add `delta` to both chroma planes; `wrap` reproduces an overflowing u8.
fn shift_chroma(p: [u8; 3], delta: f32, wrap: bool) -> [u8; 3] {
    let (yl, cb, cr) = rgb_to_ycbcr(p);
    let adj = |c: f32| {
        let v = c + delta;
        if wrap {
            v.rem_euclid(256.0)
        } else {
            v.clamp(0.0, 255.0)
        }
    };
    ycbcr_to_rgb(yl, adj(cb), adj(cr))
}

pub(crate) fn rgb_to_ycbcr(p: [u8; 3]) -> (f32, f32, f32) {
    let (r, g, b) = (p[0] as f32, p[1] as f32, p[2] as f32);
    (
        0.299 * r + 0.587 * g + 0.114 * b,
        128.0 - 0.168_736 * r - 0.331_264 * g + 0.5 * b,
        128.0 + 0.5 * r - 0.418_688 * g - 0.081_312 * b,
    )
}

pub(crate) fn ycbcr_to_rgb(y: f32, cb: f32, cr: f32) -> [u8; 3] {
    let (cb, cr) = (cb - 128.0, cr - 128.0);
    let r = y + 1.402 * cr;
    let g = y - 0.344_136 * cb - 0.714_136 * cr;
    let b = y + 1.772 * cb;
    [
        r.round().clamp(0.0, 255.0) as u8,
        g.round().clamp(0.0, 255.0) as u8,
        b.round().clamp(0.0, 255.0) as u8,
    ]
}

fn linear_to_srgb(v: f32) -> f32 {
    if v <= 0.003_130_8 {
        v * 12.92
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    }
}

/// SMPTE ST 2084 EOTF: normalized PQ signal → absolute nits (0..=10000).
fn pq_eotf_nits(e: f32) -> f32 {
    const M1: f32 = 0.159_301_76;
    const M2: f32 = 78.84375;
    const C1: f32 = 0.835_937_5;
    const C2: f32 = 18.851_562;
    const C3: f32 = 18.6875;
    let ep = e.clamp(0.0, 1.0).powf(1.0 / M2);
    let num = (ep - C1).max(0.0);
    let den = C2 - C3 * ep;
    10_000.0 * (num / den).powf(1.0 / M1)
}

/// Run an sRGB triple through jxl's XYB → linear-RGB inverse as if the
/// channels were (X, Y, B) planes, then re-encode to sRGB.
fn xyb_inverse_of_srgb(p: [u8; 3]) -> [u8; 3] {
    const BIAS: f32 = 0.003_793_073_2;
    const CBRT_BIAS: f32 = 0.155_954_2;
    let x = (p[0] as f32 / 255.0 - 0.5) * 0.1;
    let y = p[1] as f32 / 255.0;
    let b = p[2] as f32 / 255.0;
    let gl = y + x + CBRT_BIAS;
    let gm = y - x + CBRT_BIAS;
    let gs = b + CBRT_BIAS;
    let l = gl * gl * gl - BIAS;
    let m = gm * gm * gm - BIAS;
    let s = gs * gs * gs - BIAS;
    let r = 11.031_567 * l - 9.866_944 * m - 0.164_623 * s;
    let g = -3.254_147_4 * l + 4.418_770_4 * m - 0.164_623 * s;
    let bl = -3.658_851_3 * l + 2.712_923 * m + 1.945_928_2 * s;
    let enc = |v: f32| (linear_to_srgb(v.clamp(0.0, 1.0)) * 255.0).round() as u8;
    [enc(r), enc(g), enc(bl)]
}

/// Quantize one 8×8 block's DCT-II coefficients (per RGB channel) with a
/// flat step and reconstruct it.
fn dct_quantize_block(src: &Rgb8, out: &mut Rgb8, rect: R, step: f32) {
    let (x0, y0, rw, rh) = rect;
    let (w, h) = (src.width(), src.height());
    let cosv = |k: usize, n: usize| {
        (((2 * n + 1) as f32) * (k as f32) * std::f32::consts::PI / 16.0).cos()
    };
    for c in 0..3 {
        // Edge-replicated 8x8 sample block.
        let mut block = [[0f32; 8]; 8];
        for (j, row) in block.iter_mut().enumerate() {
            for (i, v) in row.iter_mut().enumerate() {
                let x = (x0 + (i as u32).min(rw - 1)).min(w - 1);
                let y = (y0 + (j as u32).min(rh - 1)).min(h - 1);
                *v = src.get(x, y)[c] as f32 - 128.0;
            }
        }
        let mut coef = [[0f32; 8]; 8];
        for (v, crow) in coef.iter_mut().enumerate() {
            for (u, cv) in crow.iter_mut().enumerate() {
                let mut s = 0f32;
                for (j, row) in block.iter().enumerate() {
                    for (i, &val) in row.iter().enumerate() {
                        s += val * cosv(u, i) * cosv(v, j);
                    }
                }
                let cu = if u == 0 {
                    std::f32::consts::FRAC_1_SQRT_2
                } else {
                    1.0
                };
                let cvv = if v == 0 {
                    std::f32::consts::FRAC_1_SQRT_2
                } else {
                    1.0
                };
                let q = 0.25 * cu * cvv * s;
                *cv = (q / step).round() * step;
            }
        }
        for j in 0..rh.min(8) as usize {
            for i in 0..rw.min(8) as usize {
                let mut s = 0f32;
                for (v, crow) in coef.iter().enumerate() {
                    for (u, &cv) in crow.iter().enumerate() {
                        let cu = if u == 0 {
                            std::f32::consts::FRAC_1_SQRT_2
                        } else {
                            1.0
                        };
                        let cvv = if v == 0 {
                            std::f32::consts::FRAC_1_SQRT_2
                        } else {
                            1.0
                        };
                        s += cu * cvv * cv * cosv(u, i) * cosv(v, j);
                    }
                }
                let (x, y) = (x0 + i as u32, y0 + j as u32);
                let mut p = out.get(x, y);
                p[c] = (0.25 * s + 128.0).round().clamp(0.0, 255.0) as u8;
                out.set(x, y, p);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Chromatic, textured content so every pattern has something to act on.
    fn textured(w: u32, h: u32) -> Rgb8 {
        let mut img = Rgb8::filled(w, h, [0, 0, 0]);
        for y in 0..h {
            for x in 0..w {
                let k = x / 4 + (y / 4) * 7;
                img.set(
                    x,
                    y,
                    [
                        ((k * 29 + x * 3) % 256) as u8,
                        ((k * 67 + y * 5) % 256) as u8,
                        ((k * 13 + x + y) % 256) as u8,
                    ],
                );
            }
        }
        img
    }

    fn changed(a: &Rgb8, b: &Rgb8) -> usize {
        (0..a.height())
            .flat_map(|y| (0..a.width()).map(move |x| (x, y)))
            .filter(|&(x, y)| a.get(x, y) != b.get(x, y))
            .count()
    }

    #[test]
    fn table_has_every_row_of_the_issue() {
        let all = RealBugId::all();
        assert_eq!(all.len(), 41);
        let count = |repo: &str| all.iter().filter(|b| b.info().repo == repo).count();
        assert_eq!(count("zenjpeg"), 10);
        assert_eq!(count("zenwebp"), 11);
        assert_eq!(count("zengif"), 2);
        assert_eq!(count("zenpng"), 3);
        assert_eq!(count("zenavif"), 4);
        assert_eq!(count("heic"), 5);
        assert_eq!(count("imageflow"), 6);
        let mut refs: Vec<String> = all.iter().map(|b| b.source_reference()).collect();
        refs.sort();
        refs.dedup();
        assert_eq!(refs.len(), 41, "source references are unique");
        let mut slugs: Vec<String> = all.iter().map(|b| b.slug()).collect();
        slugs.sort();
        slugs.dedup();
        assert_eq!(slugs.len(), 41);
        assert_eq!(
            RealBugId::ZenjpegProgressiveAcTruncation.source_reference(),
            "real-bug:zenjpeg#08ef601"
        );
        assert_eq!(
            RealBugId::ImageflowWithinConstraintTransparentBorder.slug(),
            "imageflow_within_constraint_transparent_border"
        );
        for b in all {
            assert!(b.source_reference().starts_with("real-bug:"));
            assert!(!b.info().summary.is_empty() && !b.info().pattern.is_empty());
        }
        // Every family with a documented real-bug analogue is represented.
        for fam in [
            "channel",
            "block",
            "edge",
            "tone",
            "chroma_boundary",
            "composite",
        ] {
            assert!(
                all.iter().any(|b| b.info().family_analog == Some(fam)),
                "no real-bug analogue tagged for family {fam}"
            );
        }
    }

    #[test]
    fn every_real_bug_changes_pixels_keeps_dims_and_is_deterministic() {
        let reference = textured(96, 80);
        for &bug in RealBugId::all() {
            let mut a = reference.clone();
            bug.apply(&mut a, 1.0, &mut SplitMix64::new(7));
            let mut b = reference.clone();
            bug.apply(&mut b, 1.0, &mut SplitMix64::new(7));
            assert_eq!((a.width(), a.height()), (96, 80), "{bug:?}");
            assert_eq!(a, b, "{bug:?} is not deterministic");
            assert!(
                changed(&reference, &a) > 0,
                "{bug:?} changed zero pixels — a repro that is an identity is not a defect (#9)"
            );
        }
    }

    #[test]
    fn subtle_off_by_one_bugs_stay_subtle() {
        let reference = textured(64, 64);
        for bug in [
            RealBugId::ZenpngU16ToU8Truncation,
            RealBugId::ZenavifUnpremultiplyTruncation,
            RealBugId::ZenavifScaleFromU16Rounding,
        ] {
            let mut a = reference.clone();
            bug.apply(&mut a, 1.0, &mut SplitMix64::new(3));
            let max = (0..64)
                .flat_map(|y| (0..64).map(move |x| (x, y)))
                .map(|(x, y)| {
                    let (p, q) = (reference.get(x, y), a.get(x, y));
                    (0..3).map(|c| p[c].abs_diff(q[c])).max().unwrap()
                })
                .max()
                .unwrap();
            assert!(
                max <= 1,
                "{bug:?}: max abs diff {max}, expected an off-by-one pattern"
            );
            assert!(
                changed(&reference, &a) * 10 > 64 * 64,
                "{bug:?} should touch >10% of pixels"
            );
        }
        let mut uv = reference.clone();
        RealBugId::ZenwebpUvRoundingOffByOne.apply(&mut uv, 1.0, &mut SplitMix64::new(1));
        assert!(
            changed(&reference, &uv) * 2 > 64 * 64,
            "U/V off-by-one is systematic"
        );
    }

    #[test]
    fn exact_patterns() {
        let reference = textured(40, 24);
        let mut rng = SplitMix64::new(1);

        let mut white = reference.clone();
        RealBugId::ZenjpegXybDoubleScaleWhite.apply(&mut white, 1.0, &mut rng);
        assert_eq!(white, Rgb8::filled(40, 24, [255, 255, 255]));

        let mut planes = reference.clone();
        RealBugId::ZenjpegXybPlaneOrder.apply(&mut planes, 1.0, &mut rng);
        let p = reference.get(5, 5);
        assert_eq!(planes.get(5, 5), [p[1], p[2], p[0]]);

        let mut border = reference.clone();
        RealBugId::ImageflowWithinConstraintTransparentBorder.apply(&mut border, 1.0, &mut rng);
        assert_eq!(changed(&reference, &border), 24, "exactly the last column");
        assert_eq!(border.get(39, 3), [255, 255, 255]);
        assert_eq!(border.get(38, 3), reference.get(38, 3));

        let mut adam7 = reference.clone();
        RealBugId::ZenpngAdam7BufferTooSmall.apply(&mut adam7, 1.0, &mut rng);
        for y in 0..24 {
            for x in 0..40 {
                assert_eq!(adam7.get(x, y), reference.get(x & !1, y & !1));
            }
        }

        let mut cut = reference.clone();
        RealBugId::ZenjpegXybDcClampBitstream.apply(&mut cut, 1.0, &mut rng);
        assert_eq!(cut.get(0, 0), reference.get(0, 0));
        assert_eq!(cut.get(0, 23), [128, 128, 128]);

        let mut stride = reference.clone();
        RealBugId::ZenjpegProgressiveStrideMismatch.apply(&mut stride, 1.0, &mut rng);
        for x in 0..40 {
            assert_eq!(
                stride.get(x, 3),
                reference.get(x, 3),
                "block row 0 unshifted"
            );
            assert_eq!(
                stride.get(x, 9),
                reference.get((x + 40 - 8) % 40, 9),
                "row 1 shifted 8"
            );
        }

        let mut mirror = reference.clone();
        RealBugId::ImageflowExifOrientationCropRegion.apply(&mut mirror, 1.0, &mut rng);
        assert_eq!(mirror.get(20, 12), reference.get(19, 12));
        assert_eq!(mirror.get(0, 0), reference.get(0, 0));

        let mut shear = reference.clone();
        RealBugId::ZenwebpLosslessAlphaPaddedStride.apply(&mut shear, 1.0, &mut rng);
        // Index 40..48 of the stride-48 layout is padding: pixels (0..8, 1).
        assert_eq!(shear.get(3, 1), [255, 255, 255]);
        assert_eq!(shear.get(20, 1), reference.get(20, 1));

        let mut chroma4 = Rgb8::filled(8, 8, [140, 120, 130]);
        let before = rgb_to_ycbcr([140, 120, 130]);
        RealBugId::HeicChromaMcShift4.apply(&mut chroma4, 1.0, &mut rng);
        let after = rgb_to_ycbcr(chroma4.get(0, 0));
        assert!((after.1 - 128.0).abs() > 3.0 * (before.1 - 128.0).abs());
    }

    #[test]
    fn opacity_blends_toward_the_defect() {
        let reference = textured(32, 32);
        let mut full = reference.clone();
        RealBugId::HeicChromaMcShift4.apply(&mut full, 1.0, &mut SplitMix64::new(2));
        let mut half = reference.clone();
        RealBugId::HeicChromaMcShift4.apply(&mut half, 0.5, &mut SplitMix64::new(2));
        let (p, f, hh) = (reference.get(9, 9), full.get(9, 9), half.get(9, 9));
        for c in 0..3 {
            let lo = p[c].min(f[c]);
            let hi = p[c].max(f[c]);
            assert!(hh[c] >= lo && hh[c] <= hi, "half-opacity lies between");
        }
        assert_ne!(half, reference);
        assert_ne!(half, full);
    }

    #[test]
    fn tiny_images_do_not_panic() {
        for (w, h) in [(1, 1), (3, 2), (2, 9), (17, 1), (65, 65)] {
            let reference = textured(w, h);
            for &bug in RealBugId::all() {
                let mut img = reference.clone();
                bug.apply(&mut img, 1.0, &mut SplitMix64::new(5));
                assert_eq!((img.width(), img.height()), (w, h), "{bug:?} {w}x{h}");
            }
        }
    }

    #[test]
    fn pq_and_dct_helpers_are_sane() {
        assert!(pq_eotf_nits(0.0) < 1e-3);
        assert!((pq_eotf_nits(1.0) - 10_000.0).abs() < 1.0);
        assert!(
            (pq_eotf_nits(0.58) - 203.0).abs() < 8.0,
            "{}",
            pq_eotf_nits(0.58)
        );
        // A flat block survives DCT quantization unchanged (DC-only).
        let flat = Rgb8::filled(8, 8, [100, 150, 200]);
        let mut out = flat.clone();
        dct_quantize_block(&flat, &mut out, (0, 0, 8, 8), 8.0);
        for y in 0..8 {
            for x in 0..8 {
                let p = out.get(x, y);
                for c in 0..3 {
                    assert!(p[c].abs_diff(flat.get(x, y)[c]) <= 4, "{p:?}");
                }
            }
        }
    }
}
