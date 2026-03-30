#!/bin/bash
# Fetch and curate RAW samples from raw.pixls.us for the conformance suite.
#
# Prerequisites:
#   - Git LFS installed (git lfs install)
#   - ~2 GB free disk for the temporary clone
#
# Usage:
#   ./fetch.sh [--keep-clone]
#
# This script:
#   1. Clones the raw.pixls.us LFS data repo to a temp directory
#   2. Selects one representative file per camera/format category
#   3. Copies them into the conformance directory structure
#   4. Generates invalid variants (truncated, corrupted)
#   5. Cleans up the temporary clone (unless --keep-clone)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
CLONE_DIR="/tmp/raw-pixls-data"
KEEP_CLONE=false

if [[ "${1:-}" == "--keep-clone" ]]; then
    KEEP_CLONE=true
fi

# Check prerequisites
if ! command -v git-lfs &>/dev/null && ! git lfs version &>/dev/null; then
    echo "ERROR: Git LFS is required. Install it: https://git-lfs.github.com/"
    exit 1
fi

echo "=== Step 1: Clone raw.pixls.us data ==="
if [[ -d "$CLONE_DIR/.git" ]]; then
    echo "Using existing clone at $CLONE_DIR"
    cd "$CLONE_DIR" && git lfs pull
else
    echo "Cloning raw.pixls.us (this may take a while)..."
    git clone https://raw.pixls.us/data.lfs.git "$CLONE_DIR"
fi

echo ""
echo "=== Step 2: Create directory structure ==="
mkdir -p "$SCRIPT_DIR/valid/bayer"
mkdir -p "$SCRIPT_DIR/valid/xtrans"
mkdir -p "$SCRIPT_DIR/valid/linear"
mkdir -p "$SCRIPT_DIR/valid/compressed"
mkdir -p "$SCRIPT_DIR/valid/mobile"
mkdir -p "$SCRIPT_DIR/valid/proprietary"
mkdir -p "$SCRIPT_DIR/invalid"
mkdir -p "$SCRIPT_DIR/edge-cases"

echo ""
echo "=== Step 3: Select representative samples ==="

# Helper: find first matching file by extension in a camera dir
find_sample() {
    local search_dir="$1"
    local ext="$2"
    find "$search_dir" -iname "*.$ext" -type f 2>/dev/null | head -1
}

# Bayer CFA - one per brand
declare -A BAYER_TARGETS=(
    ["Canon_EOS_350D"]="cr2"
    ["Canon_EOS_R"]="cr3"
    ["Nikon_D40"]="nef"
    ["Sony_NEX-3"]="arw"
)

for camera in "${!BAYER_TARGETS[@]}"; do
    ext="${BAYER_TARGETS[$camera]}"
    # raw.pixls.us organizes by Make/Model
    sample=$(find "$CLONE_DIR" -path "*${camera}*" -iname "*.${ext}" -type f 2>/dev/null | head -1)
    if [[ -n "$sample" ]]; then
        dest_name="$(echo "$camera" | tr '/' '_' | tr ' ' '_').${ext}"
        cp "$sample" "$SCRIPT_DIR/valid/bayer/$dest_name"
        echo "  Bayer: $dest_name ($(du -h "$sample" | cut -f1))"
    else
        echo "  WARNING: No sample found for $camera (.$ext)"
    fi
done

# X-Trans CFA
sample=$(find "$CLONE_DIR" -path "*Fuji*" -iname "*.raf" -type f 2>/dev/null | head -1)
if [[ -n "$sample" ]]; then
    cp "$sample" "$SCRIPT_DIR/valid/xtrans/fuji_xtrans.raf"
    echo "  X-Trans: fuji_xtrans.raf"
fi

# Proprietary formats - one each
declare -A PROP_FORMATS=(
    ["Olympus"]="orf"
    ["Panasonic"]="rw2"
)

for brand in "${!PROP_FORMATS[@]}"; do
    ext="${PROP_FORMATS[$brand]}"
    sample=$(find "$CLONE_DIR" -path "*${brand}*" -iname "*.${ext}" -type f 2>/dev/null | head -1)
    if [[ -n "$sample" ]]; then
        dest_name="${brand,,}_sample.${ext}"
        cp "$sample" "$SCRIPT_DIR/valid/proprietary/$dest_name"
        echo "  Proprietary: $dest_name"
    fi
done

# DNG samples (if available in the repo)
sample=$(find "$CLONE_DIR" -iname "*.dng" -type f 2>/dev/null | head -1)
if [[ -n "$sample" ]]; then
    cp "$sample" "$SCRIPT_DIR/valid/bayer/sample.dng"
    echo "  DNG: sample.dng"
fi

echo ""
echo "=== Step 4: Generate invalid variants ==="

# Pick a valid file to corrupt
VALID_FILE=$(find "$SCRIPT_DIR/valid" -type f | head -1)
if [[ -n "$VALID_FILE" ]]; then
    # Truncated header (first 100 bytes)
    head -c 100 "$VALID_FILE" > "$SCRIPT_DIR/invalid/truncated_header.raw"
    echo "  Created: truncated_header.raw"

    # Truncated data (first 50%)
    FILE_SIZE=$(stat -c%s "$VALID_FILE")
    HALF=$((FILE_SIZE / 2))
    head -c "$HALF" "$VALID_FILE" > "$SCRIPT_DIR/invalid/truncated_data.raw"
    echo "  Created: truncated_data.raw"

    # Corrupt byte order marker
    cp "$VALID_FILE" "$SCRIPT_DIR/invalid/bad_byte_order.raw"
    printf '\x00\x00' | dd of="$SCRIPT_DIR/invalid/bad_byte_order.raw" bs=1 seek=0 count=2 conv=notrunc 2>/dev/null
    echo "  Created: bad_byte_order.raw"

    # Empty file
    : > "$SCRIPT_DIR/invalid/empty.raw"
    echo "  Created: empty.raw"

    # Not a TIFF (JPEG header)
    printf '\xff\xd8\xff\xe0' > "$SCRIPT_DIR/invalid/not_tiff.raw"
    echo "  Created: not_tiff.raw"
fi

echo ""
echo "=== Step 5: Summary ==="
echo "Valid files:   $(find "$SCRIPT_DIR/valid" -type f | wc -l)"
echo "Invalid files: $(find "$SCRIPT_DIR/invalid" -type f | wc -l)"
echo "Edge cases:    $(find "$SCRIPT_DIR/edge-cases" -type f 2>/dev/null | wc -l)"
echo "Total size:    $(du -sh "$SCRIPT_DIR" | cut -f1)"

if [[ "$KEEP_CLONE" == "false" ]]; then
    echo ""
    echo "Cleaning up temporary clone..."
    rm -rf "$CLONE_DIR"
fi

echo ""
echo "Done! Don't forget to track new files with Git LFS:"
echo "  cd $(dirname "$SCRIPT_DIR")"
echo "  git lfs track 'raw-conformance/valid/**'"
echo "  git add .gitattributes raw-conformance/"
echo "  git commit -m 'feat: add RAW/DNG conformance suite (Git LFS)'"
