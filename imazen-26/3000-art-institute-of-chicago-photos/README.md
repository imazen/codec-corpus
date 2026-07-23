# High-Resolution CC0 Art Corpus (`art-cc0`)

High-resolution **CC0 / public-domain color artwork scans** from two keyless open
APIs: the Metropolitan Museum of Art Open Access and the Art Institute of Chicago.
Paintings, prints, and objects with fine detail, smooth tonal gradients, and large
dimensions — a continuous-tone color class that stresses codecs differently from
photos or document scans.

- **39 artworks (24 Met + 15 AIC) · 31 chromatic · ~95 MB**
- Built `2026-06-07` by `fetch_art.py` (reproducible)

## License

Every item is flagged **public-domain / CC0** by the source institution
(`isPublicDomain: true` at the Met; `is_public_domain: true` at AIC). CC0 means no
rights reserved — freely redistributable. Per-item attribution (artist, title,
date, museum object ID, source URL, sha256) is in `MANIFEST.json`.

## Content profile

- **Genres** (via diverse API queries): landscape, portrait, still life, ukiyo-e,
  marine, interior, abstract, ceramics, manuscript.
- **Resolution:** Met = native full-res JPEG; AIC = IIIF up to 4000 px wide.
  Width 676 → 4000 px, **median 3000 px**. Note: a few Met items (notably some
  "manuscript" hits) are low-res thumbnails (676–1000 px); the bulk are 2–8 MP.
  Width/height are in `MANIFEST.json` if you want to filter to ≥2000 px.
- **Chroma:** 31/39 are chromatic; the 8 near-monochrome ones are mostly B&W
  prints / sepia ukiyo-e / ceramic vessels (each flagged `chromatic` in the
  manifest).

## Layout

```
fetch_art.py       queries Met + AIC, downloads PD images, writes manifest
chroma_index.tsv   per-image width/height/colorspace/saturation
MANIFEST.json       per-image CC0 attribution + dims/chroma
images/met/<objectID>_<slug>.jpg
images/aic/<id>_<slug>.jpg
```

## Reproduce / extend

```bash
python3 fetch_art.py        # re-queries the APIs (selection may vary slightly)
```

- Tune genre coverage via `met_queries` / `aic_queries`; `PER_QUERY` controls
  how many PD works per genre; `AIC_WIDTH` sets IIIF target width.
- To add **Rijksmuseum** (also CC0, very high-res): it needs a free API key —
  not wired in here to keep the pipeline keyless. Say the word and I'll add it.
- Selection is API-relevance-ordered, so re-running may pick slightly different
  works; the manifest records exactly what was fetched.
