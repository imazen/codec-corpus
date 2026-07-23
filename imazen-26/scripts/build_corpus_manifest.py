#!/usr/bin/env python3
"""Regenerate the unified CORPUS-MANIFEST.tsv from the per-folder MANIFEST.tsv files.

The per-folder `<NNNN-folder>/MANIFEST.tsv` are the source of truth (one row per
image, with provenance + license). This aggregates their common columns into the
root CORPUS-MANIFEST.tsv. Run after editing any per-folder manifest.
"""
import os, re, csv

BASE = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
COMMON = ["path", "folder", "category", "number", "descriptor", "location", "width",
          "height", "format", "bytes", "original_filename", "source", "license"]

def main():
    folders = sorted(d for d in os.listdir(BASE)
                     if re.match(r"\d{4}-", d) and os.path.isdir(os.path.join(BASE, d)))
    rows = []
    for f in folders:
        mp = os.path.join(BASE, f, "MANIFEST.tsv")
        if not os.path.exists(mp):
            print(f"WARN: no MANIFEST.tsv in {f}"); continue
        for r in csv.DictReader(open(mp), delimiter="\t"):
            rows.append({"path": f"{f}/{r['filename']}", "folder": f,
                         **{k: r.get(k, "") for k in COMMON if k not in ("path", "folder")}})
    rows.sort(key=lambda x: (x["number"] or "0", x["path"]))
    out = os.path.join(BASE, "CORPUS-MANIFEST.tsv")
    with open(out, "w", newline="") as fh:
        wr = csv.DictWriter(fh, fieldnames=COMMON, delimiter="\t")
        wr.writeheader(); wr.writerows(rows)
    print(f"wrote {out}: {len(rows)} rows from {len(folders)} folders")

if __name__ == "__main__":
    main()
