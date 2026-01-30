#!/usr/bin/env bash
# Download subsets from the QOI Benchmark Suite
# Source: https://qoiformat.org/benchmark/
#
# Usage:
#   ./download.sh                    # Download all subsets (default)
#   ./download.sh icon_512 icon_64   # Download specific subsets
#   ./download.sh --list             # List available subsets
#
# Available subsets:
#   screenshot_web   -  15 files, ~39 MB  (CC0)           [committed to repo]
#   icon_512         - 214 files, ~12 MB  (Public Domain)
#   icon_64          - 214 files, ~1.3 MB (Public Domain)
#   screenshot_game  - 619 files, ~256 MB (CC BY-SA 3.0)
#   textures_pk      - 1004 files, ~44 MB
#   textures_pk01    - 115 files, ~19 MB
#   textures_pk02    - 237 files, ~99 MB
#   textures_plants  -  61 files, ~50 MB
#   textures_photo   -  21 files, ~37 MB
#   photo_kodak      -  25 files, ~15 MB
#   photo_tecnick    - 101 files, ~228 MB
#   photo_wikipedia  -  50 files, ~85 MB
#   pngimg           - 189 files, ~220 MB (CC BY-NC 4.0)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
TAR_URL="https://qoiformat.org/benchmark/qoi_benchmark_suite.tar"
TMPDIR="${SCRIPT_DIR}/.tmp_download"

ALL_SUBSETS=(
    screenshot_web icon_512 icon_64 screenshot_game
    textures_pk textures_pk01 textures_pk02 textures_plants textures_photo
    photo_kodak photo_tecnick photo_wikipedia pngimg
)

if [ "${1:-}" = "--list" ]; then
    echo "Available subsets:"
    for s in "${ALL_SUBSETS[@]}"; do echo "  $s"; done
    exit 0
fi

if [ $# -gt 0 ]; then
    SUBSETS=("$@")
else
    SUBSETS=("${ALL_SUBSETS[@]}")
fi

echo "Downloading QOI Benchmark Suite..."
echo "This downloads the full ~1.1 GB tarball, then extracts selected subsets."
echo "Subsets: ${SUBSETS[*]}"
echo ""

mkdir -p "$TMPDIR"
TAR_FILE="${TMPDIR}/qoi_benchmark_suite.tar"

if [ ! -f "$TAR_FILE" ]; then
    curl -L -o "$TAR_FILE" "$TAR_URL"
else
    echo "Tarball already downloaded, reusing."
fi

echo ""
echo "Extracting subsets..."

for subset in "${SUBSETS[@]}"; do
    dest="${SCRIPT_DIR}/${subset}"
    if [ -d "$dest" ]; then
        echo "  ${subset}/ already exists, skipping."
        continue
    fi
    mkdir -p "$dest"
    tar xf "$TAR_FILE" -C "$dest" --strip-components=2 "images/${subset}/"
    count=$(find "$dest" -type f | wc -l)
    echo "  ${subset}/  (${count} files)"
done

echo ""
echo "Cleaning up tarball..."
rm -rf "$TMPDIR"

echo ""
echo "Done. Extracted subsets:"
for subset in "${SUBSETS[@]}"; do
    dest="${SCRIPT_DIR}/${subset}"
    [ -d "$dest" ] && du -sh "$dest"
done
