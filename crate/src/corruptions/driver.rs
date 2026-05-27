//! Driver: turn a reference image into a corpus quad
//! `(reference, corruption, q20-anchor, q10-anchor)`.
//!
//! This is the part of the corpus that needs the `image` crate, so it lives
//! behind the `driver` feature. It loads a reference PNG, applies a
//! [`CorruptionParams`](super::CorruptionParams), and produces the two honest
//! low-quality JPEG anchors the gate compares against:
//!
//! ```text
//! score(ref, corruption) < score(ref, q20-anchor)
//! ```
//!
//! All four outputs are returned as in-memory [`Rgb8`] buffers (the JPEG
//! anchors are encoded then decoded back to pixels so the gate scores pixels,
//! not bitstreams). Nothing is written to disk by the library; the
//! `corruption_corpus` example writes PNGs/JPEGs on demand.

use image::{ImageEncoder, ImageReader, codecs::jpeg::JpegEncoder};

use super::{CorruptionParams, Rgb8, prng};

/// The four images a corpus entry compares.
#[derive(Debug, Clone)]
pub struct CorpusQuad {
    /// The pristine reference (decoded RGB).
    pub reference: Rgb8,
    /// The structurally-corrupted variant.
    pub corruption: Rgb8,
    /// Honest lossy anchor: JPEG q20, re-decoded to RGB.
    pub q20_anchor: Rgb8,
    /// Honest lossy anchor: JPEG q10, re-decoded to RGB.
    pub q10_anchor: Rgb8,
}

/// Errors the driver can produce.
#[derive(Debug)]
pub enum DriverError {
    /// Failed to read or decode the reference image.
    Image(image::ImageError),
    /// I/O error reading the reference file.
    Io(std::io::Error),
}

impl std::fmt::Display for DriverError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DriverError::Image(e) => write!(f, "image error: {e}"),
            DriverError::Io(e) => write!(f, "I/O error: {e}"),
        }
    }
}

impl std::error::Error for DriverError {}

impl From<image::ImageError> for DriverError {
    fn from(e: image::ImageError) -> Self {
        DriverError::Image(e)
    }
}

impl From<std::io::Error> for DriverError {
    fn from(e: std::io::Error) -> Self {
        DriverError::Io(e)
    }
}

/// Load a reference image from disk as [`Rgb8`].
pub fn load_reference(path: impl AsRef<std::path::Path>) -> Result<Rgb8, DriverError> {
    let img = ImageReader::open(path)?.with_guessed_format()?.decode()?;
    let rgb = img.to_rgb8();
    let (w, h) = (rgb.width(), rgb.height());
    Ok(Rgb8::from_raw(w, h, rgb.into_raw()))
}

/// Encode an [`Rgb8`] to JPEG at `quality` (1..=100), then decode it back to
/// RGB — the "honest lossy" anchor pixels the gate scores against.
pub fn jpeg_anchor(img: &Rgb8, quality: u8) -> Result<Rgb8, DriverError> {
    let mut buf = Vec::new();
    {
        let encoder = JpegEncoder::new_with_quality(&mut buf, quality);
        encoder.write_image(
            img.as_bytes(),
            img.width(),
            img.height(),
            image::ExtendedColorType::Rgb8,
        )?;
    }
    let decoded = image::load_from_memory_with_format(&buf, image::ImageFormat::Jpeg)?;
    let rgb = decoded.to_rgb8();
    let (w, h) = (rgb.width(), rgb.height());
    Ok(Rgb8::from_raw(w, h, rgb.into_raw()))
}

/// Build the full `(reference, corruption, q20, q10)` quad for one corpus entry.
///
/// `ref_id` and `base_seed` derive the deterministic corruption seed via
/// [`prng::seed_for`], so the same `(ref_id, base_seed, params)` always
/// reproduces the same corrupted bytes.
pub fn build_quad(
    reference: &Rgb8,
    ref_id: &str,
    params: &CorruptionParams,
    base_seed: u64,
) -> Result<CorpusQuad, DriverError> {
    let seed = prng::seed_for(ref_id, base_seed);
    let mut corruption = reference.clone();
    params.apply(&mut corruption, seed);

    let q20_anchor = jpeg_anchor(reference, 20)?;
    let q10_anchor = jpeg_anchor(reference, 10)?;

    Ok(CorpusQuad {
        reference: reference.clone(),
        corruption,
        q20_anchor,
        q10_anchor,
    })
}

/// Encode an [`Rgb8`] to a PNG byte vector (for writing corpus outputs to disk).
pub fn encode_png(img: &Rgb8) -> Result<Vec<u8>, DriverError> {
    let mut buf = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut buf);
    encoder.write_image(
        img.as_bytes(),
        img.width(),
        img.height(),
        image::ExtendedColorType::Rgb8,
    )?;
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::corruptions::{ChannelOp, Family, Region, Severity};

    fn checker(w: u32, h: u32) -> Rgb8 {
        let mut data = Vec::with_capacity((w * h * 3) as usize);
        for y in 0..h {
            for x in 0..w {
                let on = ((x / 4) + (y / 4)) % 2 == 0;
                // Distinct R/G/B so channel swaps are observable.
                data.push(if on { 220 } else { 40 });
                data.push((x * 255 / w.max(1)) as u8);
                data.push((y * 255 / h.max(1)) as u8);
            }
        }
        Rgb8::from_raw(w, h, data)
    }

    #[test]
    fn jpeg_anchor_roundtrips_dimensions() {
        let img = checker(32, 32);
        let a = jpeg_anchor(&img, 20).unwrap();
        assert_eq!((a.width(), a.height()), (32, 32));
        // q20 must actually degrade the image.
        assert_ne!(a, img);
    }

    #[test]
    fn build_quad_is_deterministic() {
        let reference = checker(48, 48);
        let params = CorruptionParams {
            family: Family::Channel(ChannelOp::SwapRb),
            region: Region::Fraction(2),
            severity: Severity::Opaque,
        };
        let a = build_quad(&reference, "test/ref", &params, 1).unwrap();
        let b = build_quad(&reference, "test/ref", &params, 1).unwrap();
        assert_eq!(a.corruption, b.corruption);
        assert_eq!(a.q20_anchor, b.q20_anchor);
        assert_eq!(a.q10_anchor, b.q10_anchor);
        // Corruption differs from reference.
        assert_ne!(a.corruption, a.reference);
    }

    #[test]
    fn png_encode_decodes_back() {
        let img = checker(16, 16);
        let png = encode_png(&img).unwrap();
        let decoded = image::load_from_memory(&png).unwrap().to_rgb8();
        assert_eq!((decoded.width(), decoded.height()), (16, 16));
        // PNG is lossless → exact roundtrip.
        assert_eq!(decoded.into_raw(), img.into_bytes());
    }
}
