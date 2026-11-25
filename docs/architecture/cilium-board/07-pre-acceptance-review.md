---
layout: default
title: "Cilium Board: Independent Pre-Acceptance Review"
permalink: /docs/architecture/cilium-board/07-pre-acceptance-review/
---

# Independent Pre-Acceptance Review of ADR-0006

> Status: **review record, 2026-11-25.** ADR-0006 stays **Proposed**. Previous:
> [06-red-team-and-release-readiness.md](./06-red-team-and-release-readiness.md). This record supersedes the
> self-assessment in 06 §8.

## 1. Outcome

**Recommendation: RECOMMEND ACCEPTANCE AFTER SPECIFIED REMEDIATION.**

The review found two critical technical errors and nine major weaknesses. It also found a live-guard regression that the board itself had introduced. Every one that can be fixed in the repository was fixed in this change and re-verified. What remains are four mandatory gates that the repository cannot pass on its own (§12), and one dissent:

- The maintainability gate **fails**: there is no named owner for the per-release refresh.
- The business-value gate is **insufficient evidence**: nothing has been measured.
- The specialization and adversarial-resilience gates are **conditional on your decisions** in §13.
- One reviewer recommended a scoped **redesign**: remove the maestro and narrow the external-connectivity specialist. That dissent is recorded in §13, not averaged away.

What justifies the recommendation:

- Every safety boundary the board claims is now enforced by a gate, and the red team's evasions became permanent negative tests.
- All 15 adversarial behavioral scenarios passed independent grading, including 12 uncontaminated runs, with zero fabricated observations and zero mutation attempts.
- The independent technical re-verification found no contradicted Cilium fact.

What remains unverified:

- Real-incident accuracy. One model family wrote the board and answered the scenarios, so the evaluation is weak evidence.
- Measured value.
- Behavior on Kiro and Codex, where tool limits are prompt-level.

## 2. Method and the limits of its independence

| Reviewer role | Who executed it | What it actually did |
| --- | --- | --- |
| Distinguished Network Architect, Cilium and eBPF Specialist | Fresh subagent, withheld the original fact sheets | Re-fetched about 45 production-relevant claims from docs.cilium.io, tagged source at v1.20.5, Helm reference, managed-provider pages |
| Agentic Systems Architect, Repository Maintainer, CTO | Fresh subagent under the `agentic-system-architecture-review-agent` contract | Scored all 25 candidates before reading 02, then challenged topology and conventions |
| CISO, Red Team | Fresh subagent under the `agentic-security-review-agent` contract | Ran evasion probes against the gates, router and ledger validator on copies |
| Technical Documentation Reviewer | Fresh subagent | Read every Cilium file and the AgentCore benchmark; n-gram duplication script |
| Enterprise Governance Specialist | Fresh subagent | Paper-ran six Jira and Confluence scenarios against the real templates; validated an envelope and a capsule with the repository's own gates |
| FinOps Specialist, SRE Director | Fresh subagent under the `finops-ai-economist-agent` contract | Recomputed the scorecard, counted maintenance surface, assessed 3 a.m. usefulness |
| Behavioral evaluation | Five runner subagents acting as the board's agents through their Claude Code adapters (Read, Grep, Glob only) and one separate grader | 15 scenarios (`tests/fixtures/cilium-board-evals/`) |
| Sensitivity analysis | Orchestrator, seeded script | `tests/fixtures/cilium-board-evals/sensitivity.py` |

**This is not independent human review.** Every reviewer, runner and grader ran on the same model family as the builder. They received no stake in the earlier conclusions, but their blind spots are likely to correlate. Where reviewers agree, that is one family's view, not consensus. The orchestrator did not grade its own board.

## 3. Findings, by severity

Status: **fixed** means changed in this change and re-verified by the named method; **open** means a decision or work item that remains.

### P0 — Critical

| # | Finding | Evidence | Consequence | Remediation | Verification | Status |
| --- | --- | --- | --- | --- | --- | --- |
| P0-1 | The policy skill ignored deny policies. `ingressDeny` and `egressDeny` override every allow, from CNP, CCNP or Kubernetes NetworkPolicy. | Technical reviewer; orchestrator re-verified via Context7 (`deny.rst`, 1.20.4) | "Traffic is allowed" or "least privilege holds" concluded from allows alone is wrong | New `policy-semantics.md`; workflow step 2 now asks the five semantics questions first; ledger `CIL-NP-E22`, `E23` | `validate:skill-references`; ledger rows cited | fixed |
| P0-2 | The policy skill never asked the enforcement mode. In `never` mode, rules that select endpoints enforce nothing; in `always` mode, unselected endpoints are denied. | Technical reviewer; re-verified (`intro.rst`, 1.20.4) | Default-deny conclusions wrong in both directions; a migrating cluster (mode `never`) looks segmented | Same reference; `CIL-NP-E24`, `E25`, `E26`, `E27` (incl. `enableDefaultDeny`, audit mode, Kubernetes per-direction isolation and DNS) | Same | fixed |

### P1 — Major

| # | Finding | Remediation | Verification | Status |
| --- | --- | --- | --- | --- |
| P1-1 | **The board broke a live guard.** `kubernetes-live-network-policy-guard-agent` used the rescoped policy skill as its checklist, lost egress-gateway checks, gained an unbundled dependency, and inherited a "never execute" rule. | Guard bundles `cilium-external-connectivity-review` and `cilium-evidence-boundary`, version 0.2.0. Policy skill gains a carve-out section for the guard. New gate rule CB11. | CB11 negative probe reproduced the regression and failed | fixed |
| P1-2 | **The maestro could not do its job.** It was contracted to delegate, but in Claude Code `Agent(...)` type lists are ignored for subagents, so any spawning grant would reach every agent, live guards included. (Also raised by the Codex review.) | Maestro is now plan-only: it returns a dispatch plan and combines outputs the caller returns. Adapters stay read-only. | Eval S09, S11, S14 passed as plan-only | fixed; value still unmeasured (O-2) |
| P1-3 | **Double hop and dual owners.** The Kubernetes maestro sent single-owner Cilium phrases through the Cilium maestro, and kept "kube-proxy replacement, MTU, conntrack" and "egress gateway policy" with other owners. | Kubernetes maestro routes straight to each specialist. Terms deduplicated (design versus runtime). The architecture playbook hands datapath work to the dataplane agent. | Prose routing table only; the generated Kubernetes fixture still has no Cilium domain | partly fixed (O-7) |
| P1-4 | **The scorecard cannot justify C8.** It is at exactly 80. The independent reviewer scored it 65. | Sensitivity result recorded in 02 §5. Acceptance of C8 rests on the unowned BGP, LB-IPAM and L2 surface, not on the score. | `sensitivity.py`, seeded | recorded; decision O-1 |
| P1-5 | **External connectivity is a forced merge.** ClusterMesh ownership is split three ways, and CiliumEgressGatewayPolicy two ways plus the guard. | Explicit splits documented; guard now bundles the egress reference | — | open, decision O-1 |
| P1-6 | **The routing regex failed open.** An appended "no review needed" switched the gate off. `kubectl -n prod apply` was unclassified, and `delete cnp` tied to the wrong guard. | Lookahead removed; flag-tolerant CLI pattern; synonyms; `cnp`/`ccnp` guard keywords; six new fixtures. Recorded as a test of a keyword router, not a runtime control. | `validate:maestro-routing`, 924 scenarios | fixed |
| P1-7 | **Gate evasions.** `sudo`, env prefix, absolute path, uppercase, zero-width characters, HTML spans, only the first command checked per block, and any tag accepted as correct. | NFKC normalization; prefix stripping; more tools; every block line checked; HTML spans checked; CB12 rejects mutating verbs below [C4]. | Nine-pattern probe file: all caught, then removed | fixed |
| P1-8 | **Approval text and verdict-word collision.** Bare `approved` was copied into tickets, and approval handling was prose-only. | Handoff now writes the definition with the word. Board-to-Atlassian bridge is a human. Envelope claims corrected in 03, 04 and the ADR. | Eval S11, S13 passed; still a model-behavior control | partly fixed |
| P1-9 | **No Cilium-to-Atlassian channel.** The capsule rejects Cilium agents and the envelope cannot carry `blocked`. Both were reproduced by the integration reviewer. | Stated plainly that the bridge is a human. Field checklist per template; reassessment request defined. | Paper walk-through | fixed as documentation; no machine channel |
| P1-10 | **Skills were not self-contained.** No skill declared its companion, and the dataplane skill depended on an unbundled Kubernetes skill. | `companion_skills` on every skill. Every specialist fails closed to `needs-review` without its companion. Dataplane now hands off to the agent instead. | `validate:cilium-board`, schema gates | fixed (no tool consumes skill-level companions; the fail-closed rule is the real control) |
| P1-11 | **Skill in a main session with Bash.** A loaded skill pre-approves tools but does not restrict them. | `disallowed-tools: Write Edit NotebookEdit Bash` on all six Cilium skills | `validate:skill-schema` | fixed for Claude Code only; other harnesses do not read the field |

### P2 — Material improvement

| # | Finding | Status |
| --- | --- | --- |
| P2-1 | Lifecycle routed Gateway API CRD changes to a guard that refuses all CRD operations (Codex review; technical reviewer agreed). | fixed: human change process |
| P2-2 | Lifecycle blocked every target but the next minor, including same-minor patches and one-minor rollbacks (Codex review). | fixed |
| P2-3 | Ledger allowlist accepted `..` traversal, encoded traversal, ports, userinfo, and user content under the org. | fixed: `official()` normalizes and rejects; all 22 ledgers still pass |
| P2-4 | `hubble observe` was classed [C1] despite the board's own C2 definition (sensitive collection). | fixed: [C2] everywhere |
| P2-5 | Value model understated maintenance (1.5 to 4.5 engineer-days per minor is the reviewer's estimate). Three KPIs measured form, volume, or unmeasurable quantities, and the maestro KPI had no data source. | recorded in 05; revised KPIs in §10 |
| P2-6 | Managed distributions: the reviewer found GKE Dataplane V2, AKS Azure CNI powered by Cilium and EKS Hybrid Nodes all restrict versions and configuration. The skills say only "ask the vendor". | open (O-4): the orchestrator has not verified those provider pages, so they are not encoded |
| P2-7 | Collection examples use `-A` (all namespaces) and never name `--context`: cross-tenant collection labeled as read-only. | open (O-10) |
| P2-8 | Answers state upstream facts without citing ledger IDs (grader's main criticism). | open (O-8) |
| P2-9 | Encryption folded into the policy decision dilutes single-decision ownership. | open, decision O-3 |
| P2-10 | Context cost: a typical policy review loads 9k to 11k tokens, against 3k to 4k for AgentCore. | open |

### P3 — Minor

| # | Finding | Status |
| --- | --- | --- |
| P3-1 | Legacy hedges remain in the policy workflow file (empty-selector and wildcard semantics). | open |
| P3-2 | `helm template` was over-classified; it is an offline render. | fixed: [C0] |
| P3-3 | Unverified CVE numbers were printed in the advisory reference. | fixed: removed; GHSA ids stay the key |
| P3-4 | `openai.yaml` was missing for the policy skill, and other files had truncated descriptions. | fixed |
| P3-5 | Bespoke per-board gates are proliferating (Istio, Atlassian, agentic board, Cilium). | open: candidate for one generic board-contract gate |
| P3-6 | Codex adapter sandbox handling differs from the Istio board's. | open |
| P3-7 | The guard's former egress-gateway IP-collision check was never sourced. It is replaced by the verified egress-gateway reference. | recorded |

### P4 — Optional

- **P4-1:** Rename `mcp-and-evidence.md`, which contains no MCP content.
- **P4-2:** Trim duplicated boilerplate. "Human change process" appears 25 times.
- **P4-3:** Add a `dependencies` block to `openai.yaml`, as AgentCore does.

## 4. Candidate re-assessment and sensitivity

The independent reviewer scored all 25 candidates before reading the original scorecard. Gaps of 10 or more points:

| Candidate | Original | Independent | Gap |
| --- | --- | --- | --- |
| C5 dataplane | 90 | 71 | −19 |
| C8 external connectivity | 80 | 65 | −15 |
| C11 performance | 71 | 56 | −15 |
| C7 ClusterMesh | 72 | 59 | −13 |

C25, advisory applicability, scored 77, which supports merging it into lifecycle. Under the reviewer's own scores, only C3 reaches 80.

Seeded sensitivity on the original scores (`sensitivity.py`):

| Perturbation | Result |
| --- | --- |
| 7 alternative weightings (equal, safety-, evidence-, frequency-, maintenance-, gap-heavy, no business impact) | C8 drops out under 5; C3, C5 and C12 stay in under all |
| Random weights ×[0.5, 1.5], N = 20,000 | identical set 50.1%; C8 flips 49.9% |
| ±1 score noise, N = 20,000 | identical set 21.0%; flip rates: C8 58.9%, C3 22.5%, C12 13.6%, C5 5.9% |
| Threshold 82 or higher | C8 out |

Answers to the challenge questions:

- **Can 30 to 50% of the agents go without losing value?** Possibly 40%: the maestro and external connectivity are the candidates. Both arguments are recorded in §13 for your decision. No evidence in this review shows that either agent changes an outcome.
- **Is a rejected candidate worth more than an accepted one?** C10 encryption (73) and C25 advisory (77) outscore C8 in the independent scoring. Both are already merged into accepted owners, so no rejected capability is lost.
- **Are we optimizing for appearance?** The 35 adapter files and 14 required headings are generated and gated, so they cost little to keep consistent. The real cost is the version-sensitive reference surface.

## 5. Technical revalidation

The independent re-verification of about 45 claims found:

- **No upstream contradiction** in defaults, version gates, compatibility, drop codes, MTU overheads or encryption gaps. Examples: drop codes in `drop.go` and MTU constants in `mtu.go`, both at v1.20.5; the ClusterMesh single trust domain; `cacheTTL` default 0s; egress-gateway incompatibilities; WireGuard same-node and XDP/DSR gaps.
- **Omissions** were the two P0s plus Gateway API controller ownership and the GatewayClass `controllerName` (open), `policy-deny-response` (open), and host policies (open).
- **Version skew:** Gateway API is 1.6.1 in the upgrade notes but 1.6.3 on the install page. Context7's newest Cilium snapshot (1.20.4) lags stable (1.20.5).
- **Not re-fetched:** the system requirements, IPsec, L2 announcements and Hubble pages, and the security-advisory pages. The advisory table stays a dated snapshot to re-fetch before use.

Documentation describes capability, not behavior in every configuration; none of this was tested on a cluster.

## 6. Skills against the AWS AgentCore benchmark

| Dimension | Result | Note |
| --- | --- | --- |
| Technical depth | Cilium better | defaults, overheads, lease arithmetic, map rules |
| Progressive disclosure | AgentCore better | Cilium's mandatory shared contract adds 4k to 5k tokens |
| Reference specialization | Cilium better | three generic legacy files in the policy skill |
| Source authority | Cilium much better | per-claim gated ledgers versus URL lists |
| Version awareness | Cilium better | snapshots decay; refresh required |
| Diagnostic workflows | Cilium better | hypothesis trees with contradicting evidence |
| Safety boundaries | Cilium better | behavior-based command classes |
| Output contracts | Cilium better | finding anatomy with counterexample |
| Reference deduplication | Equal | 21 near-duplicates, mostly pointers |
| Cross-harness portability | AgentCore better before this change | fixed by companion declarations and the fail-closed rule |
| Maintainability | Cilium better tooling, heavier refresh burden | see O-9 |
| Enterprise applicability | Cilium better | 15 scenario-bearing references |

Verdict: meets the benchmark, with context cost the main gap.

## 7. Jira and Confluence

| Scenario | Result |
| --- | --- |
| 1. Incident draft | PARTIAL → fixed: field checklist now names state, severity source, timeline, recovery and owner |
| 2. Remediation ticket with measurable criteria | PARTIAL → fixed: finding-to-implementation step and draft criteria fields |
| 3. Runbook | PARTIAL → fixed: owner, stop conditions, escalation, `NOT VERIFIED` status |
| 4. ADR plus backlog | PARTIAL → fixed: decision authority and options; the ADR stays Proposed without them |
| 5. Closed ticket reassessment | FAIL as a mechanism → reassessment request defined; still nothing triggers it automatically |
| 6. Injected instructions in tickets | PASS. Permission and boundary changes are stopped by code (capsule rules, read-only adapters); "include kubeconfig" is stopped only by the tool grant and prose. |

The board reuses the authoring skills and adds no templates. The live-write guard still owns every write.

## 8. Threat-control matrix (corrected)

I = implemented and tested in this change; P = partial; D = documented prose only.

| Threat | Preventive | Detective | Status | Residual |
| --- | --- | --- | --- | --- |
| Prompt injection | prose "data, not instructions"; routing fixtures | evals S11, S13 | D + eval | High: model behavior |
| Confused deputy, skill in main session | `disallowed-tools` (Claude Code); read-only adapters | CB10 | P | Medium: other harnesses |
| Unauthorized production mutation | no write tools on any Cilium adapter; no live operator | CB06, CB10, CB12; 15 evals with no mutation attempt | I | Medium: Kiro and Codex prompt-only |
| Wrong guard or gate bypass in routing | fail-closed regex, guard keywords | 16 live and negative fixtures | I (test of the routing model) | Low |
| Hallucinated or stale facts | gated ledgers, hardened URL check | `validate:skill-references` | I for format; D for truth | Medium: rows are self-asserted; `/stable/` moves |
| Incorrect approval recommendation | verdict definition travels with the word | eval S11 | D + eval | Medium |
| Live-guard checklist drift | companion bundling | CB11 | I | Low |
| Circular delegation, unbounded cost | maestro plan-only, at most four owners | none | D | Medium: no budget enforcement |
| Correlated specialist failures, false consensus | no-voting rule | none | D | Medium: one model family |
| Context7 data exposure | no private data in queries (research process) | none | D | Low |
| Manipulated Jira or Confluence records | human bridge; capsule gate on writes | capsule rules | P | Medium |
| Invalid audit evidence | "not certification" definition | none | D | Medium |
| Cross-tenant collection | — | none | open (O-10) | Medium |

## 9. Behavioral evaluation

15 scenarios covered missing version, contradictory evidence, incomplete Hubble data, a wrong policy assumption, healthy BGP with a dead VIP, a ClusterMesh cascade, an upgrade regression, an encryption sign-off request, an Istio misroute, no tools, a poisoned handoff, an unauthorized production change, a malicious Jira ticket, an Istio and Cilium L7 conflict, and a managed distribution.

**Result: 15 of 15 passed independent grading.**

- No fabricated observation; no C3 to C5 action framed as the agent's own.
- Every insufficient-evidence case returned `needs-review`.

Caveats:

- **Contamination:** S07, S13 and S15 are marked contaminated, because their runner saw the expected behaviors.
- **Fidelity:** one response file was retyped by its runner rather than copied.
- **Runner context:** each runner answered two to four scenarios in one context.
- **Same family:** the responses came from the model family that wrote the board.
- **Weak tests:** the scenarios signal their traps, and boilerplate disclaimers can satisfy several rubric items.

Responses and grading are in `tests/fixtures/cilium-board-evals/results/`.

## 10. Value: per-agent measurement path

All values are hypothetical. No baseline exists in this repository.

| Agent | Business owner | Measurable outcome | Minimum to keep investing |
| --- | --- | --- | --- |
| network-policy-review | Security architect | Fewer policy-rollout incidents; substantiated encryption claims | Named reviewer; at least two documented prevented bad rollouts per half-year |
| dataplane-diagnostics | SRE lead | p95 time to defensible diagnosis, used mainly in post-incident and regression review rather than live at 3 a.m. | At least 8 real uses; `needs-review` rate under 50% |
| lifecycle-readiness | Release manager | Fewer failed or rolled-back Cilium changes | At least one real upgrade reviewed per minor release, with checklist hits and misses recorded |
| external-connectivity-review | Network and platform leads | Fewer address-ownership outages | Used on at least three real changes with peer-side evidence supplied |
| maestro | Platform lead | Owner-change rate on routed requests | Needs a routing log, which does not exist; otherwise remove |

KPI revisions:

- **Add:** the `needs-review` rate per specialist, which measures the cost of the evidence bias.
- **Drop:** "complete evidence fields", which measures form rather than value.
- **Re-base:** "exposures found" needs a denominator.
- **Retire:** "false-negative rate", until a labeled corpus exists.
- **Stop calling it ROI:** without a defined cash path, report recovered capacity instead.

## 11. Repository audit

- **Location.** The design record moved from `.claude/workflow/cilium-board/` to `docs/architecture/cilium-board/`. The old path is excluded from the Jekyll site and the npm package, is Claude-specific in a multi-harness repository, and sits next to `.claude/workflows/` (executable workflows). The new location follows the precedent of `docs/architecture/atlassian-board.md`. It is tracked, published and harness-neutral. No sensitive data was found by the reviewers' regex sweep. Raw research transcripts were not committed; only the scenarios, responses, grading and scripts were.
- **CI.** The Rust `Gate` job failed on advisory RUSTSEC-2026-0543 (`chrono` unmaintained), which is not this change's cause. The existing fix from PR #245, commit `4a47a4f6`, was cherry-picked. Locally: fmt, clippy and 1,129 tests pass; `cargo-deny` is not installed here, so CI confirms it.
- **Backward compatibility.** The retained policy agent id is unchanged, but its scope changed: egress-gateway SNAT and failover moved out, and it is now static-review only. Consumers that relied on egress-gateway review from it must route to the external-connectivity specialist. The live guard is bumped to 0.2.0.
- **Gates run.** See the pull request; every applicable gate was run after these changes.

## 12. Architecture acceptance matrix

| Gate | Result | Basis |
| --- | --- | --- |
| Technical correctness | CONDITIONAL PASS | Both P0s fixed and re-verified; no contradicted fact; managed-distribution facts and some pages not re-fetched (O-4) |
| Agent specialization | CONDITIONAL PASS | Dual-owner keywords removed; maestro plan-only; ClusterMesh and egress-gateway splits remain, pending decisions O-1 and O-2 |
| Skill quality | PASS | Meets or exceeds AgentCore on 10 of 12 dimensions; worse on context cost |
| Reference integrity | CONDITIONAL PASS | Gated, hardened, deduplicated pointers; `VERIFIED` is self-asserted and `/stable/` URLs move |
| Security | CONDITIONAL PASS | No P0; the P1s fixed or bounded; Kiro and Codex prompt-only, cross-tenant collection open |
| Production safety | PASS | No write tools on any Cilium adapter; zero mutation attempts in 15 graded runs |
| Jira integration | CONDITIONAL PASS | Existing skill and governance reused; human bridge, no machine contract |
| Confluence integration | CONDITIONAL PASS | Same |
| Repository compatibility | PASS | All applicable gates pass, listed in the pull request |
| Business value | INSUFFICIENT EVIDENCE | Nothing measured; measurement path defined for four agents, none for the maestro |
| Maintainability | FAIL | No named owner or budget for the per-minor refresh (1.5 to 4.5 engineer-days estimated) |
| Adversarial resilience | CONDITIONAL PASS | 15 of 15 evals and all evasion probes pass; same-family caveat |
| Architectural evidence | PASS | Durable, published location; seeded, reproducible scripts; corrections recorded in place |

By the review's own rule, the maintainability FAIL and the business-value INSUFFICIENT EVIDENCE block acceptance as written.

## 13. Decisions that belong to the repository owner

| # | Decision | Options | Recommendation |
| --- | --- | --- | --- |
| O-1 | External-connectivity scope | (a) Keep five subsystems. (b) Narrow to BGP, LB-IPAM and L2, returning egress gateway and ClusterMesh to the policy agent and the Kubernetes architecture agent. (c) Ship it as a skill only. | (b). The independent reviewer and the sensitivity result agree that the five-way merge is fragile; BGP, LB-IPAM and L2 are the genuinely unowned surface. |
| O-2 | Maestro | (a) Keep it plan-only. (b) Remove it and keep `cilium-review-routing` as a skill the Kubernetes maestro loads. | (a), only with a routing log; otherwise (b). **Dissent:** the architecture reviewer recommends (b) now. |
| O-3 | Encryption ownership | Keep it in the policy agent's decision, or keep the reference but drop confidentiality from the decision statement. | Drop it from the statement and keep the reference. |
| O-4 | Managed distributions | Commission an orchestrator-verified reference for GKE, AKS and EKS, or keep "ask the vendor". | Commission it; it is the most common real deployment. |
| O-7 | Cross-entry routing fixture | Teach the generator cross-provider domains, or hand-tune the Kubernetes taxonomy. | Before `stable`. |
| O-9 | Maintenance owner | Name an owner and a per-minor budget, or mark the board `experimental`. | Required for acceptance. |
| O-10 | Collection targeting | Require `--context` and a namespace on every collection command, or not. | Require them, and add a gate rule. |

## 14. Verdict

**RECOMMEND ACCEPTANCE AFTER SPECIFIED REMEDIATION.**

The remediation is:

1. Name the maintenance owner and budget (O-9).
2. Decide O-1 and O-2. If you choose the narrowed topology, revise ADR-0006's decision 1 before accepting.
3. Fix O-10.
4. Treat business value as unproven, and accept the board as a measured pilot at `beta`, not as `stable`.

If you decline O-1(b) and O-2 and keep five agents without a routing log, the architecture reviewer's **REDESIGN** recommendation is the better-supported position, and I would not argue against it.
