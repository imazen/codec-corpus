# Improving the zensim training corpus — review (2026-05-27, corrected)

## What v47 actually trains on (verified: `zensim/weights/manifests/v47_strict.toml`)
`target_column = "human_score"` (per-group normalized [0,1]); losses MSE 0.6 + **RankNet 0.6** +
monotonicity 1.0. Groups:

| group | rows | target | train_w |
|---|--:|---|--:|
| safesyn | 196,086 | synthetic | 1.0 |
| cid22_train | 17,611 | **CID22 MCOS (human)** | 1.5 |
| kadid | 10,125 | **KADID-10k MOS (human)** | 0.5 |
| tid | 3,000 | **TID2013 MOS (human)** | 0.5 |
| konjnd_dense | 20,160 | **KonJND-1k JND (human)** | 1.2 |

CVVDP / multiband-anchor are **post-training spline calibration only, NOT training targets**
(`v47_cvvdp_target_FALSIFIED_2026-05-27.md` rejected CVVDP-as-target vs human). So: human-target
training and a ranking loss are **already in place** — they are NOT gaps.

v47-strict held-out (bake_verdict): CID22 SROCC 0.8547, KADID 0.8030, TID 0.7965, KonJND **0.485**,
AIC-3 0.770, AIC-4 **0.8902**. v47 traded SROCC for correctness vs V39 (KADID −0.122, TID −0.135 from
dropping 72 sign-flip features for monotonicity; blur>identity FIXED). That cost is **architectural
(monotonicity mask), not corpus** — corpus changes won't recover it.

## Real gaps & opportunities

### G1 — Human sets we now have on disk but are NOT in the training mix
Add (split-aware; keep CID22 49-ref val + AIC held-out per the holdout design):
- **UPIQ** (JOD, cross-source HDR+SDR) — not trained; complements the high-fidelity regime.
- **JPEG-AI-SDR25** (SVQA paper 2504.06301; 181 imgs + BTC/PTC triplets, local) + **AIC-3 BTC raw triplets**
  (419,760 votes / 778 workers, local) — high-fidelity human JND + raw pairwise for the RankNet term.
- **AIC-HDR2025** (HDR fine-grained, 2506.12505) — NOT yet released; watch github.com/jpeg-aic/AIC-HDR2025.
- **AIC-4** — keep eval-only. The public release is just the CfP **example** (5 source images / 305
  PTC test images + JND scores at github.com/jpeg-aic/JPEG-AIC-4-datasets); too small to train on, and
  the larger AIC-4 set is a committee-held hidden test. High-fidelity TRAINING signal comes from CID22
  (Cloudinary 250-image pairwise set, already trained; NOT the same as the 5-source AIC-3 CTC study) + the Fine-Grained/SVQA paper datasets if their full data is obtainable.
- **CSIQ**, **KonIQ-10k**, **SPAQ** — not trained; breadth (CSIQ FR; KonIQ/SPAQ in-the-wild).
- **PieAPP / BAPPS pairwise** — feed the existing RankNet objective *real human pairwise* directly.
- More **JND** data (MCL-JCI, picture-wise-JND) — KonJND SROCC 0.485 is the weakest corpus → the
  JND regime is under-supplied.

### G2 — Where imazen-26-synth fits: diversify the `safesyn` synthetic group
safesyn (196k) is the only non-human group. The imazen-26-synth references add **content- and
size-diversity** safesyn may lack. To use them they need **distortion generation** (still missing —
this set is clean references only): codec q-sweep (q5–q95, dense low-q) + traditional distortions
(blur/noise/banding/ringing/chroma/over-smooth/halo). Targets for these synthetic pairs = the
existing safesyn-style proxy/relative labels (NOT a new metric target — see falsification above);
their value is content diversity + free ranking pairs (same ref, two qualities = known order) for
the RankNet term, then the human groups carry the absolute supervision.

### G3 — Discipline (unchanged)
Persist distorted variants content-addressed + diffmaps + `build_commit` manifests; canonical store
+ Tower mirror + repo pointer (CLAUDE.md ML rules). Holdout: CID22 49-ref + AIC val never-train;
split by reference; beware UPIQ⊃{TID2013,LIVE} and TID2008⊂TID2013; report per-reference SROCC.

## Concrete next step
`05_distort_sweep.py`: codec q-sweep + traditional distortions on the curated 500 (and a size-
stratified slice of the 10.7k), persisted as `(ref_sha,dist_sha,distortion,param,bytes,bpp)` +
content-addressed PNGs → a new **safesyn-style synthetic group** for the v47 mix. Separately, ingest
UPIQ/AIC-4/CSIQ/SPAQ + PieAPP/BAPPS into the canonical training store as new human groups.
