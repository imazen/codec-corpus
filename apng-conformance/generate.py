#!/usr/bin/env python3
"""
APNG conformance test suite generator.

Generates valid, invalid, and edge-case APNG files using only Python stdlib.
No external dependencies required.

APNG spec: https://wiki.mozilla.org/APNG_Specification
"""

import struct
import zlib
import os
from pathlib import Path

# --- PNG / APNG constants ---

PNG_SIGNATURE = b"\x89PNG\r\n\x1a\n"

# Color types
CT_GRAY = 0
CT_RGB = 2
CT_PALETTE = 3
CT_GRAY_ALPHA = 4
CT_RGBA = 6

# Dispose operations
DISPOSE_NONE = 0
DISPOSE_BACKGROUND = 1
DISPOSE_PREVIOUS = 2

# Blend operations
BLEND_SOURCE = 0
BLEND_OVER = 1

# Bytes per pixel for each color type (at 8-bit depth)
BPP = {CT_GRAY: 1, CT_RGB: 3, CT_PALETTE: 1, CT_GRAY_ALPHA: 2, CT_RGBA: 4}


# --- Low-level PNG chunk helpers ---

def make_chunk(chunk_type: bytes, data: bytes) -> bytes:
    """Create a PNG chunk: length(4) + type(4) + data + crc32(4)."""
    assert len(chunk_type) == 4
    crc = zlib.crc32(chunk_type + data) & 0xFFFFFFFF
    return struct.pack(">I", len(data)) + chunk_type + data + struct.pack(">I", crc)


def make_ihdr(width: int, height: int, bit_depth: int = 8,
              color_type: int = CT_RGBA) -> bytes:
    data = struct.pack(">IIBBBBB", width, height, bit_depth, color_type,
                       0,  # compression
                       0,  # filter
                       0)  # interlace
    return make_chunk(b"IHDR", data)


def make_plte(palette: list[tuple[int, int, int]]) -> bytes:
    """Create a PLTE chunk from a list of (R, G, B) tuples."""
    data = b""
    for r, g, b in palette:
        data += struct.pack("BBB", r, g, b)
    return make_chunk(b"PLTE", data)


def make_trns_palette(alphas: list[int]) -> bytes:
    """Create a tRNS chunk for palette images."""
    return make_chunk(b"tRNS", bytes(alphas))


def make_actl(num_frames: int, num_plays: int = 0) -> bytes:
    """Create an acTL (animation control) chunk."""
    data = struct.pack(">II", num_frames, num_plays)
    return make_chunk(b"acTL", data)


def make_fctl(sequence_number: int, width: int, height: int,
              x_offset: int = 0, y_offset: int = 0,
              delay_num: int = 500, delay_den: int = 1000,
              dispose_op: int = DISPOSE_NONE,
              blend_op: int = BLEND_SOURCE) -> bytes:
    """Create an fcTL (frame control) chunk."""
    data = struct.pack(">IIIIIHHBB",
                       sequence_number,
                       width, height,
                       x_offset, y_offset,
                       delay_num, delay_den,
                       dispose_op, blend_op)
    return make_chunk(b"fcTL", data)


def make_idat(raw_scanlines: bytes) -> bytes:
    """Compress raw scanlines and wrap in IDAT chunk."""
    compressed = zlib.compress(raw_scanlines)
    return make_chunk(b"IDAT", compressed)


def make_fdat(sequence_number: int, raw_scanlines: bytes) -> bytes:
    """Compress raw scanlines and wrap in fdAT chunk."""
    compressed = zlib.compress(raw_scanlines)
    data = struct.pack(">I", sequence_number) + compressed
    return make_chunk(b"fdAT", data)


def make_iend() -> bytes:
    return make_chunk(b"IEND", b"")


# --- Scanline helpers ---

def solid_scanlines(width: int, height: int, pixel: bytes,
                    bpp: int | None = None) -> bytes:
    """Generate raw scanlines (filter byte 0 + pixel data) for a solid color."""
    if bpp is None:
        bpp = len(pixel)
    row = b"\x00" + pixel * width  # filter=None + pixel data
    return row * height


def gradient_scanlines(width: int, height: int, value: int,
                       color_type: int = CT_RGBA) -> bytes:
    """Generate scanlines for a solid gray value (0-255)."""
    if color_type == CT_RGBA:
        pixel = bytes([value, value, value, 255])
    elif color_type == CT_RGB:
        pixel = bytes([value, value, value])
    elif color_type == CT_GRAY:
        pixel = bytes([value])
    elif color_type == CT_GRAY_ALPHA:
        pixel = bytes([value, 255])
    else:
        pixel = bytes([value])
    return solid_scanlines(width, height, pixel)


def palette_scanlines(width: int, height: int, index: int) -> bytes:
    """Generate scanlines for a palette-based image with a single color index."""
    row = b"\x00" + bytes([index]) * width
    return row * height


# --- File writing helper ---

def write_apng(path: str, chunks: list[bytes]):
    """Write PNG signature + chunks to file."""
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "wb") as f:
        f.write(PNG_SIGNATURE)
        for chunk in chunks:
            f.write(chunk)


# --- Sequence number tracker ---

class SeqNum:
    """Track APNG sequence numbers (shared counter for fcTL and fdAT)."""
    def __init__(self, start: int = 0):
        self.val = start

    def next(self) -> int:
        v = self.val
        self.val += 1
        return v


# ============================================================
# VALID files
# ============================================================

def gen_2frame_simple(base: str):
    """2 frames, 8x8, alternating red/blue, 500ms delay."""
    w, h = 8, 8
    seq = SeqNum()

    red = solid_scanlines(w, h, b"\xff\x00\x00\xff")
    blue = solid_scanlines(w, h, b"\x00\x00\xff\xff")

    chunks = [
        make_ihdr(w, h, 8, CT_RGBA),
        make_actl(2, 0),
        # Frame 0 (default image, uses IDAT)
        make_fctl(seq.next(), w, h, 0, 0, 500, 1000, DISPOSE_NONE, BLEND_SOURCE),
        make_idat(red),
        # Frame 1
        make_fctl(seq.next(), w, h, 0, 0, 500, 1000, DISPOSE_NONE, BLEND_SOURCE),
        make_fdat(seq.next(), blue),
        make_iend(),
    ]
    write_apng(f"{base}/valid/2frame_simple.png", chunks)


def gen_3frame_rgb(base: str):
    """3 frames cycling R, G, B solid colors, 4x4."""
    w, h = 4, 4
    seq = SeqNum()

    red = solid_scanlines(w, h, b"\xff\x00\x00\xff")
    green = solid_scanlines(w, h, b"\x00\xff\x00\xff")
    blue = solid_scanlines(w, h, b"\x00\x00\xff\xff")

    chunks = [
        make_ihdr(w, h, 8, CT_RGBA),
        make_actl(3, 0),
        make_fctl(seq.next(), w, h, 0, 0, 333, 1000, DISPOSE_NONE, BLEND_SOURCE),
        make_idat(red),
        make_fctl(seq.next(), w, h, 0, 0, 333, 1000, DISPOSE_NONE, BLEND_SOURCE),
        make_fdat(seq.next(), green),
        make_fctl(seq.next(), w, h, 0, 0, 333, 1000, DISPOSE_NONE, BLEND_SOURCE),
        make_fdat(seq.next(), blue),
        make_iend(),
    ]
    write_apng(f"{base}/valid/3frame_rgb.png", chunks)


def gen_10frame_gradient(base: str):
    """10 frames fading black to white, 8x8."""
    w, h = 8, 8
    seq = SeqNum()
    n_frames = 10

    chunks = [
        make_ihdr(w, h, 8, CT_RGBA),
        make_actl(n_frames, 0),
    ]

    for i in range(n_frames):
        value = int(255 * i / (n_frames - 1))
        scanlines = gradient_scanlines(w, h, value, CT_RGBA)
        if i == 0:
            chunks.append(make_fctl(seq.next(), w, h, 0, 0, 100, 1000,
                                    DISPOSE_NONE, BLEND_SOURCE))
            chunks.append(make_idat(scanlines))
        else:
            chunks.append(make_fctl(seq.next(), w, h, 0, 0, 100, 1000,
                                    DISPOSE_NONE, BLEND_SOURCE))
            chunks.append(make_fdat(seq.next(), scanlines))

    chunks.append(make_iend())
    write_apng(f"{base}/valid/10frame_gradient.png", chunks)


def gen_dispose_none(base: str):
    """3 frames, dispose_op=APNG_DISPOSE_OP_NONE (0).
    Each frame overwrites full canvas. Previous frame buffer persists."""
    w, h = 8, 8
    seq = SeqNum()

    colors = [b"\xff\x00\x00\xff", b"\x00\xff\x00\xff", b"\x00\x00\xff\xff"]
    chunks = [
        make_ihdr(w, h, 8, CT_RGBA),
        make_actl(3, 0),
    ]

    for i, color in enumerate(colors):
        scanlines = solid_scanlines(w, h, color)
        if i == 0:
            chunks.append(make_fctl(seq.next(), w, h, 0, 0, 500, 1000,
                                    DISPOSE_NONE, BLEND_SOURCE))
            chunks.append(make_idat(scanlines))
        else:
            chunks.append(make_fctl(seq.next(), w, h, 0, 0, 500, 1000,
                                    DISPOSE_NONE, BLEND_SOURCE))
            chunks.append(make_fdat(seq.next(), scanlines))

    chunks.append(make_iend())
    write_apng(f"{base}/valid/dispose_none.png", chunks)


def gen_dispose_background(base: str):
    """3 frames, dispose_op=APNG_DISPOSE_OP_BACKGROUND (1).
    After displaying each frame, the frame region is cleared to transparent black."""
    w, h = 8, 8
    seq = SeqNum()

    # Use sub-region frames to make disposal visible
    colors = [b"\xff\x00\x00\xff", b"\x00\xff\x00\xff", b"\x00\x00\xff\xff"]
    chunks = [
        make_ihdr(w, h, 8, CT_RGBA),
        make_actl(3, 0),
    ]

    for i, color in enumerate(colors):
        # Each frame is a 4x4 region at different positions
        fw, fh = 4, 4
        xoff = (i % 2) * 4
        yoff = (i // 2) * 4
        scanlines = solid_scanlines(fw, fh, color)
        if i == 0:
            # First frame must cover full canvas for IDAT, so use full canvas
            full_scanlines = solid_scanlines(w, h, color)
            chunks.append(make_fctl(seq.next(), w, h, 0, 0, 500, 1000,
                                    DISPOSE_BACKGROUND, BLEND_SOURCE))
            chunks.append(make_idat(full_scanlines))
        else:
            chunks.append(make_fctl(seq.next(), fw, fh, xoff, yoff, 500, 1000,
                                    DISPOSE_BACKGROUND, BLEND_SOURCE))
            chunks.append(make_fdat(seq.next(), scanlines))

    chunks.append(make_iend())
    write_apng(f"{base}/valid/dispose_background.png", chunks)


def gen_dispose_previous(base: str):
    """3 frames, dispose_op=APNG_DISPOSE_OP_PREVIOUS (2).
    After displaying, restore the frame region to what it was before the frame was drawn."""
    w, h = 8, 8
    seq = SeqNum()

    colors = [b"\xff\x00\x00\xff", b"\x00\xff\x00\xff", b"\x00\x00\xff\xff"]
    chunks = [
        make_ihdr(w, h, 8, CT_RGBA),
        make_actl(3, 0),
    ]

    for i, color in enumerate(colors):
        scanlines = solid_scanlines(w, h, color)
        if i == 0:
            # First frame: DISPOSE_NONE (DISPOSE_PREVIOUS on first frame
            # is treated as DISPOSE_NONE per spec)
            chunks.append(make_fctl(seq.next(), w, h, 0, 0, 500, 1000,
                                    DISPOSE_PREVIOUS, BLEND_SOURCE))
            chunks.append(make_idat(scanlines))
        else:
            chunks.append(make_fctl(seq.next(), w, h, 0, 0, 500, 1000,
                                    DISPOSE_PREVIOUS, BLEND_SOURCE))
            chunks.append(make_fdat(seq.next(), scanlines))

    chunks.append(make_iend())
    write_apng(f"{base}/valid/dispose_previous.png", chunks)


def gen_blend_source(base: str):
    """3 frames, blend_op=APNG_BLEND_OP_SOURCE (0).
    Frame pixels fully replace the canvas region."""
    w, h = 8, 8
    seq = SeqNum()

    # Use semi-transparent frames to show SOURCE behavior
    colors = [
        b"\xff\x00\x00\x80",  # semi-transparent red
        b"\x00\xff\x00\x80",  # semi-transparent green
        b"\x00\x00\xff\x80",  # semi-transparent blue
    ]
    chunks = [
        make_ihdr(w, h, 8, CT_RGBA),
        make_actl(3, 0),
    ]

    for i, color in enumerate(colors):
        scanlines = solid_scanlines(w, h, color)
        if i == 0:
            chunks.append(make_fctl(seq.next(), w, h, 0, 0, 500, 1000,
                                    DISPOSE_NONE, BLEND_SOURCE))
            chunks.append(make_idat(scanlines))
        else:
            chunks.append(make_fctl(seq.next(), w, h, 0, 0, 500, 1000,
                                    DISPOSE_NONE, BLEND_SOURCE))
            chunks.append(make_fdat(seq.next(), scanlines))

    chunks.append(make_iend())
    write_apng(f"{base}/valid/blend_source.png", chunks)


def gen_blend_over(base: str):
    """3 frames with alpha, blend_op=APNG_BLEND_OP_OVER (1).
    Frame is alpha-composited onto the canvas."""
    w, h = 8, 8
    seq = SeqNum()

    # Semi-transparent overlapping frames
    colors = [
        b"\xff\x00\x00\xff",  # opaque red background
        b"\x00\xff\x00\x80",  # semi-transparent green overlay
        b"\x00\x00\xff\x80",  # semi-transparent blue overlay
    ]
    chunks = [
        make_ihdr(w, h, 8, CT_RGBA),
        make_actl(3, 0),
    ]

    for i, color in enumerate(colors):
        scanlines = solid_scanlines(w, h, color)
        if i == 0:
            chunks.append(make_fctl(seq.next(), w, h, 0, 0, 500, 1000,
                                    DISPOSE_NONE, BLEND_SOURCE))
            chunks.append(make_idat(scanlines))
        else:
            chunks.append(make_fctl(seq.next(), w, h, 0, 0, 500, 1000,
                                    DISPOSE_NONE, BLEND_OVER))
            chunks.append(make_fdat(seq.next(), scanlines))

    chunks.append(make_iend())
    write_apng(f"{base}/valid/blend_over.png", chunks)


def gen_offset_frames(base: str):
    """4x4 canvas, frames at different x_offset/y_offset positions.
    Demonstrates sub-frame rendering."""
    w, h = 8, 8
    seq = SeqNum()

    # Full canvas first frame, then 2x2 patches at different corners
    full_gray = solid_scanlines(w, h, b"\x80\x80\x80\xff")
    patch_red = solid_scanlines(2, 2, b"\xff\x00\x00\xff")
    patch_green = solid_scanlines(2, 2, b"\x00\xff\x00\xff")
    patch_blue = solid_scanlines(2, 2, b"\x00\x00\xff\xff")
    patch_yellow = solid_scanlines(2, 2, b"\xff\xff\x00\xff")

    chunks = [
        make_ihdr(w, h, 8, CT_RGBA),
        make_actl(5, 0),
        # Frame 0: full gray canvas
        make_fctl(seq.next(), w, h, 0, 0, 500, 1000, DISPOSE_NONE, BLEND_SOURCE),
        make_idat(full_gray),
        # Frame 1: red patch top-left
        make_fctl(seq.next(), 2, 2, 0, 0, 500, 1000, DISPOSE_NONE, BLEND_SOURCE),
        make_fdat(seq.next(), patch_red),
        # Frame 2: green patch top-right
        make_fctl(seq.next(), 2, 2, 6, 0, 500, 1000, DISPOSE_NONE, BLEND_SOURCE),
        make_fdat(seq.next(), patch_green),
        # Frame 3: blue patch bottom-left
        make_fctl(seq.next(), 2, 2, 0, 6, 500, 1000, DISPOSE_NONE, BLEND_SOURCE),
        make_fdat(seq.next(), patch_blue),
        # Frame 4: yellow patch bottom-right
        make_fctl(seq.next(), 2, 2, 6, 6, 500, 1000, DISPOSE_NONE, BLEND_SOURCE),
        make_fdat(seq.next(), patch_yellow),
        make_iend(),
    ]
    write_apng(f"{base}/valid/offset_frames.png", chunks)


def gen_loop(base: str, name: str, num_plays: int):
    """Generate loop test: num_plays=0 infinite, 1 once, 3 three times."""
    w, h = 4, 4
    seq = SeqNum()

    red = solid_scanlines(w, h, b"\xff\x00\x00\xff")
    blue = solid_scanlines(w, h, b"\x00\x00\xff\xff")

    chunks = [
        make_ihdr(w, h, 8, CT_RGBA),
        make_actl(2, num_plays),
        make_fctl(seq.next(), w, h, 0, 0, 500, 1000, DISPOSE_NONE, BLEND_SOURCE),
        make_idat(red),
        make_fctl(seq.next(), w, h, 0, 0, 500, 1000, DISPOSE_NONE, BLEND_SOURCE),
        make_fdat(seq.next(), blue),
        make_iend(),
    ]
    write_apng(f"{base}/valid/{name}.png", chunks)


def gen_fast_animation(base: str):
    """3 frames, delay=10ms per frame."""
    w, h = 4, 4
    seq = SeqNum()

    colors = [b"\xff\x00\x00\xff", b"\x00\xff\x00\xff", b"\x00\x00\xff\xff"]
    chunks = [
        make_ihdr(w, h, 8, CT_RGBA),
        make_actl(3, 0),
    ]

    for i, color in enumerate(colors):
        scanlines = solid_scanlines(w, h, color)
        if i == 0:
            chunks.append(make_fctl(seq.next(), w, h, 0, 0, 10, 1000,
                                    DISPOSE_NONE, BLEND_SOURCE))
            chunks.append(make_idat(scanlines))
        else:
            chunks.append(make_fctl(seq.next(), w, h, 0, 0, 10, 1000,
                                    DISPOSE_NONE, BLEND_SOURCE))
            chunks.append(make_fdat(seq.next(), scanlines))

    chunks.append(make_iend())
    write_apng(f"{base}/valid/fast_animation.png", chunks)


def gen_slow_animation(base: str):
    """3 frames, delay=2000ms per frame."""
    w, h = 4, 4
    seq = SeqNum()

    colors = [b"\xff\x00\x00\xff", b"\x00\xff\x00\xff", b"\x00\x00\xff\xff"]
    chunks = [
        make_ihdr(w, h, 8, CT_RGBA),
        make_actl(3, 0),
    ]

    for i, color in enumerate(colors):
        scanlines = solid_scanlines(w, h, color)
        if i == 0:
            chunks.append(make_fctl(seq.next(), w, h, 0, 0, 2000, 1000,
                                    DISPOSE_NONE, BLEND_SOURCE))
            chunks.append(make_idat(scanlines))
        else:
            chunks.append(make_fctl(seq.next(), w, h, 0, 0, 2000, 1000,
                                    DISPOSE_NONE, BLEND_SOURCE))
            chunks.append(make_fdat(seq.next(), scanlines))

    chunks.append(make_iend())
    write_apng(f"{base}/valid/slow_animation.png", chunks)


def gen_variable_delay(base: str):
    """3 frames with different delays: 100ms, 500ms, 1000ms."""
    w, h = 4, 4
    seq = SeqNum()

    colors = [b"\xff\x00\x00\xff", b"\x00\xff\x00\xff", b"\x00\x00\xff\xff"]
    delays = [(100, 1000), (500, 1000), (1000, 1000)]
    chunks = [
        make_ihdr(w, h, 8, CT_RGBA),
        make_actl(3, 0),
    ]

    for i, (color, (dnum, dden)) in enumerate(zip(colors, delays)):
        scanlines = solid_scanlines(w, h, color)
        if i == 0:
            chunks.append(make_fctl(seq.next(), w, h, 0, 0, dnum, dden,
                                    DISPOSE_NONE, BLEND_SOURCE))
            chunks.append(make_idat(scanlines))
        else:
            chunks.append(make_fctl(seq.next(), w, h, 0, 0, dnum, dden,
                                    DISPOSE_NONE, BLEND_SOURCE))
            chunks.append(make_fdat(seq.next(), scanlines))

    chunks.append(make_iend())
    write_apng(f"{base}/valid/variable_delay.png", chunks)


def gen_rgba_8bit(base: str):
    """RGBA color type 6, 8-bit, 2 frames."""
    w, h = 4, 4
    seq = SeqNum()

    frame0 = solid_scanlines(w, h, b"\xff\x00\x00\x80")  # semi-transparent red
    frame1 = solid_scanlines(w, h, b"\x00\x00\xff\xc0")  # semi-transparent blue

    chunks = [
        make_ihdr(w, h, 8, CT_RGBA),
        make_actl(2, 0),
        make_fctl(seq.next(), w, h, 0, 0, 500, 1000, DISPOSE_NONE, BLEND_SOURCE),
        make_idat(frame0),
        make_fctl(seq.next(), w, h, 0, 0, 500, 1000, DISPOSE_NONE, BLEND_SOURCE),
        make_fdat(seq.next(), frame1),
        make_iend(),
    ]
    write_apng(f"{base}/valid/rgba_8bit.png", chunks)


def gen_rgb_8bit(base: str):
    """RGB color type 2, 8-bit, 2 frames."""
    w, h = 4, 4
    seq = SeqNum()

    frame0 = solid_scanlines(w, h, b"\xff\x00\x00", bpp=3)
    frame1 = solid_scanlines(w, h, b"\x00\x00\xff", bpp=3)

    chunks = [
        make_ihdr(w, h, 8, CT_RGB),
        make_actl(2, 0),
        make_fctl(seq.next(), w, h, 0, 0, 500, 1000, DISPOSE_NONE, BLEND_SOURCE),
        make_idat(frame0),
        make_fctl(seq.next(), w, h, 0, 0, 500, 1000, DISPOSE_NONE, BLEND_SOURCE),
        make_fdat(seq.next(), frame1),
        make_iend(),
    ]
    write_apng(f"{base}/valid/rgb_8bit.png", chunks)


def gen_gray_8bit(base: str):
    """Grayscale color type 0, 8-bit, 2 frames."""
    w, h = 4, 4
    seq = SeqNum()

    frame0 = solid_scanlines(w, h, b"\x40", bpp=1)
    frame1 = solid_scanlines(w, h, b"\xc0", bpp=1)

    chunks = [
        make_ihdr(w, h, 8, CT_GRAY),
        make_actl(2, 0),
        make_fctl(seq.next(), w, h, 0, 0, 500, 1000, DISPOSE_NONE, BLEND_SOURCE),
        make_idat(frame0),
        make_fctl(seq.next(), w, h, 0, 0, 500, 1000, DISPOSE_NONE, BLEND_SOURCE),
        make_fdat(seq.next(), frame1),
        make_iend(),
    ]
    write_apng(f"{base}/valid/gray_8bit.png", chunks)


def gen_palette_8bit(base: str):
    """Palette color type 3, 8-bit, 3 frames cycling palette colors."""
    w, h = 4, 4
    seq = SeqNum()

    palette = [(255, 0, 0), (0, 255, 0), (0, 0, 255), (255, 255, 0)]

    chunks = [
        make_ihdr(w, h, 8, CT_PALETTE),
        make_plte(palette),
        make_actl(3, 0),
    ]

    for i in range(3):
        scanlines = palette_scanlines(w, h, i)
        if i == 0:
            chunks.append(make_fctl(seq.next(), w, h, 0, 0, 500, 1000,
                                    DISPOSE_NONE, BLEND_SOURCE))
            chunks.append(make_idat(scanlines))
        else:
            chunks.append(make_fctl(seq.next(), w, h, 0, 0, 500, 1000,
                                    DISPOSE_NONE, BLEND_SOURCE))
            chunks.append(make_fdat(seq.next(), scanlines))

    chunks.append(make_iend())
    write_apng(f"{base}/valid/palette_8bit.png", chunks)


def gen_default_is_first(base: str):
    """First IDAT frame IS part of animation.
    fcTL for first frame appears BEFORE IDAT."""
    w, h = 4, 4
    seq = SeqNum()

    red = solid_scanlines(w, h, b"\xff\x00\x00\xff")
    blue = solid_scanlines(w, h, b"\x00\x00\xff\xff")

    chunks = [
        make_ihdr(w, h, 8, CT_RGBA),
        make_actl(2, 0),
        # fcTL BEFORE IDAT = first frame is part of animation
        make_fctl(seq.next(), w, h, 0, 0, 500, 1000, DISPOSE_NONE, BLEND_SOURCE),
        make_idat(red),
        make_fctl(seq.next(), w, h, 0, 0, 500, 1000, DISPOSE_NONE, BLEND_SOURCE),
        make_fdat(seq.next(), blue),
        make_iend(),
    ]
    write_apng(f"{base}/valid/default_is_first.png", chunks)


def gen_default_separate(base: str):
    """IDAT is a static fallback, animation starts with fdAT only.
    No fcTL before IDAT = default image is NOT part of animation."""
    w, h = 4, 4
    seq = SeqNum()

    # Static fallback image (gray)
    gray = solid_scanlines(w, h, b"\x80\x80\x80\xff")
    red = solid_scanlines(w, h, b"\xff\x00\x00\xff")
    blue = solid_scanlines(w, h, b"\x00\x00\xff\xff")

    chunks = [
        make_ihdr(w, h, 8, CT_RGBA),
        # acTL says 2 frames in the animation
        make_actl(2, 0),
        # NO fcTL before IDAT = IDAT is NOT part of animation
        make_idat(gray),
        # Animation frames start here
        make_fctl(seq.next(), w, h, 0, 0, 500, 1000, DISPOSE_NONE, BLEND_SOURCE),
        make_fdat(seq.next(), red),
        make_fctl(seq.next(), w, h, 0, 0, 500, 1000, DISPOSE_NONE, BLEND_SOURCE),
        make_fdat(seq.next(), blue),
        make_iend(),
    ]
    write_apng(f"{base}/valid/default_separate.png", chunks)


def gen_single_frame(base: str):
    """Single frame APNG (valid, acTL says 1 frame)."""
    w, h = 4, 4
    seq = SeqNum()

    red = solid_scanlines(w, h, b"\xff\x00\x00\xff")

    chunks = [
        make_ihdr(w, h, 8, CT_RGBA),
        make_actl(1, 0),
        make_fctl(seq.next(), w, h, 0, 0, 0, 1000, DISPOSE_NONE, BLEND_SOURCE),
        make_idat(red),
        make_iend(),
    ]
    write_apng(f"{base}/valid/single_frame.png", chunks)


# ============================================================
# INVALID files
# ============================================================

def gen_missing_actl(base: str):
    """Has fcTL/fdAT but no acTL chunk. Should be treated as static PNG."""
    w, h = 4, 4
    seq = SeqNum()

    red = solid_scanlines(w, h, b"\xff\x00\x00\xff")
    blue = solid_scanlines(w, h, b"\x00\x00\xff\xff")

    chunks = [
        make_ihdr(w, h, 8, CT_RGBA),
        # NO acTL!
        make_fctl(seq.next(), w, h, 0, 0, 500, 1000, DISPOSE_NONE, BLEND_SOURCE),
        make_idat(red),
        make_fctl(seq.next(), w, h, 0, 0, 500, 1000, DISPOSE_NONE, BLEND_SOURCE),
        make_fdat(seq.next(), blue),
        make_iend(),
    ]
    write_apng(f"{base}/invalid/missing_actl.png", chunks)


def gen_bad_sequence(base: str):
    """Sequence numbers out of order (duplicate sequence number)."""
    w, h = 4, 4

    red = solid_scanlines(w, h, b"\xff\x00\x00\xff")
    blue = solid_scanlines(w, h, b"\x00\x00\xff\xff")

    chunks = [
        make_ihdr(w, h, 8, CT_RGBA),
        make_actl(2, 0),
        make_fctl(0, w, h, 0, 0, 500, 1000, DISPOSE_NONE, BLEND_SOURCE),
        make_idat(red),
        # Duplicate sequence number 0 instead of 1
        make_fctl(0, w, h, 0, 0, 500, 1000, DISPOSE_NONE, BLEND_SOURCE),
        make_fdat(0, blue),  # Also duplicate
        make_iend(),
    ]
    write_apng(f"{base}/invalid/bad_sequence.png", chunks)


def gen_frame_out_of_bounds(base: str):
    """fcTL with x_offset+width > canvas width."""
    w, h = 4, 4
    seq = SeqNum()

    red = solid_scanlines(w, h, b"\xff\x00\x00\xff")
    # Patch that extends beyond canvas
    patch = solid_scanlines(3, 3, b"\x00\xff\x00\xff")

    chunks = [
        make_ihdr(w, h, 8, CT_RGBA),
        make_actl(2, 0),
        make_fctl(seq.next(), w, h, 0, 0, 500, 1000, DISPOSE_NONE, BLEND_SOURCE),
        make_idat(red),
        # x_offset=3, width=3 => 3+3=6 > 4 (canvas width)
        make_fctl(seq.next(), 3, 3, 3, 3, 500, 1000, DISPOSE_NONE, BLEND_SOURCE),
        make_fdat(seq.next(), patch),
        make_iend(),
    ]
    write_apng(f"{base}/invalid/frame_out_of_bounds.png", chunks)


def gen_zero_delay_den(base: str):
    """delay_den = 0. Per spec, should be treated as 100, but some decoders may error."""
    w, h = 4, 4
    seq = SeqNum()

    red = solid_scanlines(w, h, b"\xff\x00\x00\xff")
    blue = solid_scanlines(w, h, b"\x00\x00\xff\xff")

    chunks = [
        make_ihdr(w, h, 8, CT_RGBA),
        make_actl(2, 0),
        make_fctl(seq.next(), w, h, 0, 0, 50, 0, DISPOSE_NONE, BLEND_SOURCE),
        make_idat(red),
        make_fctl(seq.next(), w, h, 0, 0, 50, 0, DISPOSE_NONE, BLEND_SOURCE),
        make_fdat(seq.next(), blue),
        make_iend(),
    ]
    write_apng(f"{base}/invalid/zero_delay_den.png", chunks)


def gen_truncated_fdat(base: str):
    """fdAT chunk with less compressed data than needed (truncated)."""
    w, h = 4, 4
    seq = SeqNum()

    red = solid_scanlines(w, h, b"\xff\x00\x00\xff")
    blue_scanlines = solid_scanlines(w, h, b"\x00\x00\xff\xff")

    # Create a truncated fdAT manually
    compressed = zlib.compress(blue_scanlines)
    # Truncate the compressed data to half
    truncated = compressed[:len(compressed) // 2]
    fdat_data = struct.pack(">I", seq.next()) + truncated  # placeholder, will fix below

    chunks = [
        make_ihdr(w, h, 8, CT_RGBA),
        make_actl(2, 0),
        make_fctl(0, w, h, 0, 0, 500, 1000, DISPOSE_NONE, BLEND_SOURCE),
        make_idat(red),
        make_fctl(1, w, h, 0, 0, 500, 1000, DISPOSE_NONE, BLEND_SOURCE),
    ]

    # Manually construct the truncated fdAT
    fdat_seqnum = 2
    fdat_payload = struct.pack(">I", fdat_seqnum) + truncated
    chunks.append(make_chunk(b"fdAT", fdat_payload))
    chunks.append(make_iend())
    write_apng(f"{base}/invalid/truncated_fdat.png", chunks)


def gen_no_fdat(base: str):
    """acTL says 3 frames but only 1 frame of data (the IDAT)."""
    w, h = 4, 4
    seq = SeqNum()

    red = solid_scanlines(w, h, b"\xff\x00\x00\xff")

    chunks = [
        make_ihdr(w, h, 8, CT_RGBA),
        make_actl(3, 0),  # Claims 3 frames
        make_fctl(seq.next(), w, h, 0, 0, 500, 1000, DISPOSE_NONE, BLEND_SOURCE),
        make_idat(red),
        # Missing frames 2 and 3!
        make_iend(),
    ]
    write_apng(f"{base}/invalid/no_fdat.png", chunks)


# ============================================================
# EDGE CASE files
# ============================================================

def gen_zero_delay(base: str):
    """delay_num=0 (render as fast as possible)."""
    w, h = 4, 4
    seq = SeqNum()

    colors = [b"\xff\x00\x00\xff", b"\x00\xff\x00\xff", b"\x00\x00\xff\xff"]
    chunks = [
        make_ihdr(w, h, 8, CT_RGBA),
        make_actl(3, 0),
    ]

    for i, color in enumerate(colors):
        scanlines = solid_scanlines(w, h, color)
        if i == 0:
            chunks.append(make_fctl(seq.next(), w, h, 0, 0, 0, 1000,
                                    DISPOSE_NONE, BLEND_SOURCE))
            chunks.append(make_idat(scanlines))
        else:
            chunks.append(make_fctl(seq.next(), w, h, 0, 0, 0, 1000,
                                    DISPOSE_NONE, BLEND_SOURCE))
            chunks.append(make_fdat(seq.next(), scanlines))

    chunks.append(make_iend())
    write_apng(f"{base}/edge/zero_delay.png", chunks)


def gen_many_frames(base: str):
    """50 frames cycling through hues, stress test frame count."""
    w, h = 4, 4
    seq = SeqNum()
    n_frames = 50

    chunks = [
        make_ihdr(w, h, 8, CT_RGBA),
        make_actl(n_frames, 0),
    ]

    for i in range(n_frames):
        # Cycle through colors
        r = int(127.5 + 127.5 * __import__("math").sin(2 * 3.14159 * i / n_frames))
        g = int(127.5 + 127.5 * __import__("math").sin(2 * 3.14159 * i / n_frames + 2.094))
        b = int(127.5 + 127.5 * __import__("math").sin(2 * 3.14159 * i / n_frames + 4.189))
        r, g, b = max(0, min(255, r)), max(0, min(255, g)), max(0, min(255, b))
        pixel = bytes([r, g, b, 255])
        scanlines = solid_scanlines(w, h, pixel)

        if i == 0:
            chunks.append(make_fctl(seq.next(), w, h, 0, 0, 50, 1000,
                                    DISPOSE_NONE, BLEND_SOURCE))
            chunks.append(make_idat(scanlines))
        else:
            chunks.append(make_fctl(seq.next(), w, h, 0, 0, 50, 1000,
                                    DISPOSE_NONE, BLEND_SOURCE))
            chunks.append(make_fdat(seq.next(), scanlines))

    chunks.append(make_iend())
    write_apng(f"{base}/edge/many_frames.png", chunks)


def gen_1x1_animated(base: str):
    """1x1 pixel animation with 3 frames."""
    w, h = 1, 1
    seq = SeqNum()

    colors = [b"\xff\x00\x00\xff", b"\x00\xff\x00\xff", b"\x00\x00\xff\xff"]
    chunks = [
        make_ihdr(w, h, 8, CT_RGBA),
        make_actl(3, 0),
    ]

    for i, color in enumerate(colors):
        scanlines = solid_scanlines(w, h, color)
        if i == 0:
            chunks.append(make_fctl(seq.next(), w, h, 0, 0, 500, 1000,
                                    DISPOSE_NONE, BLEND_SOURCE))
            chunks.append(make_idat(scanlines))
        else:
            chunks.append(make_fctl(seq.next(), w, h, 0, 0, 500, 1000,
                                    DISPOSE_NONE, BLEND_SOURCE))
            chunks.append(make_fdat(seq.next(), scanlines))

    chunks.append(make_iend())
    write_apng(f"{base}/edge/1x1_animated.png", chunks)


# ============================================================
# Main
# ============================================================

def main():
    base = os.path.dirname(os.path.abspath(__file__))

    print("Generating APNG conformance test files...")
    print()

    # --- Valid files ---
    print("=== Valid files ===")

    gen_2frame_simple(base)
    print("  valid/2frame_simple.png")

    gen_3frame_rgb(base)
    print("  valid/3frame_rgb.png")

    gen_10frame_gradient(base)
    print("  valid/10frame_gradient.png")

    gen_dispose_none(base)
    print("  valid/dispose_none.png")

    gen_dispose_background(base)
    print("  valid/dispose_background.png")

    gen_dispose_previous(base)
    print("  valid/dispose_previous.png")

    gen_blend_source(base)
    print("  valid/blend_source.png")

    gen_blend_over(base)
    print("  valid/blend_over.png")

    gen_offset_frames(base)
    print("  valid/offset_frames.png")

    gen_loop(base, "loop_infinite", 0)
    print("  valid/loop_infinite.png")

    gen_loop(base, "loop_once", 1)
    print("  valid/loop_once.png")

    gen_loop(base, "loop_3times", 3)
    print("  valid/loop_3times.png")

    gen_fast_animation(base)
    print("  valid/fast_animation.png")

    gen_slow_animation(base)
    print("  valid/slow_animation.png")

    gen_variable_delay(base)
    print("  valid/variable_delay.png")

    gen_rgba_8bit(base)
    print("  valid/rgba_8bit.png")

    gen_rgb_8bit(base)
    print("  valid/rgb_8bit.png")

    gen_gray_8bit(base)
    print("  valid/gray_8bit.png")

    gen_palette_8bit(base)
    print("  valid/palette_8bit.png")

    gen_default_is_first(base)
    print("  valid/default_is_first.png")

    gen_default_separate(base)
    print("  valid/default_separate.png")

    gen_single_frame(base)
    print("  valid/single_frame.png")

    # --- Invalid files ---
    print()
    print("=== Invalid files ===")

    gen_missing_actl(base)
    print("  invalid/missing_actl.png")

    gen_bad_sequence(base)
    print("  invalid/bad_sequence.png")

    gen_frame_out_of_bounds(base)
    print("  invalid/frame_out_of_bounds.png")

    gen_zero_delay_den(base)
    print("  invalid/zero_delay_den.png")

    gen_truncated_fdat(base)
    print("  invalid/truncated_fdat.png")

    gen_no_fdat(base)
    print("  invalid/no_fdat.png")

    # --- Edge cases ---
    print()
    print("=== Edge case files ===")

    gen_zero_delay(base)
    print("  edge/zero_delay.png")

    gen_many_frames(base)
    print("  edge/many_frames.png")

    gen_1x1_animated(base)
    print("  edge/1x1_animated.png")

    # --- Summary ---
    print()
    valid_count = len([f for f in os.listdir(f"{base}/valid") if f.endswith(".png")])
    invalid_count = len([f for f in os.listdir(f"{base}/invalid") if f.endswith(".png")])
    edge_count = len([f for f in os.listdir(f"{base}/edge") if f.endswith(".png")])
    total = valid_count + invalid_count + edge_count
    print(f"Generated {total} files: {valid_count} valid, {invalid_count} invalid, {edge_count} edge cases")


if __name__ == "__main__":
    main()
