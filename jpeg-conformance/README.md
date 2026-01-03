# JPEG Conformance Test Suite

Test files for JPEG decoder conformance testing, organized by expected behavior.

## Directory Structure

```
jpeg-conformance/
├── valid/              # MUST decode correctly
├── invalid/            # MUST reject gracefully
├── non-conformant/     # MAY reject or recover
│   ├── truncated/      # Incomplete files
│   ├── extraneous-data/# Extra bytes
│   ├── marker-quirks/  # Unusual marker sequences
│   └── metadata-quirks/# ICC/EXIF edge cases
└── SOURCES.md          # File attribution
```

## Categories

### valid/ (41 files)

Valid JPEG files that any conformant decoder MUST handle correctly.

- **Reference images**: jpeg-decoder reftest, libjpeg-turbo testimages
- **Camera samples**: 12 manufacturers (Canon, Nikon, Sony, Fujifilm, etc.)
- **Feature variants**: Restart markers, CMYK/YCCK color models
- **Encoding modes**: Baseline, progressive, arithmetic, 12-bit

### invalid/ (116 files)

Invalid JPEG files that decoders MUST reject gracefully (return error, not crash/hang).

- **Crash tests**: Missing markers, overflows, invalid dimensions
- **Malformed files**: imagetestsuite corpus (98 files)
- **Corrupted metadata**: Invalid EXIF data

### non-conformant/ (19 files)

Files that technically violate the JPEG spec but are common in the wild.
Decoder behavior varies - strict decoders reject, lenient decoders recover.

Each file has a companion `.txt` explaining:
- What's non-conformant
- Expected strict behavior
- Expected lenient behavior

**Categories:**
- `truncated/`: Files cut off at various positions
- `extraneous-data/`: Extra bytes in unusual places
- `marker-quirks/`: Unusual but potentially recoverable marker issues
- `metadata-quirks/`: ICC profile chunk ordering/numbering issues
- `progressive-quirks/`: Progressive JPEG edge cases (AC refinement, scan structure)

## Usage

### Testing a Decoder

```rust
use std::fs;

// Test valid files - must succeed
for entry in fs::read_dir("jpeg-conformance/valid")? {
    let path = entry?.path();
    if path.extension() == Some("jpg".as_ref()) {
        decoder.decode(&path).expect("valid JPEG must decode");
    }
}

// Test invalid files - must error (not panic)
for entry in fs::read_dir("jpeg-conformance/invalid")? {
    let path = entry?.path();
    if path.extension() == Some("jpg".as_ref()) {
        assert!(decoder.decode(&path).is_err());
    }
}

// Test non-conformant files - document behavior
for entry in fs::read_dir("jpeg-conformance/non-conformant/truncated")? {
    let path = entry?.path();
    if path.extension() == Some("jpg".as_ref()) {
        match decoder.decode(&path) {
            Ok(_) => println!("{}: accepted (lenient)", path.display()),
            Err(_) => println!("{}: rejected (strict)", path.display()),
        }
    }
}
```

### Fuzzing Seed Corpus

```bash
# Excellent seed corpus for cargo-fuzz
cp -r jpeg-conformance/valid/* fuzz/corpus/
cp -r jpeg-conformance/invalid/* fuzz/corpus/
cp -r jpeg-conformance/non-conformant/*/* fuzz/corpus/

cargo +nightly fuzz run decode_jpeg
```

## Attribution

See [SOURCES.md](SOURCES.md) for complete file attribution and licensing.

## Contributing

When adding files:

1. Determine category (valid/invalid/non-conformant)
2. For non-conformant files, create a companion `.txt` file
3. Update SOURCES.md with attribution
4. Include license information
