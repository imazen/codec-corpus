//! Reference-set selection: the "≥ 5 content classes × ≥ 10 references each"
//! half of the corpus (imazen/codec-corpus#7).
//!
//! The references come from the public **imazen-26** corpus, whose
//! `imazen-26/manifests/{train,val}.tsv` files in this repo list one PNG per
//! image with a direct public URL. Those manifests carry a 21-way *category*
//! (`1400-lilith-nature`, `8100-lilith-web-screenshots`, ...), which
//! [`content_class_for_category`] folds into the five [`ContentClass`]es the
//! corruption sweep stratifies by. The fold is by category, not by
//! per-image inspection — it is documented in that function and a test pins
//! it against the checked-in manifest so a new category cannot silently go
//! unclassified.
//!
//! Selection is deterministic ([`select_per_class`]): for every class the
//! candidate categories are round-robined so a class is not drawn from a
//! single category, and each category's order is a seeded shuffle.

use std::collections::BTreeMap;

use crate::{ContentClass, prng};

/// One candidate reference image from an imazen-26 manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceEntry {
    /// Manifest `id` (stable numeric id as a string).
    pub id: String,
    /// The manifest's 21-way category, e.g. `1400-lilith-nature`.
    pub category: String,
    /// The corruption-corpus content class the category folds into.
    pub class: ContentClass,
    /// Path relative to the PNG-v3 layer root, e.g. `1400-lilith-nature/1400_….sdr.png`.
    pub relative_path: String,
    /// Direct public URL of the PNG.
    pub url: String,
}

impl ReferenceEntry {
    /// Stable corpus `ref_id` for manifests: `imazen-26/<category>/<id>`.
    pub fn ref_id(&self) -> String {
        format!("imazen-26/{}/{}", self.category, self.id)
    }

    /// File name to store the download under (the manifest's basename).
    pub fn file_name(&self) -> &str {
        self.relative_path
            .rsplit('/')
            .next()
            .unwrap_or(&self.relative_path)
    }
}

/// Fold an imazen-26 category into a corruption-corpus [`ContentClass`].
///
/// Returns `None` for categories that fit none of the five classes well
/// enough to stratify on (currently `9094-lilith-ai-illustrations`: painterly
/// AI illustrations, neither line art nor photo). Unknown categories also
/// return `None`; the test `mapping_covers_checked_in_manifest` fails when
/// the repo's manifest gains a category this table does not know.
pub fn content_class_for_category(category: &str) -> Option<ContentClass> {
    use ContentClass::*;
    Some(match category {
        // Natural / captured photographs.
        "1000-lilith-photos-general"
        | "1200-lilith-interiors"
        | "1400-lilith-nature"
        | "1600-lilith-food"
        | "2000-unsplash-people"
        | "2400-unsplash-textures"
        | "3000-art-institute-of-chicago-photos"
        | "3300-met-museum-photos" => Photo,
        // Screenshots / UI.
        "8000-lilith-mobile-screenshots" | "8100-lilith-web-screenshots" => Screen,
        // Line art: charts, clipart, engraved plates.
        "7000-lilith-plots"
        | "9000-lilith-ai-clipart"
        | "6600-ia-scans-manuscript-illustrations" => LineArt,
        // Text-dominant scans and documents.
        "6000-lilith-scans-public-patents"
        | "6800-ia-scans-manuscript-text"
        | "5000-national-park-service-brochures"
        | "5200-epa-climate-impact-2021-report"
        | "5300-noaa-hurricane-documents" => Text,
        // Smooth synthetic renders (abstract renders, studio product shots).
        "2200-unsplash-renders" | "9226-lilith-ai-products" => Gradient,
        _ => return None,
    })
}

/// Parse an imazen-26 manifest TSV (`id, split, content_class, variant,
/// relative_path, url`, header-driven so column order does not matter).
///
/// Keeps only `variant == "sdr"` rows (one per image; the HDR renders are a
/// second rendering of the same 64 gain-map images) whose category folds
/// into a [`ContentClass`]. Returns an error if a required column is absent.
pub fn parse_imazen26_manifest(tsv: &str) -> Result<Vec<ReferenceEntry>, String> {
    let mut lines = tsv.lines().filter(|l| !l.trim().is_empty());
    let header = lines.next().ok_or("empty manifest")?;
    let cols: Vec<&str> = header.split('\t').map(str::trim).collect();
    let col = |name: &str| {
        cols.iter()
            .position(|c| *c == name)
            .ok_or_else(|| format!("manifest is missing the '{name}' column"))
    };
    let (ci, cc, cv, cp, cu) = (
        col("id")?,
        col("content_class")?,
        col("variant")?,
        col("relative_path")?,
        col("url")?,
    );
    let mut out = Vec::new();
    for line in lines {
        let f: Vec<&str> = line.split('\t').collect();
        let get = |i: usize| f.get(i).map(|s| s.trim()).unwrap_or("");
        if get(cv) != "sdr" {
            continue;
        }
        let category = get(cc);
        let Some(class) = content_class_for_category(category) else {
            continue;
        };
        out.push(ReferenceEntry {
            id: get(ci).to_string(),
            category: category.to_string(),
            class,
            relative_path: get(cp).to_string(),
            url: get(cu).to_string(),
        });
    }
    Ok(out)
}

/// Deterministically pick up to `per_class` references for every
/// [`ContentClass`], round-robining across that class's categories (each
/// category's order is a seeded shuffle) so no class is drawn from one
/// category alone. Classes with fewer candidates than `per_class` yield what
/// they have; the caller decides whether that is enough.
pub fn select_per_class(
    entries: &[ReferenceEntry],
    per_class: usize,
    seed: u64,
) -> Vec<ReferenceEntry> {
    // class -> category -> entries (BTreeMap for a stable iteration order).
    let mut by_class: BTreeMap<u8, BTreeMap<&str, Vec<&ReferenceEntry>>> = BTreeMap::new();
    for e in entries {
        by_class
            .entry(e.class as u8)
            .or_default()
            .entry(e.category.as_str())
            .or_default()
            .push(e);
    }
    let mut out = Vec::new();
    for class in ContentClass::all() {
        let Some(cats) = by_class.get(&(class as u8)) else {
            continue;
        };
        let mut queues: Vec<Vec<&ReferenceEntry>> = cats
            .iter()
            .map(|(cat, list)| {
                let mut v = list.clone();
                v.sort_by(|a, b| a.id.cmp(&b.id));
                let mut rng = prng::SplitMix64::new(prng::seed_for(cat, seed));
                // Fisher-Yates.
                for i in (1..v.len()).rev() {
                    let j = rng.below(i as u64 + 1) as usize;
                    v.swap(i, j);
                }
                v.reverse(); // so `pop` yields the shuffled order front-to-back
                v
            })
            .collect();
        let mut picked = 0;
        while picked < per_class {
            let mut progressed = false;
            for q in queues.iter_mut() {
                if picked == per_class {
                    break;
                }
                if let Some(e) = q.pop() {
                    out.push(e.clone());
                    picked += 1;
                    progressed = true;
                }
            }
            if !progressed {
                break;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The repo's `imazen-26/manifests/train.tsv`, embedded at compile time
    /// (test builds only) so the test is hermetic on every target — the
    /// wasm32-wasip1 CI job runs under `wasmtime --dir .` with only the
    /// package root preopened, where a host-absolute path is not readable.
    fn checked_in_manifest() -> &'static str {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../imazen-26/manifests/train.tsv"
        ))
    }

    /// Every category in the repo's manifest is either folded into a class
    /// or deliberately listed as unmapped — nothing falls through by accident.
    #[test]
    fn mapping_covers_checked_in_manifest() {
        const DELIBERATELY_UNMAPPED: &[&str] = &["9094-lilith-ai-illustrations"];
        let text = checked_in_manifest();
        let mut lines = text.lines();
        let header: Vec<&str> = lines.next().unwrap().split('\t').collect();
        let cc = header.iter().position(|c| *c == "content_class").unwrap();
        let mut cats: Vec<String> = lines
            .filter(|l| !l.trim().is_empty())
            .map(|l| l.split('\t').nth(cc).unwrap().to_string())
            .collect();
        cats.sort();
        cats.dedup();
        assert!(
            cats.len() >= 20,
            "expected the 21-way category set, got {}",
            cats.len()
        );
        for c in &cats {
            let mapped = content_class_for_category(c).is_some();
            let listed = DELIBERATELY_UNMAPPED.contains(&c.as_str());
            assert!(
                mapped ^ listed,
                "category '{c}' must be mapped or deliberately unmapped (not both / neither)"
            );
        }
    }

    /// The headline #7 requirement: ≥ 5 content classes × ≥ 10 references,
    /// from the checked-in manifest, deterministically.
    #[test]
    fn at_least_ten_references_per_class_from_train_manifest() {
        let entries = parse_imazen26_manifest(checked_in_manifest()).unwrap();
        assert!(entries.len() > 1000, "got {}", entries.len());
        assert!(entries.iter().all(|e| e.url.starts_with("https://")));
        assert!(
            entries
                .iter()
                .all(|e| e.relative_path.ends_with(".sdr.png"))
        );

        let picked = select_per_class(&entries, 10, 1);
        for class in ContentClass::all() {
            let n = picked.iter().filter(|e| e.class == class).count();
            assert!(n >= 10, "{class:?}: only {n} references selectable");
        }
        assert_eq!(picked.len(), 50);
        // No duplicates.
        let mut ids: Vec<&str> = picked.iter().map(|e| e.id.as_str()).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), 50);
        // Round-robin: photo draws from more than one category.
        let mut photo_cats: Vec<&str> = picked
            .iter()
            .filter(|e| e.class == ContentClass::Photo)
            .map(|e| e.category.as_str())
            .collect();
        photo_cats.sort_unstable();
        photo_cats.dedup();
        assert!(photo_cats.len() >= 2, "{photo_cats:?}");

        assert_eq!(
            picked,
            select_per_class(&entries, 10, 1),
            "not deterministic"
        );
        assert_ne!(
            picked,
            select_per_class(&entries, 10, 2),
            "seed has no effect"
        );
    }

    #[test]
    fn parse_is_header_driven_and_filters() {
        let tsv = "url\tid\tvariant\tcontent_class\tsplit\trelative_path\n\
                   https://x/a.sdr.png\t7\tsdr\t7000-lilith-plots\ttrain\t7000-lilith-plots/a.sdr.png\n\
                   https://x/a.hdr.png\t7\thdr\t7000-lilith-plots\ttrain\t7000-lilith-plots/a.hdr.png\n\
                   https://x/b.sdr.png\t8\tsdr\t9094-lilith-ai-illustrations\ttrain\tb.sdr.png\n\
                   https://x/c.sdr.png\t9\tsdr\tno-such-category\ttrain\tc.sdr.png\n";
        let entries = parse_imazen26_manifest(tsv).unwrap();
        assert_eq!(entries.len(), 1, "{entries:?}");
        let e = &entries[0];
        assert_eq!(e.id, "7");
        assert_eq!(e.class, ContentClass::LineArt);
        assert_eq!(e.ref_id(), "imazen-26/7000-lilith-plots/7");
        assert_eq!(e.file_name(), "a.sdr.png");

        let missing = "id\tsplit\tvariant\n1\ttrain\tsdr\n";
        let err = parse_imazen26_manifest(missing).unwrap_err();
        assert!(err.contains("content_class"), "{err}");
    }

    #[test]
    fn select_handles_short_classes_and_zero() {
        let mk = |id: &str, cat: &str| ReferenceEntry {
            id: id.into(),
            category: cat.into(),
            class: content_class_for_category(cat).unwrap(),
            relative_path: format!("{cat}/{id}.sdr.png"),
            url: format!("https://x/{id}"),
        };
        let entries = vec![
            mk("1", "8100-lilith-web-screenshots"),
            mk("2", "8000-lilith-mobile-screenshots"),
            mk("3", "8100-lilith-web-screenshots"),
        ];
        let picked = select_per_class(&entries, 10, 0);
        assert_eq!(picked.len(), 3, "short class yields what it has");
        assert!(select_per_class(&entries, 0, 0).is_empty());
        // Round-robin: with 2 picks, both categories appear.
        let two = select_per_class(&entries, 2, 0);
        let cats: std::collections::BTreeSet<&str> =
            two.iter().map(|e| e.category.as_str()).collect();
        assert_eq!(cats.len(), 2);
    }
}
