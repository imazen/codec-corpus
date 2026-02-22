# BMP Conformance Test Suite - Sources

Attribution for all files in this test suite.

## License Summary

| Source | License | URL |
|--------|---------|-----|
| BMPSuite | Public domain | https://entropymine.com/jason/bmpsuite/ |
| Pillow | HPND (PIL Software License) | https://github.com/python-pillow/Pillow |
| zune-bmp | MIT/Apache-2.0/Zlib | https://github.com/etemesi254/zune-image |

The BMPSuite files (by Jason Summers) are public domain. The Pillow BMP test images
are redistributable under the PIL Software License (HPND). The zune-image test files
are from BMPSuite and are public domain.

---

## valid/

Valid BMP files that conformant decoders must handle correctly.

### BMPSuite reference images (public domain)

Files with `g` prefix are from the BMPSuite by Jason Summers. These are the
canonical conformance test images for BMP decoders.

| File | Description |
|------|-------------|
| g01bg.bmp | 1-bit, black on green background |
| g01bw.bmp | 1-bit, black and white |
| g01p1.bmp | 1-bit, 1-entry palette |
| g01wb.bmp | 1-bit, white and black |
| g04.bmp | 4-bit uncompressed |
| g04p4.bmp | 4-bit, 4-entry palette |
| g04rle.bmp | 4-bit, RLE4 compressed |
| g08.bmp | 8-bit uncompressed |
| g08offs.bmp | 8-bit, non-zero data offset |
| g08os2.bmp | 8-bit, OS/2 BITMAPCOREHEADER |
| g08p256.bmp | 8-bit, full 256-entry palette |
| g08p64.bmp | 8-bit, 64-entry palette |
| g08pi256.bmp | 8-bit, 256-entry palette, interleaved |
| g08pi64.bmp | 8-bit, 64-entry palette, interleaved |
| g08res11.bmp | 8-bit, 1:1 aspect ratio |
| g08res21.bmp | 8-bit, 2:1 aspect ratio |
| g08res22.bmp | 8-bit, 2:2 aspect ratio |
| g08rle.bmp | 8-bit, RLE8 compressed |
| g08s0.bmp | 8-bit, image size = 0 in header |
| g08w124.bmp | 8-bit, width 124 (row padding test) |
| g08w125.bmp | 8-bit, width 125 (row padding test) |
| g08w126.bmp | 8-bit, width 126 (row padding test) |
| g16bf555.bmp | 16-bit, 5-5-5 bitfield |
| g16bf565.bmp | 16-bit, 5-6-5 bitfield |
| g16def555.bmp | 16-bit, default 5-5-5 |
| g24.bmp | 24-bit RGB |
| g32bf.bmp | 32-bit with bitfield masks |
| g32def.bmp | 32-bit default (no bitfield) |

### zune-image / BMPSuite (MIT/Apache-2.0/Zlib, public domain)

These files are from the zune-image test suite, which includes BMPSuite files
and additional test images.

| File | Description |
|------|-------------|
| david.bmp | Reference photo (55 KB) |
| pal1bg.bmp | 1-bit, background color |
| pal1.bmp | 1-bit paletted |
| pal1wb.bmp | 1-bit, white/black |
| pal2.bmp | 2-bit paletted |
| pal2color.bmp | 2-bit colored palette |
| pal4.bmp | 4-bit paletted |
| pal4gs.bmp | 4-bit grayscale palette |
| pal4rle.bmp | 4-bit RLE compressed |
| pal8-0.bmp | 8-bit, variant 0 |
| pal8.bmp | 8-bit paletted |
| pal8gs.bmp | 8-bit grayscale palette |
| pal8nonsquare.bmp | 8-bit, non-square dimensions |
| pal8os2.bmp | 8-bit OS/2 format |
| pal8rle.bmp | 8-bit RLE compressed |
| pal8topdown.bmp | 8-bit top-down row order |
| pal8v4.bmp | 8-bit, BITMAPV4HEADER |
| pal8v5.bmp | 8-bit, BITMAPV5HEADER |
| pal8w124.bmp | 8-bit, width 124 |
| pal8w125.bmp | 8-bit, width 125 |
| pal8w126.bmp | 8-bit, width 126 |
| rgb16-565.bmp | 16-bit, 5-6-5 |
| rgb16-565pal.bmp | 16-bit, 5-6-5 with palette |
| rgb16.bmp | 16-bit default |
| rgb16bfdef.bmp | 16-bit bitfield default |
| rgb24.bmp | 24-bit RGB |
| rgb24pal.bmp | 24-bit with palette |
| rgb32bf.bmp | 32-bit bitfield |
| rgb32bfdef.bmp | 32-bit bitfield default |
| rgb32.bmp | 32-bit standard |
| rgba16-4444.bmp | 16-bit RGBA 4:4:4:4 |
| rgba16-5551.bmp | 16-bit RGBA 5:5:5:1 |
| rgba32-1.bmp | 32-bit RGBA variant 1 |
| rgba32-2.bmp | 32-bit RGBA variant 2 |
| rgba32abf.bmp | 32-bit RGBA alpha bitfield |
| rgba32h56.bmp | 32-bit RGBA, 56-byte header |

### Pillow (HPND)

| File | Description |
|------|-------------|
| pal8v4-pillow.bmp | 8-bit v4 header (Pillow variant, differs from zune-image) |
| hopper.bmp | 128x128 reference photo |
| hopper_emboss.bmp | Emboss filter applied |
| hopper_emboss_more.bmp | Enhanced emboss |
| hopper_rle8.bmp | 8-bit RLE reference photo |
| hopper_rle8_grayscale.bmp | 8-bit grayscale RLE |

---

## non-conformant/

Files that deviate from the BMP specification but are accepted by many decoders.
Includes unusual bitfield layouts, OS/2 header variants, embedded color profiles,
RLE edge cases, and uncommon compression types.

### zune-image / BMPSuite (MIT/Apache-2.0/Zlib, public domain)

| File | Description |
|------|-------------|
| pal1p1.bmp | 1-bit, 1-entry palette (ambiguous) |
| pal1hufflsb.bmp | 1-bit Huffman 1D LSB (OS/2) |
| pal1huffmsb.bmp | 1-bit Huffman 1D MSB (OS/2) |
| pal4rlecut.bmp | 4-bit RLE, truncated early |
| pal4rletrns.bmp | 4-bit RLE with transparency |
| pal8offs.bmp | 8-bit with unusual data offset |
| pal8os2-hs.bmp | 8-bit OS/2, halftone stretch |
| pal8os2sp.bmp | 8-bit OS/2 special variant |
| pal8os2-sz.bmp | 8-bit OS/2 with explicit size |
| pal8os2v2-16.bmp | 8-bit OS/2 v2, 16-byte variant |
| pal8os2v2-40sz.bmp | 8-bit OS/2 v2, 40-byte with size |
| pal8os2v2.bmp | 8-bit OS/2 v2 standard |
| pal8os2v2-sz.bmp | 8-bit OS/2 v2 with explicit size |
| pal8oversizepal.bmp | 8-bit with oversized palette |
| pal8rlecut.bmp | 8-bit RLE, truncated early |
| pal8rletrns.bmp | 8-bit RLE with transparency |
| rgb16-231.bmp | 16-bit, 2-3-1 bitfield |
| rgb16-3103.bmp | 16-bit, 3-10-3 bitfield |
| rgb16faketrns.bmp | 16-bit with fake transparency |
| rgb24jpeg.bmp | 24-bit with JPEG compression marker |
| rgb24largepal.bmp | 24-bit with unnecessarily large palette |
| rgb24lprof.bmp | 24-bit with linked color profile |
| rgb24png.bmp | 24-bit with PNG compression marker |
| rgb24prof2.bmp | 24-bit with embedded profile (variant 2) |
| rgb24prof.bmp | 24-bit with embedded color profile |
| rgb24rle24.bmp | 24-bit with RLE24 compression |
| rgb32-111110.bmp | 32-bit, 1:1:1:1:1:0 bitfield |
| rgb32-7187.bmp | 32-bit, 7-1-8-7 bitfield |
| rgb32fakealpha.bmp | 32-bit with zeroed alpha channel |
| rgb32h52.bmp | 32-bit, 52-byte header |
| rgb32-xbgr.bmp | 32-bit xBGR bitfield layout |
| rgba16-1924.bmp | 16-bit RGBA, 1-9-2-4 bitfield |
| rgba32-1010102.bmp | 32-bit RGBA, 10:10:10:2 |
| rgba32-61754.bmp | 32-bit RGBA, 6-17-5-4 bitfield |
| rgba32-81284.bmp | 32-bit RGBA, 8-12-8-4 bitfield |

### Pillow (HPND)

| File | Description |
|------|-------------|
| rgb32bf-abgr.bmp | 32-bit aBGR bitfield layout |
| rgb32bf-rgba.bmp | 32-bit RGBA bitfield layout |
| hopper_rle8_row_overflow.bmp | RLE with row overflow |
| pal8_offset.bmp | 8-bit with non-standard offset |
| l2rgb_read.bmp | Minimal luminance-to-RGB test |
| mmap_error.bmp | Memory mapping edge case |

---

## invalid/

Corrupt or intentionally broken BMP files. Decoders should reject these cleanly
without panicking or producing incorrect output.

### Pillow (HPND)

| File | Description |
|------|-------------|
| badbitcount.bmp | Invalid bits-per-pixel value |
| badbitssize.bmp | Invalid image data size field |
| baddens1.bmp | Invalid horizontal resolution |
| baddens2.bmp | Invalid vertical resolution |
| badfilesize.bmp | File size field doesn't match actual size |
| badheadersize.bmp | Invalid info header size |
| badpalettesize.bmp | Invalid palette size |
| badplanes.bmp | Planes field != 1 |
| badrle.bmp | Corrupt RLE compressed data |
| badwidth.bmp | Invalid width value |
| pal8badindex.bmp | Palette index out of range |
| reallybig.bmp | Unreasonably large dimensions |
| rletopdown.bmp | Top-down with RLE (forbidden by spec) |
| shortfile.bmp | Truncated file (incomplete data) |
