#!/usr/bin/env python3
"""Generate farbfeld conformance test files.

Farbfeld format (https://tools.suckless.org/farbfeld/):
  - Magic: "farbfeld" (8 bytes ASCII)
  - Width: 4 bytes big-endian u32
  - Height: 4 bytes big-endian u32
  - Pixel data: width*height pixels, each = 4 channels (R,G,B,A) x 2 bytes big-endian u16
  - Total header: 16 bytes
  - Total file size: 16 + width * height * 8

This script is idempotent: running it multiple times produces identical output.
"""

import os
import struct
from pathlib import Path

SCRIPT_DIR = Path(__file__).parent
VALID_DIR = SCRIPT_DIR / "valid"
INVALID_DIR = SCRIPT_DIR / "invalid"
EDGE_DIR = SCRIPT_DIR / "edge-cases"

MAGIC = b"farbfeld"


def write_ff(path: Path, width: int, height: int, pixels: list[tuple[int, int, int, int]]) -> None:
    """Write a valid farbfeld file.

    pixels: list of (R, G, B, A) tuples, each channel u16 [0, 65535].
    Length must equal width * height.
    """
    assert len(pixels) == width * height, (
        f"Expected {width * height} pixels, got {len(pixels)}"
    )
    with open(path, "wb") as f:
        f.write(MAGIC)
        f.write(struct.pack(">II", width, height))
        for r, g, b, a in pixels:
            f.write(struct.pack(">HHHH", r, g, b, a))


def write_raw(path: Path, data: bytes) -> None:
    """Write raw bytes to a file (for invalid test cases)."""
    with open(path, "wb") as f:
        f.write(data)


def make_header(width: int, height: int) -> bytes:
    """Build a 16-byte farbfeld header."""
    return MAGIC + struct.pack(">II", width, height)


def pixel_bytes(r: int, g: int, b: int, a: int) -> bytes:
    """Encode a single pixel as 8 bytes big-endian."""
    return struct.pack(">HHHH", r, g, b, a)


# ---------------------------------------------------------------------------
# Valid files
# ---------------------------------------------------------------------------

def generate_valid() -> None:
    VALID_DIR.mkdir(parents=True, exist_ok=True)

    # 1x1_black.ff
    write_ff(VALID_DIR / "1x1_black.ff", 1, 1, [(0, 0, 0, 65535)])

    # 1x1_white.ff
    write_ff(VALID_DIR / "1x1_white.ff", 1, 1, [(65535, 65535, 65535, 65535)])

    # 1x1_red.ff
    write_ff(VALID_DIR / "1x1_red.ff", 1, 1, [(65535, 0, 0, 65535)])

    # 1x1_transparent.ff
    write_ff(VALID_DIR / "1x1_transparent.ff", 1, 1, [(0, 0, 0, 0)])

    # 4x4_solid_blue.ff
    write_ff(VALID_DIR / "4x4_solid_blue.ff", 4, 4, [(0, 0, 65535, 65535)] * 16)

    # 4x4_gradient.ff — horizontal gradient from black to white
    # Each row: 4 pixels going from black to white
    pixels_grad = []
    for _row in range(4):
        for col in range(4):
            v = col * 65535 // 3  # 0, 21845, 43690, 65535
            pixels_grad.append((v, v, v, 65535))
    write_ff(VALID_DIR / "4x4_gradient.ff", 4, 4, pixels_grad)

    # 8x8_checkerboard.ff — alternating black/white
    pixels_check = []
    for row in range(8):
        for col in range(8):
            if (row + col) % 2 == 0:
                pixels_check.append((0, 0, 0, 65535))
            else:
                pixels_check.append((65535, 65535, 65535, 65535))
    write_ff(VALID_DIR / "8x8_checkerboard.ff", 8, 8, pixels_check)

    # 4x4_semitransparent.ff — all pixels at alpha=32768 (50%)
    write_ff(
        VALID_DIR / "4x4_semitransparent.ff", 4, 4,
        [(32768, 32768, 32768, 32768)] * 16,
    )

    # 2x3_colors.ff — 6 different colors: R, G, B, C, M, Y (row-major)
    colors = [
        (65535, 0, 0, 65535),      # Red
        (0, 65535, 0, 65535),      # Green
        (0, 0, 65535, 65535),      # Blue
        (0, 65535, 65535, 65535),   # Cyan
        (65535, 0, 65535, 65535),   # Magenta
        (65535, 65535, 0, 65535),   # Yellow
    ]
    write_ff(VALID_DIR / "2x3_colors.ff", 2, 3, colors)

    # 16x16_rgb_ramp.ff — smooth RGB ramp across 256 pixels
    pixels_ramp = []
    for i in range(256):
        r = (i % 16) * 65535 // 15
        g = (i // 16) * 65535 // 15
        b = ((i % 16) + (i // 16)) * 65535 // 30
        pixels_ramp.append((r, g, b, 65535))
    write_ff(VALID_DIR / "16x16_rgb_ramp.ff", 16, 16, pixels_ramp)

    # 100x1_wide.ff — single-row image, gradient
    pixels_wide = []
    for col in range(100):
        v = col * 65535 // 99
        pixels_wide.append((v, v, v, 65535))
    write_ff(VALID_DIR / "100x1_wide.ff", 100, 1, pixels_wide)

    # 1x100_tall.ff — single-column image, gradient
    pixels_tall = []
    for row in range(100):
        v = row * 65535 // 99
        pixels_tall.append((v, v, v, 65535))
    write_ff(VALID_DIR / "1x100_tall.ff", 1, 100, pixels_tall)


# ---------------------------------------------------------------------------
# Invalid files
# ---------------------------------------------------------------------------

def generate_invalid() -> None:
    INVALID_DIR.mkdir(parents=True, exist_ok=True)

    # bad_magic.ff — starts with "farbfool"
    header = b"farbfool" + struct.pack(">II", 1, 1)
    data = header + pixel_bytes(0, 0, 0, 65535)
    write_raw(INVALID_DIR / "bad_magic.ff", data)

    # empty.ff — 0 bytes
    write_raw(INVALID_DIR / "empty.ff", b"")

    # header_only.ff — 16-byte header claiming 4x4 but no pixel data
    write_raw(INVALID_DIR / "header_only.ff", make_header(4, 4))

    # truncated_header.ff — only 10 bytes (magic + partial width)
    write_raw(INVALID_DIR / "truncated_header.ff", MAGIC + b"\x00\x04")

    # truncated_pixels.ff — valid header for 4x4 but only half the pixel data
    header = make_header(4, 4)
    # Full data would be 4*4*8 = 128 bytes; write only 64
    half_pixels = b"\x00" * 64
    write_raw(INVALID_DIR / "truncated_pixels.ff", header + half_pixels)

    # extra_data.ff — valid 1x1 image with 100 extra bytes appended
    header = make_header(1, 1)
    px = pixel_bytes(65535, 0, 0, 65535)
    extra = b"\xDE\xAD" * 50  # 100 bytes of junk
    write_raw(INVALID_DIR / "extra_data.ff", header + px + extra)

    # zero_width.ff — width=0, height=4
    write_raw(INVALID_DIR / "zero_width.ff", make_header(0, 4))

    # zero_height.ff — width=4, height=0
    write_raw(INVALID_DIR / "zero_height.ff", make_header(4, 0))

    # zero_both.ff — width=0, height=0
    write_raw(INVALID_DIR / "zero_both.ff", make_header(0, 0))


# ---------------------------------------------------------------------------
# Edge cases
# ---------------------------------------------------------------------------

def generate_edge_cases() -> None:
    EDGE_DIR.mkdir(parents=True, exist_ok=True)

    # max_dimension_1d.ff — width=1000, height=1 (~8 KB)
    pixels_wide = []
    for col in range(1000):
        v = col * 65535 // 999
        pixels_wide.append((v, 0, 65535 - v, 65535))
    write_ff(EDGE_DIR / "max_dimension_1d.ff", 1000, 1, pixels_wide)

    # large_square.ff — 64x64 gradient (32 KB)
    pixels_sq = []
    for row in range(64):
        for col in range(64):
            r = col * 65535 // 63
            g = row * 65535 // 63
            b = 65535 - ((row + col) * 65535 // 126)
            pixels_sq.append((r, g, b, 65535))
    write_ff(EDGE_DIR / "large_square.ff", 64, 64, pixels_sq)

    # single_channel_illusion.ff — grayscale: R=G=B, 4x4
    pixels_gray = []
    for i in range(16):
        v = i * 65535 // 15
        pixels_gray.append((v, v, v, 65535))
    write_ff(EDGE_DIR / "single_channel_illusion.ff", 4, 4, pixels_gray)


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main() -> None:
    generate_valid()
    generate_invalid()
    generate_edge_cases()
    print("Generated all farbfeld conformance test files.")


if __name__ == "__main__":
    main()
