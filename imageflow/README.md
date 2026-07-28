# Imageflow test inputs

33 test images sourced from the [Imageflow project's
test_inputs/](https://github.com/imazen/resources/tree/master/test_inputs)
collection. Imageflow uses these to exercise its decoder, color-management,
EXIF orientation, WebP, CMYK, and corrupt-input code paths.

## Provenance & license — mixed; verify before redistribution

The Imageflow project itself is **AGPL-3.0**, but its test inputs are
a curated mix of third-party samples gathered for codec development.
Most are well-known reference images in the imaging community; a few
were captured or constructed by the Imageflow authors. **No single
license covers the whole directory** — treat as research / fair-use
test material and verify the upstream source before redistributing
any individual file.

Notable third-party samples (best-effort attribution):

| File(s) | Origin | Typical license |
|---|---|---|
| `frymire.png`, `frymire-srgb.png` | Frymire screen-content test image (a well-known synthetic test image attributed to Karl Frymire) | Public domain (commonly cited as such in IQA literature) |
| `MarsRGB_tagged.jpg`, `MarsRGB_v4_sYCC_8bit.jpg` | Argyll CMS / sRGB ICC profile test (Mars surface NASA imagery) | NASA imagery generally public-domain; Argyll integration MIT/GPL |
| `dice.png` | Common synthetic dice render (origin unclear) | Verify upstream |
| `canon_5d_srgb.jpg`, `5760_x_4320.jpg` | Camera capture samples | Likely original to the Imageflow authors; verify |
| `wrenches.jpg`, `red-leaf.jpg`, `red-night.png`, `roof_test_800x600.jpg`, `mountain_800.gif`, `lossy_mountain.webp`, `shirt_transparent.png`, `waterhouse.jpg` | Stock / capture test images of unknown origin | Verify upstream |
| `*.webp` (1_webp_a.sm.png, 1_webp_a.webp, 1_webp_ll.webp, 5_webp_ll.webp) | WebP decoder test samples | Likely from libwebp or Google sample set — verify |
| `gradients.png`, `little_gradient_whitespace.jpg`, `whitespace-issue.png`, `gamma_test.jpg`, `rings2.png`, `dct_overflow_patterns/` | Synthetic / generated test patterns for specific bug repros | Likely original to Imageflow contributors (AGPL via the project) |
| `cmyk_logo.jpg`, `color_profile_error.jpg`, `corrupt.jpg`, `cropissue.jpg`, `png_turns_empty_2.png` | Bug-trigger samples | Origin varies — verify case-by-case |
| `orientation/` | EXIF orientation test set | Commonly the test set from <https://github.com/recurser/exif-orientation-examples> (MIT) |
| `pngsuite/` | PngSuite by Willem van Schaik | Freeware, see [`../pngsuite/LICENSE`](../pngsuite/LICENSE) |

The `dct_overflow_patterns/` subdirectory was added per
[mozilla/mozjpeg#453](https://github.com/mozilla/mozjpeg/issues/453)
to exercise a specific DCT overflow path — these are synthetic
patterns generated for that bug and are AGPL-3.0 with the rest of
Imageflow.

## Recommended use

✅ Decoder development, regression tests, codec research, internal
benchmarking — falls under fair-use research for any codec project.

⚠ **Do NOT redistribute individual files commercially without
verifying the upstream source.** The README in
[imageflow/resources](https://github.com/imazen/resources) is the
authoritative provenance reference; this README is best-effort.

If you find a file here whose license is more permissive (or more
restrictive) than what's documented, please open an issue or PR.
