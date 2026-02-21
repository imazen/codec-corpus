#!/bin/bash
# Generate clic2025-1024/ — center-cropped 1024x1024 variants of CLIC 2025 images.
#
# Uses vipsthumbnail for high-quality Lanczos resize + smartcrop, then adds
# a proper sRGB PNG chunk (vips doesn't write one by default).
#
# Requirements: vips (libvips), python3
#
# Usage: bash generate-clic2025-1024.sh
# Output: clic2025-1024/*.png (1024x1024, 8-bit sRGB)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
OUTDIR="$SCRIPT_DIR/clic2025-1024"
SOURCES=("$SCRIPT_DIR/clic2025/training" "$SCRIPT_DIR/clic2025/final-test")
SIZE=1024

# Check dependencies
for cmd in vipsthumbnail python3; do
    if ! command -v "$cmd" &>/dev/null; then
        echo "ERROR: $cmd not found" >&2
        exit 1
    fi
done

mkdir -p "$OUTDIR"

count=0
skipped=0
for srcdir in "${SOURCES[@]}"; do
    if [ ! -d "$srcdir" ]; then
        echo "WARNING: source directory not found: $srcdir" >&2
        continue
    fi
    for img in "$srcdir"/*.png; do
        [ -f "$img" ] || continue
        name=$(basename "$img")
        out="$OUTDIR/$name"

        if [ -f "$out" ]; then
            skipped=$((skipped + 1))
            continue
        fi

        printf "\r  %s" "$name" >&2
        vipsthumbnail "$img" -s "${SIZE}x${SIZE}" --smartcrop centre -o "$out"
        count=$((count + 1))
    done
done

echo "" >&2

# Add sRGB chunk to all generated PNGs (vips writes none by default).
# This ensures consistent color space signaling across the corpus.
echo "Adding sRGB chunks..." >&2

python3 - "$OUTDIR" <<'PYTHON'
import struct
import zlib
import sys
import os

def add_srgb_chunk(path):
    """Add sRGB chunk after IHDR, remove any gAMA/cHRM/iCCP if present."""
    with open(path, "rb") as f:
        sig = f.read(8)
        if sig != b"\x89PNG\r\n\x1a\n":
            return False
        chunks = []
        while True:
            header = f.read(8)
            if len(header) < 8:
                break
            length = struct.unpack(">I", header[:4])[0]
            chunk_type = header[4:8]
            data = f.read(length)
            crc = f.read(4)
            chunks.append((chunk_type, data, crc))
            if chunk_type == b"IEND":
                break

    # Check if sRGB already present
    has_srgb = any(ct == b"sRGB" for ct, _, _ in chunks)
    has_gama = any(ct == b"gAMA" for ct, _, _ in chunks)
    has_chrm = any(ct == b"cHRM" for ct, _, _ in chunks)
    has_iccp = any(ct == b"iCCP" for ct, _, _ in chunks)

    if has_srgb and not has_gama and not has_chrm and not has_iccp:
        return False  # Already clean

    # Rebuild: strip gAMA/cHRM/iCCP, add sRGB after IHDR
    new_chunks = []
    srgb_added = False
    for chunk_type, data, crc in chunks:
        if chunk_type in (b"gAMA", b"cHRM", b"iCCP", b"sRGB"):
            continue
        new_chunks.append((chunk_type, data, crc))
        if chunk_type == b"IHDR" and not srgb_added:
            srgb_data = struct.pack("B", 0)  # rendering intent: perceptual
            srgb_crc = struct.pack(">I", zlib.crc32(b"sRGB" + srgb_data) & 0xFFFFFFFF)
            new_chunks.append((b"sRGB", srgb_data, srgb_crc))
            srgb_added = True

    with open(path, "wb") as f:
        f.write(sig)
        for chunk_type, data, crc in new_chunks:
            f.write(struct.pack(">I", len(data)))
            f.write(chunk_type)
            f.write(data)
            f.write(crc)
    return True

outdir = sys.argv[1]
fixed = 0
for fn in sorted(os.listdir(outdir)):
    if fn.lower().endswith(".png"):
        if add_srgb_chunk(os.path.join(outdir, fn)):
            fixed += 1

print(f"  {fixed} files updated with sRGB chunk", file=sys.stderr)
PYTHON

total=$((count + skipped))
echo "Done. ${count} generated, ${skipped} skipped (already exist). ${total} total in ${OUTDIR}/" >&2
