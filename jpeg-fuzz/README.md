# JPEG Fuzz & Edge Case Test Suite

Test files for JPEG decoder robustness testing, organized by expected behavior.

## Directory Structure

```
jpeg-fuzz/
├── must_decode/     # Valid JPEGs - decoder MUST handle correctly
├── may_decode/      # Edge cases - behavior varies by decoder
└── should_fail/     # Invalid JPEGs - decoder MUST reject gracefully
```

## Categories

### must_decode/ (43 files, ~2.0 MB)

Valid JPEG files that any conformant decoder must handle correctly.

| Source | Files | Description |
|--------|-------|-------------|
| `jpeg-decoder/reftest/` | 17 | Reference images with known-good decodes |
| `libjpeg-turbo/` | 4 | Baseline, progressive, arithmetic, 12-bit |
| `camera-samples/` | 12 | Real camera JPEGs with diverse EXIF (Canon, Nikon, Sony, etc.) |
| `restart-intervals/` | 6 | Various RST marker intervals (1/2/4 rows, 1/8/16 blocks) |
| `cmyk-ycck/` | 3 | CMYK and YCCK color model files |

### may_decode/ (20 files, ~220 KB)

Edge cases where decoder behavior varies. Some decoders accept these, others reject.

| Source | Files | Description |
|--------|-------|-------------|
| `jpeg-decoder/` | 8 | Extraneous bytes, ICC chunk edge cases |
| `truncated/` | 12 | Files truncated at various positions |

**jpeg-decoder edge cases:**
- `extraneous-bytes-after-sos.jpg` - Extra data after scan data
- `multiple-0xff-before-eoi.jpg` - Multiple 0xFF bytes before EOI marker
- `missing-frame-image-1410.jpg` - Truncated file
- `icc_chunk_*.jpeg` - ICC profile chunk ordering/numbering edge cases

**Truncated variants:**
- `after_soi.jpg` - Only SOI marker (2 bytes)
- `after_app0.jpg` - Truncated in APP0 marker
- `mid_header.jpg` - Truncated mid-header
- `after_sos.jpg` - Truncated after SOS marker
- `scan_10pct.jpg`, `scan_50pct.jpg`, `scan_90pct.jpg` - Truncated in scan data
- `missing_eoi.jpg` - Complete except missing EOI marker
- `progressive_*.jpg` - Progressive JPEGs truncated at 25/50/75%

### should_fail/ (116 files, ~2.9 MB)

Invalid or malformed JPEGs. Decoders should reject these gracefully (return error, not crash/hang).

| Source | Files | Description |
|--------|-------|-------------|
| `jpeg-decoder/` | 10 | Targeted crash tests (overflow, missing markers) |
| `imagetestsuite/` | 98 | Malformed files from Google's imagetestsuite |
| `exif-invalid/` | 8 | Corrupted/invalid EXIF metadata |

**Notable crash test files:**
- `empty.jpg` - Zero-length file
- `missing-sof.jpg` - No Start Of Frame marker
- `missing-sos.jpg` - No Start Of Scan marker
- `null_height.jpg` - Height = 0 in SOF
- `invalid-dimensions.jpg` - Invalid image dimensions
- `derive-huffman-codes-overflow.jpg` - Huffman table overflow
- `dc-predictor-overflow.jpg` - DC coefficient overflow
- `subtract-with-overflow.jpg` - Arithmetic overflow case

## Sources

| Project | License | URL |
|---------|---------|-----|
| jpeg-decoder | MIT | https://github.com/image-rs/jpeg-decoder |
| libjpeg-turbo | IJG + BSD | https://github.com/libjpeg-turbo/libjpeg-turbo |
| imagetestsuite | Various | https://code.google.com/p/imagetestsuite/ |
| exif-samples | MIT | https://github.com/ianare/exif-samples |
| imageflow | AGPL-3.0 | https://github.com/imazen/imageflow |

## Camera Samples

The `camera-samples/` directory contains JPEGs from various camera manufacturers:

| Camera | Manufacturer | Notable Features |
|--------|--------------|------------------|
| Canon_40D.jpg | Canon | DSLR, standard EXIF |
| Nikon_D70.jpg | Nikon | DSLR, Nikon MakerNote |
| Sony_HDR-HC3.jpg | Sony | Camcorder |
| Fujifilm_FinePix_E500.jpg | Fujifilm | Compact camera |
| Olympus_C8080WZ.jpg | Olympus | Compact camera |
| Panasonic_DMC-FZ30.jpg | Panasonic | Bridge camera |
| Pentax_K10D.jpg | Pentax | DSLR |
| Samsung_Digimax_i50_MP3.jpg | Samsung | Compact with MP3 |
| Ricoh_Caplio_RR330.jpg | Ricoh | Compact camera |
| Kodak_CX7530.jpg | Kodak | Compact camera |
| Konica_Minolta_DiMAGE_Z3.jpg | Konica Minolta | Bridge camera |
| Reconyx_HC500_Hyperfire.jpg | Reconyx | Trail camera |

## Usage

### Rust (jpeg-decoder)

```rust
use jpeg_decoder::Decoder;
use std::fs::File;

fn test_should_fail(path: &str) {
    let file = File::open(path).unwrap();
    let mut decoder = Decoder::new(file);
    // Should return Err, not panic
    assert!(decoder.decode().is_err());
}

fn test_must_decode(path: &str) {
    let file = File::open(path).unwrap();
    let mut decoder = Decoder::new(file);
    // Should succeed
    decoder.decode().expect("valid JPEG must decode");
}
```

### Fuzzing

These files make excellent seed corpora for cargo-fuzz:

```bash
# Copy to fuzz corpus
cp -r jpeg-fuzz/should_fail/* fuzz/corpus/decode/
cp -r jpeg-fuzz/must_decode/* fuzz/corpus/decode/

# Run fuzzer
cargo +nightly fuzz run decode
```

## Adding New Files

When adding files, categorize based on:

1. **must_decode**: Has reference output, known valid
2. **may_decode**: Valid but uses optional/obscure features
3. **should_fail**: Intentionally malformed, causes crashes in some decoders

Include source attribution in this README.
