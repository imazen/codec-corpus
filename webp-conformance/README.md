# WebP Conformance Test Suite

This directory contains WebP test files for codec conformance testing. Files are organized by RFC compliance and test coverage.

## Directory Structure

```
webp-conformance/
├── README.md                          # This file
├── generate_corpus.py                 # Script to regenerate synthetic test files
├── sources/                           # Source images for generation
│   ├── src_grad_16.png               # 16×16 gradient
│   ├── src_checker_odd.png           # 129×131 checkerboard (odd dimensions)
│   └── src_noise.png                 # 64×64 Perlin noise
├── valid/                             # RFC 6386 compliant (225 files, 1.2 MB)
│   └── *.webp                        # Synthetic and real-world test images
├── invalid/                           # RFC violations (pending)
│   ├── truncated/                    # Incomplete files
│   ├── malformed/                    # Invalid syntax
│   ├── oversized/                    # Dimension violations
│   └── reserved/                     # Reserved field violations
└── non-conformant/                    # Gray area (pending)
    ├── loop_filter_edge/             # Loop filter boundary conditions
    ├── color_space/                  # Color space interpretation
    ├── alpha_blend/                  # Alpha channel semantics
    └── rounding/                     # Rounding differences
```

## File Sources

### Valid Files (225 WebP images)

#### Synthetic Test Suite (216 files from libwebp-rs)

Procedurally generated WebP files testing VP8 encoder parameters:

**Parameters:**
- **Quality levels**: 0 (minimum), 50 (medium), 90 (high)
- **Encoding methods**: 0 (fast), 4 (balanced)
- **Loop filters**: default, -f 0 (disabled), -f 50 -strong, -f 50 -nostrong
- **Segmentation**: default, -segments 2 -sns 80, -segments 4

This generates 3 × 2 × 4 × 3 = **72 variations per source image × 3 sources = 216 files**

**Source Images:**
- `src_grad_16.png` — Smooth 16×16 gradient (tests compression of low-frequency content)
- `src_checker_odd.png` — 129×131 checkerboard (tests high-frequency patterns and odd dimensions)
- `src_noise.png` — 64×64 Perlin noise (tests mid-frequency complexity)

**Characteristics:**
- ✅ All RFC 6386 compliant
- ✅ Covers VP8 compression parameter space
- ✅ Tests edge cases (Q=0, odd dimensions)
- ✅ Reproducible (can regenerate from sources)
- ❌ Synthetic only (not photographic)
- ❌ Small images (max 129×131)

#### Real-World Test Images (10 files from image-rs)

Additional test files from the image-rs test suite:

**Lossless:**
- `simple.webp` (44 KB) — Simple image, VP8L
- `2-color.webp` (314 bytes) — Minimal 2-color lossless
- `simple_xmp.webp` (47 KB) — Lossless with XMP metadata
- `multi-color.webp` (152 KB) — Complex color lossless

**Lossy:**
- `simple-rgb.webp` (2.2 KB) — RGB VP8
- `simple-gray.webp` (1.3 KB) — Grayscale VP8

**Extended (VP8/VP8L with additional features):**
- `anim.webp` (11 KB) — Animated WebP
- `lossy_alpha.webp` (1.3 KB) — VP8 with alpha channel
- `advertises_rgba_but_frames_are_rgb.webp` (52 KB) — Edge case in format specification

## Test Categories

### ✅ VALID/

All files in `valid/` strictly conform to RFC 6386 and should decode identically on conformant decoders.

**Test expectations:**
- Must decode without errors
- Dimensions should match file headers
- Output format (RGB, RGBA, YUV) should match headers
- Alpha channel (if present) should decode correctly

### ❌ INVALID/ (Pending)

Files that violate RFC 6386. Behavior is undefined by spec.

**Categories:**
- **truncated/** — Incomplete files (bitstream or container cut off mid-frame)
- **malformed/** — Invalid syntax (bad chunk sizes, invalid headers, corrupted frames)
- **oversized/** — Dimension limits exceeded (width/height > 16384)
- **reserved/** — Reserved fields/FourCCs used incorrectly

**Test expectations:**
- Must NOT crash (memory safety is critical)
- May reject with error or produce garbage (both acceptable)
- Useful for robustness/fuzzing

### ⚠️ NON-CONFORMANT/ (Pending)

Files that are syntactically valid but decode differently across decoders due to spec ambiguities.

**Categories:**
- **loop_filter_edge/** — Loop filter boundary conditions
- **color_space/** — ICC profile and colorspace interpretation
- **alpha_blend/** — Alpha premultiplication semantics
- **rounding/** — Numerical precision differences (loop filter, dequantization)

**Test expectations:**
- Must decode without error
- Output may differ from reference decoders (acceptable)
- Useful for regression testing within a single decoder

## Testing Usage

### Rust Integration

Add to your test suite:

```rust
#[test]
fn test_webp_valid_files() {
    let corpus_dir = std::env::var("CORPUS_DIR")
        .unwrap_or_else(|_| format!("{}/ codec-corpus/webp-conformance/valid",
                                     std::env::var("HOME").unwrap()));

    for entry in std::fs::read_dir(&corpus_dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().map_or(false, |ext| ext == "webp") {
            let data = std::fs::read(&path).unwrap();
            let result = decode_webp(&data);

            assert!(result.is_ok(), "Valid file should decode: {:?}", path);
            let image = result.unwrap();
            assert!(image.width > 0 && image.height > 0);
        }
    }
}

#[test]
fn test_webp_invalid_files() {
    let corpus_dir = std::env::var("CORPUS_DIR")
        .unwrap_or_else(|_| format!("{}/ codec-corpus/webp-conformance/invalid",
                                     std::env::var("HOME").unwrap()));

    for entry in std::fs::read_dir(&corpus_dir).unwrap_or_else(|_| return) {
        let path = entry.unwrap().path();
        if path.extension().map_or(false, |ext| ext == "webp") {
            let data = std::fs::read(&path).unwrap();
            // Should not crash (main requirement)
            let _ = decode_webp(&data);
        }
    }
}
```

### CI Integration

```yaml
conformance:
  name: WebP Conformance
  runs-on: ubuntu-latest
  if: github.ref == 'refs/heads/main'
  continue-on-error: true
  steps:
    - uses: actions/checkout@v6
    - uses: dtolnay/rust-toolchain@stable
    - name: Get Corpus
      run: |
        [ -d ~/codec-corpus ] || \
          git clone --depth=1 https://github.com/imazen/codec-corpus.git ~/codec-corpus
    - name: Test WebP Decoding
      run: cargo test --release test_webp_valid -- --ignored --nocapture
      env:
        CORPUS_DIR: ~/codec-corpus/webp-conformance/valid
```

## Regenerating Synthetic Files

To regenerate the synthetic test suite from source images:

```bash
python3 generate_corpus.py
```

This requires `cwebp` (from libwebp):

```bash
# macOS
brew install webp

# Ubuntu
sudo apt-get install webp

# Or build from source: https://github.com/webmproject/libwebp
```

## File Statistics

| Category | Files | Size |
|----------|-------|------|
| **valid/** | 225 | 1.2 MB |
| **invalid/** (pending) | TBD | TBD |
| **non-conformant/** (pending) | TBD | TBD |
| **sources/** | 3 PNG | 12 KB |

**Total (current):** 225 WebP files, 1.2 MB

## Creating Invalid Files

To create invalid test cases, corrupt valid files:

```bash
# Truncate a file (incomplete bitstream)
truncate -s 500 valid/file.webp > invalid/truncated/incomplete.webp

# Corrupt chunk size (Python)
python3 << 'EOF'
import struct
data = open('valid/file.webp', 'rb').read()
modified = bytearray(data)
# RIFF chunk size at bytes 4-7
modified[4:8] = struct.pack('<I', len(data) - 100)
open('invalid/malformed/bad_chunk_size.webp', 'wb').write(modified)
EOF

# Corrupt VP8 frame header
python3 << 'EOF'
data = bytearray(open('valid/file.webp', 'rb').read())
vp8_pos = data.find(b'VP8 ')
if vp8_pos >= 0:
    data[vp8_pos + 8] = 0xFF  # Invalid frame tag
open('invalid/malformed/bad_vp8_header.webp', 'wb').write(data)
EOF
```

## References

- **RFC 6386**: WebP format specification (https://tools.ietf.org/html/rfc6386)
- **libwebp**: Reference encoder/decoder (https://github.com/webmproject/libwebp)
- **image-rs**: Rust image library with WebP test vectors (https://github.com/image-rs/image)

## Related

For corpus integration strategy, see: `~/work/work-maintenance/CORPUS-FROM-CODEC-EVAL.md`

For detailed conformance structure design, see: `~/work/work-maintenance/WEBP-CONFORMANCE-STRUCTURE.md`
