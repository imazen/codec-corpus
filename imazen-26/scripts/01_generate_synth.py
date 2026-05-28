import os, subprocess, csv, random, glob, sys
random.seed(26)
SRC="/home/lilith/work/codec-corpus/imazen-26"
OUT="/home/lilith/work/codec-corpus/imazen-26-synth"
os.makedirs(OUT, exist_ok=True)
DOWN=[2048,1536,1024,768,512,384,256,192,128]          # longest-edge targets, downscale-only
CROPS=[("c",512),("c",256),("r",512)]                   # native-res crops: center512, center256, random512
exts=('.png','.jpg','.jpeg','.tif','.tiff','.heic','.JPG')
srcs=sorted(p for p in glob.glob(SRC+"/**/*",recursive=True) if os.path.isfile(p) and p.endswith(exts))
def dims(p):
    try:
        w=int(subprocess.check_output(["vipsheader","-f","width",p],stderr=subprocess.DEVNULL))
        h=int(subprocess.check_output(["vipsheader","-f","height",p],stderr=subprocess.DEVNULL))
        return w,h
    except Exception: return None
man=open(OUT+"/_manifest.csv","w",newline="")
W=csv.writer(man); W.writerow(["id","filename","source","op","param","src_w","src_h"])
i=0; made=0; skipped=0
for s in srcs:
    d=dims(s)
    if not d: skipped+=1; continue
    w,h=d; le=max(w,h); rel=os.path.relpath(s,SRC)
    # downscales (only strictly smaller than source longest edge)
    for sz in DOWN:
        if sz>=le: continue
        i+=1; fn=f"{i:06d}.png"; out=os.path.join(OUT,fn)
        try:
            subprocess.run(["vipsthumbnail",s,"-s",f"{sz}x{sz}","-o",out],
                           check=True,stderr=subprocess.DEVNULL,timeout=120)
            W.writerow([i,fn,rel,"downscale",sz,w,h]); made+=1
        except Exception: i-=1
    # native-res crops (no resampling)
    for kind,cs in CROPS:
        if cs>w or cs>h: continue
        if kind=="c": left=(w-cs)//2; top=(h-cs)//2
        else: left=random.randint(0,w-cs); top=random.randint(0,h-cs)
        i+=1; fn=f"{i:06d}.png"; out=os.path.join(OUT,fn)
        try:
            subprocess.run(["vips","crop",s,out,str(left),str(top),str(cs),str(cs)],
                           check=True,stderr=subprocess.DEVNULL,timeout=120)
            W.writerow([i,fn,rel,f"crop_{kind}",f"{cs}@{left},{top}",w,h]); made+=1
        except Exception: i-=1
    if made and made%500==0: man.flush(); print(f"  {made} made ({i} ids), src {srcs.index(s)+1}/{len(srcs)}",flush=True)
man.close()
print(f"DONE: {made} synth images, {skipped} unreadable sources, from {len(srcs)} sources",flush=True)
