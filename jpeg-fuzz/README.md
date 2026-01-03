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

### must_decode/ (21 files, ~500 KB)

Valid JPEG files that any conformant decoder must handle correctly.

| Source | Files | Description |
|--------|-------|-------------|
| `jpeg-decoder/reftest/` | 17 | Reference images with known-good decodes |
| `libjpeg-turbo/` | 4 | Baseline, progressive, arithmetic, 12-bit |

### may_decode/ (8 files, ~200 KB)

Edge cases where decoder behavior varies. Some decoders accept these, others reject.

| Source | Files | Description |
|--------|-------|-------------|
| `jpeg-decoder/` | 8 | Extraneous bytes, ICC chunk edge cases |

Files:
- `extraneous-bytes-after-sos.jpg` - Extra data after scan data
- `multiple-0xff-before-eoi.jpg` - Multiple 0xFF bytes before EOI marker
- `missing-frame-image-1410.jpg` - Truncated file
- `icc_chunk_*.jpeg` - ICC profile chunk ordering/numbering edge cases

### should_fail/ (108 files, ~2.8 MB)

Invalid or malformed JPEGs. Decoders should reject these gracefully (return error, not crash/hang).

| Source | Files | Description |
|--------|-------|-------------|
| `jpeg-decoder/` | 10 | Targeted crash tests (overflow, missing markers) |
| `imagetestsuite/` | 98 | Malformed files from Google's imagetestsuite |

Notable files:
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
