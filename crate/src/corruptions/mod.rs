//! Structural-corruption distortion corpus.
//!
//! This module builds a held-out **falsification set for a perceptual metric's
//! negative tail**: corruptions that no honest encoder would produce (a decoder
//! bug, a channel swap, an off-by-one edge) and that a faithful metric must rank
//! *below* an honestly-lossy encode of the same reference.
//!
//! See `docs/structural_corruption_corpus_spec_2026-05-27.md` in the zensim repo
//! for the full motivation. The gate every entry asserts is:
//!
//! ```text
//! score(ref, corruption) < score(ref, honest_lq_anchor)   // q20 / q10 JPEG
//! ```
//!
//! ## Design
//!
//! - **Generators are deterministic** — seeded by `(ref_id, seed, params)` via
//!   [`prng::SplitMix64`]. No `rand`, no OS entropy. Same inputs → same bytes
//!   on every platform.
//! - **Generators are pure RGB-buffer math** — they take an [`Rgb8`] image and
//!   mutate it in place. No image decoding is needed to *apply* a corruption, so
//!   the generators stay in the default (and wasm) build.
//! - **The driver** ([`driver`], behind the `driver` feature) loads reference
//!   PNGs and emits `(ref, corruption, q20-anchor, q10-anchor)` so the zensim
//!   gate can run directly. It is the only part that needs the `image` crate.
//! - **No large images are committed.** The corpus is reproduced on demand from
//!   `(reference_id, seed, params)` plus the curated reference set that already
//!   ships in this repo (CLIC2025, CID22, KADID-10k, GB82, GB82-SC, ...).
//!
//! ## Example
//!
//! ```
//! use codec_corpus::corruptions::{Rgb8, Family, ChannelOp, CorruptionParams, Region, Severity};
//!
//! // A flat gray 64x64 reference.
//! let mut img = Rgb8::filled(64, 64, [128, 128, 128]);
//! let params = CorruptionParams {
//!     family: Family::Channel(ChannelOp::SwapRb),
//!     region: Region::Fraction(4), // 1/4 of the image
//!     severity: Severity::Opaque,
//! };
//! params.apply(&mut img, /* seed */ 1);
//! // img now has a region with R and B swapped.
//! ```

pub mod prng;

mod families;

#[cfg(feature = "driver")]
pub mod driver;

use serde::{Deserialize, Serialize};

pub use families::*;

// ---------------------------------------------------------------------------
// Pixel buffer
// ---------------------------------------------------------------------------

/// A tightly-packed 8-bit RGB image buffer (3 bytes per pixel, row-major).
///
/// Stride is always `width * 3` (tightly packed); the generators iterate rows
/// internally so a strided variant could be added without changing the family
/// math. Pixels are plain `[u8; 3]` in R, G, B order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rgb8 {
    width: u32,
    height: u32,
    /// Row-major RGB bytes, `width * height * 3` long.
    data: Vec<u8>,
}

impl Rgb8 {
    /// Create an image from raw RGB bytes. Panics if `data.len() != w*h*3`.
    pub fn from_raw(width: u32, height: u32, data: Vec<u8>) -> Self {
        assert_eq!(
            data.len(),
            width as usize * height as usize * 3,
            "Rgb8::from_raw: data length must equal width*height*3"
        );
        Self {
            width,
            height,
            data,
        }
    }

    /// Create an image with every pixel set to `rgb`.
    pub fn filled(width: u32, height: u32, rgb: [u8; 3]) -> Self {
        let mut data = Vec::with_capacity(width as usize * height as usize * 3);
        for _ in 0..(width as usize * height as usize) {
            data.extend_from_slice(&rgb);
        }
        Self {
            width,
            height,
            data,
        }
    }

    /// Image width in pixels.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Image height in pixels.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Raw RGB bytes (row-major, tightly packed).
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Mutable raw RGB bytes.
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }

    /// Consume and return the raw RGB bytes.
    pub fn into_bytes(self) -> Vec<u8> {
        self.data
    }

    /// Byte index of pixel `(x, y)` within `data`.
    #[inline]
    fn idx(&self, x: u32, y: u32) -> usize {
        (y as usize * self.width as usize + x as usize) * 3
    }

    /// Read pixel `(x, y)`. Panics if out of bounds.
    #[inline]
    pub fn get(&self, x: u32, y: u32) -> [u8; 3] {
        let i = self.idx(x, y);
        [self.data[i], self.data[i + 1], self.data[i + 2]]
    }

    /// Write pixel `(x, y)`. Panics if out of bounds.
    #[inline]
    pub fn set(&mut self, x: u32, y: u32, rgb: [u8; 3]) {
        let i = self.idx(x, y);
        self.data[i] = rgb[0];
        self.data[i + 1] = rgb[1];
        self.data[i + 2] = rgb[2];
    }
}

// ---------------------------------------------------------------------------
// Region + severity
// ---------------------------------------------------------------------------

/// The size of the region a corruption is applied to.
///
/// The sweep runs whole-image → 1/4 → 1/16 → 64x64 → 16x16 → 8x8, plus a 1-pixel
/// region for edge/geometric families. The subtle end (small region) is the hard
/// case where a metric saturates and lets a real bug pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Region {
    /// The whole image.
    Whole,
    /// A fractional sub-rectangle: side length = image side / `n` (so `Fraction(2)`
    /// is 1/4 of the image area, `Fraction(4)` is 1/16, ...). Placed at a
    /// seed-chosen offset.
    Fraction(u32),
    /// A fixed `size x size` square block at a seed-chosen offset (clamped to the
    /// image). Used for the 64x64 / 16x16 / 8x8 sweep points.
    Square(u32),
}

impl Region {
    /// Resolve the region to a concrete rectangle `(x0, y0, w, h)` within
    /// `width x height`, choosing the offset deterministically from `rng`.
    pub fn resolve(&self, width: u32, height: u32, rng: &mut prng::SplitMix64) -> Rect {
        match *self {
            Region::Whole => Rect {
                x0: 0,
                y0: 0,
                w: width,
                h: height,
            },
            Region::Fraction(n) => {
                let n = n.max(1);
                let w = (width / n).max(1);
                let h = (height / n).max(1);
                Self::place(width, height, w, h, rng)
            }
            Region::Square(s) => {
                let w = s.min(width).max(1);
                let h = s.min(height).max(1);
                Self::place(width, height, w, h, rng)
            }
        }
    }

    fn place(width: u32, height: u32, w: u32, h: u32, rng: &mut prng::SplitMix64) -> Rect {
        let max_x = width.saturating_sub(w);
        let max_y = height.saturating_sub(h);
        let x0 = if max_x == 0 {
            0
        } else {
            rng.below((max_x + 1) as u64) as u32
        };
        let y0 = if max_y == 0 {
            0
        } else {
            rng.below((max_y + 1) as u64) as u32
        };
        Rect { x0, y0, w, h }
    }

    /// A short stable label for manifests / file names.
    pub fn label(&self) -> String {
        match self {
            Region::Whole => "whole".to_string(),
            Region::Fraction(n) => format!("frac{n}"),
            Region::Square(s) => format!("sq{s}"),
        }
    }
}

/// A resolved pixel rectangle `(x0, y0)` with size `w x h`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    /// Left edge (inclusive).
    pub x0: u32,
    /// Top edge (inclusive).
    pub y0: u32,
    /// Width in pixels.
    pub w: u32,
    /// Height in pixels.
    pub h: u32,
}

impl Rect {
    /// Iterate every `(x, y)` in the rectangle, clamped to `width x height`.
    pub fn for_each_pixel(&self, width: u32, height: u32, mut f: impl FnMut(u32, u32)) {
        let x_end = (self.x0 + self.w).min(width);
        let y_end = (self.y0 + self.h).min(height);
        for y in self.y0..y_end {
            for x in self.x0..x_end {
                f(x, y);
            }
        }
    }
}

/// Corruption magnitude. `Opaque` is the obvious end; lower opacities are the
/// subtle, hard cases the metric is most at risk of letting pass.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// Fully replace the corrupted pixels (opacity 1.0).
    Opaque,
    /// Blend the corruption over the original at `0.0..=1.0` opacity.
    Opacity(f32),
}

impl Severity {
    /// The opacity in `0.0..=1.0`.
    pub fn opacity(&self) -> f32 {
        match self {
            Severity::Opaque => 1.0,
            Severity::Opacity(p) => p.clamp(0.0, 1.0),
        }
    }

    /// A short stable label for manifests / file names.
    pub fn label(&self) -> String {
        match self {
            Severity::Opaque => "op100".to_string(),
            Severity::Opacity(p) => format!("op{}", (p.clamp(0.0, 1.0) * 100.0).round() as u32),
        }
    }
}

/// Alpha-blend `corrupt` over `orig` at `opacity` (0 = orig, 1 = corrupt).
#[inline]
pub(crate) fn blend(orig: [u8; 3], corrupt: [u8; 3], opacity: f32) -> [u8; 3] {
    if opacity >= 1.0 {
        return corrupt;
    }
    if opacity <= 0.0 {
        return orig;
    }
    let mut out = [0u8; 3];
    for c in 0..3 {
        let o = orig[c] as f32;
        let k = corrupt[c] as f32;
        out[c] = (o + (k - o) * opacity).round().clamp(0.0, 255.0) as u8;
    }
    out
}

// ---------------------------------------------------------------------------
// CorruptionParams — the full description of one corpus entry's distortion
// ---------------------------------------------------------------------------

/// A fully-specified corruption: which [`Family`] (with its family-specific
/// variant), what [`Region`] size, and what [`Severity`].
///
/// This is the unit the manifest serializes and the driver applies. Applying it
/// is deterministic given a seed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CorruptionParams {
    /// The distortion family + its variant.
    pub family: Family,
    /// Region size.
    pub region: Region,
    /// Magnitude / opacity.
    pub severity: Severity,
}

impl CorruptionParams {
    /// Apply this corruption to `img` in place, deterministically from `seed`.
    pub fn apply(&self, img: &mut Rgb8, seed: u64) {
        let mut rng = prng::SplitMix64::new(seed);
        self.family.apply(img, self.region, self.severity, &mut rng);
    }

    /// Whether, for an honest metric, this corruption is egregious enough that
    /// the *calibrated* score is expected to go negative (vs merely ranking
    /// below the lq anchor). Large opaque region + a structure-destroying family.
    pub fn expected_negative(&self) -> bool {
        let big = matches!(
            self.region,
            Region::Whole | Region::Fraction(1) | Region::Fraction(2)
        );
        let opaque = self.severity.opacity() >= 0.99;
        big && opaque && self.family.is_structure_destroying()
    }

    /// A short stable identifier combining family, region, and severity — used
    /// for manifest ids and generated file names.
    pub fn slug(&self) -> String {
        format!(
            "{}__{}__{}",
            self.family.slug(),
            self.region.label(),
            self.severity.label()
        )
    }
}

// ---------------------------------------------------------------------------
// Manifest
// ---------------------------------------------------------------------------

/// Where a corpus entry came from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EntrySource {
    /// Procedurally generated by this module's seeded generators.
    Synthetic,
    /// A reproduction of a documented historical decoder/renderer bug. The
    /// synthetic families in this module are `Synthetic`; real-bug repros are
    /// tracked separately (a parallel GitHub issue) and slot in here.
    RealBug {
        /// e.g. `"zenjpeg#123"` — `<repo>#<issue>`.
        reference: String,
    },
}

/// One entry in the structural-corruption corpus manifest.
///
/// The corpus is reproduced on demand: given `ref_id` (a reference image that
/// already ships in this repo) + `seed` + `params`, the generators recreate the
/// exact corrupted bytes. Nothing large is committed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ManifestEntry {
    /// Identifier of the reference image within the corpus (e.g.
    /// `"gb82-sc/imac_g3_strip"`). The driver maps this to a concrete path.
    pub ref_id: String,
    /// Content-class label of the reference (photo / screen / line_art / text /
    /// gradient). Lets evaluators stratify by content.
    pub content_class: ContentClass,
    /// The distortion family this entry uses (denormalized for easy filtering).
    pub family_name: String,
    /// The full corruption description.
    pub params: CorruptionParams,
    /// Deterministic seed used to reproduce the corruption.
    pub seed: u64,
    /// The gate every entry asserts: `score(corruption) < score(q20 anchor)`.
    /// Always `true` — it is the property under test.
    pub expected_below_lq: bool,
    /// Whether the calibrated score is expected to go *negative* (egregious
    /// corruptions only; subtle ones only need to rank below the lq anchor).
    pub expected_negative: bool,
    /// Provenance.
    pub source: EntrySource,
}

/// Content class of a reference image. Used to stratify the corpus across the
/// required ≥5 content classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentClass {
    /// Natural photograph.
    Photo,
    /// Screenshot / UI / desktop content.
    Screen,
    /// Line art / vector-like / icons.
    LineArt,
    /// Text-heavy / document.
    Text,
    /// Smooth gradient / synthetic.
    Gradient,
}

impl ContentClass {
    /// All classes, in a stable order.
    pub fn all() -> [ContentClass; 5] {
        [
            ContentClass::Photo,
            ContentClass::Screen,
            ContentClass::LineArt,
            ContentClass::Text,
            ContentClass::Gradient,
        ]
    }
}

// ---------------------------------------------------------------------------
// Sweep catalog
// ---------------------------------------------------------------------------

/// The canonical region-size sweep for region-confined families: whole-image →
/// 1/4 → 1/16 → 64x64 → 16x16 → 8x8. (The 1-pixel point is added per-family for
/// edge/geometric families that have explicit 1px variants.)
pub fn region_sweep() -> Vec<Region> {
    vec![
        Region::Whole,
        Region::Fraction(2), // 1/4 area
        Region::Fraction(4), // 1/16 area
        Region::Square(64),
        Region::Square(16),
        Region::Square(8),
    ]
}

/// The canonical severity sweep: opaque (obvious) → 50% → 20% (subtle, hard).
pub fn severity_sweep() -> Vec<Severity> {
    vec![
        Severity::Opaque,
        Severity::Opacity(0.5),
        Severity::Opacity(0.2),
    ]
}

/// Every family variant, used to expand the full catalog. One representative
/// per distinct mechanism; the region × severity sweep multiplies them out.
pub fn all_family_variants() -> Vec<Family> {
    let mut v = Vec::new();
    for op in [
        ChannelOp::Invert,
        ChannelOp::SwapRb,
        ChannelOp::SwapRg,
        ChannelOp::SwapGb,
        ChannelOp::ZeroR,
        ChannelOp::ZeroG,
        ChannelOp::ZeroB,
        ChannelOp::MaxR,
    ] {
        v.push(Family::Channel(op));
    }
    for op in [
        BlockOp::Zero,
        BlockOp::Gray,
        BlockOp::Garbage,
        BlockOp::CopyWrong,
        BlockOp::RepeatNeighbor,
    ] {
        v.push(Family::Block(op));
    }
    for op in [
        EdgeOp::BorderTop { k: 1 },
        EdgeOp::BorderTop { k: 2 },
        EdgeOp::BorderTop { k: 4 },
        EdgeOp::BorderAll { k: 1 },
        EdgeOp::BorderAll { k: 2 },
        EdgeOp::BorderAll { k: 4 },
        EdgeOp::ShiftInterior1px,
        EdgeOp::DuplicateTopRow,
    ] {
        v.push(Family::Edge(op));
    }
    for count in [1u32, 16, 256] {
        v.push(Family::Noise(NoiseOp::SaltPepper { count }));
        v.push(Family::Noise(NoiseOp::BitFlip { count }));
    }
    for op in [
        ToneOp::GammaEncode,
        ToneOp::GammaDecode,
        ToneOp::ContrastBoost,
        ToneOp::Brightness { delta: 40 },
        ToneOp::Brightness { delta: -40 },
    ] {
        v.push(Family::Tone(op));
    }
    for op in [OverlayShape::Rect, OverlayShape::Line, OverlayShape::Glyph] {
        v.push(Family::Overlay(op));
    }
    v.push(Family::ChromaBoundary);
    v.push(Family::Aliasing);
    for op in [
        GeometricOp::Shift1px,
        GeometricOp::FlipH,
        GeometricOp::FlipV,
        GeometricOp::Rotate90,
    ] {
        v.push(Family::Geometric(op));
    }
    for op in [
        CompositeOp::PremulAsStraight,
        CompositeOp::WrongBgBlack,
        CompositeOp::WrongBgWhite,
    ] {
        v.push(Family::Composite(op));
    }
    v
}

/// Whether a family's region must be the whole image (edge/geometric families
/// that operate on the global frame rather than a placed sub-rectangle).
fn is_whole_frame_family(f: &Family) -> bool {
    matches!(
        f,
        Family::Edge(EdgeOp::BorderTop { .. })
            | Family::Edge(EdgeOp::BorderAll { .. })
            | Family::Edge(EdgeOp::ShiftInterior1px)
            | Family::Edge(EdgeOp::DuplicateTopRow)
    )
}

/// Build the full catalog of [`CorruptionParams`] for one reference: every
/// family variant × the region sweep × the severity sweep. Edge families that
/// act on the whole frame use only `Region::Whole`.
pub fn catalog() -> Vec<CorruptionParams> {
    let mut out = Vec::new();
    for family in all_family_variants() {
        let regions: Vec<Region> = if is_whole_frame_family(&family) {
            vec![Region::Whole]
        } else {
            region_sweep()
        };
        for region in regions {
            for severity in severity_sweep() {
                out.push(CorruptionParams {
                    family,
                    region,
                    severity,
                });
            }
        }
    }
    out
}

/// Build manifest entries for one reference across the full [`catalog`].
///
/// `base_seed` is mixed with `ref_id` per entry (so the same family on two
/// references gets different placements but stays reproducible).
pub fn manifest_for_reference(
    ref_id: &str,
    content_class: ContentClass,
    base_seed: u64,
) -> Vec<ManifestEntry> {
    catalog()
        .into_iter()
        .map(|params| {
            let seed = prng::seed_for(ref_id, base_seed);
            ManifestEntry {
                ref_id: ref_id.to_string(),
                content_class,
                family_name: params.family.name().to_string(),
                expected_negative: params.expected_negative(),
                expected_below_lq: true,
                source: EntrySource::Synthetic,
                seed,
                params,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rgb8_roundtrip_and_pixel_access() {
        let mut img = Rgb8::filled(4, 3, [10, 20, 30]);
        assert_eq!(img.width(), 4);
        assert_eq!(img.height(), 3);
        assert_eq!(img.as_bytes().len(), 4 * 3 * 3);
        assert_eq!(img.get(2, 1), [10, 20, 30]);
        img.set(2, 1, [99, 98, 97]);
        assert_eq!(img.get(2, 1), [99, 98, 97]);
    }

    #[test]
    fn region_resolve_within_bounds() {
        let mut rng = prng::SplitMix64::new(3);
        for region in [
            Region::Whole,
            Region::Fraction(2),
            Region::Fraction(4),
            Region::Square(8),
            Region::Square(64),
        ] {
            let r = region.resolve(100, 80, &mut rng);
            assert!(r.x0 + r.w <= 100, "{region:?} -> {r:?}");
            assert!(r.y0 + r.h <= 80, "{region:?} -> {r:?}");
            assert!(r.w >= 1 && r.h >= 1);
        }
    }

    #[test]
    fn square_larger_than_image_is_clamped() {
        let mut rng = prng::SplitMix64::new(1);
        let r = Region::Square(256).resolve(64, 64, &mut rng);
        assert_eq!((r.x0, r.y0, r.w, r.h), (0, 0, 64, 64));
    }

    #[test]
    fn blend_endpoints() {
        assert_eq!(blend([0, 0, 0], [100, 100, 100], 0.0), [0, 0, 0]);
        assert_eq!(blend([0, 0, 0], [100, 100, 100], 1.0), [100, 100, 100]);
        assert_eq!(blend([0, 0, 0], [100, 100, 100], 0.5), [50, 50, 50]);
    }

    #[test]
    fn severity_and_region_labels_are_stable() {
        assert_eq!(Severity::Opaque.label(), "op100");
        assert_eq!(Severity::Opacity(0.2).label(), "op20");
        assert_eq!(Region::Whole.label(), "whole");
        assert_eq!(Region::Fraction(4).label(), "frac4");
        assert_eq!(Region::Square(8).label(), "sq8");
    }

    #[test]
    fn manifest_entry_serde_roundtrip() {
        let entry = ManifestEntry {
            ref_id: "gb82-sc/imac_g3_strip".to_string(),
            content_class: ContentClass::Screen,
            family_name: "channel".to_string(),
            params: CorruptionParams {
                family: Family::Channel(ChannelOp::SwapRb),
                region: Region::Fraction(2),
                severity: Severity::Opaque,
            },
            seed: 42,
            expected_below_lq: true,
            expected_negative: true,
            source: EntrySource::Synthetic,
        };
        let json = serde_json::to_string(&entry).unwrap();
        let back: ManifestEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(entry, back);
    }

    #[test]
    fn real_bug_source_serde() {
        let src = EntrySource::RealBug {
            reference: "zenjpeg#123".to_string(),
        };
        let json = serde_json::to_string(&src).unwrap();
        assert!(json.contains("real_bug"));
        let back: EntrySource = serde_json::from_str(&json).unwrap();
        assert_eq!(src, back);
    }

    #[test]
    fn catalog_covers_all_ten_families() {
        let cat = catalog();
        assert!(!cat.is_empty());
        let mut names: Vec<&str> = cat.iter().map(|p| p.family.name()).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(
            names.len(),
            10,
            "expected all 10 families in the catalog, got {names:?}"
        );
    }

    #[test]
    fn catalog_slugs_are_unique() {
        let cat = catalog();
        let mut slugs: Vec<String> = cat.iter().map(|p| p.slug()).collect();
        let total = slugs.len();
        slugs.sort();
        slugs.dedup();
        assert_eq!(slugs.len(), total, "catalog slugs must be unique");
    }

    #[test]
    fn manifest_for_reference_is_deterministic() {
        let a = manifest_for_reference("photo/01", ContentClass::Photo, 7);
        let b = manifest_for_reference("photo/01", ContentClass::Photo, 7);
        assert_eq!(a, b);
        assert!(!a.is_empty());
        // Every entry asserts the gate and carries the right ref id.
        for e in &a {
            assert!(e.expected_below_lq);
            assert_eq!(e.ref_id, "photo/01");
        }
    }

    #[test]
    fn whole_frame_edge_families_use_whole_region() {
        for entry in catalog() {
            if is_whole_frame_family(&entry.family) {
                assert_eq!(
                    entry.region,
                    Region::Whole,
                    "{} should only sweep whole region",
                    entry.slug()
                );
            }
        }
    }

    #[test]
    fn egregious_corruptions_expect_negative() {
        // A whole-image opaque channel swap is structure-destroying → negative.
        let p = CorruptionParams {
            family: Family::Channel(ChannelOp::SwapRb),
            region: Region::Whole,
            severity: Severity::Opaque,
        };
        assert!(p.expected_negative());
        // An 8x8 20%-opacity overlay is subtle → only ranks below lq, not negative.
        let q = CorruptionParams {
            family: Family::Overlay(OverlayShape::Rect),
            region: Region::Square(8),
            severity: Severity::Opacity(0.2),
        };
        assert!(!q.expected_negative());
    }
}
