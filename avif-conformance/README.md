# AVIF Conformance Test Suite

137 AVIF files organized for decoder conformance testing. Covers still images, animation, alpha, HDR, gain maps, grids, multilayer, transforms, ICC profiles, and various chroma/bit-depth combinations.

## Directory Structure

```
avif-conformance/
  valid/         106 files — MUST decode successfully
  invalid/        12 files — MUST error gracefully (no crash, no hang)
  edge-cases/     19 files — decoder-dependent behavior, unusual but spec-legal
```

## Sources and Licenses

| Source | License | Files | Prefix | Notes |
|--------|---------|-------|--------|-------|
| [AOMediaCodec/av1-avif](https://github.com/AOMediaCodec/av1-avif) (Microsoft) | BSD-2-Clause (repo) + CC-BY 3.0 (Blender content) | 16 | `ms_` | Still, grid, crop, rotate, 4K, mono, alpha, HDR metadata, thumbnails |
| [AOMediaCodec/av1-avif](https://github.com/AOMediaCodec/av1-avif) (Netflix) | CC-BY-NC-ND 4.0 | 11 | `netflix_` | HDR PQ, SDR, lossless, yuv420/444, animated sequences |
| [AOMediaCodec/av1-avif](https://github.com/AOMediaCodec/av1-avif) (Apple) | CC-BY-SA 4.0 | 7 valid + 2 edge + 1 invalid | `apple_` | Multilayer (a1lx/a1op/lsel/grid), truncated stream, unknown properties |
| [AOMediaCodec/av1-avif](https://github.com/AOMediaCodec/av1-avif) (Xiph) | CC-BY-SA 3.0 / CC-BY 3.0 | 5 | `xiph_` | Film grain, multi-layer, multi-resolution scalability |
| [AOMediaCodec/libavif](https://github.com/AOMediaCodec/libavif) | BSD-2-Clause | 36 valid + 17 edge + 3 invalid | `libavif_` | Gain maps, grids, HDR, WCG, ICC/EXIF/XMP, animation, idat, custom properties |
| [link-u/avif-sample-images](https://github.com/link-u/avif-sample-images) | CC-BY-SA 4.0 | 30 | `linku_` | All profiles (0/1/2), 8/10/12bpc, yuv420/422/444, mono, alpha, odd dims, transforms, ICC |
| Unknown | Unknown | 1 | `2.avif` | Pre-existing animated AVIF sequence |

**License note:** Netflix files are CC-BY-NC-ND 4.0, which restricts commercial use and derivatives. They are included for testing purposes only. All other files use permissive licenses (BSD, CC-BY, CC-BY-SA).

## Feature Coverage Matrix

| Feature | Files | Directory |
|---------|-------|-----------|
| **Chroma 4:2:0** | `linku_fox_p0_*`, `ms_Mexico.avif`, `libavif_sofa_grid*`, Netflix yuv420 | valid/ |
| **Chroma 4:2:2** | `linku_fox_p2_*_yuv422.*`, `linku_hato_p2_*` | valid/ |
| **Chroma 4:4:4** | `linku_fox_p1_*_yuv444.*`, `linku_fox_p2_12bpc_yuv444.*`, `ms_Mexico_YUV444.avif`, Netflix yuv444 | valid/ |
| **8-bit** | `linku_fox_p0_8bpc_*`, `linku_hato_p0_8bpc_*`, `ms_Chimera_8bit_*` | valid/ |
| **10-bit** | `linku_fox_p0_10bpc_*`, `linku_fox_p1_10bpc_*`, `ms_Chimera_10bit_*`, Netflix HDR/SDR | valid/ |
| **12-bit** | `linku_fox_p2_12bpc_*`, `linku_hato_p2_12bpc_*`, `libavif_colors-animated-12bpc*` | valid/ |
| **Monochrome** | `linku_fox_p0_10bpc_mono.*`, `linku_hato_p2_12bpc_mono.*`, `ms_Monochrome.avif` | valid/ |
| **Alpha** | `linku_plum_*_alpha_*`, `ms_bbb_alpha_inverted.avif`, `netflix_alpha_video.avif`, `libavif_abc_color_irot_alpha_*` | valid/ |
| **HDR (PQ/HLG)** | `netflix_hdr_*`, `libavif_colors_hdr_*`, `libavif_seine_hdr_*`, `ms_Chimera_10bit_*_with_HDR_metadata.avif` | valid/ |
| **Wide Color Gamut** | `libavif_colors_wcg_*`, `libavif_colors_text_wcg_*` | valid/ |
| **Gain Maps** | `libavif_seine_*_gainmap_*`, `libavif_color_*_gainmap_*` | valid/ + edge-cases/ |
| **Animation** | `2.avif`, `netflix_chimera_animated_10bit.avif`, `netflix_alpha_video.avif`, `libavif_colors-animated-*` | valid/ |
| **Grid / Multi-tile** | `ms_Summer_in_Tomsk_720p_5x4_grid.avif`, `libavif_sofa_grid1x5_420.avif`, `apple_*_grid_*` | valid/ |
| **Multilayer** | `apple_animals_00_multilayer_*` (a1lx, a1op, lsel variants) | valid/ |
| **Multi-resolution** | `xiph_tiger_3layer_3res.avif`, `xiph_fruits_2layer_thumbsize.avif` | valid/ |
| **Film Grain** | `xiph_abandoned_filmgrain.avif` | valid/ |
| **Rotation (irot)** | `linku_kimono_rotate90.avif`, `linku_kimono_rotate270.avif`, `ms_Ronda_rotate90.avif` | valid/ |
| **Mirror (imir)** | `linku_kimono_mirror_h.avif`, `linku_kimono_mirror_v.avif` | valid/ |
| **Crop (clap)** | `linku_kimono_crop.avif`, `ms_Chimera_8bit_cropped_480x256.avif`, `ms_Chimera_10bit_cropped_*` | valid/ |
| **Combined Transforms** | `linku_kimono_mirror_v_rotate270.avif` | valid/ |
| **ICC Profiles** | `linku_icc_profile_8bpc.avif`, `linku_icc_profile_10bpc.avif`, `linku_icc_profile_12bpc.avif`, `libavif_paris_icc_exif_xmp.avif` | valid/ |
| **EXIF + XMP** | `libavif_paris_icc_exif_xmp.avif`, `libavif_colors-animated-8bpc-alpha-exif-xmp.avif` | valid/ |
| **Thumbnails** | `ms_Tomsk_with_thumbnails.avif` | valid/ |
| **Still Picture Header** | `ms_still_picture.avif`, `ms_reduced_still_picture_header.avif` | valid/ |
| **Odd Dimensions** | `linku_fox_odd_width.avif`, `linku_fox_odd_height.avif`, `linku_fox_odd_both.avif` | valid/ |
| **1x1 Minimum** | `libavif_white_1x1.avif` | valid/ |
| **4K Resolution** | `ms_bbb_4k.avif`, `ms_Summer_Nature_4k.avif` | valid/ |
| **P3 Color Space** | `libavif_colors_hdr_p3.avif`, `libavif_colors_text_hdr_p3.avif` | valid/ |
| **Rec.2020** | `libavif_colors_hdr_rec2020.avif`, `libavif_seine_hdr_rec2020.avif`, `libavif_colors_wcg_hdr_rec2020.avif` | valid/ |

## Invalid Files (must error gracefully)

| File | Description | Expected Behavior |
|------|-------------|-------------------|
| `empty.avif` | 0 bytes | Return error immediately |
| `not_avif.avif` | JPEG data with .avif extension | Reject at format detection |
| `truncated_header.avif` | First 100 bytes of valid file | Error during box parsing |
| `truncated_data.avif` | Valid header, pixel data cut to 50% | Error during AV1 decode |
| `bad_ftyp.avif` | ftyp box type overwritten with 'xxxx' | Reject at box parsing |
| `wrong_brand.avif` | Major brand set to 'mp41' instead of 'avif' | Reject or warn on brand check |
| `corrupted_mdat.avif` | Valid ISOBMFF structure, corrupted AV1 payload | Error during AV1 decode |
| `zero_dimensions.avif` | ispe box zeroed to 0x0 | Error on dimension validation |
| `apple_truncated_elementary_stream.avif` | AV1 elementary stream truncated mid-frame | Error during AV1 decode |
| `libavif_unsupported_gainmap_minimum_version.avif` | Gain map with unsupported minimum version | Error or ignore gain map |
| `libavif_unsupported_gainmap_version.avif` | Gain map with unsupported version | Error or ignore gain map |
| `libavif_unsupported_gainmap_writer_version_with_extra_bytes.avif` | Unsupported gain map writer version + trailing bytes | Error or ignore gain map |

## Edge Case Files (decoder-dependent behavior)

| File | Description | Notes |
|------|-------------|-------|
| `apple_free_property.avif` | Contains 'free' box as property | Decoders should ignore unknown properties |
| `apple_unknown_nonessential_property.avif` | Unknown non-essential property box | Must not cause rejection per spec |
| `libavif_alpha_noispe.avif` | Alpha plane without ispe property | Some decoders may reject |
| `libavif_arc_triomphe_extent1000_nullbyte_extent1310.avif` | Unusual item extent with null byte gap | Tests extent handling |
| `libavif_circle_custom_properties.avif` | Custom/unknown property boxes | Tests property tolerance |
| `libavif_clap_irot_imir_non_essential.avif` | Clap/irot/imir marked non-essential | Spec says these should be essential |
| `libavif_clop_irot_imor.avif` | Uses 'clop' and 'imor' box types | Newer ISOBMFF box names |
| `libavif_draw_points_idat.avif` | Pixel data in idat box (not mdat) | Tests idat-based storage |
| `libavif_draw_points_idat_metasize0.avif` | idat storage with meta size=0 | Edge case in box sizing |
| `libavif_draw_points_idat_progressive.avif` | Progressive idat delivery | Tests progressive decode |
| `libavif_draw_points_idat_progressive_metasize0.avif` | Progressive idat with meta size=0 | Combined edge case |
| `libavif_extended_pixi.avif` | Extended pixi (pixel information) box | Non-standard pixi variant |
| `libavif_sofa_grid1x5_420_dimg_repeat.avif` | Grid with repeated dimg references | Tests duplicate tile handling |
| `libavif_sofa_grid1x5_420_reversed_dimg_order.avif` | Grid with reversed dimg order | Tests non-sequential tile ordering |
| `libavif_color_grid_alpha_grid_tile_shared_in_dimg.avif` | Grid tiles shared between color and alpha | Tests tile deduplication |
| `libavif_seine_hdr_gainmap_wrongaltr.avif` | Gain map with incorrect altr grouping | Tests altr validation |
| `libavif_seine_sdr_gainmap_gammazero.avif` | Gain map with gamma=0 | Tests degenerate gain map params |
| `libavif_seine_sdr_gainmap_notmapbrand.avif` | Gain map without 'map1' brand | Tests brand-based gain map detection |
| `libavif_supported_gainmap_writer_version_with_extra_bytes.avif` | Supported gain map version with trailing bytes | Tests tolerance of extra data |

## AV1 Profile Coverage

| Profile | Chroma | Bit Depths | Example Files |
|---------|--------|------------|---------------|
| 0 (Main) | 4:2:0 | 8, 10 | `linku_fox_p0_*`, most Microsoft/Netflix files |
| 1 (High) | 4:4:4 | 8, 10 | `linku_fox_p1_*`, `ms_Mexico_YUV444.avif` |
| 2 (Professional) | 4:2:2, 4:4:4 | 8, 10, 12 | `linku_fox_p2_*`, `linku_hato_p2_*` |

## CICP (Color) Coverage

| Primaries | Transfer | Matrix | Files |
|-----------|----------|--------|-------|
| BT.709 (1) | sRGB (13) | BT.601 (6) | Netflix SDR, most still images |
| BT.2020 (9) | PQ (16) | BT.2020 NCL (9) | Netflix HDR |
| BT.2020 (9) | PQ (16) | Identity (0) | Netflix HDR lossless (RGB) |
| Display P3 | PQ | — | `libavif_colors_hdr_p3.avif` |

## Total Size

~33 MB across 137 files.
