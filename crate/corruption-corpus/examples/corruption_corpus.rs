//! Driver example for the structural-corruption distortion corpus.
//!
//! Given one or more reference images, this:
//!   1. expands the full corruption catalog (every family × region × severity),
//!   2. for each entry, builds the quad `(reference, corruption, q20, q10)`,
//!   3. writes a `_MANIFEST.json` describing every entry, and
//!   4. (optionally) writes the corrupted PNG + the two JPEG anchors to disk so
//!      a metric (e.g. zensim) can score the gate `score(corruption) < score(q20)`.
//!
//! Nothing large is committed to git: outputs land in a `--out` dir (gitignored
//! `corruption-out/` by default) and are reproducible on demand from
//! `(ref_id, seed, params)`.
//!
//! ## Usage
//!
//! ```text
//! # Score-ready quads + manifest for a single reference:
//! cargo run --example corruption_corpus --features driver -- \
//!     --ref ../gb82-sc/imac_g3_strip.png --ref-id gb82-sc/imac_g3_strip \
//!     --class screen --out /tmp/corruption-out
//!
//! # Manifest only (no image bytes written), e.g. for inspecting the catalog:
//! cargo run --example corruption_corpus --features driver -- \
//!     --ref ../gb82/dog-lossless.png --ref-id gb82/dog --class photo \
//!     --manifest-only
//! ```

use std::path::{Path, PathBuf};

use corruption_corpus::{ContentClass, ManifestEntry, driver, manifest_for_reference};

struct Args {
    references: Vec<(PathBuf, String, ContentClass)>,
    out: PathBuf,
    manifest_only: bool,
    base_seed: u64,
    /// If set, only emit entries whose family name matches (substring).
    family_filter: Option<String>,
}

fn parse_class(s: &str) -> ContentClass {
    match s {
        "photo" => ContentClass::Photo,
        "screen" => ContentClass::Screen,
        "line_art" | "lineart" => ContentClass::LineArt,
        "text" => ContentClass::Text,
        "gradient" => ContentClass::Gradient,
        other => {
            eprintln!("unknown content class '{other}', defaulting to photo");
            ContentClass::Photo
        }
    }
}

fn parse_args() -> Args {
    let mut references = Vec::new();
    let mut out = PathBuf::from("corruption-out");
    let mut manifest_only = false;
    let mut base_seed = 1u64;
    let mut family_filter = None;

    // A reference is described by a `--ref` followed (in any order) by its
    // `--ref-id` and `--class`. We accumulate the in-progress reference and
    // flush it whenever the next `--ref` appears or at end of args, so the
    // class isn't lost regardless of flag order.
    let mut pending_ref: Option<PathBuf> = None;
    let mut pending_id: Option<String> = None;
    let mut pending_class = ContentClass::Photo;

    let flush = |references: &mut Vec<(PathBuf, String, ContentClass)>,
                 r: &mut Option<PathBuf>,
                 id: &Option<String>,
                 class: ContentClass| {
        if let Some(path) = r.take() {
            let ref_id = id
                .clone()
                .unwrap_or_else(|| path.to_string_lossy().into_owned());
            references.push((path, ref_id, class));
        }
    };

    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        match a.as_str() {
            "--ref" => {
                // A new reference starts: flush the previous one first.
                flush(
                    &mut references,
                    &mut pending_ref,
                    &pending_id,
                    pending_class,
                );
                pending_id = None;
                pending_class = ContentClass::Photo;
                pending_ref = it.next().map(PathBuf::from);
            }
            "--ref-id" => pending_id = it.next(),
            "--class" => pending_class = parse_class(&it.next().unwrap_or_default()),
            "--out" => out = it.next().map(PathBuf::from).unwrap_or(out),
            "--manifest-only" => manifest_only = true,
            "--seed" => base_seed = it.next().and_then(|s| s.parse().ok()).unwrap_or(1),
            "--family" => family_filter = it.next(),
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            other => eprintln!("ignoring unknown arg: {other}"),
        }
    }
    // Flush the final in-progress reference.
    flush(
        &mut references,
        &mut pending_ref,
        &pending_id,
        pending_class,
    );

    if references.is_empty() {
        eprintln!("no --ref/--ref-id given; nothing to do. Pass --help for usage.");
    }

    Args {
        references,
        out,
        manifest_only,
        base_seed,
        family_filter,
    }
}

fn print_help() {
    eprintln!(
        "corruption_corpus — generate the structural-corruption corpus\n\
         \n\
         Required (repeatable in pairs):\n\
           --ref <path>        reference image (PNG/JPEG)\n\
           --ref-id <id>       stable identifier (e.g. gb82-sc/imac_g3_strip)\n\
           --class <c>         content class: photo|screen|line_art|text|gradient\n\
         \n\
         Optional:\n\
           --out <dir>         output dir (default: corruption-out/)\n\
           --manifest-only     write _MANIFEST.json only, no image bytes\n\
           --seed <u64>        base seed (default 1)\n\
           --family <name>     only emit entries whose family contains <name>\n"
    );
}

fn write_entry_images(
    out_dir: &Path,
    entry: &ManifestEntry,
    reference: &corruption_corpus::Rgb8,
) -> Result<(), Box<dyn std::error::Error>> {
    let quad = driver::build_quad(reference, &entry.ref_id, &entry.params, entry.seed)?;
    let slug = entry.params.slug();
    let stem = format!("{}__{}", entry.ref_id.replace('/', "_"), slug);
    std::fs::write(
        out_dir.join(format!("{stem}__corruption.png")),
        driver::encode_png(&quad.corruption)?,
    )?;
    std::fs::write(
        out_dir.join(format!("{stem}__q20.png")),
        driver::encode_png(&quad.q20_anchor)?,
    )?;
    std::fs::write(
        out_dir.join(format!("{stem}__q10.png")),
        driver::encode_png(&quad.q10_anchor)?,
    )?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args();
    if args.references.is_empty() {
        return Ok(());
    }

    std::fs::create_dir_all(&args.out)?;

    let mut all_entries: Vec<ManifestEntry> = Vec::new();
    let mut ref_count = 0usize;

    for (path, ref_id, class) in &args.references {
        let reference = match driver::load_reference(path) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("skip {ref_id}: failed to load {}: {e}", path.display());
                continue;
            }
        };
        ref_count += 1;

        let mut entries = manifest_for_reference(ref_id, *class, args.base_seed);
        if let Some(filter) = &args.family_filter {
            entries.retain(|e| e.params.family.slug().contains(filter.as_str()));
        }

        if !args.manifest_only {
            // Write the (gitignored) pristine reference once per ref.
            let ref_png = driver::encode_png(&reference)?;
            std::fs::write(
                args.out
                    .join(format!("{}__reference.png", ref_id.replace('/', "_"))),
                ref_png,
            )?;
            for entry in &entries {
                if let Err(e) = write_entry_images(&args.out, entry, &reference) {
                    eprintln!("entry {} failed: {e}", entry.params.slug());
                }
            }
        }

        println!(
            "{ref_id}: {} entries ({}x{})",
            entries.len(),
            reference.width(),
            reference.height()
        );
        all_entries.extend(entries);
    }

    let manifest_path = args.out.join("_MANIFEST.json");
    let json = serde_json::to_string_pretty(&all_entries)?;
    std::fs::write(&manifest_path, json)?;

    println!(
        "wrote {} manifest entries across {ref_count} references to {}",
        all_entries.len(),
        manifest_path.display()
    );
    if args.manifest_only {
        println!("(manifest-only: no image bytes written)");
    }
    Ok(())
}
