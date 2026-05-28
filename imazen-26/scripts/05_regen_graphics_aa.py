#!/usr/bin/env python3
"""Regenerate the curated synthetic graphics with anti-aliasing.

Background
----------
The PNGs under ``lilith/generated-graphics/{charts,grids,line-patterns,lines,
polygons}/`` are a hand-curated subset of the output of zensim's
``synth_nonphoto.py`` (the V_06 "rebalance" non-photo generator). The chart
category is matplotlib (already AA); the line-art category is raw PIL
``ImageDraw`` — which has **no anti-aliasing**, so every polygon / line / grid
has jagged edges.

This script reproduces each curated image *byte-exactly* from the seed embedded
in its filename, then re-renders it with anti-aliasing:

  * line-art : supersampled SS_LINE× via ``ScaledDraw`` (all geometry + stroke
               widths scaled at draw time, RNG kept on the logical 1024 grid),
               then Lanczos-downsampled to 1024².
  * charts   : matplotlib at SS_CHART× DPI (``.mpl.png``) **and** quickchart.io /
               Chart.js from the same seeded data (``.qc.png``), side-by-side.
  * heatmaps : matplotlib only (Chart.js has no native heatmap).

Faithful reproduction recipe (the subtle bit)
---------------------------------------------
The driver creates ONE ``sub_rng = random.Random(sub_seed)`` per source and
reuses it across both sizes ``[512, 1024]`` in order. So the 1024² image is
drawn from the RNG state *after* the 512² render already consumed values. To
reproduce a ``*_1024sq.png`` you must render 512 first (and discard it) to
advance the RNG, then render 1024. This is validated byte-exact in --validate.

Sorting rule (folder = f(type, kind))
-------------------------------------
    chart/heatmap -> grids        chart/{line,bar,scatter,area,stack} -> charts
    line/grid-pattern -> grids    line/polygons -> polygons
    line/tiling -> line-patterns  line/{concentric,voronoi-ish,stars-burst} -> lines

Naming
------
    gen-<chart|line>-<kindslug>__<origidx>_s<seed>_1024sq[.mpl|.qc].png
"""
from __future__ import annotations

import argparse
import csv
import json
import re
import random
import sys
import urllib.request
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt

# --------------------------------------------------------------------------
# Constants copied verbatim from synth_nonphoto.py so the RNG sequence and the
# drawing match the original byte-for-byte.
# --------------------------------------------------------------------------
LINE_KINDS = ["polygons", "tiling", "voronoi-ish", "concentric", "grid-pattern", "stars-burst"]
CHART_KINDS = ["line", "bar", "scatter", "heatmap", "area", "stack"]
CHART_STYLES = ["seaborn-v0_8-whitegrid", "ggplot", "default", "seaborn-v0_8-darkgrid", "Solarize_Light2"]
LINE_BG = [(255, 255, 255), (250, 250, 248), (15, 18, 22), (240, 235, 225)]

WORDS_EN_COMMON = (
    "the of and to a in is it that for as with on this be by are have not but at "
    "from or had has was were they you we he she them their which when what about "
    "into other after before than over under between through during without these "
    "those some many much such where here there because while until against above "
    "below within across each every another however therefore moreover furthermore "
    "indeed example perhaps maybe always never sometimes often rarely usually "
    "system value memory buffer index offset length kernel thread process queue "
    "module function pointer struct vector matrix tensor packet stream pipeline "
    "decoder encoder filter quality lossy lossless bitrate entropy compression "
    "frequency wavelet transform residual coefficient quantization prediction "
    "reference codec format version commit branch release benchmark profile data"
).split()

# kind -> destination folder (the user's curated taxonomy)
def folder_for(typ: str, kind: str) -> str:
    if typ == "chart":
        return "grids" if kind == "heatmap" else "charts"
    return {
        "polygons": "polygons",
        "tiling": "line-patterns",
        "grid-pattern": "grids",
        "concentric": "lines",
        "voronoi-ish": "lines",
        "stars-burst": "lines",
    }[kind]

KIND_SLUG = {"grid-pattern": "grid", "voronoi-ish": "voronoi", "stars-burst": "starburst"}
def kind_slug(k: str) -> str:
    return KIND_SLUG.get(k, k)


# --------------------------------------------------------------------------
# ScaledDraw: a transparent ImageDraw wrapper that multiplies every coordinate
# and stroke width by `s`, so we can render the *unchanged* line-art logic onto
# an s× canvas and Lanczos-downsample for anti-aliasing. RNG is untouched.
# --------------------------------------------------------------------------
class ScaledDraw:
    def __init__(self, img: Image.Image, s: int):
        self._d = ImageDraw.Draw(img)
        self._s = s

    def _xy(self, xy):
        s = self._s
        if len(xy) > 0 and isinstance(xy[0], (tuple, list)):
            return [(p[0] * s, p[1] * s) for p in xy]
        return [c * s for c in xy]

    def _kw(self, kw):
        if kw.get("width"):
            kw = dict(kw)
            kw["width"] = max(1, int(round(kw["width"] * self._s)))
        return kw

    def line(self, xy, **kw):      self._d.line(self._xy(xy), **self._kw(kw))
    def rectangle(self, xy, **kw): self._d.rectangle(self._xy(xy), **self._kw(kw))
    def ellipse(self, xy, **kw):   self._d.ellipse(self._xy(xy), **self._kw(kw))
    def polygon(self, xy, **kw):   self._d.polygon(self._xy(xy), **self._kw(kw))


# --------------------------------------------------------------------------
# gen_lineart — line-for-line copy of synth_nonphoto.gen_lineart, with `ss`
# supersampling threaded through. At ss=1 it is byte-identical to the original.
# --------------------------------------------------------------------------
def gen_lineart(rng: random.Random, size: int, ss: int = 1) -> Image.Image:
    bg = rng.choice(LINE_BG)
    fg = (255, 255, 255) if sum(bg) < 380 else (10, 10, 10)
    img = Image.new("RGB", (size * ss, size * ss), bg)
    d = ScaledDraw(img, ss)

    kind = rng.choice(LINE_KINDS)

    if kind == "polygons":
        for _ in range(rng.randint(8, 40)):
            cx, cy = rng.randint(0, size), rng.randint(0, size)
            r = rng.randint(20, size // 3)
            sides = rng.randint(3, 9)
            phase = rng.uniform(0, 6.28)
            pts = []
            for k in range(sides):
                a = phase + 2 * 3.14159 * k / sides
                pts.append((cx + r * np.cos(a), cy + r * np.sin(a)))
            color = (rng.randint(0, 255), rng.randint(0, 255), rng.randint(0, 255)) if rng.random() < 0.4 else fg
            if rng.random() < 0.5:
                d.polygon(pts, outline=color, width=rng.randint(1, 4))
            else:
                d.polygon(pts, fill=color, outline=fg)
    elif kind == "tiling":
        cell = rng.randint(16, 64)
        for x in range(0, size, cell):
            for y in range(0, size, cell):
                shape = rng.choice(["box", "diag1", "diag2", "circle", "tri"])
                color = (rng.randint(0, 255), rng.randint(0, 255), rng.randint(0, 255)) if rng.random() < 0.3 else fg
                if shape == "box":
                    d.rectangle([x, y, x + cell - 2, y + cell - 2], outline=color, width=1)
                elif shape == "diag1":
                    d.line([x, y, x + cell, y + cell], fill=color, width=2)
                elif shape == "diag2":
                    d.line([x + cell, y, x, y + cell], fill=color, width=2)
                elif shape == "circle":
                    d.ellipse([x + 2, y + 2, x + cell - 2, y + cell - 2], outline=color, width=1)
                else:
                    d.polygon([(x + cell // 2, y), (x, y + cell), (x + cell, y + cell)], outline=color, width=1)
    elif kind == "voronoi-ish":
        pts = [(rng.randint(0, size), rng.randint(0, size)) for _ in range(rng.randint(12, 50))]
        for i, p in enumerate(pts):
            for q in pts[i + 1: i + 4]:
                d.line([p, q], fill=fg, width=1)
            d.ellipse([p[0] - 3, p[1] - 3, p[0] + 3, p[1] + 3], fill=fg)
    elif kind == "concentric":
        cx, cy = size // 2, size // 2
        for r in range(8, size // 2, rng.randint(6, 22)):
            color = (rng.randint(0, 255), rng.randint(0, 255), rng.randint(0, 255)) if rng.random() < 0.3 else fg
            shape = rng.choice(["ellipse", "rect", "polygon"])
            if shape == "ellipse":
                d.ellipse([cx - r, cy - r, cx + r, cy + r], outline=color, width=rng.randint(1, 3))
            elif shape == "rect":
                d.rectangle([cx - r, cy - r, cx + r, cy + r], outline=color, width=rng.randint(1, 3))
            else:
                sides = 6
                phase = rng.uniform(0, 6.28)
                pts = [(cx + r * np.cos(phase + 2 * 3.14159 * k / sides),
                        cy + r * np.sin(phase + 2 * 3.14159 * k / sides))
                       for k in range(sides)]
                d.polygon(pts, outline=color, width=rng.randint(1, 3))
    elif kind == "grid-pattern":
        spacing = rng.randint(8, 32)
        for i in range(0, size, spacing):
            d.line([(i, 0), (i, size)], fill=fg, width=1)
            d.line([(0, i), (size, i)], fill=fg, width=1)
        for _ in range(rng.randint(20, 80)):
            cx = rng.randint(0, size // spacing) * spacing
            cy = rng.randint(0, size // spacing) * spacing
            color = (rng.randint(0, 255), rng.randint(0, 255), rng.randint(0, 255))
            d.rectangle([cx, cy, cx + spacing, cy + spacing], fill=color)
    else:  # stars-burst
        cx, cy = rng.randint(size // 4, 3 * size // 4), rng.randint(size // 4, 3 * size // 4)
        for k in range(rng.randint(40, 200)):
            ang = 2 * 3.14159 * k / rng.randint(40, 200)
            r = rng.randint(20, size // 2)
            x2 = cx + r * np.cos(ang)
            y2 = cy + r * np.sin(ang)
            d.line([(cx, cy), (x2, y2)], fill=fg, width=1)

    if ss > 1:
        img = img.resize((size, size), Image.Resampling.LANCZOS)
    return img, kind


# --------------------------------------------------------------------------
# gen_chart — copy of synth_nonphoto.gen_chart, with `ss` DPI supersampling and
# a captured `spec` (the exact seeded data) returned for the quickchart path.
# At ss=1 the rendered image is byte-identical to the original.
# --------------------------------------------------------------------------
def gen_chart(rng: random.Random, size: int, ss: int = 1):
    dpi = 100 * ss
    fig_in = size / 100.0
    fig, ax = plt.subplots(figsize=(fig_in, fig_in), dpi=dpi)

    chart_kind = rng.choice(CHART_KINDS)
    n = rng.randint(8, 40)
    style = rng.choice(CHART_STYLES)
    plt.style.use(style)
    fig, ax = plt.subplots(figsize=(fig_in, fig_in), dpi=dpi)

    spec = {"kind": chart_kind, "n": n, "style": style}

    if chart_kind == "line":
        series = []
        for s in range(rng.randint(2, 5)):
            xs = np.arange(n)
            steps = np.array([rng.gauss(0, 1) for _ in range(n)])
            ys = np.cumsum(steps)
            marker = rng.choice(["o", "s", "^", "x", ".", None])
            ax.plot(xs, ys, marker=marker, label=f"series {s + 1}")
            series.append({"label": f"series {s + 1}", "ys": ys.tolist(), "marker": marker})
        ax.legend(loc="best")
        spec["xs"] = list(range(n)); spec["series"] = series
    elif chart_kind == "bar":
        xs = np.arange(n)
        ys = np.array([rng.uniform(0.1, 1.0) for _ in range(n)])
        ax.bar(xs, ys, color=plt.cm.viridis(np.linspace(0, 1, n)))
        spec["xs"] = list(range(n)); spec["ys"] = ys.tolist()
    elif chart_kind == "scatter":
        xs = np.array([rng.gauss(0, 1) for _ in range(n * 5)])
        ys = np.array([rng.gauss(0, 1) for _ in range(n * 5)])
        ax.scatter(xs, ys, c=np.arange(n * 5), cmap="plasma", alpha=0.7)
        spec["xs"] = xs.tolist(); spec["ys"] = ys.tolist()
    elif chart_kind == "heatmap":
        m = np.array([[rng.uniform(0, 1) for _ in range(n)] for _ in range(n)])
        ax.imshow(m, cmap=rng.choice(["viridis", "magma", "plasma", "inferno", "coolwarm"]), aspect="auto")
        spec["matrix"] = m.tolist()
    elif chart_kind == "area":
        xs = np.arange(n)
        steps = np.array([rng.gauss(0, 1) for _ in range(n)])
        ys = np.cumsum(steps)
        ax.fill_between(xs, ys, alpha=0.5)
        ax.plot(xs, ys)
        spec["xs"] = list(range(n)); spec["ys"] = ys.tolist()
    elif chart_kind == "stack":
        xs = np.arange(n)
        ys_list = [np.array([rng.uniform(0, 1) for _ in range(n)]) for _ in range(rng.randint(3, 6))]
        ax.stackplot(xs, *ys_list, labels=[f"s{i + 1}" for i in range(len(ys_list))])
        ax.legend(loc="best")
        spec["xs"] = list(range(n)); spec["stack"] = [y.tolist() for y in ys_list]

    title = " ".join(rng.choice(WORDS_EN_COMMON).capitalize() for _ in range(rng.randint(2, 5)))
    ax.set_title(title)
    xlabel = rng.choice(WORDS_EN_COMMON).capitalize()
    ylabel = rng.choice(WORDS_EN_COMMON).capitalize()
    ax.set_xlabel(xlabel)
    ax.set_ylabel(ylabel)
    spec["title"] = title; spec["xlabel"] = xlabel; spec["ylabel"] = ylabel

    fig.tight_layout()
    fig.canvas.draw()
    arr = np.frombuffer(fig.canvas.tostring_argb(), dtype=np.uint8)
    arr = arr.reshape(fig.canvas.get_width_height()[::-1] + (4,))
    rgb = arr[..., 1:4]
    plt.close(fig)
    img = Image.fromarray(rgb).resize((size, size), Image.Resampling.LANCZOS)
    return img, chart_kind, spec


# --------------------------------------------------------------------------
# quickchart.io — render the same seeded chart data via Chart.js (v2 default).
# --------------------------------------------------------------------------
def quickchart_config(spec: dict) -> dict | None:
    k = spec["kind"]
    title = {"display": True, "text": spec.get("title", "")}
    if k == "line":
        datasets = [{"label": s["label"], "data": s["ys"], "fill": False, "borderWidth": 2,
                     "pointRadius": 2} for s in spec["series"]]
        return {"type": "line", "data": {"labels": spec["xs"], "datasets": datasets},
                "options": {"title": title}}
    if k == "bar":
        return {"type": "bar", "data": {"labels": spec["xs"],
                "datasets": [{"label": spec.get("title", "data"), "data": spec["ys"]}]},
                "options": {"title": title, "legend": {"display": False}}}
    if k == "scatter":
        pts = [{"x": x, "y": y} for x, y in zip(spec["xs"], spec["ys"])]
        return {"type": "scatter", "data": {"datasets": [{"label": "points", "data": pts,
                "pointRadius": 3}]}, "options": {"title": title}}
    if k == "area":
        return {"type": "line", "data": {"labels": spec["xs"],
                "datasets": [{"data": spec["ys"], "fill": True, "borderWidth": 2, "pointRadius": 0}]},
                "options": {"title": title, "legend": {"display": False}}}
    if k == "stack":
        datasets = [{"label": f"s{i + 1}", "data": ys, "fill": True, "borderWidth": 1, "pointRadius": 0}
                    for i, ys in enumerate(spec["stack"])]
        return {"type": "line", "data": {"labels": spec["xs"], "datasets": datasets},
                "options": {"title": title,
                            "scales": {"yAxes": [{"stacked": True}], "xAxes": [{"stacked": True}]}}}
    return None  # heatmap unsupported


def render_quickchart(spec: dict, size: int, endpoint: str, timeout: int = 30) -> Image.Image | None:
    cfg = quickchart_config(spec)
    if cfg is None:
        return None
    body = json.dumps({"width": size, "height": size, "devicePixelRatio": 2,
                       "format": "png", "backgroundColor": "white", "chart": cfg}).encode()
    req = urllib.request.Request(endpoint, data=body, headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(req, timeout=timeout) as r:
        if r.status != 200:
            raise RuntimeError(f"quickchart HTTP {r.status}")
        from io import BytesIO
        img = Image.open(BytesIO(r.read())).convert("RGB")
    if img.size != (size, size):
        img = img.resize((size, size), Image.Resampling.LANCZOS)
    return img


# --------------------------------------------------------------------------
NAME_RE = re.compile(r"gen-(chart|line)__(\d+)_s([0-9a-f]{8})_(\d+)sq\.png")


def scan_reference(ref_dir: Path):
    """Yield (folder, type, idx, seed_hex, seed_int, path) for every curated file."""
    for folder in sorted(p.name for p in ref_dir.iterdir() if p.is_dir()):
        for f in sorted((ref_dir / folder).iterdir()):
            m = NAME_RE.match(f.name)
            if not m:
                continue
            typ, idx, sh, sz = m.groups()
            if sz != "1024":
                continue
            yield folder, typ, idx, sh, int(sh, 16), f


def main() -> int:
    ap = argparse.ArgumentParser()
    here = Path(__file__).resolve().parent
    root = here.parent  # imazen-26/
    ap.add_argument("--ref", type=Path, default=root / "lilith" / "generated-graphics",
                    help="curated reference tree to mirror (seeds + organization)")
    ap.add_argument("--out", type=Path, default=root / "lilith" / "generated-graphics-aa",
                    help="output tree (default: sibling -aa dir, non-destructive)")
    ap.add_argument("--size", type=int, default=1024)
    ap.add_argument("--ss-line", type=int, default=4, help="line-art supersample factor")
    ap.add_argument("--ss-chart", type=int, default=2, help="chart matplotlib DPI supersample")
    ap.add_argument("--quickchart-endpoint", default="https://quickchart.io/chart")
    ap.add_argument("--no-quickchart", action="store_true", help="skip the .qc.png variant")
    ap.add_argument("--include-charts", action="store_true",
                    help="also render the seeded matplotlib charts (NOT byte-exact; "
                         "charts are normally owned by 06_quickchart_gallery.py)")
    ap.add_argument("--validate", action="store_true",
                    help="render each at ss=1 and assert byte-exact vs the reference PNG, then exit")
    args = ap.parse_args()

    items = list(scan_reference(args.ref))
    if not items:
        print(f"no gen-*_1024sq.png found under {args.ref}", file=sys.stderr)
        return 1
    # Charts are produced by 06_quickchart_gallery.py (interesting real data,
    # full type catalog, themed). The seeded matplotlib charts here cannot be
    # reproduced byte-exact anyway (matplotlib global-style leakage across the
    # original driver's chart sequence). Line-art IS byte-exact, so 05 owns it.
    if not args.include_charts:
        items = [it for it in items if it[1] == "line"]
    print(f"{len(items)} curated 1024sq references under {args.ref} "
          f"({'incl' if args.include_charts else 'line-art only; charts -> 06'})")

    # ---- validation pass: prove logic fidelity (ss=1 must be byte-exact) ----
    if args.validate:
        bad = 0
        for folder, typ, idx, sh, seed, path in items:
            rng = random.Random(seed)
            if typ == "line":
                gen_lineart(rng, 512, ss=1)
                img, kind = gen_lineart(rng, args.size, ss=1)
            else:
                gen_chart(rng, 512, ss=1)
                img, kind, _ = gen_chart(rng, args.size, ss=1)
            a = np.asarray(img.convert("RGB"), dtype=np.int16)
            b = np.asarray(Image.open(path).convert("RGB"), dtype=np.int16)
            md = int(np.abs(a - b).max()) if a.shape == b.shape else 999
            want = folder_for(typ, kind)
            ok_pix = (md == 0)
            ok_sort = (want == folder)
            if not (ok_pix and ok_sort):
                bad += 1
                print(f"  MISMATCH {path.name}: maxdiff={md} kind={kind} "
                      f"sort={want} (in {folder})")
        if bad:
            print(f"VALIDATE FAILED: {bad}/{len(items)} mismatched")
            return 1
        print(f"VALIDATE OK: all {len(items)} byte-exact and sort-consistent")
        return 0

    # ---- generation pass: anti-aliased, sorted, renamed ----
    args.out.mkdir(parents=True, exist_ok=True)
    manifest = []
    qc_fail = []
    for folder, typ, idx, sh, seed, path in items:
        rng = random.Random(seed)
        if typ == "line":
            gen_lineart(rng, 512, ss=1)  # advance RNG exactly like the driver
            img, kind = gen_lineart(rng, args.size, ss=args.ss_line)
            spec = None
        else:
            gen_chart(rng, 512, ss=1)
            img, kind, spec = gen_chart(rng, args.size, ss=args.ss_chart)

        dest_folder = folder_for(typ, kind)
        assert dest_folder == folder, f"{path.name}: sort rule {dest_folder} != curated {folder}"
        (args.out / dest_folder).mkdir(parents=True, exist_ok=True)
        base = f"gen-{typ}-{kind_slug(kind)}__{idx}_s{sh}_{args.size}sq"

        is_dual = (typ == "chart" and kind != "heatmap" and not args.no_quickchart)
        mpl_name = f"{base}.mpl.png" if is_dual else f"{base}.png"
        img.save(args.out / dest_folder / mpl_name, optimize=True, compress_level=6)
        engine = "matplotlib" if typ == "chart" else "pil-supersampled"
        manifest.append((dest_folder, mpl_name, typ, kind, idx, sh, engine, path.name))

        if is_dual:
            qc_name = f"{base}.qc.png"
            try:
                qc = render_quickchart(spec, args.size, args.quickchart_endpoint)
                if qc is not None:
                    qc.save(args.out / dest_folder / qc_name, optimize=True, compress_level=6)
                    manifest.append((dest_folder, qc_name, typ, kind, idx, sh, "quickchart.io", path.name))
            except Exception as e:
                qc_fail.append((path.name, str(e)))
                print(f"  quickchart FAILED {base}: {e}", file=sys.stderr)

    # ---- manifest ----
    mpath = args.out / "_regen_manifest.csv"
    with mpath.open("w", newline="") as f:
        w = csv.writer(f)
        w.writerow(["folder", "filename", "type", "kind", "orig_idx", "seed", "engine", "orig_filename"])
        w.writerows(manifest)

    from collections import Counter
    by_folder = Counter(m[0] for m in manifest)
    print(f"wrote {len(manifest)} images to {args.out}")
    for k, v in sorted(by_folder.items()):
        print(f"  {k}: {v}")
    if qc_fail:
        print(f"quickchart failures: {len(qc_fail)} (matplotlib variants still written)")
    print(f"manifest: {mpath}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
