//! The ten structural-corruption families.
//!
//! Each family is a deterministic, in-place mutation of an [`Rgb8`] buffer,
//! parameterized by [`Region`] and [`Severity`]. They model the *structural*
//! defects an honest encoder never produces — channel swaps, dropped blocks,
//! off-by-one edges — as opposed to the uniform softening of honest lossy
//! compression.
//!
//! Every family routes through [`Family::apply`], which resolves the region,
//! computes the corrupted pixels, and blends them at the requested opacity.

use serde::{Deserialize, Serialize};

use super::{Rect, Region, Rgb8, Severity, blend, prng::SplitMix64};

// ---------------------------------------------------------------------------
// Family-specific variants
// ---------------------------------------------------------------------------

/// Channel-corruption variant (family 1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChannelOp {
    /// `v -> 255 - v` per channel.
    Invert,
    /// RGB -> BGR (swap R and B).
    SwapRb,
    /// Swap R and G.
    SwapRg,
    /// Swap G and B.
    SwapGb,
    /// Zero the red channel.
    ZeroR,
    /// Zero the green channel.
    ZeroG,
    /// Zero the blue channel.
    ZeroB,
    /// Set the red channel to 255.
    MaxR,
}

/// Block-corruption variant (family 2). Models dropped/corrupted MCUs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockOp {
    /// Fill with black.
    Zero,
    /// Fill with mid-gray.
    Gray,
    /// Fill with deterministic garbage (random per pixel).
    Garbage,
    /// Copy from a wrong location elsewhere in the image (MCU mispredict).
    CopyWrong,
    /// Repeat the block immediately to the left (or above, if at the left edge).
    RepeatNeighbor,
}

/// Edge / border variant (family 3). Models partial-MCU edge handling and
/// padding/cropping off-by-one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeOp {
    /// Draw a `k`-pixel black border on the top edge only.
    BorderTop { k: u32 },
    /// Draw a `k`-pixel black border on all four edges.
    BorderAll { k: u32 },
    /// Shift the entire interior 1px horizontally (off-by-one crop), leaving a
    /// duplicated column at the seam.
    ShiftInterior1px,
    /// Duplicate the top row downward by one (dropped/duplicated edge row).
    DuplicateTopRow,
}

/// Salt-and-pepper / bit-error variant (family 4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoiseOp {
    /// Set `count` random pixels to black or white (salt & pepper).
    SaltPepper { count: u32 },
    /// Flip a single (high) bit in a channel of `count` random pixels.
    BitFlip { count: u32 },
}

/// Local tone / gamma variant (family 5). Models a region-confined
/// color-management bug.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToneOp {
    /// Apply sRGB encode gamma where it shouldn't be (treat linear as sRGB).
    GammaEncode,
    /// Apply sRGB decode gamma (treat sRGB as linear).
    GammaDecode,
    /// Boost local contrast ×1.5 around the region mean.
    ContrastBoost,
    /// Add a fixed brightness offset (`delta`, signed, in 0..=255 magnitude).
    Brightness { delta: i32 },
}

/// Low-opacity overlay shape (family 6). Models a render leak / watermark bleed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OverlayShape {
    /// A filled rectangle.
    Rect,
    /// A diagonal line across the region.
    Line,
    /// A plus/cross glyph centered in the region.
    Glyph,
}

/// Background-compositing variant (family 10). Models alpha-handling bugs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompositeOp {
    /// Treat premultiplied alpha as straight: divide RGB by a synthetic alpha
    /// derived from luma (brightens / clips where alpha is small).
    PremulAsStraight,
    /// Composite over black using a synthetic alpha (darkens edges).
    WrongBgBlack,
    /// Composite over white using a synthetic alpha (lightens edges).
    WrongBgWhite,
}

// ---------------------------------------------------------------------------
// Family enum
// ---------------------------------------------------------------------------

/// One of the ten structural-corruption families, carrying its variant.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "family", rename_all = "snake_case")]
pub enum Family {
    /// 1. Channel corruption (invert / swap / zero a plane) in a rectangle.
    Channel(ChannelOp),
    /// 2. Block corruption (zero / gray / garbage / copy / repeat).
    Block(BlockOp),
    /// 3. Edge / border artifacts (k-px border, 1px interior shift, dup row).
    Edge(EdgeOp),
    /// 4. Salt-and-pepper / bit errors.
    Noise(NoiseOp),
    /// 5. Local tone / gamma confined to a region.
    Tone(ToneOp),
    /// 6. Low-opacity overlay shape.
    Overlay(OverlayShape),
    /// 7. Chroma-boundary mismatch (wrong-phase chroma upsample at block edges).
    ChromaBoundary,
    /// 8. Aliasing / moiré (nearest-neighbor down-then-up resample).
    Aliasing,
    /// 9. Geometric (1px shift / region flip / small rotate).
    Geometric(GeometricOp),
    /// 10. Wrong-background compositing.
    Composite(CompositeOp),
}

/// Geometric variant (family 9).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GeometricOp {
    /// Translate the region 1px to the right.
    Shift1px,
    /// Horizontally flip the region.
    FlipH,
    /// Vertically flip the region.
    FlipV,
    /// Rotate the region 90° (square regions; otherwise transpose-and-clip).
    Rotate90,
}

impl Family {
    /// A short stable slug for manifest ids / file names. Parameterized
    /// variants (border width `k`, noise `count`, brightness `delta`) encode
    /// their parameter so slugs stay unique across the catalog.
    pub fn slug(&self) -> String {
        match self {
            Family::Channel(op) => format!("channel_{}", variant_slug(op)),
            Family::Block(op) => format!("block_{}", variant_slug(op)),
            Family::Edge(op) => {
                let v = match op {
                    EdgeOp::BorderTop { k } => format!("border_top_k{k}"),
                    EdgeOp::BorderAll { k } => format!("border_all_k{k}"),
                    EdgeOp::ShiftInterior1px => "shift_interior1px".to_string(),
                    EdgeOp::DuplicateTopRow => "duplicate_top_row".to_string(),
                };
                format!("edge_{v}")
            }
            Family::Noise(op) => {
                let v = match op {
                    NoiseOp::SaltPepper { count } => format!("salt_pepper_n{count}"),
                    NoiseOp::BitFlip { count } => format!("bit_flip_n{count}"),
                };
                format!("noise_{v}")
            }
            Family::Tone(op) => {
                let v = match op {
                    ToneOp::GammaEncode => "gamma_encode".to_string(),
                    ToneOp::GammaDecode => "gamma_decode".to_string(),
                    ToneOp::ContrastBoost => "contrast_boost".to_string(),
                    ToneOp::Brightness { delta } => {
                        format!("brightness_d{}", delta.unsigned_abs())
                            + if *delta < 0 { "neg" } else { "pos" }
                    }
                };
                format!("tone_{v}")
            }
            Family::Overlay(op) => format!("overlay_{}", variant_slug(op)),
            Family::ChromaBoundary => "chroma_boundary".to_string(),
            Family::Aliasing => "aliasing".to_string(),
            Family::Geometric(op) => format!("geometric_{}", variant_slug(op)),
            Family::Composite(op) => format!("composite_{}", variant_slug(op)),
        }
    }

    /// The top-level family name (without the variant).
    pub fn name(&self) -> &'static str {
        match self {
            Family::Channel(_) => "channel",
            Family::Block(_) => "block",
            Family::Edge(_) => "edge",
            Family::Noise(_) => "noise",
            Family::Tone(_) => "tone",
            Family::Overlay(_) => "overlay",
            Family::ChromaBoundary => "chroma_boundary",
            Family::Aliasing => "aliasing",
            Family::Geometric(_) => "geometric",
            Family::Composite(_) => "composite",
        }
    }

    /// Whether this family destroys structure so thoroughly that, at a large
    /// opaque region, the calibrated metric score is expected to go negative.
    pub fn is_structure_destroying(&self) -> bool {
        matches!(
            self,
            Family::Channel(_) | Family::Block(_) | Family::Aliasing
        )
    }

    /// Apply this family to `img` in place.
    pub fn apply(&self, img: &mut Rgb8, region: Region, severity: Severity, rng: &mut SplitMix64) {
        let opacity = severity.opacity();
        let rect = region.resolve(img.width(), img.height(), rng);
        match self {
            Family::Channel(op) => apply_channel(img, rect, *op, opacity),
            Family::Block(op) => apply_block(img, rect, *op, opacity, rng),
            Family::Edge(op) => apply_edge(img, *op, opacity),
            Family::Noise(op) => apply_noise(img, rect, *op, opacity, rng),
            Family::Tone(op) => apply_tone(img, rect, *op, opacity),
            Family::Overlay(op) => apply_overlay(img, rect, *op, opacity),
            Family::ChromaBoundary => apply_chroma_boundary(img, rect, opacity),
            Family::Aliasing => apply_aliasing(img, rect, opacity),
            Family::Geometric(op) => apply_geometric(img, rect, *op, opacity),
            Family::Composite(op) => apply_composite(img, rect, *op, opacity),
        }
    }
}

/// Slug for a `Debug`-printable variant (snake_cases the enum variant name and
/// strips struct-field detail). Stable for manifest ids.
fn variant_slug<T: std::fmt::Debug>(v: &T) -> String {
    let dbg = format!("{v:?}");
    // Take the identifier up to the first space / brace / paren.
    let head: String = dbg.chars().take_while(|c| c.is_alphanumeric()).collect();
    to_snake(&head)
}

fn to_snake(camel: &str) -> String {
    let mut out = String::with_capacity(camel.len() + 4);
    for (i, ch) in camel.chars().enumerate() {
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

// ---------------------------------------------------------------------------
// Family 1: channel corruption
// ---------------------------------------------------------------------------

fn apply_channel(img: &mut Rgb8, rect: Rect, op: ChannelOp, opacity: f32) {
    let (w, h) = (img.width(), img.height());
    rect.for_each_pixel(w, h, |x, y| {
        let p = img.get(x, y);
        let c = match op {
            ChannelOp::Invert => [255 - p[0], 255 - p[1], 255 - p[2]],
            ChannelOp::SwapRb => [p[2], p[1], p[0]],
            ChannelOp::SwapRg => [p[1], p[0], p[2]],
            ChannelOp::SwapGb => [p[0], p[2], p[1]],
            ChannelOp::ZeroR => [0, p[1], p[2]],
            ChannelOp::ZeroG => [p[0], 0, p[2]],
            ChannelOp::ZeroB => [p[0], p[1], 0],
            ChannelOp::MaxR => [255, p[1], p[2]],
        };
        img.set(x, y, blend(p, c, opacity));
    });
}

// ---------------------------------------------------------------------------
// Family 2: block corruption
// ---------------------------------------------------------------------------

fn apply_block(img: &mut Rgb8, rect: Rect, op: BlockOp, opacity: f32, rng: &mut SplitMix64) {
    let (w, h) = (img.width(), img.height());
    match op {
        BlockOp::Zero | BlockOp::Gray => {
            let fill = if matches!(op, BlockOp::Zero) {
                [0, 0, 0]
            } else {
                [128, 128, 128]
            };
            rect.for_each_pixel(w, h, |x, y| {
                let p = img.get(x, y);
                img.set(x, y, blend(p, fill, opacity));
            });
        }
        BlockOp::Garbage => {
            // Pre-generate garbage so the RNG draw order is independent of the
            // blend (deterministic regardless of opacity).
            let mut bytes = Vec::new();
            rect.for_each_pixel(w, h, |_x, _y| {
                bytes.push([rng.next_u8(), rng.next_u8(), rng.next_u8()]);
            });
            let mut i = 0;
            rect.for_each_pixel(w, h, |x, y| {
                let p = img.get(x, y);
                img.set(x, y, blend(p, bytes[i], opacity));
                i += 1;
            });
        }
        BlockOp::CopyWrong => {
            // Copy from a deterministically-chosen wrong source offset.
            let src_dx = (rng.below((w as u64).max(1)) as i64 - (w as i64 / 2)) as i32;
            let src_dy = (rng.below((h as u64).max(1)) as i64 - (h as i64 / 2)) as i32;
            // Read the whole source region first (avoid read-after-write).
            let mut src = Vec::new();
            rect.for_each_pixel(w, h, |x, y| {
                let sx = (x as i32 + src_dx).rem_euclid(w as i32) as u32;
                let sy = (y as i32 + src_dy).rem_euclid(h as i32) as u32;
                src.push(img.get(sx, sy));
            });
            let mut i = 0;
            rect.for_each_pixel(w, h, |x, y| {
                let p = img.get(x, y);
                img.set(x, y, blend(p, src[i], opacity));
                i += 1;
            });
        }
        BlockOp::RepeatNeighbor => {
            // Copy the adjacent block of the same size (see
            // `repeat_neighbor_source`). Read the whole source first so the
            // copy never reads pixels it has already overwritten.
            let mut src = Vec::new();
            rect.for_each_pixel(w, h, |x, y| {
                let (sx, sy) = repeat_neighbor_source(rect, w, h, x, y);
                src.push(img.get(sx, sy));
            });
            let mut i = 0;
            rect.for_each_pixel(w, h, |x, y| {
                let p = img.get(x, y);
                img.set(x, y, blend(p, src[i], opacity));
                i += 1;
            });
        }
    }
}

/// Source pixel for `BlockOp::RepeatNeighbor` at `(x, y)` inside `rect`.
///
/// The neighbor is the adjacent block of the same size: to the left when
/// there is room for one, else to the right; failing that, above, else below
/// (wrapping at the image edge). When the rect spans the whole image on both
/// axes there is no neighbor block at all, so the op models a decoder stuck
/// re-emitting the first MCU of each row: every 8-px column band repeats the
/// first band.
///
/// The previous version always picked "above" for a whole-image rect, which
/// resolved to `(y - H).rem_euclid(H) == y` — an exact identity
/// (imazen/codec-corpus#9).
fn repeat_neighbor_source(rect: Rect, w: u32, h: u32, x: u32, y: u32) -> (u32, u32) {
    if rect.w < w {
        let dx = if rect.x0 >= rect.w {
            -(rect.w as i32)
        } else {
            rect.w as i32
        };
        ((x as i32 + dx).rem_euclid(w as i32) as u32, y)
    } else if rect.h < h {
        let dy = if rect.y0 >= rect.h {
            -(rect.h as i32)
        } else {
            rect.h as i32
        };
        (x, (y as i32 + dy).rem_euclid(h as i32) as u32)
    } else {
        (x % 8, y)
    }
}

// ---------------------------------------------------------------------------
// Family 3: edge / border artifacts
// ---------------------------------------------------------------------------

fn apply_edge(img: &mut Rgb8, op: EdgeOp, opacity: f32) {
    let (w, h) = (img.width(), img.height());
    match op {
        EdgeOp::BorderTop { k } => {
            let k = k.min(h);
            for y in 0..k {
                for x in 0..w {
                    let p = img.get(x, y);
                    img.set(x, y, blend(p, [0, 0, 0], opacity));
                }
            }
        }
        EdgeOp::BorderAll { k } => {
            let kx = k.min(w);
            let ky = k.min(h);
            for y in 0..h {
                for x in 0..w {
                    let on_edge = x < kx || x >= w - kx || y < ky || y >= h - ky;
                    if on_edge {
                        let p = img.get(x, y);
                        img.set(x, y, blend(p, [0, 0, 0], opacity));
                    }
                }
            }
        }
        EdgeOp::ShiftInterior1px => {
            // Shift everything 1px right; duplicate column 0 into column 0 stays,
            // last column gets dropped. Read a full snapshot first.
            if w < 2 {
                return;
            }
            let snapshot = img.clone();
            for y in 0..h {
                for x in 0..w {
                    let sx = if x == 0 { 0 } else { x - 1 };
                    let shifted = snapshot.get(sx, y);
                    let p = img.get(x, y);
                    img.set(x, y, blend(p, shifted, opacity));
                }
            }
        }
        EdgeOp::DuplicateTopRow => {
            if h < 2 {
                return;
            }
            // Shift all rows down by 1; row 0 duplicates into row 1's slot.
            let snapshot = img.clone();
            for y in 0..h {
                let sy = if y == 0 { 0 } else { y - 1 };
                for x in 0..w {
                    let shifted = snapshot.get(x, sy);
                    let p = img.get(x, y);
                    img.set(x, y, blend(p, shifted, opacity));
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Family 4: salt-and-pepper / bit errors
// ---------------------------------------------------------------------------

fn apply_noise(img: &mut Rgb8, rect: Rect, op: NoiseOp, opacity: f32, rng: &mut SplitMix64) {
    let (w, h) = (img.width(), img.height());
    match op {
        NoiseOp::SaltPepper { count } => {
            let x_end = (rect.x0 + rect.w).min(w);
            let y_end = (rect.y0 + rect.h).min(h);
            let rw = x_end.saturating_sub(rect.x0).max(1);
            let rh = y_end.saturating_sub(rect.y0).max(1);
            for _ in 0..count {
                let x = rect.x0 + rng.below(rw as u64) as u32;
                let y = rect.y0 + rng.below(rh as u64) as u32;
                let val = if rng.next_u64() & 1 == 0 { 0 } else { 255 };
                let p = img.get(x, y);
                img.set(x, y, blend(p, [val, val, val], opacity));
            }
        }
        NoiseOp::BitFlip { count } => {
            let x_end = (rect.x0 + rect.w).min(w);
            let y_end = (rect.y0 + rect.h).min(h);
            let rw = x_end.saturating_sub(rect.x0).max(1);
            let rh = y_end.saturating_sub(rect.y0).max(1);
            for _ in 0..count {
                let x = rect.x0 + rng.below(rw as u64) as u32;
                let y = rect.y0 + rng.below(rh as u64) as u32;
                let ch = (rng.next_u64() % 3) as usize;
                let bit = 1u8 << (4 + (rng.next_u64() % 4) as u8); // a high-ish bit
                let mut p = img.get(x, y);
                let flipped = p[ch] ^ bit;
                let mut corrupt = p;
                corrupt[ch] = flipped;
                p = blend(p, corrupt, opacity);
                img.set(x, y, p);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Family 5: local tone / gamma
// ---------------------------------------------------------------------------

fn srgb_encode(v: u8) -> u8 {
    let l = v as f32 / 255.0;
    let e = if l <= 0.003_130_8 {
        12.92 * l
    } else {
        1.055 * l.powf(1.0 / 2.4) - 0.055
    };
    (e.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn srgb_decode(v: u8) -> u8 {
    let e = v as f32 / 255.0;
    let l = if e <= 0.040_45 {
        e / 12.92
    } else {
        ((e + 0.055) / 1.055).powf(2.4)
    };
    (l.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn apply_tone(img: &mut Rgb8, rect: Rect, op: ToneOp, opacity: f32) {
    let (w, h) = (img.width(), img.height());
    // Region mean for contrast boost.
    let mean = if matches!(op, ToneOp::ContrastBoost) {
        let mut sum = [0u64; 3];
        let mut n = 0u64;
        rect.for_each_pixel(w, h, |x, y| {
            let p = img.get(x, y);
            for c in 0..3 {
                sum[c] += p[c] as u64;
            }
            n += 1;
        });
        let n = n.max(1);
        [
            (sum[0] / n) as f32,
            (sum[1] / n) as f32,
            (sum[2] / n) as f32,
        ]
    } else {
        [0.0; 3]
    };

    rect.for_each_pixel(w, h, |x, y| {
        let p = img.get(x, y);
        let c = match op {
            ToneOp::GammaEncode => [srgb_encode(p[0]), srgb_encode(p[1]), srgb_encode(p[2])],
            ToneOp::GammaDecode => [srgb_decode(p[0]), srgb_decode(p[1]), srgb_decode(p[2])],
            ToneOp::ContrastBoost => {
                let mut out = [0u8; 3];
                for ch in 0..3 {
                    let v = (mean[ch] + (p[ch] as f32 - mean[ch]) * 1.5).round();
                    out[ch] = v.clamp(0.0, 255.0) as u8;
                }
                out
            }
            ToneOp::Brightness { delta } => {
                let mut out = [0u8; 3];
                for ch in 0..3 {
                    out[ch] = (p[ch] as i32 + delta).clamp(0, 255) as u8;
                }
                out
            }
        };
        img.set(x, y, blend(p, c, opacity));
    });
}

// ---------------------------------------------------------------------------
// Family 6: low-opacity overlay
// ---------------------------------------------------------------------------

fn apply_overlay(img: &mut Rgb8, rect: Rect, shape: OverlayShape, opacity: f32) {
    let (w, h) = (img.width(), img.height());
    // Overlays are deliberately drawn at the given (low) opacity; a fully-opaque
    // overlay collapses to a solid shape, which is still valid.
    let color = [255u8, 0, 0]; // a leak / watermark color
    let x_end = (rect.x0 + rect.w).min(w);
    let y_end = (rect.y0 + rect.h).min(h);
    match shape {
        OverlayShape::Rect => {
            rect.for_each_pixel(w, h, |x, y| {
                let p = img.get(x, y);
                img.set(x, y, blend(p, color, opacity));
            });
        }
        OverlayShape::Line => {
            // Diagonal from (x0,y0) to (x_end, y_end).
            let span = (x_end.saturating_sub(rect.x0)).max(y_end.saturating_sub(rect.y0));
            for t in 0..span {
                let x = rect.x0 + t * (x_end.saturating_sub(rect.x0)).max(1) / span.max(1);
                let y = rect.y0 + t * (y_end.saturating_sub(rect.y0)).max(1) / span.max(1);
                if x < w && y < h {
                    let p = img.get(x, y);
                    img.set(x, y, blend(p, color, opacity));
                }
            }
        }
        OverlayShape::Glyph => {
            // A plus/cross through the region center.
            let cx = (rect.x0 + x_end) / 2;
            let cy = (rect.y0 + y_end) / 2;
            for x in rect.x0..x_end {
                if cy < h {
                    let p = img.get(x, cy);
                    img.set(x, cy, blend(p, color, opacity));
                }
            }
            for y in rect.y0..y_end {
                if cx < w {
                    let p = img.get(cx, y);
                    img.set(cx, y, blend(p, color, opacity));
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Family 7: chroma-boundary mismatch
// ---------------------------------------------------------------------------

/// Model a chroma upsampler that mishandles 8x8 block boundaries: in the
/// 1-px bands on either side of every block edge — **columns and rows** — a
/// pixel keeps its own luma but takes the chroma of the same position in the
/// adjacent block, one full block (8 px) across the boundary. This is the
/// wrong-block / edge-replicated chroma a decoder produces when it reuses the
/// previous MCU's chroma row instead of interpolating into the next one (the
/// zenjpeg `bd0f8d7` bottom-boundary bug is exactly this on the row axis).
///
/// The previous version only touched column bands and took chroma from the
/// 1-px neighbor, which is an identity wherever chroma varies slowly — i.e.
/// on essentially all real content — and was measured at zero changed pixels
/// for every localized region (imazen/codec-corpus#9). Chroma is still a
/// no-op on achromatic content by nature; use
/// [`CorruptionParams::is_identity_on`](super::CorruptionParams::is_identity_on)
/// / [`manifest_for_image`](super::manifest_for_image) to drop such entries.
fn apply_chroma_boundary(img: &mut Rgb8, rect: Rect, opacity: f32) {
    let (w, h) = (img.width(), img.height());
    let snapshot = img.clone();
    rect.for_each_pixel(w, h, |x, y| {
        let sx = across_block_boundary(x, w);
        let sy = across_block_boundary(y, h);
        if sx == x && sy == y {
            return;
        }
        let here = snapshot.get(x, y);
        let neigh = snapshot.get(sx, sy);
        // Keep this pixel's luma, take the neighbor block's chroma (BT.601).
        let (yl, _, _) = rgb_to_ycbcr(here);
        let (_, cb, cr) = rgb_to_ycbcr(neigh);
        let c = ycbcr_to_rgb(yl, cb, cr);
        img.set(x, y, blend(here, c, opacity));
    });
}

/// For a coordinate in an 8-px block-boundary band (`v % 8` is 0 or 7), the
/// same position in the block across that boundary (8 px away, mirrored when
/// that would leave the image); otherwise `v` itself.
fn across_block_boundary(v: u32, len: u32) -> u32 {
    match v % 8 {
        0 => {
            if v >= 8 {
                v - 8
            } else {
                (v + 8).min(len.saturating_sub(1))
            }
        }
        7 => {
            if v + 8 < len {
                v + 8
            } else {
                v.saturating_sub(8)
            }
        }
        _ => v,
    }
}

fn rgb_to_ycbcr(p: [u8; 3]) -> (f32, f32, f32) {
    let (r, g, b) = (p[0] as f32, p[1] as f32, p[2] as f32);
    let y = 0.299 * r + 0.587 * g + 0.114 * b;
    let cb = 128.0 - 0.168_736 * r - 0.331_264 * g + 0.5 * b;
    let cr = 128.0 + 0.5 * r - 0.418_688 * g - 0.081_312 * b;
    (y, cb, cr)
}

fn ycbcr_to_rgb(y: f32, cb: f32, cr: f32) -> [u8; 3] {
    let r = y + 1.402 * (cr - 128.0);
    let g = y - 0.344_136 * (cb - 128.0) - 0.714_136 * (cr - 128.0);
    let b = y + 1.772 * (cb - 128.0);
    [
        r.round().clamp(0.0, 255.0) as u8,
        g.round().clamp(0.0, 255.0) as u8,
        b.round().clamp(0.0, 255.0) as u8,
    ]
}

// ---------------------------------------------------------------------------
// Family 8: aliasing / moiré
// ---------------------------------------------------------------------------

/// Nearest-neighbor downscale by 2x then upscale by 2x (within the region) —
/// destroys high-frequency detail the way a buggy resampler would, distinct
/// from honest lossy blur.
fn apply_aliasing(img: &mut Rgb8, rect: Rect, opacity: f32) {
    let (w, h) = (img.width(), img.height());
    let snapshot = img.clone();
    rect.for_each_pixel(w, h, |x, y| {
        // NN downscale: map to the even-pixel grid, then read that sample.
        let sx = (x & !1).min(w - 1);
        let sy = (y & !1).min(h - 1);
        let c = snapshot.get(sx, sy);
        let p = img.get(x, y);
        img.set(x, y, blend(p, c, opacity));
    });
}

// ---------------------------------------------------------------------------
// Family 9: geometric
// ---------------------------------------------------------------------------

fn apply_geometric(img: &mut Rgb8, rect: Rect, op: GeometricOp, opacity: f32) {
    let (w, h) = (img.width(), img.height());
    let snapshot = img.clone();
    let x_end = (rect.x0 + rect.w).min(w);
    let y_end = (rect.y0 + rect.h).min(h);
    let rw = x_end.saturating_sub(rect.x0);
    let rh = y_end.saturating_sub(rect.y0);
    if rw == 0 || rh == 0 {
        return;
    }
    for y in rect.y0..y_end {
        for x in rect.x0..x_end {
            let (sx, sy) = match op {
                GeometricOp::Shift1px => {
                    // Source is 1px to the left within the region (wrap at edge).
                    let sx = if x == rect.x0 { x } else { x - 1 };
                    (sx, y)
                }
                GeometricOp::FlipH => (rect.x0 + (x_end - 1 - x), y),
                GeometricOp::FlipV => (x, rect.y0 + (y_end - 1 - y)),
                GeometricOp::Rotate90 => {
                    // Rotate within the square sub-region; for non-square clip to
                    // the min side so the mapping stays in-bounds.
                    let s = rw.min(rh);
                    let lx = x - rect.x0;
                    let ly = y - rect.y0;
                    if lx >= s || ly >= s {
                        (x, y) // leave the non-square remainder untouched
                    } else {
                        // (lx, ly) <- (ly, s-1-lx)
                        (rect.x0 + ly, rect.y0 + (s - 1 - lx))
                    }
                }
            };
            let c = snapshot.get(sx, sy);
            let p = img.get(x, y);
            img.set(x, y, blend(p, c, opacity));
        }
    }
}

// ---------------------------------------------------------------------------
// Family 10: wrong-background compositing
// ---------------------------------------------------------------------------

fn apply_composite(img: &mut Rgb8, rect: Rect, op: CompositeOp, opacity: f32) {
    let (w, h) = (img.width(), img.height());
    rect.for_each_pixel(w, h, |x, y| {
        let p = img.get(x, y);
        // Synthesize an alpha from luma so the bug has something to act on
        // (mirrors an RGBA buffer where alpha correlates with brightness).
        let (yl, _, _) = rgb_to_ycbcr(p);
        let a = (yl / 255.0).clamp(0.02, 1.0);
        let c = match op {
            CompositeOp::PremulAsStraight => {
                // Treat the stored RGB as premultiplied and "un-premultiply" by
                // dividing by alpha — over-brightens where alpha is small.
                let mut out = [0u8; 3];
                for ch in 0..3 {
                    out[ch] = ((p[ch] as f32 / a).round()).clamp(0.0, 255.0) as u8;
                }
                out
            }
            CompositeOp::WrongBgBlack => {
                // Composite premultiplied-style over black: RGB stays, but edges
                // (low alpha) darken toward 0.
                let mut out = [0u8; 3];
                for ch in 0..3 {
                    out[ch] = (p[ch] as f32 * a).round().clamp(0.0, 255.0) as u8;
                }
                out
            }
            CompositeOp::WrongBgWhite => {
                // Composite over white: low-alpha edges lighten toward 255.
                let mut out = [0u8; 3];
                for ch in 0..3 {
                    let v = p[ch] as f32 * a + 255.0 * (1.0 - a);
                    out[ch] = v.round().clamp(0.0, 255.0) as u8;
                }
                out
            }
        };
        img.set(x, y, blend(p, c, opacity));
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gradient(w: u32, h: u32) -> Rgb8 {
        let mut data = Vec::with_capacity((w * h * 3) as usize);
        for y in 0..h {
            for x in 0..w {
                data.push((x * 255 / w.max(1)) as u8);
                data.push((y * 255 / h.max(1)) as u8);
                data.push(((x + y) * 255 / (w + h).max(1)) as u8);
            }
        }
        Rgb8::from_raw(w, h, data)
    }

    #[test]
    fn to_snake_basic() {
        assert_eq!(to_snake("SwapRb"), "swap_rb");
        assert_eq!(to_snake("Invert"), "invert");
        assert_eq!(to_snake("ZeroR"), "zero_r");
    }

    #[test]
    fn slugs_are_stable() {
        assert_eq!(Family::Channel(ChannelOp::SwapRb).slug(), "channel_swap_rb");
        assert_eq!(Family::Block(BlockOp::Garbage).slug(), "block_garbage");
        assert_eq!(Family::ChromaBoundary.slug(), "chroma_boundary");
        assert_eq!(Family::Aliasing.slug(), "aliasing");
        assert_eq!(
            Family::Geometric(GeometricOp::FlipH).slug(),
            "geometric_flip_h"
        );
    }

    /// Every family must (a) change the image and (b) preserve dimensions.
    #[test]
    fn every_family_changes_pixels_and_keeps_dims() {
        let families = sample_families();
        for fam in families {
            let orig = gradient(64, 64);
            let mut img = orig.clone();
            let mut rng = SplitMix64::new(99);
            fam.apply(&mut img, Region::Whole, Severity::Opaque, &mut rng);
            assert_eq!(img.width(), 64);
            assert_eq!(img.height(), 64);
            assert_ne!(img, orig, "family {} did not change any pixels", fam.slug());
        }
    }

    /// Determinism: same seed + params → identical bytes.
    #[test]
    fn deterministic_per_family() {
        for fam in sample_families() {
            let mut a = gradient(48, 40);
            let mut b = gradient(48, 40);
            let mut ra = SplitMix64::new(7);
            let mut rb = SplitMix64::new(7);
            fam.apply(&mut a, Region::Fraction(2), Severity::Opacity(0.5), &mut ra);
            fam.apply(&mut b, Region::Fraction(2), Severity::Opacity(0.5), &mut rb);
            assert_eq!(a, b, "family {} not deterministic", fam.slug());
        }
    }

    /// Opacity 0 must be a no-op for blend-based families.
    #[test]
    fn opacity_zero_is_noop_for_blend_families() {
        // Noise picks pixels with the RNG but blends at opacity 0 → no change.
        for fam in sample_families() {
            let orig = gradient(32, 32);
            let mut img = orig.clone();
            let mut rng = SplitMix64::new(3);
            fam.apply(&mut img, Region::Whole, Severity::Opacity(0.0), &mut rng);
            assert_eq!(
                img,
                orig,
                "family {} changed pixels at opacity 0",
                fam.slug()
            );
        }
    }

    /// Tiny images must not panic across all families / regions.
    #[test]
    fn tiny_images_do_not_panic() {
        for fam in sample_families() {
            for (w, h) in [(1, 1), (2, 1), (1, 2), (3, 3), (8, 8)] {
                for region in [Region::Whole, Region::Square(1), Region::Fraction(4)] {
                    let mut img = gradient(w, h);
                    let mut rng = SplitMix64::new(11);
                    fam.apply(&mut img, region, Severity::Opaque, &mut rng);
                    assert_eq!(img.width(), w);
                    assert_eq!(img.height(), h);
                }
            }
        }
    }

    /// A representative variant of each of the 10 families.
    fn sample_families() -> Vec<Family> {
        vec![
            Family::Channel(ChannelOp::SwapRb),
            Family::Block(BlockOp::Garbage),
            Family::Edge(EdgeOp::ShiftInterior1px),
            Family::Noise(NoiseOp::SaltPepper { count: 20 }),
            Family::Tone(ToneOp::ContrastBoost),
            Family::Overlay(OverlayShape::Rect),
            Family::ChromaBoundary,
            Family::Aliasing,
            Family::Geometric(GeometricOp::FlipH),
            Family::Composite(CompositeOp::WrongBgWhite),
        ]
    }

    #[test]
    fn channel_invert_is_exact() {
        let mut img = Rgb8::filled(4, 4, [10, 20, 250]);
        let mut rng = SplitMix64::new(0);
        Family::Channel(ChannelOp::Invert).apply(
            &mut img,
            Region::Whole,
            Severity::Opaque,
            &mut rng,
        );
        assert_eq!(img.get(0, 0), [245, 235, 5]);
    }

    #[test]
    fn block_zero_fills_region() {
        let mut img = Rgb8::filled(16, 16, [200, 200, 200]);
        let mut rng = SplitMix64::new(0);
        Family::Block(BlockOp::Zero).apply(&mut img, Region::Whole, Severity::Opaque, &mut rng);
        assert_eq!(img.get(8, 8), [0, 0, 0]);
    }

    // -----------------------------------------------------------------------
    // Regression tests for imazen/codec-corpus#9: chroma_boundary and
    // block_repeat_neighbor must not be identities on content that has
    // something for them to act on.
    // -----------------------------------------------------------------------

    /// Number of pixels that differ between two same-sized images.
    fn changed_pixels(a: &Rgb8, b: &Rgb8) -> usize {
        let mut n = 0;
        for y in 0..a.height() {
            for x in 0..a.width() {
                if a.get(x, y) != b.get(x, y) {
                    n += 1;
                }
            }
        }
        n
    }

    /// Every 8x8 block has its own distinct, saturated color.
    fn block_colored(w: u32, h: u32) -> Rgb8 {
        let mut img = Rgb8::filled(w, h, [0, 0, 0]);
        for y in 0..h {
            for x in 0..w {
                let (bx, by) = (x / 8, y / 8);
                let k = bx + by * w.div_ceil(8);
                img.set(
                    x,
                    y,
                    [
                        (40 + (k * 37) % 200) as u8,
                        (30 + (k * 91) % 200) as u8,
                        (20 + (k * 53) % 200) as u8,
                    ],
                );
            }
        }
        img
    }

    /// Horizontal stripes 8 px tall, each a distinct color; constant along x.
    /// Chroma changes only across rows, so any op that samples horizontal
    /// neighbors only is an identity here.
    fn hstripes(w: u32, h: u32) -> Rgb8 {
        let mut img = Rgb8::filled(w, h, [0, 0, 0]);
        for y in 0..h {
            let band = y / 8;
            let c = [
                200,
                (40 + (band % 5) * 30) as u8,
                (60 + (band % 3) * 40) as u8,
            ];
            for x in 0..w {
                img.set(x, y, c);
            }
        }
        img
    }

    #[test]
    fn block_repeat_neighbor_whole_image_is_not_identity() {
        // Issue #9: a whole-image rect took the `above` branch, which wrapped
        // to the pixel's own row — zero changed pixels on every image.
        let orig = block_colored(64, 64);
        let mut img = orig.clone();
        let mut rng = SplitMix64::new(5);
        Family::Block(BlockOp::RepeatNeighbor).apply(
            &mut img,
            Region::Whole,
            Severity::Opaque,
            &mut rng,
        );
        let changed = changed_pixels(&orig, &img);
        assert!(
            changed >= 64 * 64 / 2,
            "whole-image repeat_neighbor changed only {changed} of {} pixels",
            64 * 64
        );
        // Every 8-px column band now repeats the first band of its row.
        for y in 0..64 {
            for x in 0..64 {
                assert_eq!(img.get(x, y), orig.get(x % 8, y), "at ({x},{y})");
            }
        }
    }

    #[test]
    fn block_repeat_neighbor_copies_the_adjacent_block() {
        let orig = block_colored(64, 64);
        for seed in 0..16u64 {
            for region in [Region::Square(8), Region::Square(16), Region::Fraction(2)] {
                let mut img = orig.clone();
                let mut rng = SplitMix64::new(seed);
                Family::Block(BlockOp::RepeatNeighbor).apply(
                    &mut img,
                    region,
                    Severity::Opaque,
                    &mut rng,
                );
                // `resolve` is the first RNG draw inside `apply`, so replaying
                // it from the same seed yields the rect that was corrupted.
                let rect = region.resolve(64, 64, &mut SplitMix64::new(seed));
                let mut changed = 0;
                rect.for_each_pixel(64, 64, |x, y| {
                    let (sx, sy) = repeat_neighbor_source(rect, 64, 64, x, y);
                    assert_ne!((sx, sy), (x, y), "source must be a different pixel");
                    assert_eq!(img.get(x, y), orig.get(sx, sy), "at ({x},{y})");
                    if img.get(x, y) != orig.get(x, y) {
                        changed += 1;
                    }
                });
                assert!(
                    changed > 0,
                    "{region:?} seed {seed}: repeat_neighbor changed no pixels"
                );
            }
        }
    }

    #[test]
    fn chroma_boundary_is_visible_when_chroma_varies_only_across_rows() {
        // Issue #9: the old op only sampled the 1-px horizontal neighbor, so
        // content whose chroma varies across rows (or slowly in every
        // direction) came back unchanged at every localized region size.
        let orig = hstripes(64, 64);
        for region in [
            Region::Whole,
            Region::Fraction(4),
            Region::Square(64),
            Region::Square(16),
            Region::Square(8),
        ] {
            for seed in 0..8u64 {
                let mut img = orig.clone();
                let mut rng = SplitMix64::new(seed);
                Family::ChromaBoundary.apply(&mut img, region, Severity::Opaque, &mut rng);
                let changed = changed_pixels(&orig, &img);
                assert!(
                    changed > 0,
                    "{region:?} seed {seed}: chroma_boundary changed no pixels"
                );
                // Luma is preserved (it is a chroma-only defect) wherever the
                // foreign chroma did not push a channel into 0/255 clipping.
                for y in 0..64 {
                    for x in 0..64 {
                        let p = img.get(x, y);
                        if p.iter().any(|&c| c == 0 || c == 255) {
                            continue;
                        }
                        let (ya, _, _) = rgb_to_ycbcr(orig.get(x, y));
                        let (yb, _, _) = rgb_to_ycbcr(p);
                        assert!((ya - yb).abs() <= 2.5, "luma drift at ({x},{y})");
                    }
                }
            }
        }
    }

    #[test]
    fn chroma_boundary_only_touches_block_edge_bands() {
        let orig = block_colored(64, 64);
        let mut img = orig.clone();
        let mut rng = SplitMix64::new(1);
        Family::ChromaBoundary.apply(&mut img, Region::Whole, Severity::Opaque, &mut rng);
        for y in 0..64 {
            for x in 0..64 {
                let on_band = matches!(x % 8, 0 | 7) || matches!(y % 8, 0 | 7);
                if !on_band {
                    assert_eq!(img.get(x, y), orig.get(x, y), "interior ({x},{y})");
                }
            }
        }
        assert!(changed_pixels(&orig, &img) > 0);
    }

    #[test]
    fn across_block_boundary_maps_into_the_adjacent_block() {
        assert_eq!(across_block_boundary(8, 64), 0);
        assert_eq!(across_block_boundary(16, 64), 8);
        assert_eq!(across_block_boundary(7, 64), 15);
        assert_eq!(across_block_boundary(15, 64), 23);
        // Interior columns are untouched.
        for v in 1..7 {
            assert_eq!(across_block_boundary(v, 64), v);
        }
        // Edges mirror inward instead of leaving the image.
        assert_eq!(across_block_boundary(0, 64), 8);
        assert_eq!(across_block_boundary(63, 64), 55);
        // Tiny images stay in bounds.
        assert_eq!(across_block_boundary(0, 1), 0);
        assert_eq!(across_block_boundary(0, 4), 3);
    }
}
