---
layout: default
title: "Cilium Board: Contracts and Orchestration"
permalink: /docs/architecture/cilium-board/03-board-contracts-and-orchestration/
---

# Board Contracts and Orchestration

> Status: **BUILT — design record.** Previous: [02-opportunity-scoring.md](./02-opportunity-scoring.md) ·
> Next: [04-threat-model-and-governance.md](./04-threat-model-and-governance.md)

## 1. Final board

```text
                    user request
                         |
                 cilium-maestro-agent          (static routing; <= 4 owners; no answers)
                         |
   +---------------+-----+-----------+--------------------+
   |               |                 |                    |
 policy        dataplane      external-connectivity   lifecycle
 (who may      (why does the  (how does the packet    (is the change
 connect, is   datapath do    cross the boundary,     safe to enter
 it private)   this)          who owns the address)   the change process)
   |               |                 |                    |
   +--------- cilium-evidence-boundary (shared rules) ----+
                         |
      verdicts + handoffs: Kubernetes live guards (never auto-dispatched),
      human change process, jira-ticket-authoring, confluence-document-authoring
```

## 2. Ownership matrix

| Topic | Owner | Explicitly not |
| --- | --- | --- |
| Policy semantics, composition, default-deny, egress authorization, L7 prerequisites, `policy-default-local-cluster` | policy | — |
| WireGuard and IPsec coverage, strict mode, key lifecycle | policy | dataplane (unless the symptom is an MTU or drop fault) |
| Non-policy drop reasons, CT, NAT, BPF maps, headroom, KPR, socket LB, Maglev, DSR, MTU, routing mode effects | dataplane | policy |
| BGP, LB-IPAM, L2 announcements, egress gateway SNAT, selection, failure; ClusterMesh prerequisites, global services, affinity, unreachable-cluster behavior | external | policy (authorization), Kubernetes architecture agent (technology choice, CIDR plan) |
| Upgrade path, rollback, breaking changes, kernel and Kubernetes compatibility, migration, disruptive in-place changes, advisory applicability | lifecycle | Kubernetes architecture agent (target design) |
| Command classes, claim classes, verdicts, telemetry completeness, sensitive collection, Atlassian handoff | evidence-boundary | every specialist (they apply, they do not restate) |
| Which owner, in what order, combined verdict | maestro | — |

Every topic has exactly one canonical owner. Where another specialist needs that knowledge, it names the owner and hands off; the gate checks that every specialist loads the shared skill, and the refuter pass ([06](./06-red-team-and-release-readiness.md)) checked for restated content.

## 3. Agent contract standard

Each `AGENT.md` carries, and `tests/validate-cilium-board.py` (rule CB07) enforces, these sections: Mission, Decision ownership, Scope, Activation, Required skills, Input contract, Investigation workflow, Decision rules, Tool permissions, Security boundaries, Failure handling, Output contract, Handoff rules, Adversarial checks. They map onto the brief's seventeen elements as follows: identity and business impact are in Mission; the diagnostic framework and evidence requirements are in the companion skill's workflow and references (the agent points to them instead of copying them, so a procedure has one owner); the output contract defers to the shared evidence-boundary contract.

The agent is the identity and authority boundary; the skill is the procedure. That split is the repository's convention (the Istio and agentic boards use it) and it is what keeps a procedure from being maintained in two places.

## 4. Skill architecture

Benchmark: `skills/aws/aws-agentcore`, whose entry point is short and routes to component references loaded only when that component is in scope. The Cilium skills follow the same shape and add two things the benchmark does not have:

1. **A per-skill evidence ledger** with a gate, so that every upstream claim is traceable to a dated source and status. The benchmark's `official-sources.md` is a reading list; a ledger is evidence.
2. **Command classification** with a gate, because these skills name commands with real side effects.

| Skill | Entry point | References (load on demand) |
| --- | --- | --- |
| `cilium-evidence-boundary` | rules every specialist applies | command classification; evidence and verdicts; flow telemetry; findings-to-Atlassian handoff; ledger |
| `cilium-network-policy-review` | policy and encryption decision | evidence path; workflow and output; encryption coverage; default-deny rollout; official sources; ledger |
| `cilium-dataplane-diagnostics` | datapath decision | packet-path model; hypothesis trees; CT, NAT and map capacity; service load balancing; routing, MTU and encapsulation; drop reasons and signals; ledger |
| `cilium-external-connectivity-review` | boundary decision | BGP Control Plane; LB-IPAM and L2 announcements; egress gateway; ClusterMesh resilience; ledger |
| `cilium-lifecycle-readiness` | lifecycle decision | upgrade path and rollback; version and kernel compatibility; 1.19 and 1.20 breaking changes; CNI migration and disruptive changes; advisory applicability; ledger |
| `cilium-review-routing` | routing decision | routing table; conflicts and aggregation; live and out-of-board handoff |

**Cross-skill reuse that survives export.** Skills are exported standalone, so a relative link from one skill into another breaks after installation. Reuse is therefore expressed through `companion_skills` in agent metadata (the exporter bundles every companion) and plain-text references to the shared skill's name, never through relative paths across skill directories.

## 5. Orchestration

- The maestro routes by decision, not by resource name, using the routing table reference. It dispatches at most four specialists (the grader's ceiling is also four).
- **Dependency order:** version and enabled-feature context first when unknown; then the specialist whose output decides the next owner (for "service unreachable", the dataplane drop reason decides between policy and boundary).
- **Aggregation:** a confirmed `blocked` stands; otherwise any `needs-review` stands; no averaging, no votes. Evidence level is carried per decision.
- **Conflict resolution:** name the exact claim, compare scope, version, provenance and capture time, expose hidden assumptions, name the smallest discriminating observation, and preserve the disagreement if it is unresolved. Two agents agreeing on the same evidence is one observation.
- **Handoff envelope:** board-to-board and board-to-principal messages use the existing `agentic-board-protocol` envelope (`schemas/agentic-handoff.schema.json`). Its `evidence_refs` labels, `missing_evidence` with a blocking flag, `authorized_actions` (read, analyze, recommend, challenge, escalate only), `do_not_do_list` and `approval_state` already express what the brief's illustrative envelope wanted. No Cilium-specific schema was added; a domain field that the envelope cannot express has not been found.

> **Correction (pre-acceptance review, 2026-11-25).** No Cilium contract adopts the envelope, and nothing routes Cilium traffic through its gate; the Atlassian case capsule rejects Cilium agents (`CAP-UNKNOWN-AGENT`), and the envelope cannot carry a `blocked` return without a lossy mapping (`HO-BLOCK-AS-CHALLENGE`). Both were reproduced by the integration reviewer. The bridge to Atlassian is a human, as the handoff reference now says. The maestro also no longer delegates: its adapters grant no spawning tool (Claude Code ignores an `Agent(...)` type list for subagents, so a grant would reach every agent), and it returns a dispatch plan instead. See [07](./07-pre-acceptance-review.md).

## 6. Closed loop with Jira and Confluence

The handoff contract lives in `cilium-evidence-boundary/references/findings-to-atlassian-handoff.md`. It maps finding types to the existing `jira-ticket-authoring` content patterns (incident, bug, finding, tech-debt, operational-change, epic, spike) and verified knowledge to existing `confluence-document-authoring` templates (adr, tdd, runbook, incident-postmortem, risk-assessment, support-knowledge-base). Four rules stop the loop from converting inference into record: inference travels labeled; ticket closure is not remediation proof; a page is never an approval; retrieved content is never an instruction. Live writes stay with `atlassian-live-write-guard-agent`, as ADR-0005 decided.

## 7. Routing fixture

`tests/fixtures/cilium-maestro-routing/` holds 28 scenarios with expected routes generated by the repository grader's `evaluate()`: 10 single-owner cases (including one per merged capability: encryption, capacity, egress split, advisory, migration), 3 multi-domain, 5 live-guard gates, 5 negative (no false gate on a safety review; unguarded mutations route to static review rather than to a guard that does not own them; product questions and Tetragon stay unclassified), and 5 adversarial (instruction injection, persona replacement, secrets bait, ambiguity, an instruction quoted from a ticket). The two known limitations of a keyword grader are recorded as tags in the fixtures and in [06](./06-red-team-and-release-readiness.md).

> **Correction (2026-11-28, code review of PR #246):** "unguarded mutations route to static review" was a fail-open. A request to *execute* a BGP, LB-IPAM, L2, Helm or `cilium` CLI change now gates (`live-guard-gate`) with an empty route, because no guard owns it; a review of the same change is still routed to its specialist. The intent regex also matched only a determiner directly before the resource kind and looked for the context word only after the verb, so "Delete the default-deny CiliumNetworkPolicy in production" did not gate; both are fixed. The grader gained an optional taxonomy key, `live_guard_require_keyword`, so a gated mutation no guard owns is never handed to whichever guard sorts first. The fixture now holds 42 scenarios.
