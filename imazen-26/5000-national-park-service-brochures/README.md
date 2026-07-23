# National Park Service — unigrid brochures & park maps

**63 pages, 537 MB**, rasterised to 300 dpi PNG, from **29 national parks / monuments / historical sites**. Per-file dims/sha256/PD-basis in `MANIFEST.tsv`.

The largest folder by byte size because NPS unigrid brochures are visually dense — hand-drawn relief shading on park maps + full-bleed colour photography + the iconic Vignelli grid typography. Codec-corpus gold.

## Parks included (29 units)

**Iconic large parks:**
Yosemite, Yellowstone, Grand Canyon, Great Smoky Mountains, Mount Rainier, Rocky Mountain, Sequoia/Kings Canyon, Death Valley, Denali, Shenandoah, Pinnacles, Capitol Reef, Big Bend, Black Canyon of the Gunnison, White Sands

**Alaskan wilderness:**
Wrangell–St. Elias (Chinese-language unigrid edition!), Gates of the Arctic, Katmai, Denali

**Coastal / island / desert:**
Channel Islands, Canyonlands, Redwood, American Samoa, Lake Mead

**Historical / smaller units:**
Big Hole NB (Battle of the Big Hole), Chesapeake & Ohio Canal NHP, Delaware Water Gap, Fort Vancouver NHS, Muir Woods NM, Santa Monica Mountains NRA

## About the Unigrid

Massimo Vignelli designed the NPS Unigrid System in 1977 as the visual identity for all NPS publications. The grid (Helvetica + a strict modular layout + the arrowhead motif) is iconic modernist information design. NPS Harpers Ferry Center has produced unigrid materials for every NPS unit since.

Notable inclusions:
- The full **2025 Yellowstone Unigrid** brochure
- **Sequoia/Kings Canyon Unigrid in French AND German** (visitor translations)
- **Wrangell-St. Elias Unigrid in Chinese** (Alaska's largest park, 13 million acres)
- **White Sands Unigrid 2022** (newest unit's brochure)

## Public-domain basis

**17 USC § 105.** NPS staff work product, produced by NPS Harpers Ferry Center. Park brochures, maps, and bulletins are federal government works → no copyright. Maps with hand-drawn relief by NPS cartographers, photographs by NPS photographers, typography by NPS designers — all official-duty work, all PD.

## Codec-corpus character

- **Hand-drawn relief shading** on park maps — soft topographic gradients that challenge codecs differently than satellite imagery
- **Full-bleed colour photography** on the photo side of unigrid brochures
- **Vignelli's Helvetica grid** with strict alignment — flat colour blocks + sharp edges
- **Mixed map + text + photo** in single layouts (the unigrid's strength)
- **Multi-language editions** (French/German/Chinese) — same content, different glyphs/character coverage

## What's NOT here, and why

A few parks failed to scrape because their HTML used URL-encoded entities (`&#x2f;`) or JavaScript handlers instead of plain `href=` PDF links: Zion, Bryce Canyon, Dry Tortugas, Lake Clark, Shiloh, Virgin Islands. Addable later with a smarter parser that decodes entities and follows JS handlers, or via the NPS Gallery `NPS Park Unigrid Brochures` collection at npgallery.nps.gov (71-item collection).
