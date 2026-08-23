# imazen-26 canonical split manifests

**THE RULE** (deterministic, no seed; identical to zenmetrics
`scripts/picker/origin_split.py`): the **last digit of the 4-digit image id** decides
the bucket —

| last digit | bucket |
|---|---|
| 0, 2, 4, 6, 8 | `train` |
| 1, 3, 5 | `validate` |
| 7, 9 | `test` |

Every derivative of an image — resize, crop, re-encode, feature row, metric row —
**inherits the image's bucket**. Datasets never invent their own split; they join on
`id` (the leading 4 digits of every corpus filename and of every spec-conformant
variant name).

## Files

| file | rows | what |
|---|---|---|
| `split_map.tsv` | 2,160 | bare `id → split` map (plus class + path) |
| `train.tsv` | 1,084 | full rows for the train bucket |
| `validate.tsv` | 658 | full rows for the validate bucket |
| `test.tsv` | 418 | full rows for the test bucket |

Columns: `id, split, content_class, path, width, height, format, bytes_manifest,
bytes_actual, sha256, raw_url, png_v3_sdr_url`. `sha256`/`bytes_actual` are computed
from the canonical files at generation time (`bytes_manifest` is the historical
`CORPUS-MANIFEST.tsv` value, which can lag after metadata rewrites). `raw_url` serves
the original bytes; `png_v3_sdr_url` the codec-test-ready PNG render.

Regenerate:

```
python3 imazen-26/scripts/make_canonical_split.py --repo-im26 imazen-26 --check-urls 14
```

Browsable per-bucket folder views (relative symlinks, no byte duplication) live under
[`../splits/`](../splits/).

## Supersedes

The pre-2026-08 `manifests/{train,val}.tsv` (an ~86/14 split over PNG-v3 render rows,
built from `picker-sweep-2026-06-22/imazen26_manifest.tsv`, with **no test bucket**)
are **superseded and removed**. Anything trained on them should be re-interpreted
through `split_map.tsv`; rows whose origin id lands in `validate`/`test` here must not
be treated as training data going forward.

## Known R2 drift (probed 2026-08-22)

- ids **9231, 9869, 9874** (renamed during curation): `png_v3` URL live, `raw_url`
  404 — `imazen-26-unprocessed/` still holds the pre-rename objects. Reconciliation
  pass queued.
- id **1433**: `raw_url` live, `png_v3` 404 (render missing under the current name).
- The R2 prefixes also hold objects with **no manifest row** (pre-curation working
  material). `CORPUS-MANIFEST.tsv` is the membership oracle; ignore unmanifested
  objects.

## Near-duplicate caution (the id rule does not group these)

The split is by id, so related images with different ids can land in different
buckets. If your task is sensitive to near-duplicate leakage, drop or same-bucket
these enumerable groups:

- `6000-lilith-scans-public-patents`: 3 patents × 3 scan variants of the same pages.
- `5000-national-park-service-brochures`: `color/` + `grayscale/` renders of the same
  brochures.
- `8100-lilith-web-screenshots`: the same URL captured at up to 6 viewports.
- `6600` + `6800` IA scans: illustrations vs text pages drawn from the same 6 source
  works.
