---
layout: default
title: "Cilium Board: Opportunity Scoring"
permalink: /docs/architecture/cilium-board/02-opportunity-scoring/
---

# Agent Opportunity Scoring and Elimination

> Status: **BUILT — design record.** Previous: [01-recon-capability-and-pain.md](./01-recon-capability-and-pain.md) ·
> Next: [03-board-contracts-and-orchestration.md](./03-board-contracts-and-orchestration.md)

The default answer to "should this agent exist" is **no**. Twenty-five candidates were
considered: the thirteen role hypotheses of the brief (§8), the twelve non-obvious
opportunities of the brief (§9), and none added from feature lists. Four specialists and one
router survive.

## 1. The scoring model, and why it was changed

The brief proposed nine weighted criteria scored 1–5, with
`S = Σ wᵢ · sᵢ / 5` (maximum 100). The weights were kept. Two structural defects in the
model were not:

1. **Distinctness cannot be a weighted term.** Weighted at 10%, a candidate that fully
   duplicates an existing agent (`s = 1`) loses only 8 points. A duplicate with high business
   impact, such as "CNI modernization", would score about 80 and be accepted. Duplication is
   not a matter of degree; two owners for one decision create two verdicts with no rule for
   which wins. **Distinctness ≤ 2 is a hard rejection**, scored or not.
2. **Safety cannot be averaged against value.** A candidate whose core workflow needs a live
   mutation that this repository has no guard for, such as a BGP or Helm operator, must not be
   rescued by its business impact. **Any candidate whose decision requires an unguarded
   mutation is rejected as an agent**, and the mutation is routed to a human change process.

Additional hard gates, applied before scoring:

| Gate | Rejects a candidate when |
|---|---|
| G1 Distinct decision | it owns no decision that an existing or accepted agent does not already own |
| G2 Guarded authority | its decision needs a mutation that no existing guard covers |
| G3 Evidence reachable | the evidence it needs cannot be supplied offline, as sanitized artifacts |
| G4 Supported capability | its subject is not a documented, supported upstream capability |
| G5 Judgment required | a schema, script or existing CLI decides it more reliably |

**Disposition thresholds**, chosen before scoring and not adjusted after:

- `S ≥ 80` and all gates pass: **accept** as an agent.
- `65 ≤ S < 80`, or a gate fails only on G1 or G5: **merge** into the closest accepted owner,
  delegate to an existing agent, or ship as a skill without an agent.
- `S < 65`, or a G2, G3 or G4 failure: **reject** or **defer** with named entry criteria.

The maestro is not scored on this model. A router creates no value by itself; it is justified
only when at least three accepted specialists share symptom vocabulary, so that a user cannot
pick the owner unaided. That condition is tested in §4.

**Honesty labels.** Every score below is **judgment (inference)**, made by the orchestrator
from the repository evidence in [01](./01-recon-capability-and-pain.md) and the upstream facts
in the skills' `official-sources.md` files. No score is a measurement. The board's value is
unproven until the KPIs in [05](./05-value-model-and-kpis.md) are measured.

## 2. Scorecard

Columns: BI business impact (20) · Sev severity (15) · Freq frequency (10) · Gap tooling gap (10)
· AI suitability for judgment (10) · Evid evidence availability (10) · Safe safety and
controllability (10) · Dist distinctness (10) · Maint maintenance feasibility (5).

| # | Candidate | BI | Sev | Freq | Gap | AI | Evid | Safe | Dist | Maint | S | Gates | Disposition |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| C5 | eBPF dataplane and incident investigator | 5 | 5 | 4 | 4 | 5 | 4 | 4 | 5 | 3 | **90** | pass | **Accept** `cilium-dataplane-diagnostics-agent` |
| C12 | Upgrade, lifecycle and fleet readiness | 5 | 5 | 3 | 5 | 4 | 4 | 4 | 5 | 3 | **88** | pass | **Accept** `cilium-lifecycle-readiness-agent` (absorbs C25, CNI-migration half of C2) |
| C3 | Zero-trust and microsegmentation | 5 | 5 | 4 | 3 | 4 | 4 | 4 | 5 | 3 | **86** | pass | **Retain** `cilium-network-policy-review-agent`; extend (absorbs C10, C15) |
| C8 | BGP, LB-IPAM and datacenter integration | 4 | 5 | 3 | 5 | 4 | 3 | 4 | 4 | 3 | **80** | pass | **Accept**, merged with C4 and C7 as `cilium-external-connectivity-review-agent` |
| C10 | Network encryption and trust assurance | 4 | 4 | 2 | 4 | 4 | 3 | 5 | 3 | 3 | 73 | pass | **Merge** into C3 (reference `encryption-coverage.md`) |
| C11 | Performance and capacity engineering | 4 | 4 | 3 | 3 | 4 | 3 | 5 | 2 | 3 | 71 | G1 fail | **Merge** into C5 (capacity exhaustion is a diagnostic hypothesis) |
| C7 | ClusterMesh and multi-cluster resilience | 4 | 5 | 2 | 3 | 4 | 3 | 4 | 3 | 3 | 72 | pass | **Merge** into C8 |
| C4 | Egress governance and exfiltration | 4 | 5 | 3 | 2 | 4 | 3 | 4 | 2 | 3 | 70 | G1 fail | **Split**: authorization stays in C3; gateway SNAT, HA and IP ownership go to C8 |
| C13 | Runtime threat correlation (Tetragon) | 4 | 5 | 2 | 4 | 4 | 2 | 3 | 3 | 2 | 69 | G3 fail | **Defer** to a runtime-security board |
| C6 | Hubble observability and dependency intelligence | 3 | 3 | 4 | 3 | 4 | 3 | 5 | 2 | 4 | 67 | G1 fail | **Skill without agent**: folded into `cilium-evidence-boundary` |
| C9 | Gateway API and service-mesh ownership | 3 | 4 | 3 | 2 | 4 | 3 | 4 | 2 | 3 | 63 | G1 fail | **Reject** (exists): delegate |
| C2 | Network architecture and CNI modernization | 5 | 5 | 2 | 1 | 4 | 3 | 4 | 1 | 3 | 68 | G1 fail | **Reject** (exists): `kubernetes-network-architecture-review-agent` |
| C1 | Cilium maestro | — | — | — | — | — | — | — | — | — | n/a | §4 | **Accept** `cilium-maestro-agent` |

Non-obvious opportunities from the brief (§9). Each was tested for where its value actually
comes from; none needed its own Cilium agent.

| # | Candidate | Where the value comes from | Disposition |
|---|---|---|---|
| C14 | Network change impact intelligence | Maestro routing C3 + C5 + C8 against one proposed change | **Merged** into maestro workflow |
| C15 | Zero-trust migration readiness | Default-deny readiness from observed flows | **Merged** into C3 + `cilium-evidence-boundary` |
| C16 | Fleet configuration drift | A deterministic diff of `cilium-config` and Helm values across clusters | **Rejected** as an agent (G5); deferred as a script |
| C17 | Application dependency discovery | Flow aggregation with explicit coverage limits | **Merged** into `cilium-evidence-boundary` |
| C18 | Acquisition integration readiness | Cluster ID, CIDR and identity conflicts between meshes | **Merged** into C8 and `kubernetes-network-architecture-review-agent` |
| C19 | Regulatory evidence assurance | Control-to-evidence lineage | **Delegated** to `atlassian-control-evidence-lineage-agent` |
| C20 | Network technical-debt intelligence | A `tech-debt` finding class on every specialist | **Rejected** (no distinct decision); a finding tag |
| C21 | Networking platform economics | Cost judgment on supplied technical measurements | **Delegated** to the FinOps board; C5 supplies the measurements |
| C22 | Platform self-service assurance | Needs a live mutation pipeline | **Rejected** (G2); C3 reviews the request |
| C23 | Resilience dependency analysis | Shared failure domains at the cluster edge | **Merged** into C8 |
| C24 | Incident recurrence prevention | Closed-loop verification | **Delegated** to `atlassian-incident-prevention-agent` |
| C25 | Vulnerability applicability analysis | Advisory versus installed version and enabled feature | **Merged** into C12 |

## 3. Score justifications for the accepted and the borderline

**C5 dataplane diagnostics (90).** BI and Sev are 5 because the failure it addresses, healthy
Kubernetes objects with a failing datapath, is the slowest incident class to diagnose and
directly extends customer-impacting downtime. Gap is 4, not 5: the existing
`kubernetes-network-architecture-review` playbook has a six-symptom ladder, but it names Cilium
commands without explaining drop reasons, BPF map pressure, socket load balancing or service
translation. AI is 5 because the work is ranking competing hypotheses against partial evidence,
which is judgment rather than lookup. Safe is 4, not 5: its evidence requests include privileged
observation inside the agent pod, which is why its output proposes collection but never
performs it. Maint is 3: datapath internals change across minor releases, so its references
need a per-minor refresh.

**C12 lifecycle readiness (88).** Gap is 5 because the repository has no Kubernetes or Cilium
upgrade agent, the only upgrade agents are Istio, PHP, Java and frontend. Freq is 3: upgrades
are periodic, not daily, but each is a fleet-wide change. Absorbing CNI migration is correct
because "is this transition safe" is the same decision for an upgrade and a migration: both are
judged on version compatibility, disruption mode and the rollback point. The target
architecture itself stays with the Kubernetes network architecture agent. Absorbing advisory
applicability is correct because an advisory matters only through the installed version and the
enabled features, which this agent already collects.

**C3 network policy (86, retained).** It already exists, has a live guard behind it, and is
routed to by the Kubernetes maestro. It scores Gap 3 because the base skill exists; the
extension adds default-deny readiness from flows and encryption-coverage claims. Encryption
(C10, 73) was merged rather than accepted because its user, a security architect or auditor
asking whether a zero-trust claim is substantiated, is the same as C3's, and its evidence (agent
configuration, node identity, ClusterMesh) overlaps. The merge widens C3's decision from
"authorization" to "connectivity boundary": who may connect, and whether the connection is
confidential. That is a deliberate scope change, recorded in [ADR-0006](../../adr/0006-cilium-agent-board.md).

**C8 external connectivity (80, accepted with merges).** BGP control plane is the largest
unowned gap in the catalog: no agent reviews Cilium BGP and **no live guard covers BGP
mutations** (recon, [01 §2](./01-recon-capability-and-pain.md)). ClusterMesh (C7) and egress
gateway SNAT (half of C4) were merged in because all three share one first-principles question:
*how does a packet cross the cluster boundary, who owns the address it uses, and what happens to
it when the node that owns that address fails?* Split along that line, policy agent C3 keeps
authorization (which destinations are permitted) and C8 owns reachability and address ownership.
Evid is 3: BGP and failover conclusions need peer-side state that a Kubernetes user often cannot
supply, so this agent returns `needs-review` more often than the others, by design.

**C6 Hubble (67, skill only).** Every specialist must decide whether flow telemetry supports
its own claim, so "does the telemetry support this conclusion" is a sub-decision of every agent
rather than a decision of its own. Making it an agent would create a mandatory second hop for
every investigation. Making it a shared companion skill gives one canonical owner for the
rules that matter, above all that **an unobserved flow is not a prohibited or nonexistent
flow**, without a routing hop.

**C13 Tetragon (69, deferred).** Tetragon is a separate project with its own tracing-policy
model and enforcement actions. The board has no Tetragon reference, no fixture, and no guard
for enforcement policies (G3). The existing `falco-runtime-threat-rules-review-agent` covers
runtime detection rules. Entry criteria for the deferred board are in
[06 §5](./06-red-team-and-release-readiness.md).

**C2 CNI modernization and C9 Gateway API (rejected, exist).**
`kubernetes-network-architecture-review-agent` already covers CNI choice, overlay versus native
routing, kube-proxy replacement, IPAM, MTU and Gateway API, and `istio-gateway-api-review-agent`
covers Istio's implementation. A Cilium-branded copy would be a second verdict on the same
decision. The Cilium-specific Gateway API caveats are recorded as a deferred reference.

## 4. Maestro justification test

Three symptom phrases must have more than one plausible owner for a router to earn its place.

| Symptom phrase | Plausible owners |
|---|---|
| "service unreachable" | C3 (policy drop) · C5 (service translation, MTU) · C8 (route not advertised) |
| "intermittent timeouts after the change" | C5 (conntrack pressure) · C12 (upgrade regression) · C3 (new policy) |
| "cross-cluster traffic fails" | C8 (ClusterMesh) · C3 (`policy-default-local-cluster`) · C5 (encapsulation, MTU) |

All three hold, so the maestro is accepted, under the constraints in
[03 §3](./03-board-contracts-and-orchestration.md): static routing only, at most four
specialists per request, no answers of its own, and no dispatch of a live guard.

## 5. What would change these dispositions

> **Correction (pre-acceptance review, 2026-11-25).** A seeded sensitivity analysis (`tests/fixtures/cilium-board-evals/sensitivity.py`) shows C8 is knife-edge: at exactly 80 it drops out under 5 of 7 alternative weightings, in 49.9% of random ±50% weight perturbations and in 58.9% of ±1 score-noise trials. Only C5 and C12 are robust. An independent reviewer scored C8 at 65. The acceptance of C8 therefore rests on the qualitative gap (BGP, LB-IPAM and L2 have no reviewer and no guard), not on this scorecard. See [07](./07-pre-acceptance-review.md).

- If measured routing shows the maestro adds a hop without changing the owner in more than
  80% of real requests, remove it and route from the Kubernetes maestro directly.
- If encryption questions turn out to be dominated by datapath failures (MTU, key rotation)
  rather than coverage claims, move `encryption-coverage.md` from C3 to C5.
- If C8 returns `needs-review` on most BGP reviews because peer state is never available,
  narrow it to LB-IPAM and egress gateway, and record BGP as needing a network-team review.
