---
layout: default
title: "Atlassian Agent Board: Design Record"
permalink: /docs/architecture/atlassian-board/
---

# Atlassian Agent Board: Design Record

This document records why the Atlassian board ships what it ships, what it defers, and how much of each claim is evidence rather than intent. It is a dated design record "as of 2026-11-13", not an authorization surface: nothing described here approves, executes or deploys anything. Counts are stated as of that date and are not maintained afterward; nothing here claims that counts elsewhere in the repository will match.

Evidence vocabulary. "VERIFIED" means a row in the evidence ledger (`skills/atlassian/atlassian-rovo-mcp-integration/references/evidence-ledger.md`), cited by ID such as ATL-E07. The ledger is documentation verification, not a live tenant test. "Delegate-reported" means a research delegate read it from an Atlassian-hosted page or a search snippet and the orchestrator did not independently verify it. "UNVERIFIED" means no source read supports it. "CONFLICT" means two official pages disagree.

Board members: `atlassian-maestro-agent` (routing only), three read-only specialists (`atlassian-incident-prevention-agent`, `atlassian-control-evidence-lineage-agent`, `atlassian-knowledge-integrity-agent`) and `atlassian-live-write-guard-agent`. The guard is `read-only-runtime`, holds no write tools and is gate-only: on a go, the write runs in the human's interactive session under client permission prompts, and the guard then reads back the result. No agent on this board is `mutating-runtime`. The decision record for the board is ADR-0005, referenced by the agent charters.

## 1. Atlassian capability matrix (Deliverable D)

Product availability does not imply API availability, and API availability does not imply authorized access. Tool lists change; re-run `discover` in the actual session before relying on any tool name (ledger header).

| Product | Lifecycle status | MCP support | Auth | Read/write | Limits | Board use | Confidence |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Jira | Cloud; no retirement notice in sources read (lifecycle statement itself UNVERIFIED) | Groups `read_jira`, `write_jira`, `search_jira` (ATL-E06) | OAuth 2.1 primary; API token only if admin enabled (ATL-E11); acts with the user's permissions (ATL-E12) | Read and write; deletes PROHIBITED on this board | HTTP 429 with `Retry-After`; per-issue 20 writes per 2 s and 100 per 30 s (ATL-E15, ATL-E16); retry only if idempotent (ATL-E17) | Used | VERIFIED ATL-E06, E11, E12, E15 to E17 |
| Jira Service Management | Cloud; no retirement notice in sources read | `read_jsm`, `write_jsm`, including `updateJsmOpsAlert` (ATL-E09) | CONFLICT (ATL-E13): authentication page says API token only; supported-tools page says OAuth 2.1 and API token. Neither is encoded | Read for evidence; writes are customer-visible (T4); approval decisions PROHIBITED | No numeric MCP limit found (UNVERIFIED) | Used (read evidence; writes only via the guard) | Tools VERIFIED ATL-E09; auth CONFLICT ATL-E13 |
| Confluence | Cloud; no retirement notice in sources read | `read_confluence`, `write_confluence`, `search_confluence` (ATL-E07, ATL-E08) | As Rovo MCP Server | Read and write; permission, restriction and space-instruction tools PROHIBITED | REST v2 update needs `version.number` plus one; the spec documents 409 only for content approvals, so the stale-version error status is UNVERIFIED (delegate-reported) | Used | Tools VERIFIED ATL-E07, E08; limits delegate-reported |
| Rovo MCP Server | Endpoint `https://mcp.atlassian.com/v2/mcp` over HTTP (ATL-E01); legacy SSE unsupported after 30 June 2026 (ATL-E02); GA or beta of the server as a whole UNVERIFIED | `discover` plus `executeRead`, `executeWrite`, `executeDestructive` (ATL-E03); tool set depends on auth, scopes, admin groups (ATL-E04, ATL-E05) | OAuth 2.1; API token if admin enabled (ATL-E11) | Split by risk tier | Some tools consume Rovo credits (delegate-reported); numeric request limit UNVERIFIED | Used (only integration surface) | VERIFIED ATL-E01 to E05, E11, E12, E14 |
| Jira Product Discovery | UNVERIFIED | No tool group found in sources | Not researched | Not researched | Not researched | Deferred (product discovery routes to "unsupported") | UNVERIFIED |
| Bitbucket Cloud | Cloud (lifecycle not researched) | `read_bitbucket`, `write_bitbucket`; write set includes merge, approve and pipeline tools (delegate-reported) | Not separately checked | Read and write | Not researched | Deferred (possible technical-evidence source for control lineage) | Delegate-reported |
| Teamwork Graph | Not researched | `read_teamwork_graph`, `write_teamwork_graph` (delegate-reported) | Scopes `read:all:twg`, `write:all:twg` (delegate-reported) | Read and write | 1 to 10 Rovo credits per enriched call (delegate-reported) | Deferred | Delegate-reported |
| Compass | Phase-out announced in a community post by an "Atlassian Team" author: closed to new customers 13 May 2026, access ends 31 December 2027; migration page and blog not fetched | No Compass-specific group; appears only as a Teamwork Graph entry point (delegate-reported) | n/a | n/a | n/a | Out of scope | Delegate-reported from community and search snippets; not independently verified |
| Jira Align | Not researched | `read_jira_align`, `write_jira_align`; OAuth 2.1 only (delegate-reported) | OAuth 2.1 | Read and write | Not researched | Deferred (portfolio overlap with Workflow C) | Delegate-reported |
| Atlassian Analytics | Enterprise-tier product per a licensing-page snippet; a roadmap snippet mentions dashboards moving to another app (not fetched) | No tool group found | n/a | n/a | n/a | Out of scope (overlaps Workflow C reporting) | Delegate-reported |
| Opsgenie | End of sale 4 June 2025 and end of support 5 April 2027 per a search snippet of an Atlassian changes post | No Opsgenie group; JSM alert tools exist (ATL-E09) | n/a | n/a | n/a | Out of scope | Delegate-reported from a search snippet; not independently verified |
| Forge | Excluded from the ledger's scope | Not researched | Not researched | Not researched | Points-based rate limits for Forge, Connect and OAuth 3LO apps (delegate-reported) | Out of scope | UNVERIFIED |

Group names `delete_jira` and `manage_jira` as "disabled by default" are UNVERIFIED (ATL-E10): the retrieved excerpt did not bind the label to group names. Atlassian's pages give no prompt-injection guidance; they say to use least privilege, review high-impact changes and monitor audit logs (ATL-E14).

## 2. Enterprise opportunity portfolio (Deliverable E)

Nine candidates were scored: the six master-prompt workflows A to F and three section-15 extras judged strongest: G (enterprise service ownership gaps), H (change failure pattern discovery) and I (architecture decision drift).

Weights and why. Business impact (20) and the three criteria the board cannot work around (evidence availability inside Atlassian, risk inverted, differentiation, 15 each) carry most weight. Measurability (15) is weighted equally because an unmeasurable workflow cannot be piloted honestly. Frequency and technical feasibility get 10 each because a rare but costly or hard-but-valuable workflow can still be right. Risk is inverted: 5 means low security or operational risk. The weighted total is the sum of score times weight divided by 5, giving a figure out of 100.

These scores are the board's judgment, not measurements. No customer data, baseline or cost figure was used. Differences of a few points are within judgment noise, so the accept and defer decisions below rest on the stated constraints, not on rank alone.

| ID | Workflow | Impact (20) | Freq (10) | Evidence (15) | Feasibility (10) | Risk inv. (15) | Differentiation (15) | Measurability (15) | Total |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| B | Incident-to-prevention | 4 | 4 | 4 | 4 | 4 | 4 | 4 | 80 |
| E | Knowledge and decision integrity | 3 | 5 | 4 | 4 | 4 | 3 | 4 | 75 |
| D | Audit evidence and control lineage | 4 | 3 | 4 | 3 | 3 | 4 | 4 | 73 |
| A | Customer promise-to-delivery | 5 | 4 | 2 | 2 | 3 | 4 | 3 | 68 |
| H | Change failure patterns (extra) | 4 | 3 | 3 | 3 | 4 | 3 | 3 | 67 |
| C | Delivery bottleneck intelligence | 4 | 5 | 3 | 3 | 2 | 2 | 3 | 62 |
| F | AI cost and governance | 3 | 3 | 2 | 3 | 4 | 2 | 4 | 60 |
| G | Service ownership gaps (extra) | 3 | 3 | 2 | 3 | 4 | 3 | 3 | 60 |
| I | Architecture decision drift (extra) | 3 | 3 | 3 | 3 | 4 | 3 | 2 | 60 |

Decisions:

- **Accepted: B, D, E.** They rank first, third and second, and each has a decision the board can own that no native feature owns (per-action preventive verdict, per-control chain verdict, per-page integrity verdict). Each maps to one specialist.
- **A deferred.** The deciding fact is not its score: contract and obligation evidence usually lives in CRM and contract systems, not in Atlassian, so the system could not distinguish a real obligation from an inferred promise.
- **C deferred.** Native Jira reports, Atlassian Analytics and Jira Align overlap it, workflow metadata can be unreliable, and the output risks becoming individual-performance surveillance. The routing protocol answers "unsupported (deferred)"; routing fixture 14 asserts only that no specialist is routed.
- **F routed** to the existing `finops-ai-economist-agent` (routing fixture 05); no new agent.
- **H and I** overlap B and E respectively and are treated as possible extensions, not agents. **G** depends on ownership data whose best-known catalog (Compass) is being phased out (delegate-reported).

## 3. Board topology and routing (Deliverable H)

```mermaid
flowchart TD
    H["Human requester and decision owner"] --> M["atlassian-maestro-agent: classify and route only"]
    M -->|"corrective actions after incidents"| IP["atlassian-incident-prevention-agent"]
    M -->|"control evidence chain"| CE["atlassian-control-evidence-lineage-agent"]
    M -->|"stale or contradicted pages"| KI["atlassian-knowledge-integrity-agent"]
    M -->|"plain authoring"| AU["jira-ticket-authoring and confluence-document-authoring skills"]
    M -->|"AI cost"| FN["finops-ai-economist-agent"]
    M -->|"bottleneck, promise, discovery"| UN["unsupported: deferred"]
    M -->|"ambiguous"| CL["unclassified: one clarifying question"]
    IP --> D["Drafts and proposals, T2"]
    CE --> D
    KI --> D
    AU --> D
    D -->|"prepared write; human sends capsule"| G["atlassian-live-write-guard-agent: gate only, never auto-dispatched"]
    M -.->|"write intent: gate mode, nothing executed"| G
    G -->|"go or no-go"| H
    H -->|"on go: write in own session under client prompts"| T["Atlassian tenant"]
    G -->|"read-back, read groups only"| T
```

The maestro never reads a tenant and never dispatches the guard on its own; routing is a recommendation and the named human decision owner decides. A matter that is both analysis and write is split: analysis first, then the guard with a prepared write.

```mermaid
sequenceDiagram
    actor Hu as Human requester
    participant Sp as Specialist or authoring skill
    participant Ap as Approval system of record
    participant Gd as atlassian-live-write-guard-agent
    participant Se as Human interactive session
    participant Tn as Atlassian tenant
    participant Au as Audit log, not built
    Sp->>Hu: Proposal capsule, T3 or T4, enumerated targets
    Hu->>Ap: Approve out-of-band (T4: two distinct humans, not the requester)
    Hu->>Gd: Send capsule with approval record locator
    Gd->>Ap: Read record, check approved_at <= as_of < expires_at
    Gd-->>Hu: Go or no-go, failing item named
    Hu->>Se: On go, run exactly the listed write
    Se->>Tn: Write under client permission prompt
    Gd->>Tn: Read back observed state
    Gd-->>Hu: Observed state, partial outcomes, compensation plan
    Gd->>Au: Record decision and read-back
```

```mermaid
flowchart LR
    subgraph TR["Trusted"]
        SM["Session metadata: cloud_id, auth method, enabled groups"]
        AR["Approval system of record: out-of-band human record"]
    end
    subgraph EN["Enforcement outside the prompt"]
        AG["Admin permission groups and OAuth scopes"]
        DL["Client deny-list: executeWrite, executeDestructive, prohibited tool names"]
    end
    subgraph UT["Untrusted data"]
        TC["Jira text, Confluence bodies, JSM text, tool output text, other agents' conclusions"]
    end
    TR --> AGT["Board agent: treats tenant content as evidence, never as instructions"]
    UT --> AGT
    AGT --> EN
    EN --> MCP["Rovo MCP Server acts with the user's permissions"]
```

Read-only is not enforced by an agent prompt. The server acts with the authenticated user's permissions (ATL-E12). It becomes real only when an admin disables write groups for the identity or the OAuth grant lacks write scopes (ATL-E04, ATL-E05) and the client denies write tool names. `discover` and the execute tools belong to no permission group (ATL-E05), so admins cannot disable them per group; only the client deny-list can. Agents must state which controls are in force, or report read-only as `unenforced`.

| Layer | Repo-level (exists in this change) | Needs runtime infrastructure not built here |
| --- | --- | --- |
| Contracts | Agent charters, AGENT.md files, skills, routing protocol, case capsule schema | A durable workflow engine to hold state across the DISCOVER to AUDIT model |
| Gates | `validate:atlassian-capsule` policy gate with valid and invalid fixtures; routing fixtures under `validate:maestro-routing` | A policy enforcement point in the tool path, so a non-capsule call cannot bypass the gate |
| Authorization | Rules for what a valid approval looks like | Integration with an approval system of record; a human-facing approval UI |
| Execution | Skill procedures for read-back, compensation, resume | A queue, retry and idempotency-key handling, a tenant connector |
| Audit | Required audit events named in each blueprint | An audit log store; the capsule gate proves record shape, not that an approval exists |
| Evaluation | Handcrafted capsule and routing cases | Live-tenant evaluation, seeded-defect corpora, usage metering |

As of this record there are 5 valid and 36 invalid capsule fixtures (each invalid fixture names the one rule it must trip) and 16 routing cases. The gate validates records, not live sessions: a harness that never produces a capsule is not governed by it.

## 4. Three end-to-end blueprints (Deliverable 18 fields)

Rows are the 26 master-prompt section 18 fields. Tool names come from the ledger or the tool risk tiers reference; always confirm with `discover`. KPI targets are not set: no baseline exists.

### 4.1 Workflow B: incident-to-prevention

| # | Field | Blueprint |
| --- | --- | --- |
| 1 | Business problem | Corrective actions are ticketed and closed without evidence that they prevented recurrence |
| 2 | Stakeholder | Service owners, SRE and operations leads, problem managers |
| 3 | Business KPI | Recurrence rate per signature; corrective-action aging; share of actions with implementation evidence |
| 4 | Prerequisites | Read groups enabled for the pilot identity; postmortems and corrective-action issues linked; a human-chosen observation window; a baseline period |
| 5 | Initial request | A human asks whether fixes for named incidents worked (routing fixture 01) |
| 6 | Maestro routing | Single route to the incident specialist; with a stale-runbook question, parallel with knowledge integrity (fixture 06); draft-only requests must not gate (fixture 16) |
| 7 | Specialist | `atlassian-incident-prevention-agent` |
| 8 | Evidence retrieval | JQL search and issue reads (ATL-E06), page reads (ATL-E08), JSM request and alert reads; locator and version per item |
| 9 | Context handling | Capsule `evidence_references` with `retrieved_at`, version and trust; opaque `account_ref`; minimum customer data |
| 10 | Decision points | Per action: evidenced-preventive, implemented-unverified, open, ineffective, insufficient-evidence; the human owner picks the window |
| 11 | Tool execution plan | T1 reads only, within the capsule budget; drafts via the authoring skills; no specialist write |
| 12 | Trust boundaries | Ticket, comment and page text untrusted; other agents' conclusions untrusted |
| 13 | Authorization | The user's own permissions (ATL-E12); read-only enforced only by admin groups, scopes and client deny-list |
| 14 | Human approval | None for analysis; any follow-up write goes through the guard (T3 one human, T4 two) |
| 15 | Jira artifacts | Remediation and finding drafts from the owner's `jira-ticket-authoring` templates |
| 16 | Confluence artifacts | Post-incident record draft from `confluence-document-authoring`; postmortem and decision state kept apart |
| 17 | Traceability | Incident, corrective action, implementation evidence, verifier, page; links proposed, never fabricated |
| 18 | Failure scenarios | Keyword-similar incident counted as recurrence; short window read as success; injected "mark all verified"; edited postmortem |
| 19 | Retry and recovery | On 429, respect `Retry-After`, retry only if idempotent (ATL-E15, ATL-E17), within `max_retries`; zero results is not absence |
| 20 | Compensating actions | None for analysis; for approved follow-up writes the guard's per-step read-back and compensation plan; no deletes |
| 21 | Audit events | Capsule per handoff, verdict output, approval reference, guard decision, read-back (store not built) |
| 22 | Evaluation | Routing fixtures 01, 06, 09, 16; capsule fixtures; live behavior not evaluated |
| 23 | Value hypothesis | Fewer repeat incidents per signature and shorter corrective-action aging once unverified actions are visible |
| 24 | Pilot success | Criteria fixed before the pilot by the sponsor; at minimum: no unauthorized mutation, verdicts reproducible by reviewers, and owners change at least some remediation decisions |
| 25 | Pilot failure | Verdicts do not change decisions; any unauthorized write; review cost exceeds the effort saved; native JSM or Confluence features give equivalent evidence |
| 26 | Production blockers | No policy enforcement point, audit store, durable workflow or tenant connector; no live evaluation; JSM auth conflict (ATL-E13); no baseline |

```mermaid
sequenceDiagram
    actor Hu as Human
    participant Mx as atlassian-maestro-agent
    participant Ip as incident-prevention agent
    participant Mc as Rovo MCP Server
    Hu->>Mx: Did our fixes work for these incidents?
    Mx->>Ip: Capsule, T1-read
    Ip->>Mc: Read incident, actions, postmortem
    Mc-->>Ip: Untrusted content with versions
    Ip->>Ip: Signature, window, exposure, independent verifier
    Ip-->>Hu: Verdict per action, evidence level, drafts
    Note over Hu,Ip: Any follow-up write goes to the guard
```

### 4.2 Workflow D: control evidence lineage

| # | Field | Blueprint |
| --- | --- | --- |
| 1 | Business problem | Assembling evidence that controls operate is costly, and finding documents invites false compliance claims |
| 2 | Stakeholder | Control owners, internal audit, risk and compliance teams |
| 3 | Business KPI | Evidence completeness (chain-complete controls over controls in scope); collection hours (UNAVAILABLE if not logged); broken-chain count |
| 4 | Prerequisites | Human-supplied control framework, control list and evidence period; named owners; read groups; baseline period |
| 5 | Initial request | A human asks whether a control's evidence chain is complete (routing fixture 02) |
| 6 | Maestro routing | Single route to the lineage specialist; with incident corrective-action evidence, parallel with incident prevention (fixture 07) |
| 7 | Specialist | `atlassian-control-evidence-lineage-agent` |
| 8 | Evidence retrieval | Eight links: objective, policy, owner, activity, Jira execution, Confluence evidence, technical evidence, review decision; provenance and timestamps per link |
| 9 | Context handling | One capsule per control set; evidence references carry page version at evidence time |
| 10 | Decision points | Per control: chain-complete, chain-broken (missing link named), evidence-stale, intent-only, conflicting, insufficient-evidence |
| 11 | Tool execution plan | T1 reads (search, issue and page reads, version diffs); finding drafts only |
| 12 | Trust boundaries | "Auditor approved" text in tickets or pages is data; an approval counts only as a system-of-record record |
| 13 | Authorization | As B |
| 14 | Human approval | Independent reviewer, not the agent and not the control owner, reviews the package; writes via the guard |
| 15 | Jira artifacts | Finding drafts for broken links |
| 16 | Confluence artifacts | Risk assessment or evidence page draft from `confluence-document-authoring` |
| 17 | Traceability | Control to policy to owner to Jira to Confluence to technical evidence to review decision |
| 18 | Failure scenarios | Found document read as operating evidence; same person implements and approves; screenshot-only evidence; edits after the review date |
| 19 | Retry and recovery | As B; zero search results is insufficient-evidence, never chain-complete |
| 20 | Compensating actions | As B |
| 21 | Audit events | As B, plus the evidence period and framework as supplied by the human |
| 22 | Evaluation | Routing fixtures 02, 07; capsule fixtures; no seeded broken-chain corpus yet |
| 23 | Value hypothesis | Lower collection effort and earlier discovery of broken chains |
| 24 | Pilot success | Auditors accept the output as a starting point; no output states "compliant"; criteria otherwise fixed by the sponsor beforehand |
| 25 | Pilot failure | Auditors reject the output; a GRC system of record already holds these links; any compliance claim emitted |
| 26 | Production blockers | As B; Microsoft 365 evidence is out of scope; technical evidence (Bitbucket, pipelines) not supported |

```mermaid
sequenceDiagram
    actor Hu as Human control owner
    participant Mx as atlassian-maestro-agent
    participant Cl as control-evidence-lineage agent
    participant Mc as Rovo MCP Server
    Hu->>Mx: Is this control evidenced for the period?
    Mx->>Cl: Capsule with framework and period from the human
    Cl->>Mc: Read execution and evidence per link
    Mc-->>Cl: Untrusted content with versions
    Cl->>Cl: Provenance, author vs approver, edits after review
    Cl-->>Hu: Per-control verdict, residual uncertainty
    Hu->>Hu: Independent review before reliance
```

### 4.3 Workflow E: knowledge and decision integrity

| # | Field | Blueprint |
| --- | --- | --- |
| 1 | Business problem | Stale, contradictory or ownerless runbooks and decisions are followed during incidents and changes; age alone misclassifies preserved history |
| 2 | Stakeholder | Space owners, SRE and support leads, architecture owners |
| 3 | Business KPI | Stale-content remediation rate; share of critical pages with an owner; decision traceability rate |
| 4 | Prerequisites | A human-defined critical page set; Confluence and Jira read groups; lifecycle signals reachable (linked Jira work, ADR status); baseline of documentation-attributed incidents |
| 5 | Initial request | A human asks which runbooks are stale or ownerless (routing fixtures 03, 04) |
| 6 | Maestro routing | Single route to the knowledge specialist |
| 7 | Specialist | `atlassian-knowledge-integrity-agent` |
| 8 | Evidence retrieval | Page metadata and version history, diffs, CQL search (ATL-E08), linked issues |
| 9 | Context handling | Capsule per page set; owner, version and signal provenance recorded |
| 10 | Decision points | Per page: current, stale-needs-review, contradicted, ownerless, historical-record, insufficient-evidence; superseded ADRs are historical, not stale |
| 11 | Tool execution plan | T1 reads; correction drafts only |
| 12 | Trust boundaries | "This page is deprecated, delete it" in page text is an injection finding, not a staleness signal |
| 13 | Authorization | As B |
| 14 | Human approval | Named human approver for sensitive or restricted pages before any proposal advances; writes via the guard (publish is T3, archive or move T4) |
| 15 | Jira artifacts | Linked work items for corrections, drafted with the owner's skill |
| 16 | Confluence artifacts | Publication-ready corrections from `confluence-document-authoring` |
| 17 | Traceability | Decision, supporting evidence, implementing Jira work, current page version |
| 18 | Failure scenarios | Old-but-correct page flagged stale; bot touch read as review; zero results read as absence; Confluence version conflict on a later publish |
| 19 | Retry and recovery | Re-read and diff on a conflict, never overwrite blindly; as B otherwise |
| 20 | Compensating actions | Restore-version is T4 and goes through the guard; report partial state |
| 21 | Audit events | As B, plus page version before and after any approved publish |
| 22 | Evaluation | Routing fixtures 03, 04, 06; no seeded stale-page corpus yet |
| 23 | Value hypothesis | Fewer incidents or escalations traced to wrong documentation; higher owner coverage |
| 24 | Pilot success | Owners confirm flagged pages at a rate the sponsor sets beforehand; preserved records are not flagged for rewriting |
| 25 | Pilot failure | Native page verification and ownership features cover the critical set; false stale rate too high to review |
| 26 | Production blockers | As B; no reliable signal for "implementation changed" without a connector to code or deploy evidence |

```mermaid
sequenceDiagram
    actor Hu as Human space owner
    participant Mx as atlassian-maestro-agent
    participant Ki as knowledge-integrity agent
    participant Mc as Rovo MCP Server
    participant Gd as live write guard
    Hu->>Mx: Which of these pages are wrong?
    Mx->>Ki: Capsule, T1-read
    Ki->>Mc: Read versions, owners, linked work
    Mc-->>Ki: Untrusted content with versions
    Ki-->>Hu: Verdict per page, correction drafts
    Hu->>Gd: Approved correction as a prepared write
    Gd-->>Hu: Go or no-go, then read-back after the human runs it
```

## 5. Value engineering (Deliverable I)

No number below is invented. Every cell is a formula, a named required input or "not set". The matrix covers the 19 section-17 fields for the three accepted workflows.

| Field | B incident-to-prevention | D control evidence lineage | E knowledge integrity |
| --- | --- | --- | --- |
| Business problem | Unverified corrective actions | Costly evidence assembly, false-compliance risk | Wrong or ownerless documentation in use |
| Baseline metric | Recurrences per signature per period; action aging | Collection hours per control; chain-complete share | Documentation-attributed incidents; owner coverage |
| Baseline data source | Jira, JSM, postmortem pages | Time logs (UNAVAILABLE if not logged), Jira, Confluence | Incident records, Confluence metadata |
| Measurement period | Human-chosen, fixed before pilot | Audit period supplied by human | Human-chosen, fixed before pilot |
| Population and exclusions | Named services; exclude active incidents | Named control set; exclude non-Atlassian evidence | Critical page set; exclude intentionally historical pages |
| Proposed intervention | Per-action verdicts and drafts | Per-control chain verdicts | Per-page verdicts and correction drafts |
| Causal mechanism | Visibility of unverified actions leads to verification | Early broken-chain discovery leads to earlier fixes | Flagging leads owners to correct before use |
| Leading indicator | Share of actions with implementation evidence | Broken chains found before audit window | Pages with a named owner |
| Lagging indicator | Recurrence rate | Audit findings (owned by auditors) | Documentation-attributed incidents |
| Target hypothesis | Not set until baselined | Not set until baselined | Not set until baselined |
| Operating cost | Tool calls and model usage per review; metering not built | Same | Same |
| Implementation cost | Pilot integration and admin setup; unestimated | Same | Same |
| Human review cost | Reviewer hours per verdict set (measure) | Independent reviewer hours | Owner confirmation hours |
| Maintenance cost | Tool-list and skill upkeep; unestimated | Same, plus framework changes | Same |
| Expected operational downside | False "ineffective" or false "preventive" verdicts | False chain-complete read as compliance | False stale flags; review fatigue |
| Measurement uncertainty | Recurrence is rare, so small samples | Small control counts | Documentation-attribution is subjective |
| Control or comparison group | Services or periods not reviewed by the board | Controls handled by the existing process | Spaces handled by native verification |
| Stop condition | Verdicts do not change decisions | Auditors reject the output | Flag accuracy too low to act on |
| Expansion condition | Measured benefit exceeds measured cost in the pilot | Same | Same |

Value formula (section 17.1): net annual benefit = verified annualized operational savings + verified incremental economic benefit - annual operating costs - annualized implementation costs - expected operational loss. When an input is unavailable the formula stays and the input is listed as required; a missing input is never replaced by an estimate. Time saved is not cash saved: a realization mechanism (headcount, avoided external spend, avoided loss) must be documented before hours count as benefit.

Counterfactual (section 17.3):

| Option | B | D | E |
| --- | --- | --- | --- |
| A. Existing process | Problem review meetings and ticket status | Manual evidence collection | Periodic manual page review |
| B. Deterministic automation | JQL filters on corrective-action aging, Jira automation reminders | Checklists and required links in workflows | Native page verification and owner fields, scheduled reminders |
| C. Single agent | One skill, no maestro or guard split | One skill with the lineage model | One skill with the staleness model |
| D. This board | Maestro plus specialist plus guard | Same | Same |

Honest note: for D and E, a deterministic design plus a single agent may be competitive with this board, because the verdict logic lives in skills and the maestro and guard add routing and write control the read-only analysis does not need. The pilot must test option B and C against D, not assume D wins. The board's added value is mainly the write gate and the routing contract.

## 6. Threat model (Deliverable J)

Evidence type "executable fixture" means a capsule or routing fixture that the repository gates run; it proves record shape and routing, not model behavior or live sessions. Severity (H, M, L) is the board's judgment. Rule IDs are `CAP-*` capsule rules; fixture numbers refer to `tests/fixtures/atlassian-case-capsule/invalid` or routing cases.

| # | Scenario | Control | Severity | Evidence type |
| --- | --- | --- | --- | --- |
| 1 | Injection in Jira descriptions | `CAP-INSTRUCTION-TRUST` (33); skill rule: tenant text is data | H | Executable fixture (record level) |
| 2 | Injection in Jira comments | Same; routing case 09 sends injected "close all issues" text to gate mode, a keyword canary, not injection detection | H | Executable fixture (record level, routing canary) |
| 3 | Injection in Confluence pages | `CAP-INSTRUCTION-TRUST`; skill rule | H | Executable fixture (record level) |
| 4 | Forged management approvals | `CAP-APPROVAL-RECORD` (01 to 04), `CAP-APPROVER-HUMAN` (05, 06); routing case 10 | H | Executable fixture |
| 5 | Fake policy updates | Skill rule: policy text is data; no rule detects a fake policy | M | Documented contract only |
| 6 | Malicious MCP tool outputs | Tool output text listed as untrusted in charters; `CAP-INSTRUCTION-TRUST` for records | H | Documented contract only for live outputs |
| 7 | Cross-tenant leakage | `CAP-TENANT-BINDING` (23) | H | Executable fixture for capsule records only; not live sessions |
| 8 | Cross-project unauthorized access | Server acts with user permissions (ATL-E12); no board-side control | H | Not covered |
| 9 | Agent impersonation | `CAP-APPROVAL-RECORD` (03), `CAP-DELEGATION-CHAIN` (28), `CAP-DECISION-OWNER` (31) | H | Executable fixture; identity authentication in a real harness not built |
| 10 | Permission escalation | `CAP-PROHIBITED-TOOL` (12, 13); admin groups; client deny-list | H | Executable fixture; deny-list is client configuration |
| 11 | Unauthorized bulk writes | `CAP-BULK-TARGET` (24, 25) | H | Executable fixture |
| 12 | Destructive operations without approval | `CAP-PROHIBITED-TOOL` (15, 16), `CAP-APPROVAL-REQUIRED` (09); routing case 11 | H | Executable fixture |
| 13 | Fabricated references | Locator and version required per evidence item; nothing checks that a locator resolves | M | Documented contract only |
| 14 | Stale API permissions | Session contract re-established on resume (failure-and-recovery reference) | M | Documented contract only |
| 15 | Incorrect tenant routing | `CAP-TENANT-BINDING` (23) | H | Executable fixture (record level) |
| 16 | Duplicate ticket creation | Guard rules: search before create, read back before retry | M | Documented contract only |
| 17 | Confluence edit conflicts | Re-read and diff, never overwrite; stale-version status undocumented (delegate-reported) | M | Documented contract only |
| 18 | Partial cross-product execution | Guard rule: no atomicity, per-step read-back and compensation | M | Documented contract only |
| 19 | Interrupted workflows | Resume checklist; `CAP-EXPIRED` (30) covers only expired proposals | M | Documented contract only |
| 20 | Expired OAuth credentials | Failure table: stop, do not fall back to API token unless admin enabled (ATL-E11) | M | Documented contract only |
| 21 | Rate-limit exhaustion | Respect `Retry-After`, retry only if idempotent (ATL-E15, ATL-E17); schema caps `max_retries` (35) | M | Documented contract only for 429 handling; schema cap is executable |
| 22 | Approval expiration | `CAP-APPROVAL-WINDOW` (07, 08), `CAP-EXPIRED` (30) | H | Executable fixture |
| 23 | Unauthorized data exfiltration | `CAP-SECRETS` (32) covers credential-shaped strings only; data minimization is a rule | H | Executable fixture for credentials; otherwise documented contract only |
| 24 | Circular delegation | `CAP-DELEGATION-CYCLE` (26), `-DEPTH` (27), `-CHAIN` (28) | M | Executable fixture |
| 25 | Unbounded reasoning cost | Schema cap on retries (35) and `CAP-DELEGATION-DEPTH` (27); no token or cost metering | M | Partly executable; no runtime enforcement |
| 26 | Fabricated completion | `CAP-COMPLETION-EVIDENCE` (29); the truth of the evidence is not checked | H | Executable fixture (record level) |
| 27 | Compromised retrieved documentation | `CAP-INSTRUCTION-TRUST`; `setConfluenceSpaceInstructions` PROHIBITED (14); wrong but non-instruction text is undetected | H | Executable fixture for the instruction channel; otherwise not covered |
| 28 | Untrusted instructions in tool responses | `CAP-INSTRUCTION-TRUST` | H | Executable fixture (record level) |
| 29 | Policy conflicts | Routing protocol: freeze writes, security veto stands, escalate to the human owner | M | Documented contract only |
| 30 | False compliance claims | Lineage skill rule: never output "compliant"; no check scans output text | H | Documented contract only |

## 7. Delivery roadmap (Deliverable O)

Stages follow master-prompt section 21. Owner roles are proposed, not assigned.

| Stage | Status | Entry gate | Exit gate | Owner role | Stop or retire |
| --- | --- | --- | --- | --- | --- |
| 1. Research and evidence baseline | DONE in this change: ledger, delegate research, this record; many facts remain UNVERIFIED or delegate-reported | Repository audit | Justified architecture and measurable hypotheses | Board architect | n/a |
| 2. Core board foundation | DONE in this change: charters, skills, capsule schema and gate, routing fixtures. Gate results belong to the repository gates, not to this record | Stage 1 | Repository validation passes; coherent agent and skill boundaries | Board architect | n/a |
| 3. Read-only intelligence pilots | Not started | A named pilot tenant; write groups admin-disabled for the pilot identity or write scopes withheld; a client deny-list for write, destructive and prohibited names; a baseline period chosen and recorded | Accurate, useful findings reviewed by owners; zero unauthorized mutations confirmed by audit-log review | Pilot sponsor and Atlassian admin | Any mutation, unreviewable verdicts, or no baseline: stop |
| 4. Draft-and-review automation | Not started | Stage 3 exit; reviewers named | Draft quality, traceability and approval safety accepted | Document and ticket owners | Drafts not reused by owners: retire |
| 5. Controlled execution | Not started | Stage 4 exit; policy enforcement point and approval system integration built (not in this change); guard contract unchanged | Permission safety, recovery behavior and read-back evidence shown on bounded writes | Security owner and platform engineering | Any unauthorized or duplicate write: stop; guard stays gate-only |
| 6. Measured enterprise pilot | Not started | Stage 5 exit; baselines and comparison groups recorded | Business-value evidence against simpler alternatives (section 5) at acceptable risk | Value owner and finance partner | Benefit not shown against option B or C: retire the workflow |
| 7. Hardening and expansion | Not started | Stage 6 exit | Observability, durability, support ownership, operating economics; independent production-readiness acceptance | Platform and reliability owners | Support ownership unassigned: do not expand |
| 8. Scale, redesign or retire | Not started | Stage 7 exit | Only proven workflows scale; unjustified agents retired | Board architect and sponsor | Use the charter retire conditions; do not turn a pilot failure into a success story |

Stages 3 to 8 depend on infrastructure listed in section 3 that this change does not build. Until stage 3 conditions hold, the agents are static contracts exercised only by fixtures.
