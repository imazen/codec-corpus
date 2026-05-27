//! Integration tests for the structural-corruption corpus driver.
//!
//! These require the `driver` feature (the `image`-crate-backed quad builder).
//! Run with: `cargo test --features driver --test corruption_corpus`.

#![cfg(feature = "driver")]

use codec_corpus::corruptions::{
    ChannelOp, ContentClass, CorruptionParams, Family, Region, Rgb8, Severity, catalog, driver,
    manifest_for_reference,
};

/// A synthetic reference with distinct per-channel structure so every family
/// produces an observable change.
fn synthetic_reference(w: u32, h: u32) -> Rgb8 {
    let mut data = Vec::with_capacity((w * h * 3) as usize);
    for y in 0..h {
        for x in 0..w {
            data.push((x * 255 / w.max(1)) as u8);
            data.push((y * 255 / h.max(1)) as u8);
            data.push(((x ^ y) & 0xFF) as u8);
        }
    }
    Rgb8::from_raw(w, h, data)
}

#[test]
fn build_quad_dimensions_match_reference() {
    let reference = synthetic_reference(96, 72);
    let params = CorruptionParams {
        family: Family::Channel(ChannelOp::SwapRb),
        region: Region::Fraction(2),
        severity: Severity::Opaque,
    };
    let quad = driver::build_quad(&reference, "synthetic/96x72", &params, 1).unwrap();
    for img in [
        &quad.reference,
        &quad.corruption,
        &quad.q20_anchor,
        &quad.q10_anchor,
    ] {
        assert_eq!(img.width(), 96);
        assert_eq!(img.height(), 72);
    }
    // The structural corruption and the honest lq anchors are all distinct from
    // the reference (the gate compares them, so they must differ).
    assert_ne!(quad.corruption, quad.reference);
    assert_ne!(quad.q20_anchor, quad.reference);
    assert_ne!(quad.q10_anchor, quad.reference);
}

#[test]
fn whole_catalog_builds_for_a_small_reference() {
    // Exercises every family × region × severity through the real quad builder
    // on a small image — catches any panic / dimension drift across the catalog.
    let reference = synthetic_reference(40, 40);
    let cat = catalog();
    assert!(
        cat.len() > 100,
        "catalog should be large, got {}",
        cat.len()
    );
    for params in &cat {
        let quad = driver::build_quad(&reference, "synthetic/40x40", params, 7).unwrap();
        assert_eq!(quad.corruption.width(), 40, "{}", params.slug());
        assert_eq!(quad.corruption.height(), 40, "{}", params.slug());
    }
}

#[test]
fn quad_is_reproducible_across_calls() {
    let reference = synthetic_reference(64, 48);
    let params = CorruptionParams {
        family: Family::Channel(ChannelOp::Invert),
        region: Region::Square(16),
        severity: Severity::Opacity(0.5),
    };
    let a = driver::build_quad(&reference, "synthetic/repro", &params, 3).unwrap();
    let b = driver::build_quad(&reference, "synthetic/repro", &params, 3).unwrap();
    assert_eq!(a.corruption, b.corruption);
    assert_eq!(a.q20_anchor, b.q20_anchor);
    assert_eq!(a.q10_anchor, b.q10_anchor);
}

#[test]
fn manifest_entries_drive_the_builder() {
    // The manifest's (ref_id, seed, params) must reproduce exactly what the
    // driver builds from the same inputs.
    let reference = synthetic_reference(56, 56);
    let entries = manifest_for_reference("synthetic/manifest", ContentClass::Gradient, 5);
    // Take a handful spanning families.
    for entry in entries.iter().step_by(37) {
        let from_entry =
            driver::build_quad(&reference, &entry.ref_id, &entry.params, entry.seed).unwrap();
        let recomputed_seed = codec_corpus::corruptions::prng::seed_for(&entry.ref_id, 5);
        assert_eq!(entry.seed, recomputed_seed);
        // Rebuild with the recomputed seed → identical corruption.
        let again =
            driver::build_quad(&reference, &entry.ref_id, &entry.params, recomputed_seed).unwrap();
        assert_eq!(from_entry.corruption, again.corruption);
    }
}

#[test]
fn png_roundtrip_is_lossless() {
    let reference = synthetic_reference(32, 32);
    let png = driver::encode_png(&reference).unwrap();
    let decoded = image::load_from_memory(&png).unwrap().to_rgb8();
    assert_eq!(decoded.into_raw(), reference.into_bytes());
}
