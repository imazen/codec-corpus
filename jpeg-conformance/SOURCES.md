# JPEG Conformance Test Suite - Sources

Attribution for all files in this test suite.

## License Summary

| Source | License | URL |
|--------|---------|-----|
| jpeg-decoder | MIT | https://github.com/image-rs/jpeg-decoder |
| libjpeg-turbo | IJG + BSD | https://github.com/libjpeg-turbo/libjpeg-turbo |
| imagetestsuite | Various | https://code.google.com/p/imagetestsuite/ |
| exif-samples | MIT | https://github.com/ianare/exif-samples |
| imageflow | AGPL-3.0 | https://github.com/imazen/imageflow |
| zune-image | MIT/Apache-2.0/Zlib | https://github.com/etemesi254/zune-image |
| Generated | CC0 | Created for this test suite |

---

## valid/

Valid JPEG files that conformant decoders must handle correctly.

### Reference Images (jpeg-decoder)

| File | Source | License |
|------|--------|---------|
| 16bit-qtables.jpg | jpeg-decoder/reftest | MIT |
| blank_800x280.jpg | jpeg-decoder/reftest | MIT |
| extraneous-data.jpg | jpeg-decoder/reftest | MIT |
| grayscale_*.jpg | jpeg-decoder/reftest | MIT |
| mjpeg.jpg | jpeg-decoder/reftest | MIT |
| non-interleaved-mcu.jpg | jpeg-decoder/reftest | MIT |
| partial_progressive.jpg | jpeg-decoder/reftest | MIT |
| progressive*.jpg | jpeg-decoder/reftest | MIT |
| restarts.jpg | jpeg-decoder/reftest | MIT |
| rgb.jpg | jpeg-decoder/reftest | MIT |
| ycck.jpg | jpeg-decoder/reftest | MIT |

### Reference Images (libjpeg-turbo)

| File | Source | License | Notes |
|------|--------|---------|-------|
| testorig.jpg | libjpeg-turbo/testimages | IJG+BSD | Baseline JPEG |
| testimgint.jpg | libjpeg-turbo/testimages | IJG+BSD | Progressive JPEG |
| testimgari.jpg | libjpeg-turbo/testimages | IJG+BSD | Arithmetic coding |
| testorig12.jpg | libjpeg-turbo/testimages | IJG+BSD | 12-bit JPEG |

### Camera Samples (exif-samples)

| File | Source | License | Camera |
|------|--------|---------|--------|
| Canon_40D.jpg | exif-samples | MIT | Canon EOS 40D |
| Nikon_D70.jpg | exif-samples | MIT | Nikon D70 |
| Sony_HDR-HC3.jpg | exif-samples | MIT | Sony HDR-HC3 |
| Fujifilm_FinePix_E500.jpg | exif-samples | MIT | Fujifilm FinePix E500 |
| Olympus_C8080WZ.jpg | exif-samples | MIT | Olympus C8080WZ |
| Panasonic_DMC-FZ30.jpg | exif-samples | MIT | Panasonic DMC-FZ30 |
| Pentax_K10D.jpg | exif-samples | MIT | Pentax K10D |
| Samsung_Digimax_i50_MP3.jpg | exif-samples | MIT | Samsung Digimax i50 |
| Ricoh_Caplio_RR330.jpg | exif-samples | MIT | Ricoh Caplio RR330 |
| Kodak_CX7530.jpg | exif-samples | MIT | Kodak CX7530 |
| Konica_Minolta_DiMAGE_Z3.jpg | exif-samples | MIT | Konica Minolta DiMAGE Z3 |
| Reconyx_HC500_Hyperfire.jpg | exif-samples | MIT | Reconyx HC500 (trail cam) |

### Restart Marker Variants (Generated)

| File | Source | License | Notes |
|------|--------|---------|-------|
| rst_1row.jpg | Generated | CC0 | Restart interval: 1 MCU row |
| rst_2row.jpg | Generated | CC0 | Restart interval: 2 MCU rows |
| rst_4row.jpg | Generated | CC0 | Restart interval: 4 MCU rows |
| rst_1block.jpg | Generated | CC0 | Restart interval: 1 MCU block |
| rst_8block.jpg | Generated | CC0 | Restart interval: 8 MCU blocks |
| rst_16block.jpg | Generated | CC0 | Restart interval: 16 MCU blocks |

### Progressive + Restart + Subsampling (Generated)

| File | Source | License | Notes |
|------|--------|---------|-------|
| progressive_rst_420.jpg | Generated (zenjpeg) | CC0 | Progressive 4:2:0 with DRI, 16x16. Triggers zune-jpeg 0.5 grayscale decode bug. |

### Color Model Variants

| File | Source | License | Notes |
|------|--------|---------|-------|
| cmyk_logo.jpg | imageflow | AGPL-3.0 | CMYK color model |
| cymk.jpg | zune-image | MIT | CMYK color model |
| ycck.jpg | jpeg-decoder | MIT | YCCK color model |

---

## invalid/

Invalid JPEG files that decoders must reject gracefully (error, not crash).

### Crash Tests (jpeg-decoder)

| File | Source | License | Issue |
|------|--------|---------|-------|
| empty.jpg | jpeg-decoder | MIT | Zero-length file |
| missing-sof.jpg | jpeg-decoder | MIT | No SOF marker |
| missing-sos.jpg | jpeg-decoder | MIT | No SOS marker |
| null_height.jpg | jpeg-decoder | MIT | Height = 0 in SOF |
| invalid-dimensions.jpg | jpeg-decoder | MIT | Invalid dimensions |
| max_size.jpg | jpeg-decoder | MIT | Extreme dimensions |
| derive-huffman-codes-overflow.jpg | jpeg-decoder | MIT | Huffman overflow |
| dc-predictor-overflow.jpg | jpeg-decoder | MIT | DC predictor overflow |
| subtract-with-overflow.jpg | jpeg-decoder | MIT | Arithmetic overflow |
| invalid-prediction-shift.jpg | jpeg-decoder | MIT | Invalid lossless shift |

### Malformed Files (imagetestsuite)

98 files from Google's imagetestsuite project. These are known-malformed
JPEGs collected from various sources, each with unique corruption patterns.

| Pattern | Source | License |
|---------|--------|---------|
| *.jpg (98 files) | imagetestsuite | Various |

### Invalid EXIF (exif-samples)

| File | Source | License | Issue |
|------|--------|---------|-------|
| corrupted.jpg | exif-samples | MIT | Corrupted EXIF data |
| image00971.jpg | exif-samples/invalid | MIT | Invalid EXIF |
| image01088.jpg | exif-samples/invalid | MIT | Invalid EXIF |
| image01137.jpg | exif-samples/invalid | MIT | Invalid EXIF |
| image01551.jpg | exif-samples/invalid | MIT | Invalid EXIF |
| image01713.jpg | exif-samples/invalid | MIT | Invalid EXIF |
| image01980.jpg | exif-samples/invalid | MIT | Invalid EXIF |
| image02206.jpg | exif-samples/invalid | MIT | Invalid EXIF |

---

## non-conformant/

Files that technically violate the JPEG spec but may be accepted by
lenient decoders. See companion .txt files for details on each file.

### truncated/

| File | Source | License |
|------|--------|---------|
| after_soi.jpg | Generated from testorig.jpg | CC0 |
| after_app0.jpg | Generated from testorig.jpg | CC0 |
| mid_header.jpg | Generated from testorig.jpg | CC0 |
| after_sos.jpg | Generated from testorig.jpg | CC0 |
| scan_10pct.jpg | Generated from testorig.jpg | CC0 |
| scan_50pct.jpg | Generated from testorig.jpg | CC0 |
| scan_90pct.jpg | Generated from testorig.jpg | CC0 |
| missing_eoi.jpg | Generated from testorig.jpg | CC0 |
| progressive_25pct.jpg | Generated from testimgint.jpg | CC0 |
| progressive_50pct.jpg | Generated from testimgint.jpg | CC0 |
| progressive_75pct.jpg | Generated from testimgint.jpg | CC0 |
| missing-frame-image-1410.jpg | jpeg-decoder | MIT |

### extraneous-data/

| File | Source | License |
|------|--------|---------|
| extraneous-bytes-after-sos.jpg | jpeg-decoder (Brother scanner) | MIT |

### marker-quirks/

| File | Source | License |
|------|--------|---------|
| multiple-0xff-before-eoi.jpg | jpeg-decoder | MIT |

### metadata-quirks/

| File | Source | License |
|------|--------|---------|
| icc_chunk_count_mismatch.jpeg | jpeg-decoder | MIT |
| icc_chunk_double_seq_no.jpeg | jpeg-decoder | MIT |
| icc_chunk_order.jpeg | jpeg-decoder | MIT |
| icc_chunk_seq_no_0.jpeg | jpeg-decoder | MIT |
| icc_missing_chunk.jpeg | jpeg-decoder | MIT |

### progressive-quirks/

| File | Source | License | Notes |
|------|--------|---------|-------|
| mozjpeg-rs-ac-refine-q95.jpg | mozjpeg-rs | BSD-3-Clause | AC refinement edge case, decodes with libjpeg/mozjpeg but fails with jpeg-decoder/zune-jpeg |
