#!/bin/bash
# Generate reference outputs from JXL files using libjxl (djxl)
#
# Usage: ./generate_references.sh [djxl_path]
#
# If djxl_path is not provided, looks for djxl in PATH or common locations.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REF_DIR="$SCRIPT_DIR/reference"

# Find djxl
if [ -n "$1" ]; then
    DJXL="$1"
elif command -v djxl &> /dev/null; then
    DJXL="djxl"
else
    # Try common locations
    for path in \
        "$HOME/libjxl/build/tools/djxl" \
        "/usr/local/bin/djxl" \
        "/usr/bin/djxl"; do
        if [ -x "$path" ]; then
            DJXL="$path"
            break
        fi
    done
fi

if [ -z "$DJXL" ] || ! command -v "$DJXL" &> /dev/null; then
    echo "Error: djxl not found. Please install libjxl or provide path as argument."
    echo "Usage: $0 [path/to/djxl]"
    exit 1
fi

echo "Using djxl: $DJXL"
"$DJXL" --version 2>&1 | head -1

# Create reference directories
mkdir -p "$REF_DIR"/{conformance,features,photographic,edge-cases}

count=0
failed=0

for subdir in conformance features photographic edge-cases; do
    echo ""
    echo "Processing $subdir/..."

    for jxl in "$SCRIPT_DIR/$subdir"/*.jxl; do
        [ -f "$jxl" ] || continue
        name=$(basename "$jxl" .jxl)
        out_ppm="$REF_DIR/$subdir/${name}.ppm"
        out_png="$REF_DIR/$subdir/${name}.png"

        # Try PPM first (exact pixels), fall back to PNG for grayscale/complex
        if "$DJXL" "$jxl" "$out_ppm" 2>/dev/null; then
            echo "  ✓ $name.ppm"
            ((count++))
        elif "$DJXL" "$jxl" "$out_png" 2>/dev/null; then
            echo "  ✓ $name.png"
            ((count++))
        else
            echo "  ✗ $name (FAILED)"
            ((failed++))
        fi
    done
done

echo ""
echo "=========================================="
echo "Generated $count reference outputs"
[ $failed -gt 0 ] && echo "Failed: $failed"
echo "Output directory: $REF_DIR"
du -sh "$REF_DIR"
