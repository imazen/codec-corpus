#!/usr/bin/env python3
"""Rename a selected synth subset to `NNNN-provenance-category-subcategory.png`.

Input:  <subset>/selection_manifest.csv (from 03_select_500.py) with at least
        new_id/filename, source, op, param, content_class.
Output: renames the PNGs in place and rewrites selection_manifest.csv with
        provenance/category/subcategory columns + old_id.

- provenance = source collection (imazen-26 subdir slug; 'root' for top-level)
- category   = zenanalyze content-cluster (content_class) mapped to a semantic
               name from feature centroids (see CATEGORY map / README)
- subcategory= derivation op: dsN (downscale longest-edge N), cropcN/croprN (crop NxN)
"""
import csv, os, sys

SUBSET = sys.argv[1] if len(sys.argv) > 1 else "../imazen-26-synth-500"

# content_class cluster -> semantic name (derived from feat centroids; see README)
CATEGORY = {'0':'photo','1':'illust','2':'texture','3':'detail','4':'flat','5':'graphic','6':'lineart'}
PROV = {'generated':'gen','generated-graphics':'gengfx','unsplash-people':'unsplashppl',
        'unsplash-renders':'unsplashrnd','unsplash-textures':'unsplashtex','unsplash':'unsplash',
        'skitter':'skitter','openclipart':'openclipart','flicker-pub-domain':'flickr',
        'library-of-congress-public-domain':'loc','maybe':'maybe'}

def provenance(src):
    head = src.split('/')[0] if '/' in src else 'root'
    return PROV.get(head, head.replace('-', '') or 'root')

def subcategory(op, param):
    if op == 'downscale':
        return f"ds{param}"
    size = str(param).split('@')[0]
    return ("cropc" if op == 'crop_c' else "cropr" if op == 'crop_r' else op) + size

def keycol(r):
    return r.get('new_id') or r.get('filename')

rows = list(csv.DictReader(open(os.path.join(SUBSET, "selection_manifest.csv"))))
rows.sort(key=keycol)
plan = []
for i, r in enumerate(rows, 1):
    p = provenance(r['source']); c = CATEGORY.get(r['content_class'], 'cls' + r['content_class'])
    s = subcategory(r['op'], r['param'])
    plan.append((keycol(r), f"{i:04d}-{p}-{c}-{s}.png", r, p, c, s))
assert len({x[1] for x in plan}) == len(plan), "filename collision"

for old, new, *_ in plan:
    src = os.path.join(SUBSET, old)
    if os.path.exists(src): os.rename(src, os.path.join(SUBSET, new + ".tmp"))
for _, new, *_ in plan:
    t = os.path.join(SUBSET, new + ".tmp")
    if os.path.exists(t): os.rename(t, os.path.join(SUBSET, new))

fields = ['filename','old_id','provenance','category','subcategory','src_filename','source',
          'op','param','out_w','out_h','bpp_jpeg','bpp_webp','bpp_jxl','bpp_avif','content_class','cluster_id']
w = csv.DictWriter(open(os.path.join(SUBSET, "selection_manifest.csv"), "w", newline=""), fieldnames=fields)
w.writeheader()
for old, new, r, p, c, s in plan:
    w.writerow({'filename':new,'old_id':old,'provenance':p,'category':c,'subcategory':s,
        'src_filename':r.get('src_filename',old),'source':r['source'],'op':r['op'],'param':r['param'],
        'out_w':r.get('out_w',''),'out_h':r.get('out_h',''),'bpp_jpeg':r.get('bpp_jpeg',''),
        'bpp_webp':r.get('bpp_webp',''),'bpp_jxl':r.get('bpp_jxl',''),'bpp_avif':r.get('bpp_avif',''),
        'content_class':r['content_class'],'cluster_id':r.get('cluster_id','')})
print(f"renamed {len(plan)} files in {SUBSET}")
