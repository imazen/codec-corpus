#!/usr/bin/env python3
"""Generate PNM/PAM conformance test files.

Idempotent: running twice produces identical output.
All pixel data is deterministic.
"""

import os
import struct
import sys

BASE = os.path.dirname(os.path.abspath(__file__))

DIRS = [
    "valid/pbm",
    "valid/pgm",
    "valid/ppm",
    "valid/pam",
    "invalid",
    "edge-cases",
]


def ensure_dirs():
    for d in DIRS:
        path = os.path.join(BASE, d)
        os.makedirs(path, exist_ok=True)


def write_file(relpath, data):
    """Write bytes or str to a file under BASE."""
    path = os.path.join(BASE, relpath)
    if isinstance(data, str):
        data = data.encode("ascii")
    with open(path, "wb") as f:
        f.write(data)
    print(f"  {relpath} ({len(data)} bytes)")


# ---------------------------------------------------------------------------
# PBM helpers
# ---------------------------------------------------------------------------

def checkerboard_8x8():
    """8x8 checkerboard: (row+col) % 2."""
    rows = []
    for r in range(8):
        rows.append([(r + c) % 2 for c in range(8)])
    return rows


def pbm_ascii(width, height, pixels):
    """P1 ASCII PBM. pixels is list of rows, each row list of 0/1."""
    lines = [f"P1\n{width} {height}\n"]
    for row in pixels:
        lines.append(" ".join(str(v) for v in row) + "\n")
    return "".join(lines)


def pbm_binary(width, height, pixels):
    """P4 binary PBM. Rows are packed into bytes, MSB first, padded to byte boundary."""
    header = f"P4\n{width} {height}\n".encode("ascii")
    data = bytearray()
    for row in pixels:
        byte_count = (width + 7) // 8
        for bi in range(byte_count):
            byte_val = 0
            for bit in range(8):
                col = bi * 8 + bit
                if col < width and row[col]:
                    byte_val |= (1 << (7 - bit))
            data.append(byte_val)
    return header + bytes(data)


# ---------------------------------------------------------------------------
# PGM helpers
# ---------------------------------------------------------------------------

def pgm_ascii(width, height, maxval, pixels):
    """P2 ASCII PGM. pixels is flat list or list of rows."""
    lines = [f"P2\n{width} {height}\n{maxval}\n"]
    if pixels and isinstance(pixels[0], list):
        for row in pixels:
            lines.append(" ".join(str(v) for v in row) + "\n")
    else:
        for i in range(0, len(pixels), width):
            lines.append(" ".join(str(v) for v in pixels[i:i + width]) + "\n")
    return "".join(lines)


def pgm_binary(width, height, maxval, pixels):
    """P5 binary PGM. pixels is flat list."""
    header = f"P5\n{width} {height}\n{maxval}\n".encode("ascii")
    if isinstance(pixels[0], list):
        flat = [v for row in pixels for v in row]
    else:
        flat = pixels
    if maxval > 255:
        data = b"".join(struct.pack(">H", v) for v in flat)
    else:
        data = bytes(flat)
    return header + data


# ---------------------------------------------------------------------------
# PPM helpers
# ---------------------------------------------------------------------------

def ppm_ascii(width, height, maxval, pixels):
    """P3 ASCII PPM. pixels is flat list of (r,g,b) tuples or list of rows of tuples."""
    lines = [f"P3\n{width} {height}\n{maxval}\n"]
    if pixels and isinstance(pixels[0], list):
        for row in pixels:
            vals = []
            for r, g, b in row:
                vals.extend([str(r), str(g), str(b)])
            lines.append(" ".join(vals) + "\n")
    else:
        for i in range(0, len(pixels), width):
            vals = []
            for r, g, b in pixels[i:i + width]:
                vals.extend([str(r), str(g), str(b)])
            lines.append(" ".join(vals) + "\n")
    return "".join(lines)


def ppm_binary(width, height, maxval, pixels):
    """P6 binary PPM. pixels is flat list of (r,g,b) tuples."""
    header = f"P6\n{width} {height}\n{maxval}\n".encode("ascii")
    if pixels and isinstance(pixels[0], list):
        flat = [v for row in pixels for v in row]
    else:
        flat = pixels
    if maxval > 255:
        data = b"".join(struct.pack(">HHH", r, g, b) for r, g, b in flat)
    else:
        data = b"".join(bytes([r, g, b]) for r, g, b in flat)
    return header + data


# ---------------------------------------------------------------------------
# PAM helper
# ---------------------------------------------------------------------------

def pam_file(width, height, depth, maxval, tupltype, pixels_flat_bytes):
    """P7 PAM file. pixels_flat_bytes is already packed binary data."""
    header = (
        f"P7\n"
        f"WIDTH {width}\n"
        f"HEIGHT {height}\n"
        f"DEPTH {depth}\n"
        f"MAXVAL {maxval}\n"
        f"TUPLTYPE {tupltype}\n"
        f"ENDHDR\n"
    ).encode("ascii")
    return header + pixels_flat_bytes


# ---------------------------------------------------------------------------
# Generators
# ---------------------------------------------------------------------------

def gen_valid_pbm():
    print("PBM (valid):")

    # 8x8 checkerboard
    cb = checkerboard_8x8()
    write_file("valid/pbm/checkerboard_8x8_ascii.pbm", pbm_ascii(8, 8, cb))
    write_file("valid/pbm/checkerboard_8x8_binary.pbm", pbm_binary(8, 8, cb))

    # 1x1 black (1 in PBM means black/ink)
    write_file("valid/pbm/1x1_black_ascii.pbm", "P1\n1 1\n1\n")
    write_file("valid/pbm/1x1_black_binary.pbm", pbm_binary(1, 1, [[1]]))

    # 1x1 white (0 in PBM means white/background)
    write_file("valid/pbm/1x1_white_ascii.pbm", "P1\n1 1\n0\n")
    write_file("valid/pbm/1x1_white_binary.pbm", pbm_binary(1, 1, [[0]]))

    # Wide 100x1
    wide = [[i % 2 for i in range(100)]]
    write_file("valid/pbm/wide_100x1_ascii.pbm", pbm_ascii(100, 1, wide))
    write_file("valid/pbm/wide_100x1_binary.pbm", pbm_binary(100, 1, wide))

    # Tall 1x100
    tall = [[i % 2] for i in range(100)]
    write_file("valid/pbm/tall_1x100_ascii.pbm", pbm_ascii(1, 100, tall))
    write_file("valid/pbm/tall_1x100_binary.pbm", pbm_binary(1, 100, tall))


def gen_valid_pgm():
    print("PGM (valid):")

    # 8x8 gradient maxval 255
    pixels_255 = []
    for r in range(8):
        row = []
        for c in range(8):
            row.append((r * 8 + c) * 255 // 63)
        pixels_255.append(row)
    flat_255 = [v for row in pixels_255 for v in row]
    write_file("valid/pgm/gradient_8x8_255_ascii.pgm", pgm_ascii(8, 8, 255, pixels_255))
    write_file("valid/pgm/gradient_8x8_255_binary.pgm", pgm_binary(8, 8, 255, flat_255))

    # 8x8 gradient maxval 65535 (16-bit)
    pixels_16 = []
    for r in range(8):
        row = []
        for c in range(8):
            row.append((r * 8 + c) * 65535 // 63)
        pixels_16.append(row)
    flat_16 = [v for row in pixels_16 for v in row]
    write_file("valid/pgm/gradient_8x8_65535_ascii.pgm", pgm_ascii(8, 8, 65535, pixels_16))
    write_file("valid/pgm/gradient_8x8_65535_binary.pgm", pgm_binary(8, 8, 65535, flat_16))

    # 4x4 maxval=1
    pixels_1 = []
    for r in range(4):
        row = []
        for c in range(4):
            row.append((r + c) % 2)
        pixels_1.append(row)
    flat_1 = [v for row in pixels_1 for v in row]
    write_file("valid/pgm/4x4_maxval1_ascii.pgm", pgm_ascii(4, 4, 1, pixels_1))
    write_file("valid/pgm/4x4_maxval1_binary.pgm", pgm_binary(4, 4, 1, flat_1))

    # 4x4 maxval=100
    pixels_100 = []
    for r in range(4):
        row = []
        for c in range(4):
            row.append((r * 4 + c) * 100 // 15)
        pixels_100.append(row)
    flat_100 = [v for row in pixels_100 for v in row]
    write_file("valid/pgm/4x4_maxval100_ascii.pgm", pgm_ascii(4, 4, 100, pixels_100))
    write_file("valid/pgm/4x4_maxval100_binary.pgm", pgm_binary(4, 4, 100, flat_100))


def gen_valid_ppm():
    print("PPM (valid):")

    # 4x4 color bars: columns of red, green, blue, white
    bars = []
    colors = [(255, 0, 0), (0, 255, 0), (0, 0, 255), (255, 255, 255)]
    for r in range(4):
        row = [colors[c] for c in range(4)]
        bars.append(row)
    flat_bars = [v for row in bars for v in row]
    write_file("valid/ppm/colorbars_4x4_ascii.ppm", ppm_ascii(4, 4, 255, bars))
    write_file("valid/ppm/colorbars_4x4_binary.ppm", ppm_binary(4, 4, 255, flat_bars))

    # 2x2 primary colors: red, green, blue, black
    primaries = [[(255, 0, 0), (0, 255, 0)], [(0, 0, 255), (0, 0, 0)]]
    flat_prim = [v for row in primaries for v in row]
    write_file("valid/ppm/primaries_2x2_ascii.ppm", ppm_ascii(2, 2, 255, primaries))
    write_file("valid/ppm/primaries_2x2_binary.ppm", ppm_binary(2, 2, 255, flat_prim))

    # 4x4 16-bit
    bars16 = []
    colors16 = [(65535, 0, 0), (0, 65535, 0), (0, 0, 65535), (65535, 65535, 65535)]
    for r in range(4):
        row = [colors16[c] for c in range(4)]
        bars16.append(row)
    flat_bars16 = [v for row in bars16 for v in row]
    write_file("valid/ppm/colorbars_4x4_16bit_ascii.ppm", ppm_ascii(4, 4, 65535, bars16))
    write_file("valid/ppm/colorbars_4x4_16bit_binary.ppm", ppm_binary(4, 4, 65535, flat_bars16))

    # 8x8 gradient
    grad = []
    for r in range(8):
        row = []
        for c in range(8):
            v = (r * 8 + c) * 255 // 63
            row.append((v, v // 2, 255 - v))
        grad.append(row)
    flat_grad = [v for row in grad for v in row]
    write_file("valid/ppm/gradient_8x8_ascii.ppm", ppm_ascii(8, 8, 255, grad))
    write_file("valid/ppm/gradient_8x8_binary.ppm", ppm_binary(8, 8, 255, flat_grad))


def gen_valid_pam():
    print("PAM (valid):")

    # GRAYSCALE 8-bit: 4x4 gradient
    pixels = bytearray()
    for r in range(4):
        for c in range(4):
            pixels.append((r * 4 + c) * 255 // 15)
    write_file("valid/pam/grayscale_4x4.pam",
               pam_file(4, 4, 1, 255, "GRAYSCALE", bytes(pixels)))

    # GRAYSCALE_ALPHA 8-bit: 4x4
    pixels = bytearray()
    for r in range(4):
        for c in range(4):
            gray = (r * 4 + c) * 255 // 15
            alpha = 255 - gray
            pixels.append(gray)
            pixels.append(alpha)
    write_file("valid/pam/grayscale_alpha_4x4.pam",
               pam_file(4, 4, 2, 255, "GRAYSCALE_ALPHA", bytes(pixels)))

    # RGB 8-bit: 4x4 color bars
    pixels = bytearray()
    colors = [(255, 0, 0), (0, 255, 0), (0, 0, 255), (255, 255, 255)]
    for r in range(4):
        for c in range(4):
            pixels.extend(colors[c])
    write_file("valid/pam/rgb_4x4.pam",
               pam_file(4, 4, 3, 255, "RGB", bytes(pixels)))

    # RGB_ALPHA 8-bit: 4x4
    pixels = bytearray()
    colors_a = [(255, 0, 0, 255), (0, 255, 0, 192), (0, 0, 255, 128), (255, 255, 255, 64)]
    for r in range(4):
        for c in range(4):
            pixels.extend(colors_a[c])
    write_file("valid/pam/rgb_alpha_4x4.pam",
               pam_file(4, 4, 4, 255, "RGB_ALPHA", bytes(pixels)))

    # GRAYSCALE 16-bit: 4x4
    pixels = bytearray()
    for r in range(4):
        for c in range(4):
            v = (r * 4 + c) * 65535 // 15
            pixels.extend(struct.pack(">H", v))
    write_file("valid/pam/grayscale_16bit_4x4.pam",
               pam_file(4, 4, 1, 65535, "GRAYSCALE", bytes(pixels)))

    # RGB 16-bit: 4x4
    pixels = bytearray()
    colors16 = [(65535, 0, 0), (0, 65535, 0), (0, 0, 65535), (65535, 65535, 65535)]
    for r in range(4):
        for c in range(4):
            for ch in colors16[c]:
                pixels.extend(struct.pack(">H", ch))
    write_file("valid/pam/rgb_16bit_4x4.pam",
               pam_file(4, 4, 3, 65535, "RGB", bytes(pixels)))


def gen_comments():
    print("PPM with comments (valid):")

    # Comment after magic number
    data = "P3\n# comment after magic\n2 2\n255\n255 0 0 0 255 0\n0 0 255 255 255 255\n"
    write_file("valid/ppm/comment_after_magic.ppm", data)

    # Comment between dimensions
    data = "P3\n2\n# comment between width and height\n2\n255\n255 0 0 0 255 0\n0 0 255 255 255 255\n"
    write_file("valid/ppm/comment_between_dims.ppm", data)

    # Multiple comment lines
    data = (
        "P3\n"
        "# First comment line\n"
        "# Second comment line\n"
        "# Third comment line with special chars: !@#$%^&*()\n"
        "2 2\n"
        "# Comment before maxval\n"
        "255\n"
        "# Comment before data\n"
        "255 0 0 0 255 0\n"
        "0 0 255 255 255 255\n"
    )
    write_file("valid/ppm/multiple_comments.ppm", data)


def gen_invalid():
    print("Invalid files:")

    # bad_magic.ppm — starts with P9
    write_file("invalid/bad_magic.ppm", "P9\n2 2\n255\n" + "0 " * 12 + "\n")

    # negative_width.pgm
    write_file("invalid/negative_width.pgm", "P5\n-1 2\n255\n")

    # zero_width.pgm
    write_file("invalid/zero_width.pgm", "P5\n0 2\n255\n")

    # zero_height.ppm
    write_file("invalid/zero_height.ppm", "P6\n2 0\n255\n")

    # missing_maxval.pgm — P5 with no maxval, just goes straight to data
    # Header says P5, gives dimensions, then raw bytes without maxval line
    write_file("invalid/missing_maxval.pgm", b"P5\n4 4\n" + bytes(16))

    # truncated_data.ppm — P6 4x4 but only half the pixel data
    header = "P6\n4 4\n255\n".encode("ascii")
    full_size = 4 * 4 * 3  # 48 bytes
    half_data = bytes(range(full_size // 2))  # only 24 bytes
    write_file("invalid/truncated_data.ppm", header + half_data)

    # maxval_zero.pgm
    write_file("invalid/maxval_zero.pgm", "P5\n2 2\n0\n" + "\x00" * 4)

    # maxval_too_large.pgm — maxval 70000 exceeds 65535
    write_file("invalid/maxval_too_large.pgm", "P5\n2 2\n70000\n" + "\x00" * 8)

    # bad_ascii_data.ppm — P3 with letters instead of numbers
    write_file("invalid/bad_ascii_data.ppm", "P3\n2 2\n255\nabc def ghi jkl mno pqr\nabc def ghi jkl mno pqr\n")

    # overflow_dimensions.pgm — dimensions that overflow u32 multiplication
    # 65536 * 65536 = 4294967296, overflows u32
    write_file("invalid/overflow_dimensions.pgm", "P5\n65536 65536\n255\n")


def gen_edge_cases():
    print("Edge cases:")

    # extra_whitespace.ppm — tabs, multiple spaces, mixed whitespace
    data = "P3  \t  \n  2  \t  2  \n  255  \n  255  \t  0  \t  0  \t  0  \t  255  \t  0  \n  0  \t  0  \t  255  \t  255  \t  255  \t  255  \n"
    write_file("edge-cases/extra_whitespace.ppm", data)

    # max_maxval.pgm — P5 maxval 65535
    pixels_16 = []
    for r in range(4):
        for c in range(4):
            pixels_16.append((r * 4 + c) * 65535 // 15)
    header = "P5\n4 4\n65535\n".encode("ascii")
    data = b"".join(struct.pack(">H", v) for v in pixels_16)
    write_file("edge-cases/max_maxval.pgm", header + data)

    # single_pixel_rgb.ppm — 1x1 P6
    header = "P6\n1 1\n255\n".encode("ascii")
    write_file("edge-cases/single_pixel_rgb.ppm", header + bytes([128, 64, 32]))

    # large_comment.pgm — 1000-char comment
    comment = "# " + "A" * 998 + "\n"
    pixels = bytearray()
    for r in range(4):
        for c in range(4):
            pixels.append((r * 4 + c) * 17)
    header = f"P5\n{comment}4 4\n255\n".encode("ascii")
    write_file("edge-cases/large_comment.pgm", header + bytes(pixels))

    # concatenated.ppm — two valid P6 images back to back
    img1_header = "P6\n2 2\n255\n".encode("ascii")
    img1_data = bytes([255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 0])
    img2_header = "P6\n2 2\n255\n".encode("ascii")
    img2_data = bytes([0, 255, 255, 255, 0, 255, 255, 255, 0, 128, 128, 128])
    write_file("edge-cases/concatenated.ppm", img1_header + img1_data + img2_header + img2_data)

    # crlf_lineendings.ppm — P3 with \r\n
    data = "P3\r\n2 2\r\n255\r\n255 0 0 0 255 0\r\n0 0 255 255 255 255\r\n"
    write_file("edge-cases/crlf_lineendings.ppm", data.encode("ascii"))


def main():
    print(f"Generating PNM/PAM conformance test files in {BASE}\n")
    ensure_dirs()
    gen_valid_pbm()
    gen_valid_pgm()
    gen_valid_ppm()
    gen_valid_pam()
    gen_comments()
    gen_invalid()
    gen_edge_cases()
    print(f"\nDone.")


if __name__ == "__main__":
    main()
