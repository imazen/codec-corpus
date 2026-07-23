#!/usr/bin/env python3
"""Fetch high-resolution CC0 (public-domain) color artwork scans.

Sources, both keyless and CC0:
  - The Metropolitan Museum of Art Open Access API (isPublicDomain flag,
    primaryImage = native full-resolution JPEG)
  - The Art Institute of Chicago API (is_public_domain filter, IIIF image server)

Diversity comes from querying several genres and taking a few PD works each.
Everything here is CC0 / public-domain-dedicated and freely redistributable.
"""
import hashlib, json, os, re, shutil, time, urllib.parse, urllib.request

ROOT = os.path.dirname(os.path.abspath(__file__))
IMG = os.path.join(ROOT, "images")
BUILD_DATE = "2026-06-07"
UA = "Mozilla/5.0 (X11; Linux x86_64) corpus-fetch/1.0"
PER_QUERY = 3
AIC_WIDTH = 4000  # IIIF target width (high-res but bounded)

def get_json(url):
    req = urllib.request.Request(url, headers={"User-Agent": UA})
    with urllib.request.urlopen(req, timeout=60) as r:
        return json.load(r)

def download(url, dest):
    req = urllib.request.Request(url, headers={"User-Agent": UA})
    with urllib.request.urlopen(req, timeout=300) as r, open(dest, "wb") as f:
        shutil.copyfileobj(r, f)

def slug(s):
    return (re.sub(r"[^a-z0-9]+", "-", (s or "untitled").lower()).strip("-") or "untitled")[:48]

records = []

# ---------- Metropolitan Museum of Art ----------
met_dir = os.path.join(IMG, "met"); os.makedirs(met_dir, exist_ok=True)
met_queries = ["landscape", "portrait", "still life", "ukiyo-e",
               "flowers", "marine", "interior", "manuscript"]
met_seen = set()
for q in met_queries:
    try:
        sr = get_json("https://collectionapi.metmuseum.org/public/collection/v1/search?hasImages=true&q="
                      + urllib.parse.quote(q))
    except Exception as e:
        print("met search fail", q, e); continue
    got = 0
    for oid in (sr.get("objectIDs") or [])[:80]:
        if got >= PER_QUERY:
            break
        if oid in met_seen:
            continue
        try:
            o = get_json(f"https://collectionapi.metmuseum.org/public/collection/v1/objects/{oid}")
        except Exception:
            continue
        if not o.get("isPublicDomain") or not o.get("primaryImage"):
            continue
        met_seen.add(oid)
        dest = os.path.join(met_dir, f"{oid}_{slug(o.get('title'))}.jpg")
        try:
            download(o["primaryImage"], dest)
        except Exception as e:
            print("met dl fail", oid, e); continue
        got += 1
        records.append({"source": "Met Open Access", "license": "CC0",
                        "object_id": oid, "title": o.get("title"),
                        "artist": o.get("artistDisplayName"), "date": o.get("objectDate"),
                        "medium": o.get("medium"), "query": q,
                        "image_url": o["primaryImage"],
                        "file": os.path.relpath(dest, ROOT), "bytes": os.path.getsize(dest)})
        print(f"met  {oid}  {o.get('title')!r}  {os.path.getsize(dest)//1024} KB")
        time.sleep(0.25)

# ---------- Art Institute of Chicago ----------
aic_dir = os.path.join(IMG, "aic"); os.makedirs(aic_dir, exist_ok=True)
aic_queries = ["landscape", "portrait", "still life", "woodblock print", "abstract"]
aic_seen = set()
for q in aic_queries:
    url = ("https://api.artic.edu/api/v1/artworks/search?q=" + urllib.parse.quote(q)
           + "&query[term][is_public_domain]=true"
           + "&fields=id,title,artist_title,image_id,date_display,medium_display&limit=8")
    try:
        sr = get_json(url)
    except Exception as e:
        print("aic search fail", q, e); continue
    got = 0
    for d in sr.get("data", []):
        if got >= PER_QUERY:
            break
        iid = d.get("image_id")
        if not iid or d["id"] in aic_seen:
            continue
        aic_seen.add(d["id"])
        dest = os.path.join(aic_dir, f"{d['id']}_{slug(d.get('title'))}.jpg")
        iiif = f"https://www.artic.edu/iiif/2/{iid}/full/{AIC_WIDTH},/0/default.jpg"
        try:
            download(iiif, dest)
        except Exception:
            iiif = f"https://www.artic.edu/iiif/2/{iid}/full/full/0/default.jpg"
            try:
                download(iiif, dest)
            except Exception as e:
                print("aic dl fail", d["id"], e); continue
        got += 1
        records.append({"source": "Art Institute of Chicago", "license": "CC0",
                        "object_id": d["id"], "title": d.get("title"),
                        "artist": d.get("artist_title"), "date": d.get("date_display"),
                        "medium": d.get("medium_display"), "query": q,
                        "image_url": iiif,
                        "file": os.path.relpath(dest, ROOT), "bytes": os.path.getsize(dest)})
        print(f"aic  {d['id']}  {d.get('title')!r}  {os.path.getsize(dest)//1024} KB")
        time.sleep(0.25)

for r in records:
    h = hashlib.sha256()
    with open(os.path.join(ROOT, r["file"]), "rb") as f:
        for c in iter(lambda: f.read(1 << 20), b""):
            h.update(c)
    r["sha256"] = h.hexdigest()

manifest = {
    "corpus": "art-cc0",
    "build_date": BUILD_DATE,
    "description": "High-resolution CC0 (public-domain) color artwork scans from the Met Open Access and Art Institute of Chicago open APIs.",
    "license_note": "All items flagged public-domain / CC0 by the source institution (Met isPublicDomain=true; AIC is_public_domain=true). CC0 = no rights reserved, freely redistributable.",
    "content_profile": {
        "num_images": len(records),
        "total_bytes": sum(r["bytes"] for r in records),
        "met_count": sum(1 for r in records if r["source"].startswith("Met")),
        "aic_count": sum(1 for r in records if r["source"].startswith("Art")),
        "note": "Continuous-tone color: paintings/prints with fine brushwork detail, smooth gradients, and large dimensions. Met = native full-res; AIC = IIIF up to %d px wide." % AIC_WIDTH,
    },
    "images": records,
}
with open(os.path.join(ROOT, "MANIFEST.json"), "w") as f:
    json.dump(manifest, f, indent=2)
print(f"\nTOTAL: {len(records)} CC0 artworks "
      f"({manifest['content_profile']['met_count']} Met + {manifest['content_profile']['aic_count']} AIC), "
      f"{manifest['content_profile']['total_bytes']//(1024*1024)} MB")
