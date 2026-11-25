---
name: cilium-dataplane-diagnostics
description: Explain observed Cilium datapath behavior (intermittent timeouts, Service VIP or NodePort failures, cross-node-only loss, large-payload failures, latency regressions, non-policy drop reasons) and assess conntrack, NAT and BPF map capacity from supplied status, drop lines, metrics and Helm values. Use to rank competing hypotheses and name the next least-invasive discriminating observation, including forward-looking map headroom computed only from supplied counts. Do not use for policy verdict review, BGP, LB-IPAM, egress gateway or ClusterMesh design, upgrade readiness or target architecture. Static review only; never collects live evidence.
allowed-tools: Read Grep Glob
metadata:
  author: 'github: VincentChuWaiChow'
  version: 0.1.0
  updated: '2026-11-25'
  category: networking
  lifecycle: beta
  execution_tier: static-review
---
# Cilium Dataplane Diagnostics

Rank datapath hypotheses for a Cilium cluster and name the one observation that separates them.

## Decision and boundary

What is the most defensible explanation of the observed Cilium datapath behavior, and which next observation would confirm or refute it? This includes capacity and map-pressure questions. Read supplied artifacts only; never run commands, probe, restart, or mutate. Commands in the references are evidence an authorized operator may collect. Command classes, evidence classes, verdict vocabulary, Hubble and flow-telemetry interpretation and sensitive-collection rules belong to the companion skill `cilium-evidence-boundary`.

## When to use / when not to use

Use for: symptom triage on a Cilium datapath, packet-path derivation from supplied configuration, CT/NAT/policy map exhaustion and headroom, kube-proxy-replacement and socket-LB behavior, MTU and encapsulation faults, non-policy drop reasons.

Hand off instead:

- Policy verdict drops ("Policy denied") to `cilium-network-policy-review-agent`.
- BGP, LB-IPAM, L2 announcements, egress gateway, ClusterMesh to `cilium-external-connectivity-review-agent`.
- A regression that started after an upgrade or config change: keep diagnosing, and co-route with `cilium-lifecycle-readiness-agent`.
- Target architecture or CNI choice to `kubernetes-network-architecture-review-agent`; generic DNS and EndpointSlice triage follows the skill `kubernetes-network-architecture-review`.
- Cost of larger maps or nodes to `finops-maestro-agent`.

## Required inputs

Cilium version and Helm values or agent config (routing mode, kube-proxy replacement, socket-LB, host routing, encryption, DSR), symptom with affected and unaffected paths and a time window, drop lines or `drop_count_total` series with reasons, and current versus configured map counts. Missing fields downgrade the affected conclusion to `needs-review`.

## Workflow

1. Establish affected versus unaffected traffic, time window, last change and Cilium version; preserve unknowns.
2. Derive the configuration-specific packet path from supplied settings, not from a generic diagram.
3. Classify each observation by hop: endpoint, service translation, conntrack and NAT, routing and encapsulation, encryption, proxy, policy.
4. Build competing hypotheses per symptom with supporting and contradicting evidence; one status code or one drop line is not a root cause.
5. For capacity questions, compute headroom only from supplied counts and configured maxima; a missing metric is unknown, never healthy.
6. Pick the least-invasive next observation that discriminates the leading hypotheses; state which result would refute each.
7. Route non-owned findings (policy, external connectivity, lifecycle) with the evidence gathered so far.
8. Return ranked hypotheses, headroom arithmetic and handoffs; separate observed, derived, assumed and unknown.

## Reference loading

Load companion skill `cilium-evidence-boundary` first. Then load only the active branch.

| Reference | Load when |
| --- | --- |
| [packet-path-model.md](references/packet-path-model.md) | Deriving the hop sequence for a given routing, KPR, host-routing, encryption or proxy configuration. |
| [hypothesis-trees.md](references/hypothesis-trees.md) | A symptom needs competing hypotheses and a discriminating check. |
| [conntrack-nat-and-map-capacity.md](references/conntrack-nat-and-map-capacity.md) | CT, NAT or policy map pressure, `CT: Map insertion failed`, headroom questions. |
| [service-load-balancing.md](references/service-load-balancing.md) | Service VIP, NodePort, socket-LB, Maglev, DSR or KPR toggling. |
| [routing-mtu-and-encapsulation.md](references/routing-mtu-and-encapsulation.md) | Large-payload failures, tunnel or encryption overhead, asymmetric routing. |
| [drop-reasons-and-signals.md](references/drop-reasons-and-signals.md) | Interpreting a drop reason, metrics, endpoint regeneration, or version-gating a command. |
| [evidence-ledger.md](references/evidence-ledger.md) | Verifying a cited upstream claim or its retrieval status. |

## Failure handling

- Missing version: do not choose a command form or default; state the version-dependent variants and mark `unknown`.
- Missing metrics or Prometheus disabled: report the gap as unknown, never as healthy.
- Conflicting sources (for example code comment versus constant): record both and cite the CONFLICT row.
- No tool access: produce the evidence request list only; never imply it was run.

## Deliverable

Timeline, derived packet path, ranked hypotheses with supporting and contradicting evidence, one next discriminating observation per unresolved branch (with class and required authorizer), headroom arithmetic where applicable, and owner handoffs. Never present a bounded review as production certification.
