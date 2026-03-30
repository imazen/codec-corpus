# UltraHDR / Gain Map Conformance Test Files

Test files for UltraHDR (JPEG-R) and ISO 21496-1 gain map decoder/parser validation.

UltraHDR embeds a gain map image and XMP metadata inside a standard JPEG, allowing
HDR-capable displays to reconstruct the HDR intent while remaining backward-compatible
with SDR viewers.

## Directory Structure

```
ultrahdr-conformance/
  valid/
    jpeg/
      awesome-gain-maps/        32 JPEGs + 1 .uhdr, 7.3 MB - Diverse gain map JPEGs
        cats/                   10 JPEGs, 1.5 MB            - Cat photos with gain maps
      libultrahdr-testdata/      5 JPEGs, 144 KB            - Google libultrahdr unit test images
      libultrahdr-benchmark/     2 JPEGs, 16 MB             - Google libultrahdr benchmark images
      pixel-ultrahdr/            3 JPEGs, 7.6 MB            - Pixel 6 Pro UltraHDR photos
    avif/                        (empty - no AVIF gain map samples found with permissive license)
    jxl/                         (empty - no JXL gain map samples found with permissive license)
  invalid/                       5 files                    - Corrupt/malformed files
  edge-cases/                    3 files                    - Minimal/boundary files
```

Total: 56 files, ~32 MB

## Sources

### awesome-gain-maps/ (MIT, with mixed image attributions)

- **Source:** https://github.com/NMoroney/Awesome-Gain-Maps
- **License:** Repository is MIT. Individual images have various attributions:
  - NMoroney originals: CC-BY-4.0 (test charts, procedural art, visualizations, text)
  - Photography: sourced from Wikimedia Commons (CC-licensed originals processed into gain maps)
  - Video game screenshots: from open-source games via Wikimedia Commons
  - Medical: from Kvasir dataset (https://datasets.simula.no/kvasir/)
  - `rgba.uhdr` and `rgba_uhdr.jpg`: from Google's libultrahdr (Apache-2.0)
- **Content:** Broad coverage of gain map use cases:
  - Photography (7): Tokyo neon, waterfalls, airborne, sports, flowers, astronomy, cats
  - Test charts (4): grayscale ramps, color patches, squares
  - Video games (4): various open-source game screenshots
  - Visualization (2): matplotlib, 3D scatterplot
  - Medical (1): endoscopy z-line
  - Procedural art (1): Mona Lisa variation
  - UI (1): Adobe demo app
  - Text (1): sphinx excerpt
  - Raw UltraHDR format (2): rgba.uhdr / rgba_uhdr.jpg

### libultrahdr-testdata/ (CC-BY-4.0)

- **Source:** https://github.com/google/libultrahdr (`tests/data/`)
- **License:** CC-BY-4.0 (see LICENSE file in directory)
- **Content:** 5 small JPEG files used by libultrahdr unit tests:
  - `jpeg_image.jpg` - Standard JPEG
  - `minnie-320x240-rgb.jpg` - RGB colorspace JPEG
  - `minnie-320x240-y.jpg` - Luminance-only JPEG
  - `minnie-320x240-yuv.jpg` - YUV colorspace JPEG
  - `minnie-320x240-yuv-icc.jpg` - YUV JPEG with ICC profile
  - Note: These are input test images, not necessarily UltraHDR outputs. They're used by the library's test suite for encoding validation.

### libultrahdr-benchmark/ (Apache-2.0)

- **Source:** https://storage.googleapis.com/android_media/external/libultrahdr/benchmark/UltrahdrBenchmarkTestRes-1.1.zip
- **License:** Apache-2.0 (part of google/libultrahdr, which is Apache-2.0)
- **Content:** 2 files from the benchmark resource pack:
  - `mountains.jpg` - Base SDR JPEG (4.7 MB, 3024x4032)
  - `mountains_singlechannelgainmap.jpg` - UltraHDR JPEG-R with single-channel gain map (12 MB)
  - Note: The full benchmark pack also includes multichannel and gamma variants (removed to stay under 40 MB). Available at the source URL above.

### pixel-ultrahdr/ (CC-BY-4.0)

- **Source:** https://github.com/MishaalRahmanGH/Ultra_HDR_Samples
- **License:** CC-BY-4.0
- **Attribution:** Photos by Mishaal Rahman, captured on a Pixel 6 Pro running Android 14 QPR1 Beta 2 with Google Camera v9.1.098.
- **Content:** 3 real-world UltraHDR photos (2.2-2.8 MB each):
  - `Ultra_HDR_Samples_Originals_01.jpg`
  - `Ultra_HDR_Samples_Originals_02.jpg`
  - `Ultra_HDR_Samples_Originals_05.jpg`
  - Note: The full set has 10 originals + 20 SDR emulation variants (~107 MB total). Only the 3 smallest originals are included here. The full set is at the source URL above.

## invalid/

Hand-crafted corrupt files for testing error handling:

| File | Description |
|------|-------------|
| `zero_length.jpg` | 0 bytes |
| `truncated_ultrahdr.jpg` | First 1024 bytes of a gain map JPEG |
| `no_gainmap.jpg` | Valid standard JPEG with no gain map metadata |
| `bitflip_gainmap.jpg` | Gain map JPEG with 30 random byte-flips in the second half (gain map region) |
| `wrong_format.jpg` | PNG data with .jpg extension |

## edge-cases/

| File | Description |
|------|-------------|
| `minimal_1x1_jpeg.jpg` | Minimal valid 1x1 JPEG (no gain map) |
| `xmp_only_no_gainmap_image.jpg` | Copy of a small gain map JPEG (may have XMP but test if gain map image is extractable) |
| `rgba_format.uhdr` | UltraHDR in `.uhdr` container format (from libultrahdr) |

## Additional Sources (Not Downloaded)

### Permissive but too large or requires account

- **IS&T ISO HDR Test Sets** (https://www.imaging.org/IST/IST/Standards/ISO%20HDR%20Images%20Test%20Sets.aspx):
  CC-BY-NC-4.0 license. Four test sets with 23-24 images each covering ISO 21496-1 (Adaptive HDR) and ISO 22028-5 (PQ HDR). Free but requires creating an IS&T account. Non-commercial only.

- **MishaalRahmanGH/Ultra_HDR_Samples full set**: CC-BY-4.0. 10 originals + 20 SDR emulations, ~107 MB total. Only 3 smallest originals included here.

- **libultrahdr benchmark full pack**: Apache-2.0. 152 MB zip with raw P010, YUV420, RGBA formats plus all JPEG-R variants. Only the single-channel gain map JPEG-R variant included here.

### No permissive license / unclear

- **Greg Benz HDR Gain Map Gallery** (https://gregbenzphotography.com/hdr-gain-map-gallery/): Professional HDR gain map photos in JPG, AVIF, JXL. No clear redistribution license.

- **Adobe Gain Map Demo App samples**: Available through Adobe Camera Raw docs. License unclear for redistribution.

### AVIF/JXL gain maps

No freely-licensed AVIF or JXL gain map test files were found. The ISO 21496-1 standard supports gain maps in AVIF, JXL, and HEIF containers, but sample files in these formats are not yet widely available under permissive licenses. Adobe's tools can produce them, but redistribution rights are unclear.
