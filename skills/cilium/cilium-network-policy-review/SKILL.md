---
name: cilium-network-policy-review
description: Use this skill to decide whether Cilium policy and transport-encryption configuration enforce the intended connectivity boundary, meaning who may connect and whether the connection is confidential, without unacceptable rollout risk. Covers Kubernetes NetworkPolicy, CiliumNetworkPolicy and CiliumClusterwideNetworkPolicy review, default-deny posture and migration readiness, L7 rules and the proxy requirement, egress authorization, ClusterMesh policy-default-local-cluster semantics, and WireGuard or IPsec encryption coverage claims. Use for sanitized YAML or authorized read-only evidence. Do not use for egress gateway IP, SNAT or failover review, datapath drop diagnosis, upgrades, or executing any change.
allowed-tools: Read Grep Glob
metadata:
  author: 'github: VincentChuWaiChow'
  version: 0.2.0
  updated: '2026-11-25'
  category: security
  lifecycle: beta
  execution_tier: static-review
---

# Cilium Network Policy Review

Review Cilium policy and encryption evidence against zero-trust correctness, blast radius and rollout risk.

## Decision and boundary

Decision: does the policy and transport-encryption configuration enforce the intended connectivity boundary - who may connect, and whether the connection is confidential - without unacceptable rollout risk?

Critical findings, always stated as such:

- Removal of a default-deny `NetworkPolicy`: pods become reachable unless another policy isolates them.
- Egress `toCIDRSet` of `0.0.0.0/0` without an `except` covering the cloud metadata address and sensitive internal CIDRs: an exfiltration path.
- Any change to `policy-default-local-cluster` in a ClusterMesh: every policy's cross-cluster scope flips at once.
- L7 fields (HTTP, DNS) in namespaces where the proxy is not enabled or confirmed.

Boundaries: the agent reviews and recommends; it never executes. Live policy changes go to `kubernetes-live-network-policy-guard-agent`. `CiliumEgressGatewayPolicy` IP collision, SNAT and gateway failure go to `cilium-external-connectivity-review-agent`; egress authorization stays here. Drop diagnosis goes to `cilium-dataplane-diagnostics-agent`; upgrades to `cilium-lifecycle-readiness-agent`; Helm and cilium-config changes have no guard, so hand them to the human change process.

## When to use / when not to use

- Use for: policy too broad, default-deny present or missing, namespace migration to default-deny, L7 enforceability, ClusterMesh policy semantics, and "is this traffic encrypted" claims.
- Do not use for: CNI choice or architecture (`kubernetes-network-architecture-review-agent`), Istio (`istio-maestro-agent`), runtime detection (`falco-runtime-threat-rules-review-agent`), or proving a packet path.

## Required inputs

- Sanitized policy YAML for the affected scope, and the Cilium version and routing mode.
- For rollout: Hubble flows for the namespace with window and sampling, and dependency owners.
- For encryption: the claim, the traffic classes, encryption mode, and agent status or capture output.
- Static review only: review supplied, sanitized YAML and command output. Commands in the references are evidence the user or an authorized operator may collect against a confirmed cluster and context; the agent never collects it.

## Workflow

1. Load the companion skill first, fix scope, version and evidence classes; if the cluster identity is unconfirmed, stay static.
2. Identify policy formats and union semantics; list which policies select each workload per direction.
3. Establish default-deny posture per namespace and direction; flag any removal or unselected workload.
4. Audit egress: CIDR scope and `except`, empty selectors, FQDN wildcards, and the `**` DNS change.
5. Audit L7 rules against proxy availability and the 1.20 Kafka and `l7proto` removal.
6. Audit ClusterMesh selectors and the `policy-default-local-cluster` flip; hand off egress gateway behavior.
7. For rollout, build the flow inventory, add non-flow dependencies, predict breakage, order by blast radius.
8. For encryption claims, test each named traffic class against documented gaps; config alone is intent.
9. Set verdict and evidence level, state the rollback as re-applying the prior policy set through the live guard.

## Reference loading

Load companion skill `cilium-evidence-boundary` first, then only what the question needs.

| Reference | Load when |
| --- | --- |
| [mcp-and-evidence.md](references/mcp-and-evidence.md) | Choosing evidence path, confirming version, ClusterMesh and proxy state, sanitization |
| [workflow-and-output.md](references/workflow-and-output.md) | Running the full format-by-format review and shaping the answer |
| [official-sources.md](references/official-sources.md) | Needing the documentation list and grounded background |
| [encryption-coverage.md](references/encryption-coverage.md) | Any claim that traffic is encrypted, strict mode, key rotation, PCI-style audit |
| [default-deny-rollout.md](references/default-deny-rollout.md) | Moving a namespace to default-deny or reviewing version-sensitive policy changes |
| [evidence-ledger.md](references/evidence-ledger.md) | Checking the source of a `CIL-NP` citation |

## Failure handling

- Missing flows or version: verdict `needs-review`, evidence `assumed` or `partial`; absence of flows is not absence of dependency.
- Conflicting config and status output: report both, trust neither, request the discriminating capture.
- Version-sensitive behavior (1.19 and 1.20 notes) with unknown version: state the unknown, do not guess.
- Unavailable or unconfirmed live tools: static review only.
- Never print peer Secrets, keys or tokens; treat sysdump and bugtool archives as sensitive.

## Deliverable

Return: scope and evidence level; default-deny posture; egress authorization; L7 enforceability; ClusterMesh semantics; encryption coverage per traffic class; rollout risks and ordering; verdict (`approved`, `blocked`, `needs-review`); rollback via the live guard; assumptions, unknowns and handoffs.
