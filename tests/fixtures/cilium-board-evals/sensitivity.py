"""Deterministic sensitivity analysis of the Cilium opportunity scoring (seeded)."""
import random
W0 = dict(BI=20,Sev=15,Freq=10,Gap=10,AI=10,Evid=10,Safe=10,Dist=10,Maint=5)
K = list(W0)
S = {  # scores exactly as committed in 02-opportunity-scoring.md
 "C5 dataplane":      [5,5,4,4,5,4,4,5,3],
 "C12 lifecycle":     [5,5,3,5,4,4,4,5,3],
 "C3 policy":         [5,5,4,3,4,4,4,5,3],
 "C8 ext-routing":    [4,5,3,5,4,3,4,4,3],
 "C10 encryption":    [4,4,2,4,4,3,5,3,3],
 "C7 clustermesh":    [4,5,2,3,4,3,4,3,3],
 "C11 performance":   [4,4,3,3,4,3,5,2,3],
 "C4 egress":         [4,5,3,2,4,3,4,2,3],
 "C13 tetragon":      [4,5,2,4,4,2,3,3,2],
 "C2 cni-arch":       [5,5,2,1,4,3,4,1,3],
 "C6 hubble":         [3,3,4,3,4,3,5,2,4],
 "C9 gateway-api":    [3,4,3,2,4,3,4,2,3],
}
ACCEPTED = {"C5 dataplane","C12 lifecycle","C3 policy","C8 ext-routing"}
def score(s,w): return sum(w[k]*v/5 for k,v in zip(K,s))*100/sum(w.values())
def accepted(w, thr=80, scores=S):
    # hard gate: distinctness <=2 rejects regardless (G1)
    return {c for c,s in scores.items() if score(s,w)>=thr and s[K.index("Dist")]>2}
schemes = {
 "baseline": W0,
 "equal": {k:1 for k in K},
 "safety-heavy": {**W0,"Safe":25,"BI":15,"Sev":10},
 "evidence-heavy": {**W0,"Evid":25,"BI":15,"Freq":5},
 "frequency-heavy": {**W0,"Freq":25,"Sev":10,"BI":15},
 "maintenance-heavy": {**W0,"Maint":20,"BI":15,"Sev":10},
 "gap-heavy": {**W0,"Gap":25,"BI":15,"Sev":10},
 "no-business-impact": {**W0,"BI":0},
}
print("== Named weighting schemes (threshold 80, G1 hard gate) ==")
for n,w in schemes.items():
    a=accepted(w); print(f"{n:20s} accepted={sorted(a)}  change_vs_baseline={'none' if a==ACCEPTED else sorted(a^ACCEPTED)}")
print("\n== Scores under each scheme ==")
print("candidate".ljust(18)+"".join(n[:10].rjust(11) for n in schemes))
for c,s in S.items(): print(c.ljust(18)+"".join(f"{score(s,w):11.1f}" for w in schemes.values()))
rng=random.Random(20261125); N=20000; flips={}; same=0
for _ in range(N):
    w={k:max(0.0,W0[k]*rng.uniform(0.5,1.5)) for k in K}
    a=accepted(w)
    if a==ACCEPTED: same+=1
    for c in a^ACCEPTED: flips[c]=flips.get(c,0)+1
print(f"\n== Random weight perturbation x[0.5,1.5], N={N}, seed=20261125 ==\nidentical accepted set: {same/N:.1%}\nper-candidate flip rate: " + ", ".join(f"{c}={v/N:.1%}" for c,v in sorted(flips.items(),key=lambda x:-x[1])))
# score uncertainty: each criterion score +-1 (clamped), baseline weights
rng=random.Random(7); same=0; flips={}
for _ in range(N):
    sc={c:[min(5,max(1,v+rng.choice((-1,0,1)))) for v in s] for c,s in S.items()}
    a=accepted(W0,scores=sc)
    if a==ACCEPTED: same+=1
    for c in a^ACCEPTED: flips[c]=flips.get(c,0)+1
print(f"\n== Score noise +-1 per criterion (uniform), N={N}, seed=7 ==\nidentical accepted set: {same/N:.1%}\nper-candidate flip rate: " + ", ".join(f"{c}={v/N:.1%}" for c,v in sorted(flips.items(),key=lambda x:-x[1])))
print("\n== Threshold sensitivity (baseline weights) ==")
for t in (70,75,78,80,82,85,88): print(t, sorted(accepted(W0,thr=t)))
