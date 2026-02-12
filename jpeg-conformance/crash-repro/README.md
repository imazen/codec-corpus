# JPEG Crash Reproducer Files

JPEG files from upstream decoder bug reports that triggered crashes, panics,
incorrect output, or resource exhaustion. Collected from:

- **jpeg-decoder/** — 15 files from [image-rs/jpeg-decoder](https://github.com/image-rs/jpeg-decoder) issues
- **jpeg-decoder-257/** — 27 broken real-world images from jpeg-decoder issue #257
- **zune-jpeg/** — 29 files from [etemesi254/zune-image](https://github.com/etemesi254/zune-image) issues
- **libjpeg-turbo/** — 6 files from [libjpeg-turbo](https://github.com/libjpeg-turbo/libjpeg-turbo) issues

## Classification

These files fall into two categories:

**Valid JPEGs** (triggered C-specific bugs like segfaults, heap corruption, UB,
or output-correctness bugs — the JPEG data itself is valid):

- `jd_095_exif_zero.jpg`, `jd_130_progressive_jpeg.jpg`, `jd_043_sony_ericsson.jpg`
- `jd_169_corrupt_no_error.jpg`, `jd_249_incorrect_decode.jpg`
- `zj_004_garbled.jpg`, `zj_005_black_stripe.jpg`, `zj_007_shifted_rows.jpg`
- `zj_008_shifted_rows.jpg`, `zj_064_no_more_bytes.jpg`, `zj_243_progressive_lowres.jpg`
- `zj_246_unknown_marker.jpg`, `zj_269_exhausted_data.jpg`, `zj_275_cmyk_3comp.jpg`
- `zj_292_exhausted_data_panorama.jpg`, `zj_340_decode_regression.jpg`
- `ljt_259_int_overflow_dc_first.jpg`, `ljt_347_signed_overflow_dc_*.jpg`
- `ljt_669_uaf_ycc_rgb_a.jpg`, `ljt_758_segv_adjust_quant_poc.jpg`
- Most jd_257 files (real-world camera images)

**Invalid/malformed JPEGs** (should be rejected gracefully):

- `jd_151_unset_quant_table.jpg`, `jd_052_rst_marker.jpg`, `jd_125_oob_access.jpg`
- `jd_110_invalid_dht.jpg`, `jd_148_assertion_failed.jpg`, `jd_262_truncated_eof.jpg`
- `zj_162_mcu_assert.jpg`, `zj_167_marker_soi.jpg`, `zj_207_mcu_range_cmyk.jpg`
- `zj_277_broken_decode.jpg`, `zj_278_marker_soi.jpg`
- `ljt_509_missing_huffman_table.jpg`

## Attribution

See the per-project manifests in the zenjpeg repository for detailed
per-issue descriptions, root causes, and original issue links.
