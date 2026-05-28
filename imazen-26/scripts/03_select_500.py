#!/usr/bin/env python3
"""Select a 500-image representative subset of imazen-26-synth by joint
clustering on (per-zen-codec bpp) + (zenanalyze content features).

Inputs (in /mnt/v/output/zensim/synth500/):
  bpp.csv       filename,out_px,bytes_*,bpp_jpeg,bpp_webp,bpp_jxl,bpp_avif
  features.tsv  image_path,...,feat_* (zenanalyze, native res)
Base table:     /home/lilith/work/codec-corpus/imazen-26-synth/_base_table.csv

Outputs:
  selection_500.csv  (intermediate, full columns for the manifest stage)
  report.txt         (clustering-emphasis + coverage report)
"""
import sys
import numpy as np
import pandas as pd
from sklearn.preprocessing import StandardScaler
from sklearn.decomposition import PCA
from sklearn.cluster import KMeans

RNG = 42
K = 500
OUT = "/mnt/v/output/zensim/synth500"
BASE = "/home/lilith/work/codec-corpus/imazen-26-synth/_base_table.csv"
CONTENT_CLASSES = 7  # coarse content groups
MAX_PER_SOURCE = 4

np.random.seed(RNG)

# ---- load + merge ----------------------------------------------------------
bpp = pd.read_csv(f"{OUT}/bpp.csv")
feat = pd.read_csv(f"{OUT}/features.tsv", sep="\t")
feat["filename"] = feat["image_path"].str.rsplit("/", n=1).str[-1]
# features.tsv carries placeholder source/content_class — drop so the base
# table's `source` and our computed `content_class` win without name clashes.
feat = feat.drop(columns=[c for c in ["source", "content_class"] if c in feat.columns])
base = pd.read_csv(BASE)[["filename", "source", "op", "param"]]

df = bpp.merge(feat, on="filename", how="inner").merge(base, on="filename", how="left")
print(f"merged rows: {len(df)}", file=sys.stderr)

# actual width/height from the feature extractor (native res)
df["aw"] = df["width"].astype(int)
df["ah"] = df["height"].astype(int)

# bpp codecs that are present (jxl dropped at encode time -> all-NaN)
BPP_COLS = [c for c in ["bpp_jpeg", "bpp_webp", "bpp_jxl", "bpp_avif"]
            if c in df.columns and df[c].notna().any()]
print(f"bpp codecs used: {BPP_COLS}", file=sys.stderr)

# ---- feature matrix --------------------------------------------------------
FCOLS = [c for c in df.columns if c.startswith("feat_")]
F = df[FCOLS].apply(pd.to_numeric, errors="coerce")
# drop zero-variance cols, fill any residual NaN with column median
F = F.loc[:, F.var(numeric_only=True).fillna(0) > 1e-12]
F = F.fillna(F.median())
print(f"feature cols after variance filter: {F.shape[1]}", file=sys.stderr)

Fz = StandardScaler().fit_transform(F.values)
# content signal: PCA -> 8 dims
n_pca = min(8, Fz.shape[1])
content_pca = PCA(n_components=n_pca, random_state=RNG).fit_transform(Fz)
print(f"content PCA dims: {n_pca} "
      f"(explained var {PCA(n_components=n_pca, random_state=RNG).fit(Fz).explained_variance_ratio_.sum():.3f})",
      file=sys.stderr)

# bpp signal: z-scored log(1+bpp) per codec
B = np.log1p(df[BPP_COLS].values)
Bz = StandardScaler().fit_transform(B)

# ---- three clusterings (for the report) ------------------------------------
def kmeans_labels(X, k):
    km = KMeans(n_clusters=k, n_init=10, random_state=RNG)
    return km.fit(X), km.labels_

report = []
report.append("# imazen-26-synth-500 selection report\n")
report.append(f"Total candidates: {len(df)}")
report.append(f"BPP codecs used: {BPP_COLS} (jxl dropped: zenjxl/jxl-encoder local build break)")
report.append(f"Feature cols (post variance filter): {F.shape[1]}; content PCA dims: {n_pca}")
report.append("")

# emphasis comparison at a small K to characterize what each axis splits on
KE = 8
_, lab_bpp = kmeans_labels(Bz, KE)
_, lab_content = kmeans_labels(content_pca, KE)
joint_pre = np.hstack([Bz, content_pca])
_, lab_joint = kmeans_labels(joint_pre, KE)

def describe(lab, name):
    report.append(f"## {name} clustering (K={KE}) — cluster medians")
    g = df.copy()
    g["_lab"] = lab
    cols = BPP_COLS + ["aw", "feat_edge_density", "feat_variance", "feat_distinct_color_bins"]
    cols = [c for c in cols if c in g.columns]
    agg = g.groupby("_lab")[cols].median()
    agg["n"] = g.groupby("_lab").size()
    report.append(agg.round(3).to_string())
    report.append("")

describe(lab_bpp, "BPP-only")
describe(lab_content, "Content-only")
describe(lab_joint, "Joint")

# ---- coarse content class (for coverage) -----------------------------------
_, content_class = kmeans_labels(content_pca, CONTENT_CLASSES)
df["content_class"] = content_class
cc_counts = pd.Series(content_class).value_counts().sort_index()
report.append(f"## Coarse content classes (K={CONTENT_CLASSES}) population")
report.append(cc_counts.to_string())
report.append("")

# ---- JOINT clustering K=500 -> centroid-nearest representative -------------
# joint vector = z(log1p bpp) ⊕ z(content PCA). bpp gets full weight,
# content gets full weight (both standardized so comparable scale).
JOINT = np.hstack([Bz, content_pca]).astype(np.float64)
km = KMeans(n_clusters=K, n_init=10, random_state=RNG)
clab = km.fit_predict(JOINT)
df["cluster_id"] = clab

# centroid-nearest member per cluster
centers = km.cluster_centers_
chosen = []
for cid in range(K):
    idx = np.where(clab == cid)[0]
    if len(idx) == 0:
        continue
    d = np.linalg.norm(JOINT[idx] - centers[cid], axis=1)
    chosen.append(idx[np.argmin(d)])
chosen = np.array(sorted(set(chosen)))
report.append(f"## Joint K=500: {len(chosen)} non-empty clusters -> initial picks {len(chosen)}")

sel = df.iloc[chosen].copy()

# ---- per-source cap (<=4 from one original) --------------------------------
# Greedy: keep centroid-nearest first; if a source exceeds the cap, drop the
# farthest-from-centroid picks for that source and backfill from other
# non-empty clusters' next-nearest unused members.
# list of (orig_index, dist-to-centroid, source, cluster)
recs = [(int(i), float(np.linalg.norm(JOINT[i] - centers[clab[i]])),
         str(df.iloc[i]["source"]), int(clab[i])) for i in chosen]
recs.sort(key=lambda r: r[1])  # nearest centroid first
used_clusters = set()
kept = []
src_count = {}
overflow_clusters = []
for oi, dist, src, cid in recs:
    if src_count.get(src, 0) >= MAX_PER_SOURCE:
        overflow_clusters.append((oi, dist, src, cid))
        continue
    kept.append(oi)
    used_clusters.add(cid)
    src_count[src] = src_count.get(src, 0) + 1

# Backfill to reach K from clusters that lost their rep to the cap:
# pick next-nearest member (any unused image) of those clusters under the cap.
need = K - len(kept)
if need > 0:
    kept_set = set(kept)
    # general backfill: for every cluster, list members by distance; take first
    # unused member whose source is under cap.
    by_cluster = {}
    for i in range(len(df)):
        by_cluster.setdefault(int(clab[i]), []).append(i)
    # order each cluster's members by distance to its centroid
    for cid in by_cluster:
        by_cluster[cid].sort(key=lambda i: np.linalg.norm(JOINT[i] - centers[cid]))
    # iterate clusters that currently have a kept rep removed OR any cluster,
    # round-robin nearest unused members under cap
    candidates = []
    for cid, members in by_cluster.items():
        for i in members:
            if i in kept_set:
                continue
            candidates.append((np.linalg.norm(JOINT[i] - centers[cid]), i))
    candidates.sort()
    for dist, i in candidates:
        if need <= 0:
            break
        src = str(df.iloc[i]["source"])
        if src_count.get(src, 0) >= MAX_PER_SOURCE:
            continue
        kept.append(i)
        kept_set.add(i)
        src_count[src] = src_count.get(src, 0) + 1
        need -= 1

kept = sorted(set(kept))
report.append(f"## After per-source cap (<= {MAX_PER_SOURCE}) + backfill: {len(kept)} images")
report.append(f"   dropped-for-cap before backfill: {len(overflow_clusters)}")

final = df.iloc[kept].copy()

# ---- content coverage check + rebalance ------------------------------------
cov = final["content_class"].value_counts().sort_index()
report.append("## Content-class coverage in the 500")
for c in range(CONTENT_CLASSES):
    report.append(f"   class {c}: pool={int(cc_counts.get(c,0))}  selected={int(cov.get(c,0))}")
missing = [c for c in range(CONTENT_CLASSES) if cov.get(c, 0) == 0 and cc_counts.get(c, 0) > 0]
if missing:
    report.append(f"   REBALANCE: classes with 0 reps but nonzero pool: {missing}")
    # swap in nearest-centroid member from each missing class, drop a pick from
    # the most-crowded class (keeping source cap).
    kept_set = set(kept)
    crowded = cov.idxmax()
    for mc in missing:
        # add nearest member of class mc
        cand = df[(df["content_class"] == mc)].index.tolist()
        cand = [i for i in cand if i not in kept_set]
        if not cand:
            continue
        # nearest to its joint-cluster centroid
        cand.sort(key=lambda i: np.linalg.norm(JOINT[i] - centers[int(clab[i])]))
        add_i = cand[0]
        # drop the farthest pick from the crowded class
        crowd_members = final[final["content_class"] == crowded].index.tolist()
        crowd_members.sort(key=lambda i: -np.linalg.norm(JOINT[i] - centers[int(clab[i])]))
        if crowd_members:
            drop_i = crowd_members[0]
            kept_set.discard(drop_i)
        kept_set.add(add_i)
    kept = sorted(kept_set)
    final = df.iloc[kept].copy()
    report.append(f"   after rebalance: {len(final)} images")

# ---- bpp-axis coverage sanity (span low->high per codec) -------------------
report.append("\n## BPP-axis coverage (selected 500 vs full pool)")
for c in BPP_COLS:
    fs = final[c].dropna()
    al = df[c].dropna()
    report.append(f"   {c}: selected[min={fs.min():.4f} p50={fs.median():.4f} max={fs.max():.4f}]  "
                  f"pool[min={al.min():.4f} p50={al.median():.4f} max={al.max():.4f}]")

# size span
report.append(f"\n## Dimension span (max-dim) in 500:")
final["maxdim"] = final[["aw", "ah"]].max(axis=1)
report.append(f"   maxdim min={final['maxdim'].min()} p50={int(final['maxdim'].median())} max={final['maxdim'].max()}")
report.append(f"   unique sources in 500: {final['source'].nunique()} (cap {MAX_PER_SOURCE}/source)")
report.append(f"   source-count distribution: {pd.Series([v for v in src_count.values()]).value_counts().sort_index().to_dict()}")

# ---- write ----------------------------------------------------------------
final = final.sort_values(["content_class", "cluster_id", "filename"]).reset_index(drop=True)
final.to_csv(f"{OUT}/selection_500.csv", index=False)
with open(f"{OUT}/report.txt", "w") as f:
    f.write("\n".join(str(x) for x in report) + "\n")

print(f"\nFINAL SELECTION: {len(final)} images", file=sys.stderr)
print("\n".join(str(x) for x in report))
