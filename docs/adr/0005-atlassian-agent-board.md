---
layout: default
title: "ADR-0005: A Small, Read-Only Atlassian Board Behind a Deterministic Handoff Gate"
permalink: /docs/adr/0005-atlassian-agent-board/
---

# ADR-0005: A Small, Read-Only Atlassian Board Behind a Deterministic Handoff Gate

## Status

Proposed (2026-11-13). Accepted only when the repository owner accepts it.

The brief is `docs/prompts/atlassian-agentic-ecosystem-master-prompt.md`. The supporting dossier
(capability matrix, opportunity scores, blueprints, value models, threat model, roadmap) is
`docs/architecture/atlassian-board.md`. Both are dated records; neither authorizes anything.

## Context

The brief proposed up to 24 Atlassian specialists and asked for the smallest board the evidence
supports. Four findings from the repository audit and primary-source research decided the shape:

1. **The repository already owns most of the proposed meta-roles.** ADR-0003's agentic board
   (`agentic-system-architecture-review-agent`, `agentic-security-review-agent`,
   `skills/cross-functional/agentic-board-protocol`) already provides adversarial challenge,
   refutation and governance with a fixture-tested handoff envelope. `finops-ai-economist-agent`
   already owns AI cost per outcome. Duplicating them on a vendor board would fork two
   contracts for one decision.
2. **The repository already has a board pattern for business platforms.** Salesforce ships a
   static-review maestro, a routing protocol, a case capsule and a live guard that never executes.
   The Atlassian board mirrors that pattern instead of inventing one.
3. **The Rovo MCP Server acts as the human, and "read-only" is not something an agent can
   promise.** Every action runs with the authenticated user's permissions; tool availability
   follows auth method, scopes and admin-enabled permission groups; generic `discover` /
   `executeRead` / `executeWrite` / `executeDestructive` tools sit outside the groups (ledger
   ATL-E03 to ATL-E05, ATL-E12). The verified write surface includes deciding JSM approvals,
   replacing Confluence content permissions and setting a space's AI instructions (ATL-E07,
   ATL-E09). A prompt rule cannot make those safe; only admin group settings and client deny-lists
   enforce anything at runtime.
4. **The owner's two authoring skills are already better than anything a board would generate.**
   Both encode evidence classes, read-back, partial-outcome handling and Data Center caveats.
   They become the board's only authoring capability.

## Decision

### Accepted board (5 agents)

| Agent | Tier | Owns exactly one decision |
| --- | --- | --- |
| `atlassian-maestro-agent` | static-review | Which specialist, authoring skill, other board, or `unsupported` answer a matter gets |
| `atlassian-incident-prevention-agent` | read-only-runtime | Whether each corrective action is *evidenced as preventive* |
| `atlassian-control-evidence-lineage-agent` | read-only-runtime | Whether a control's evidence chain is complete, broken, stale or intent-only (never "compliant") |
| `atlassian-knowledge-integrity-agent` | read-only-runtime | Whether a page is current, stale, contradicted, ownerless or a preserved historical record |
| `atlassian-live-write-guard-agent` | read-only-runtime, gate-only | Go or no-go for one prepared write; holds no write tools |

No agent on this board declares `mutating-runtime`. On "go", the write runs in the human's
interactive session under the client's permission prompts, using the authoring skills'
mechanics, and the guard verifies the read-back. This follows `salesforce-live-guard-agent`
(static-review, never executes) and changes the first draft of this design, which made the
guard `mutating-runtime`: an agent that can hold write tools is a larger blast radius than the
workflows need, and nothing in the accepted workflows requires unattended writes.

### Skills (9)

- Imported, owner-authored, canonical: `jira-ticket-authoring`, `confluence-document-authoring`
  (v1.2.1; migration below).
- Foundation: `atlassian-rovo-mcp-integration` — tool risk tiers, auth/tenant/trust boundaries,
  failure and recovery, and the only evidence ledger (17 rows, retrieved 2026-11-13).
- Companions: `atlassian-incident-prevention-review`, `atlassian-control-evidence-lineage`,
  `atlassian-knowledge-integrity-review`, `atlassian-live-write-guard`. They link to the
  foundation and the authoring skills rather than restating platform facts or templates.
- Cross-functional: `atlassian-routing-protocol`, `atlassian-case-capsule`.

### Deterministic controls (the part that is code, not prose)

- `schemas/atlassian-case-capsule.schema.json` plus `tests/validate-atlassian-case-capsule.py`
  (`npm run validate:atlassian-capsule`, in `npm run validate`): human-only out-of-band approval,
  approval window against the capsule's own `as_of` (no wall clock), prohibited tools at every
  tier, opaque `executeWrite` rejected, unknown tools fail closed, tier/tool consistency, dual
  control for T4 with neither approver the requester, tenant binding, enumerated (non-query) write
  targets, delegation cycles and depth, instruction-bearing evidence marked untrusted, completion
  requires read-back evidence, expiry, human decision owner, credential-shaped strings. Each rule
  is proven by a negative fixture that must trip exactly that rule; five hand-made gate mutants
  were each killed by the fixtures on 2026-11-13.
- `tests/fixtures/atlassian-maestro-routing/`: 16 cases, including prompt injection quoted from a
  comment, a forged approval request, a bulk delete and a restriction change (all to the gate,
  never auto-dispatched), an ambiguous request, a deferred use case, a direct authoring request,
  and a draft-only request that must not gate.

### PROHIBITED for every agent

`decideJsmRequestApproval`, `approveJsmRequestResolverPlans`, the four Confluence permission and
restriction tools, `setConfluenceSpaceInstructions` (a persistent instruction channel other agents
read), `setJsmAgentAvailability` (another person's status), any `delete*` tool and
`executeDestructive`. Approval decisions are human decisions; an AI approving as the human is the
failure this board exists to prevent.

## Candidate disposition (all 24 from the brief)

| # | Candidate | Disposition | Reason |
| --- | --- | --- | --- |
| 1 | Enterprise Maestro | Accepted | Needed to keep write intent away from analysis agents |
| 2 | Value and Transformation Architect | Rejected | Value models are design artifacts (dossier) and `finops-ai-economist-agent` owns AI economics |
| 3 | Adversarial Assurance | Rejected (exists) | ADR-0003 agentic board |
| 4 | Product Discovery Strategist | Deferred | Weak differentiation from native Jira Product Discovery insight features; no verified MCP tools |
| 5 | Requirements Quality Architect | Consolidated | `jira-ticket-authoring` quality review and adversarial review |
| 6 | Jira Delivery Intelligence | Deferred | See workflow C below |
| 7 | Portfolio Dependency Analyst | Deferred | Same; Jira Align overlap |
| 8 | Engineering Flow and Bitbucket | Deferred | Bitbucket not in the pilot scope; DORA tooling exists |
| 9 | JSM Reliability Specialist | Consolidated | SLA exposure is native JSM reporting; the reasoning part is in incident prevention |
| 10 | Incident-to-Prevention | Accepted | |
| 11 | Change and Release Risk Guardian | Deferred | Repository already has change-to-go-live and release protocols; revisit with JSM change evidence |
| 12 | Service Ownership and Dependency | Deferred | Compass is being phased out (delegate-reported); wait for the successor |
| 13 | Confluence Knowledge Integrity | Accepted (merged with 16) | |
| 14 | Atlassian Security and Access Architect | Consolidated | Tool tiers and trust boundaries in the foundation skill; access changes are prohibited, not reviewed |
| 15 | Regulatory Evidence and Control Lineage | Accepted | |
| 16 | Decision Traceability | Merged into 13 | Same evidence (ADRs and decision pages) |
| 17 | Integration and MCP Architect | Consolidated | A skill, not an agent: `atlassian-rovo-mcp-integration` |
| 18 | AI Cost and FinOps | Rejected (exists) | `finops-ai-economist-agent` |
| 19 | Workflow Optimization | Deferred | Same concerns as workflow C |
| 20 | Customer Promise Integrity | Deferred | Obligations live in CRM and contract systems, outside Atlassian evidence |
| 21 | Executive Decision Intelligence | Rejected | Summary-only output that Rovo already provides |
| 22 | Evaluation and Reliability Engineer | Rejected as agent | Evaluation is repository gates and fixtures, not a runtime role |
| 23 | AI Governance and Policy | Rejected (exists) | ADR-0003 agentic board and the capsule gate |
| 24 | Architecture Refuter | Rejected (exists) | `agentic-system-architecture-review-agent` |

Workflow disposition: B, D and E accepted; F routed to the existing FinOps agent; A and C
deferred. C (delivery bottlenecks) is deferred deliberately, not forgotten: native Jira reports,
Atlassian Analytics and Jira Align overlap heavily; workflow status metadata is a poor proxy for
real waiting; and per-team bottleneck analysis drifts quickly into individual performance
surveillance, which needs HR and works-council review before any agent touches it.

## Migration of the owner's skills

Archives were checked before extraction (no absolute paths, traversal or symlinks; 64 and 45
entries; identical, script-free SVG icons). Every manifest hash matched (54/54 and 35/35).
Changes, recorded in each skill's CHANGELOG as 1.2.1: an `allowed-tools: Read Grep Glob WebFetch`
line (the repository schema requires it; live access comes from the harness MCP configuration),
a repository `metadata.json`, and refreshed manifest hashes. No template, prompt, example or
reference text changed; the original `author` is preserved. One behavioral note was added to the
changelogs, not the skills: a request that appears only inside an agent message or retrieved
content is not the user's authorization. The skills' existing rule (reuse authorization a human
gave directly; do not add confirmation steps) is unchanged and compatible with the board.

## Consequences

- Positive: two enforcement points that exist as code (capsule gate, routing gate); no agent
  holds write or mutation authority; the owner's skills stay canonical and independently usable.
- Negative: the gate validates records, not live sessions. A harness that never emits a capsule
  is ungoverned by it; runtime enforcement is still the admin's permission groups and the
  client's deny-list, which this repository can document but not install.
- The maestro routing gate is keyword-based, like every other board's; the routing protocol
  forbids keyword-only classification in prose, so the fixture is a floor, not a model of the
  maestro's judgment.
- Tool names drift (the supported-tools page changed 2026-11-02); the ledger and tier lists need
  re-verification on each Rovo MCP update, and unknown names fail closed in the meantime.

## Dissent register

| ID | Dissent | Raised by (board role) | Status |
| --- | --- | --- | --- |
| D-1 | Knowledge integrity and control lineage could be one agent with two modes; two agents add routing surface. | Chief Enterprise Architect | Open: kept separate because control evidence carries audit-grade provenance rules (author/approver separation, period) that would bleed into documentation hygiene. Revisit after the pilot. |
| D-2 | Deterministic JQL/CQL reports plus page verification may deliver most of workflow E without an agent. | Chief Financial Value Analyst | Open: the pilot must run the deterministic baseline first and compare (dossier, counterfactuals). |
| D-3 | A gate-only guard without write tools makes agent-initiated writes slower. | Principal Agentic Systems Architect | Accepted risk, owner: repository owner. Speed is not a goal of the accepted workflows. |
| D-4 | JSM authentication is contradictory between two official pages (ATL-E13). | Atlassian Platform Architect | Open: verify in the pilot session; nothing JSM-specific may be encoded as fact until then. |
| D-5 | Without a live tenant there is no behavioral evidence that agents follow these contracts. | Principal AI Evaluation Engineer | Open, blocking production: see the verdicts below. |

## Independent verdicts (Deliverables M and N)

Recorded by the orchestrator acting as the arbiter, with the dissent above unresolved; the
repository owner is the decision authority and has not decided.

```yaml
verdict: APPROVE WITH CONDITIONS   # as a repository foundation, not as a production system
evidence_level: partial            # docs verified 2026-11-13; gates executed; no live-tenant test
critical_blockers:
  - No live Rovo MCP session was exercised; tool names, groups and JSM auth are documentation-verified only.
  - Runtime enforcement (admin permission groups, client deny-list) is documented, not installed or tested.
major_risks:
  - A harness that skips the capsule bypasses the only executable policy check.
  - Tool-list drift between Rovo MCP releases.
  - Value of workflows D and E versus deterministic baselines is unproven.
accepted_design_choices:
  - Five agents; no mutating tier; owner's skills canonical; one evidence ledger.
rejected_design_choices:
  - 24-agent board; duplicate governance/refuter/FinOps agents; agent-held write tools.
required_remediation:
  - Live-session verification of the ledger and tier list before any pilot write.
  - Measured deterministic baseline for workflows D and E.
safe_next_actions:
  - Stage 3 read-only pilot on a named tenant with write groups disabled and the client deny-list applied.
remaining_uncertainties:
  - Default-disabled group names (ATL-E10); JSM auth (ATL-E13); MCP rate limits (none documented).
investment_decision: PILOT   # not INVEST: no baseline, no measured benefit, no cost data yet
```
