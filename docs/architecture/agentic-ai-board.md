# Agentic AI Board: Design Record

This document records why wave 1 of the vendor-neutral Agentic AI Board ships what it ships, what it deliberately does not ship, and how much of it is evidence rather than intent. It is a dated design record, not an authorization surface: nothing described here approves, executes or deploys anything. Counts are stated "as of 2026-10-30" and are not maintained after that date; the generated count markers elsewhere in the docs are the live source.

## Status and scope

| Field | Value |
|---|---|
| Status | Experimental. Wave 1. The two agents and three skills declare `lifecycle: experimental`. |
| Date | 2026-10-30 |
| Provider | `generic` (no new provider; see the owner-ordering exception at the end) |
| Delivery | A draft pull request the repository owner can decline or defer |
| Decision record | [ADR-0003](../adr/0003-agentic-ai-board.md) |

Two products are easy to confuse here, and this record keeps them apart.

| | Product A (this repository) | Product B (not this repository) |
|---|---|---|
| What it is | A marketplace of role contracts, skills, protocols, a JSON Schema and offline gates | An executable orchestration runtime that routes work, authenticates senders, enforces authorization and stores audit records |
| What the board adds | Two review agents, three skills, one schema, one gate | Nothing |
| Who authenticates a sender | Nobody. The schema carries opaque subjects; the deploying runtime must verify them | The deploying organization's runtime |
| Who stores records | Nobody. The gate checks record sets handed to it | The deploying organization's runtime, append-only |

### Honesty boundary

- This repository ships role contracts, skills, a JSON Schema and offline gates. It is not an orchestration runtime. Nothing here authenticates senders, enforces authorization at runtime, or stores audit records.
- A valid envelope authorizes nothing. It is a data contract. A record set that passes the gate is well-formed and internally consistent; it is not proof that any approval happened or that any content is true.
- Framework mappings (NIST AI RMF, OWASP, ISO/IEC 42001, EU AI Act) are candidate review aids that an accountable owner confirms. Nothing here claims certification or compliance, and no mapping "satisfies" a framework (see docs/compliance/framework-profiles.md).
- Tool restrictions are declared only where a harness adapter carries a grant. Whether a harness enforces its declaration is a claim about that vendor's product: this record verifies it only for the Claude Code subagent `tools` field, and a SKILL.md `allowed-tools` line is documented as not restricting anything. The repository's own stable-contract document, docs/execution-tiers.md, says otherwise on three points, and that conflict is recorded as a pre-existing defect that this wave does not fix. See "Security, governance and escalation model" for the per-harness table and "Pre-existing defects found" for the conflict.

### Corrections to the seed material

New evidence contradicted three items in the research seed list. They are recorded here so no superseded claim stands.

- The seed cited MCP specification revision 2025-11-25. The current revision is 2026-07-28, per the MCP versioning page (https://modelcontextprotocol.io/specification/versioning, fetched 2026-10-30). Verify against the installed client and server before relying on any revision-specific behavior.
- The seed's OpenTelemetry GenAI semantic-convention URLs under opentelemetry.io were reported by the research delegate to serve "Moved" stubs. Per https://github.com/open-telemetry/semantic-conventions-genai the conventions live in that repository at Development status, which carries no stability guarantee, so none is presented here as a stable contract. The span names create_agent, invoke_agent and invoke_workflow were confirmed by the orchestrator at https://raw.githubusercontent.com/open-telemetry/semantic-conventions-genai/main/docs/gen-ai/gen-ai-agent-spans.md.
- EU AI Act obligations and dates were amended by the Digital Omnibus, adopted as Regulation (EU) 2026/1744 per https://eur-lex.europa.eu/eli/reg/2026/1744/oj/eng. That source was not re-verified by the orchestrator, so this record states no dates. Check EUR-Lex before using any.

### Catalog baseline before this wave (as of 2026-10-30)

735 agents, 761 skills, 39 agents whose id ends in `-maestro-agent`, and 35 maestro routing fixture directories carrying 840 scenarios (`python3 tests/validate-maestro-routing.py` reports "OK: 840 scenarios validated across 35 maestro(s)"). Agent execution tiers before this wave: 327 static-review, 44 read-only-runtime, 20 mutating-runtime, 344 unset.

## Executive verdict

**Wave 1 ships two agents, three skills, one JSON Schema and one offline gate. It ships no maestro and no business-value agent.** The two agents are agentic-system-architecture-review-agent and agentic-security-review-agent, both static-review, with a read-only declaration in four harness adapters and a prompt-level limit in the other three (enforcement of each declaration is discussed under the per-harness table). The three skills are the two companion skills plus the shared agentic-board-protocol. The gate is `npm run validate:agentic-board`.

- **Why so small.** The owner asked for the smallest coherent set, not the largest. Three candidate designs were produced and three judges each ranked the "minimal-reuse" design first at 8 out of 10. That is an internal review process, not independent evidence.
- **Why no maestro.** A1 is rejected for this wave. Only two leaves would be owned. Routing is deterministic keyword scoring. Twelve of thirteen proposed targets are already routed by their own provider maestro, so a board maestro would be a divergent second path (the Copilot Studio target conflicts with microsoft-maestro). Refuter probes of tests/validate-maestro-routing.py showed an anchored live-guard pattern routing "destroy the AgentCore runtime in production" to a single agent, aws-agentcore-agent, while the aws taxonomy gates that request. Four proposed targets have no declared execution tier, and three of those hold execute or runInTerminal tools in their Copilot adapters. Re-entry criteria are in ADR-0003.
- **Why no business agents.** E1 to E7 need baseline data that no marketplace repository can reach. The value-case contract in agentic-board-protocol lets a sponsor supply measured evidence outside the repository.
- **What the roster became.** Of 29 candidates: 9 are covered by content inside the two new agents (B1, B2, B3, B5, D3, C1, C2, C4, C5), 1 is split across three owners (B4), 9 are composed from or already exist in the catalog (A2, A4, A5, B6, C3, D1, D4, D5, E8), 1 is deferred (D2) and 9 are rejected (A1, A3, E1 to E7). The table under "Rejected and deferred candidates" is the source for this tally.
- **How far to trust it.** Not Fortune 50-grade: evidence-ready contracts, no production evidence. Every fixture is synthetic, no adopter has run the board, and every runtime guarantee belongs to a product B that does not exist here. The scorecard below shows the evidence level reached per dimension.

## Verified inventory

Everything below existed before wave 1 and was found by reading the repository on 2026-10-30. The point of the inventory is that most of what the owner's brief proposed was already here in some form, so the board composes it.

### Governance control plane

| Asset | Path | What it already does |
|---|---|---|
| python-live agents (15, including the maestro) | agents/python/python-live-* | Python-scoped control plane; the pattern is generic. policy-gate cannot create exceptions or approvals; identity-authority covers identity, JIT and approval authority; exception-governance cannot approve its own exception; change-plan holds no production credentials; control-evidence; rollback-and-recovery runs only pre-approved rollbacks; release-control |
| python-live-governance-maestro-agent | agents/python/python-live-governance-maestro-agent | Router for the python-live plane |
| Control objects | schemas/control-object.schema.json | Framework ids include NIST-AI-RMF, NIST-GenAI-Profile and ISO-IEC-42001 |
| Framework profiles | docs/compliance/framework-profiles.md | Candidate mappings only, never certification |
| Attestations | schemas/attestation.schema.json | Output contract for live-execution agents; the operator signs the document and hands it to audit |

### Risk and execution tiers

| Asset | Path | What it already does |
|---|---|---|
| Canonical R0-R5 table | docs/compliance/applicability-engine.md (rows at lines 57-62) | The one risk scale; the brief's R0-R4 collides with it, so the board crosswalks instead of duplicating |
| Audit event | schemas/audit-event.schema.json | risk_tier enum R0 to R5; R3+ approval binding is schema-enforced (approval id and plan digest on gate, approve and execute; prior-state digest and rollback on execute) |
| Execution tiers | docs/execution-tiers.md | T0 static review, T1 read-only runtime, T2 sandbox mutating, T3 production mutation prohibited for agents; the agent-level mutating-runtime tier is gate-only (one reversible operation, approval token naming the exact target, PREFLIGHT dry run, idempotency key, signed attestation, ROLLBACK with owner and time-box) |
| Tier gate | tests/validate-agent-tool-tiers.py | Polices agents that declare a tier; reports untiered agents without failing them |

### Handoff capsules

| Asset | Path | What it already does |
|---|---|---|
| legal-hr-case-capsule | skills/cross-functional/legal-hr-case-capsule | 30 required fields, one primary agent, one human decision owner, a non-empty do-not-do list |
| salesforce-case-capsule | skills/cross-functional/salesforce-case-capsule | Domain capsule for the Salesforce board |
| Communication design | docs/architecture/legal-hr-agent-communication.md | The capsule is the only channel; no free-form agent chatter |
| Routing design | docs/architecture/legal-hr-agent-routing.md and docs/adr/0001-initial-architecture.md | Three-layer maestro architecture, accepted |

These capsules are domain-specific and neither is vendor-neutral. The Salesforce capsule lists prose stop conditions about capsule readiness (no named owner, no do-not-do list, credentials present), but neither capsule defines a typed decision request, challenge payload, delegation depth or budget field. That is why the board adds an envelope rather than reusing a capsule.

### Adversarial board precedent

| Asset | Path | What it already does |
|---|---|---|
| frontend-board-chair-agent | agents/frontend/frontend-board-chair-agent | Binding approve, conditional-approve or reject; no averaging or voting; HARD gates; an evidence label per claim; a named human owner |
| enterprise-red-team-review-agent | agents/frontend/enterprise-red-team-review-agent | Second-pass adversarial review scoped to the frontend |
| salesforce-enterprise-architect-agent | agents/salesforce/salesforce-enterprise-architect-agent | Adversarial end-to-end architectural challenger for Salesforce |

### Runtime guards and routing

| Asset | Path | Note |
|---|---|---|
| Harness hooks | mods/live-guard-gate, mods/blast-radius-guard, mods/secret-redactor, mods/audit-trail, mods/context-burn | Tool-event guards and observers; disclosure, not isolation. No prompt-injection or tool-poisoning mod exists |
| Routing gate | tests/validate-maestro-routing.py | Keyword scoring; guarded-mutating-live agents appear only in live-guard-gate mode; every routed agent must exist in the catalog |
| MCP trust matrix | catalog/mcp-references.json, tests/validate-mcp-trust-matrix.py, mcp/official | Fields mutation_capable, requires_egress, requires_credentials, signed_release, pin_strategy |

### Vendor agentic assets

| Vendor | Assets (paths under agents/ unless noted) |
|---|---|
| AWS | aws/aws-agentcore-agent (with skills/aws/aws-agentcore), aws/aws-bedrock-agent-security-governor-agent |
| Azure | azure/azure-ai-foundry-ops-governor-agent |
| Databricks | databricks/databricks-genai-agent-engineering-agent, databricks/databricks-genai-evaluation-observability-agent |
| NVIDIA | nvidia/nvidia-agentic-ai-platform-review-agent, nvidia/nvidia-ngc-nim-supply-chain-governor-agent, nvidia/nvidia-model-promotion-gatekeeper-agent |
| Snowflake | snowflake/snowflake-cortex-ai-agent-security-governor-agent |
| Salesforce | salesforce/salesforce-agentforce-ai-agent, skills/salesforce/salesforce-agentforce-risk-review-skill |
| SAP | sap/sap-joule-governance-adoption-agent, skills/sap/sap-ai-core-generative-ai-hub-governance, skills/sap/sap-ai-governance-security-architecture-protocol |
| Microsoft | microsoft/copilot-studio-agent-governance-alm-agent, microsoft/m365-copilot-readiness-governance-agent, microsoft/copilot-governance-maestro-agent |
| MCP | typescript/typescript-mcp-tool-contract-agent, netsuite/netsuite-ai-connector-mcp-agent, skills/oci/oracle-oci-mcp-grounded-advisor |
| Evaluation | qa/llm-ai-pipeline-test-review-agent |
| FinOps | finops/finops-ai-economist-agent, finops/finops-maestro-agent |
| Legal and regulatory | legal/legal-regulatory-compliance-agent, marketing/eu-ai-act-marketing-system-review-agent (marketing-scoped) |
| Process | sap/sap-signavio-process-mining-value-agent |

### Gaps (searched, none found)

These were searched for on 2026-10-30; absence is as of that search.

- A vendor-neutral review of agentic system architecture: pattern selection, the single-agent versus multi-agent versus deterministic-workflow decision, topology, delegation boundaries and stop conditions.
- Generic agent state, memory and durable-execution design: checkpointing, idempotency, compensation, replay.
- The A2A protocol (searched for A2A, Agent2Agent and Agent Card).
- Agent identity as a dedicated topic: workload identity for agents, MCP audience binding, token passthrough.
- The OWASP Top 10 for Agentic Applications, MITRE ATLAS and the term "tool poisoning".
- OpenTelemetry GenAI semantic conventions (gen_ai attributes).
- A dedicated AI red-team asset (PyRIT, garak, promptfoo) and a prompt-injection mod.
- Consumer-side integrity verification for skills and MCP servers, and any AI bill of materials.
- Per-run cost attribution and cost per successful task outside the Snowflake reviewer.
- Vertex AI Agent Engine and Google ADK.
- Cross-domain AI governance: an ISO/IEC 42001 gap assessment or an AI RMF mapping for an agentic system.
- An automation decision method (eliminate, redesign, deterministic workflow, RPA, single agent, multi-agent) with a value model.
- Business-process agents E1 to E7 (partial neighbors are listed in the rejected table).

## Duplication and integration-gap findings

### What the brief proposed that already existed

| Brief proposed | Already in the repository | Disposition |
|---|---|---|
| An R0-R4 risk scale | The canonical R0-R5 scale in docs/compliance/applicability-engine.md and schemas/audit-event.schema.json | Crosswalk onto it; never redefine a label |
| Execution authority levels | docs/execution-tiers.md, including the gated mutating-runtime tier | Board agents are T0; the envelope has no execute verb |
| A handoff contract | Domain capsules and routing protocols | New vendor-neutral envelope, with the capsule rules (one owner, do-not-do list) reused as ideas |
| An adversarial decision process | frontend-board-chair-agent and enterprise-red-team-review-agent | Same rules (no voting, HARD gates, named human) generalized into a protocol |
| Governance and approval authority | The python-live control plane | Composed as the runtime authorization pattern; not duplicated |
| Tool-event guards | mods/ | Composed; the board disclaims runtime enforcement |

### Pre-existing defects found

| Defect | Evidence | Status in this wave |
|---|---|---|
| 344 agents have no execution_tier although docs/execution-tiers.md says every skill and agent must declare one | docs/execution-tiers.md opening paragraph versus tests/validate-agent-tool-tiers.py, which states the field is optional in schemas/agent.schema.json and reports untiered agents without failing them | Not fixed in wave 1. Backlog item; tier assignment is a per-agent judgment |
| copilot-governance-maestro routing table names four agent ids that are not in the catalog: m365-copilot-readiness-data-exposure-governor, purview-data-security-compliance-officer, copilot-studio-agent-governance-architect, entra-identity-conditional-access-architect | skills/microsoft/copilot-governance-maestro/references/workflow-and-output.md (also safety-checklist.md). No fixture directory carries this maestro's name; the only Microsoft fixture is tests/fixtures/microsoft-maestro-routing, so the gate never checks the table | Not fixed in wave 1. Backlog item; which real agents it should name is an owner decision |
| ROADMAP.md line 88 hardcodes a skill count ("The existing 619 skills"), contradicting ROADMAP.md lines 31-34, which say never to restate counts there | ROADMAP.md | Not fixed in wave 1. Backlog item |
| docs/execution-tiers.md contradicts the claims in this record about harness tool grants, on three points. Line 238 says a companion SKILL.md `allowed-tools` is enforced "at skill-invocation level", but Claude Code documents that it "does not restrict which tools are available: every tool remains callable" (https://code.claude.com/docs/en/skills). Line 239 says claude-code and cursor `.agent.md` carry "name + description only" and are not a tool grant, but the board's claude-code adapters carry `tools` and its cursor adapters carry `readonly: true`, and 154 of the 735 cursor adapters committed before this wave already carry `readonly: true`. Line 242 calls the Copilot adapter "the one agent-level tool grant in this catalog", which no longer holds once a claude-code adapter carries `tools` | docs/execution-tiers.md lines 238, 239 and 242 (section "Harness tool grants must match the declared tier") against the Claude Code skills and sub-agents documentation (https://code.claude.com/docs/en/skills, https://code.claude.com/docs/en/sub-agents) and the committed adapters | Not fixed in wave 1. This record follows the verified Claude Code documentation where they conflict. Backlog item; the stable-contract document is the owner's to amend, and Cursor's `readonly` behavior still has to be verified against Cursor's official documentation |
| The TypeScript board routed per-server MCP trust review to a "security board" that does not exist, and no provider or agent of that name is in catalog/agents.json | scripts/typescript_data/agents/00-typescript-maestro-agent.json and 11-typescript-mcp-tool-contract-agent.json | Partly fixed: per-server MCP trust only. The maestro's operating rule and the MCP tool-contract agent's non-owned scope, refusal and escalation lists now send per-server MCP trust review to agentic-security-review-agent and organization MCP trust policy to its named human policy owner. Other clauses were not changed and still name a board that does not exist; they are listed below the table. Backlog item with an owner decision on where application security and credential-issuance questions should route |
| The generic Kiro Power described generic agents as CI, Helm, manifest and test-quality only, and pointed readers at agents/generic/, a directory that does not exist | scripts/generate-kiro-powers.mjs | Fixed in this wave: the invariant text covers agents/qa/ and agents/cross-functional/, and a helper derives the real directories |

TypeScript clauses that still route to a board that does not exist, as of 2026-10-30 (paths under scripts/typescript_data/agents/, located by JSON field):

- 00-typescript-maestro-agent.json: the operating rule sending organization-wide secrets and identity to "the security board", and the runtime-boundary claim sending exploitation, authorization and secrets to "the application security board".
- 02-typescript-runtime-boundary-contract-agent.json: four entries (not-owned scope, refusal trigger, escalation trigger and the companion skill's when-not list) sending injection, authorization, secrets and crypto policy to "the application security board".
- 09-typescript-package-publication-integrity-agent.json: seven entries sending organization-wide secret management, token and key custody and identity policy to "the security board".
- 11-typescript-mcp-tool-contract-agent.json: the companion skill's when-not entry "whether to trust a third-party MCP server" still says "the security board", which is the per-server clause this wave meant to repoint and which also appears in skills/typescript/typescript-mcp-tool-contract/SKILL.md. Its hosting and transport clauses and two response-shape mentions also still name "the security board".
- 12-typescript-business-critical-automation-governance-agent.json: five entries sending generic application security review and credential issuance or custody to "the security board".
- scripts/gen_typescript_agents.py: a generator template line that routes organization-wide secrets, identity and MCP trust policy to "the security board".

These clauses send a reader to a board that does not exist, so the reader finds no owner. Where application security, secrets and credential-issuance questions should route is an owner decision, because agentic-security-review-agent is scoped to agentic systems and is not a general application-security reviewer.

The Kiro finding also explains a design constraint. selectMaestro() in that generator falls back to any maestro in a provider, so a new generic maestro would have silently become the generic Power's router for QA and Helm tasks. That hazard is one more reason the maestro is rejected this wave.

## Framework and pattern comparison

The architecture skill owns the full comparison: [pattern selection](../../skills/cross-functional/agentic-system-architecture-review/references/pattern-selection.md). This is a summary of the research register's framework table.

Versions drift. No version numbers are recorded here on purpose, and every cell must be re-verified against Context7 or the official documentation for the installed version before it is asserted about a system under review. The cells are research-delegate findings, cited "per" the sources below the table, with no stronger standing.

| Framework | Orchestration primitive | Human approval | Persistence and replay | Caveat |
|---|---|---|---|---|
| LangGraph | StateGraph, subgraphs, Functional API | interrupt() resumed with Command(resume=); needs a checkpointer | Checkpointers keyed by thread_id | langgraph-supervisor is archived |
| OpenAI Agents SDK | Handoffs, Agent.as_tool(), guardrails | needs_approval produces interruptions | Sessions hold history only | Durability comes from an external workflow engine |
| Microsoft Agent Framework | Sequential, concurrent, handoff, group chat, magentic | RequestPort and request_info; tool approval | Superstep checkpoints | A durable-task extension exists; its status was not verified |
| Google ADK | Sequential, Parallel, Loop, LLM delegation | Graph RequestInput; Tool Confirmation (experimental) | SessionService | The in-memory service loses state on restart. Sequential/Parallel/Loop agents are superseded by graph workflows in the current major (per https://adk.dev/workflows/) |
| CrewAI | Crews and Flows | @human_feedback | @persist and checkpointing | Default A2A protocol revision is pre-1.0 |
| LlamaIndex Workflows | Event-driven steps | InputRequiredEvent and HumanResponseEvent | Ephemeral by default | A2A support was not established |
| Bedrock AgentCore | None; it hosts frameworks | Gateway elicitation and Policy | Memory and Runtime sessions; no workflow replay | Review the hosted framework separately |
| Temporal | Workflows, activities, timers | Signals and updates | Event-history replay | Official agent-SDK integrations vary in maturity |

Sources: https://docs.langchain.com/oss/python/langgraph/persistence, https://docs.langchain.com/oss/python/langgraph/interrupts, https://openai.github.io/openai-agents-python/multi_agent/, https://openai.github.io/openai-agents-python/handoffs/, https://openai.github.io/openai-agents-python/human_in_the_loop/, https://openai.github.io/openai-agents-python/running_agents/, https://learn.microsoft.com/agent-framework/workflows/orchestrations/, https://learn.microsoft.com/agent-framework/workflows/checkpoints/, https://learn.microsoft.com/agent-framework/workflows/human-in-the-loop/, https://adk.dev/workflows/, https://adk.dev/sessions/state, https://adk.dev/a2a, https://docs.crewai.com/en/concepts/flows, https://developers.llamaindex.ai/python/llamaagents/workflows/durable_workflows/, https://docs.aws.amazon.com/bedrock-agentcore/latest/devguide/release-notes.html and https://docs.temporal.io/develop/python/integrations.

The consequence the board draws is an inference, not a documented fact: a multi-agent framework is not a durable workflow. Durable execution is not native in most of the SDKs above, so the architecture agent must ask where durability comes from before it accepts a topology.

## Board composition

| Asset | Kind | Covers roster ids | Tier and lifecycle |
|---|---|---|---|
| agentic-system-architecture-review-agent | Agent in agents/cross-functional | B1, B2, B3, B5, D3, technical rungs of E8 | static-review, experimental |
| agentic-security-review-agent | Agent in agents/cross-functional | C1, C2, C4, C5, the MCP and A2A trust half of B4 | static-review, experimental |
| agentic-system-architecture-review | Companion skill | none (carries the agent's method) | allowed-tools Read Grep Glob, experimental |
| agentic-security-review | Companion skill | none | allowed-tools Read Grep Glob, experimental |
| agentic-board-protocol | Protocol skill (layer 3) | A4 as a contract, A5 as a crosswalk, business rungs of A2 and E8, stop conditions | allowed-tools Read Grep Glob, experimental |
| schemas/agentic-handoff.schema.json | Schema | handoff, decision request, decision record, value case | contract only |
| tests/agentic-board-suite and validate:agentic-board | Gate | deterministic rule checks, sole-violation fixtures, parity and board invariants | offline |

Three small extensions keep neighbors truthful: the generic Kiro Power invariant text (scripts/generate-kiro-powers.mjs), the install-role wiring (catalog/install-roles.json adds both agents and all three skills to cloud-ai-platform-engineer, and the security agent with its companion skill and the protocol to cloud-security-engineer), and the TypeScript per-server MCP-trust routing clauses (a partial fix; see the defect table).

Composed without edits: finance-capital-allocation-advisor-agent (the designated value-case challenger; read-only-runtime), llm-ai-pipeline-test-review-agent, databricks-genai-evaluation-observability-agent, finops-ai-economist-agent, the vendor governors for single-vendor work, typescript-mcp-tool-contract-agent, legal-regulatory-compliance-agent, the python-live control plane, and mods/live-guard-gate.

## Rejected and deferred candidates

Labels follow the owner's classification (EXISTS, EXTEND, COMPOSE, NEW, REJECT), plus DEFER and SPLIT where the decision needed them. Paths are repository-relative.

### A: executive orchestration and governance

| Id | Candidate | Verdict | Reason | Existing assets |
|---|---|---|---|---|
| A1 | Enterprise Agentic AI Maestro | REJECT this wave | Two owned leaves only; deterministic keyword routing; 12 of 13 targets already routed by their own maestro; a probe showed a live-guard pattern routing a destruction request to a single agent; four targets have no tier and three hold execute tools | ADR-0003 holds the re-entry criteria |
| A2 | Portfolio Strategist | COMPOSE | The value-case procedure lives in agentic-board-protocol and is challenged by the finance agent; process evidence comes from the Signavio agent | agents/finance/finance-capital-allocation-advisor-agent, agents/sap/sap-signavio-process-mining-value-agent |
| A3 | Enterprise Architecture Review Chair | REJECT | No chair; the human decision owner resolves every decision request. The frontend chair is a domain chair over ten workflows, which two specialists do not justify | agents/frontend/frontend-board-chair-agent (precedent) |
| A4 | Adversarial Decision Auditor | COMPOSE as a contract | Designated-challenger table, deterministic rules (proposer is not challenger, unresolved dissent, HARD gate), the security HARD gate and the finance challenger. A same-model auditor agent adds procedural, not epistemic, independence | skills/cross-functional/agentic-board-protocol; agents/frontend/enterprise-red-team-review-agent |
| A5 | AI Governance and Risk Officer | COMPOSE | Risk-tier crosswalk and NIST AI RMF candidate mapping in the protocol. Re-entry: a cross-domain ISO/IEC 42001 gap-assessment need with a named owner | docs/compliance/framework-profiles.md, schemas/control-object.schema.json, agents/legal/legal-regulatory-compliance-agent, agents/python/python-live-governance-maestro-agent |

### B: orchestration engineering

| Id | Candidate | Verdict | Reason | Existing assets |
|---|---|---|---|---|
| B1 | Multi-Agent Orchestration Architect | NEW, inside the architecture agent | Pattern selection and the lowest-sufficient-pattern verdict | none generic (gap) |
| B2 | Routing and Delegation Engineer | NEW, inside the architecture agent | Delegation and handoff review, routing confidence with unclassified as a stop | tests/validate-maestro-routing.py (routing gate for existing maestros) |
| B3 | State and Memory Architect | NEW, inside the architecture agent | State scope, retention, deletion, checkpointing. Memory poisoning is owned by the security agent | none generic (gap) |
| B4 | Communication and Protocol Engineer | SPLIT | MCP schema fidelity and protocol version already exist; MCP authorization and A2A trust go to the security agent; the A2A task lifecycle as a resumability boundary goes to the architecture agent | agents/typescript/typescript-mcp-tool-contract-agent |
| B5 | Durable Workflow and Recovery Architect | NEW, inside the architecture agent | Retries, timeouts, idempotency, compensation, replay safety | none generic (gap) |
| B6 | Tool Integration and Enterprise Systems Architect | COMPOSE | Existing ERP, CRM and ITSM boards cover the systems | agents/netsuite/netsuite-ai-connector-mcp-agent and the SAP, Salesforce and Microsoft boards |

### C: trust, security and compliance

| Id | Candidate | Verdict | Reason | Existing assets |
|---|---|---|---|---|
| C1 | Agentic Threat Modeling | NEW, inside the security agent | ASI01 to ASI10 walk with review questions and plan-only tests | vendor governors for single-vendor systems |
| C2 | Agent Identity and Zero-Trust | NEW, inside the security agent | Agent identity, delegated authorization, token passthrough, A2A security schemes | none dedicated (gap) |
| C3 | AI Data Governance and Privacy | COMPOSE | The envelope carries data_classification and redactions; neighbors own the platform specifics | agents/microsoft/m365-copilot-readiness-governance-agent, skills/sap/sap-data-privacy-analytics-ai-protocol, agents/snowflake/snowflake-cortex-ai-agent-security-governor-agent |
| C4 | Agentic Red-Team | NEW as plan-only test design | The security agent designs adversarial tests and never executes them | agents/frontend/enterprise-red-team-review-agent (frontend-scoped) |
| C5 | Agent Supply-Chain Security | NEW, inside the security agent | Consumer-side provenance for tools, skills and MCP servers | agents/nvidia/nvidia-ngc-nim-supply-chain-governor-agent, tests/validate-mcp-trust-matrix.py |

### D: production engineering

| Id | Candidate | Verdict | Reason | Existing assets |
|---|---|---|---|---|
| D1 | Evaluation and Benchmark | EXISTS | The QA reviewer covers ToolCorrectness and TaskCompletion gaps and flags a missing red-team dataset as critical | agents/qa/llm-ai-pipeline-test-review-agent, agents/databricks/databricks-genai-evaluation-observability-agent |
| D2 | Observability and Incident Response | DEFER | OTel GenAI conventions are at Development status and moved repositories (per https://github.com/open-telemetry/semantic-conventions-genai); observability hooks live in the architecture reliability reference | none generic |
| D3 | Reliability and Resilience | NEW, inside the architecture agent | Cascading-failure containment is owned here: budgets, fan-out caps, circuit breakers, kill switch | none generic (gap) |
| D4 | AI FinOps and Model Economics | COMPOSE | Unit prices stay with the FinOps agent; the board adds cost per successful task to the value case | agents/finops/finops-ai-economist-agent, skills/finops/fetch-foundation-model-pricing, agents/finops/finops-maestro-agent |
| D5 | Deployment and Runtime | COMPOSE; gap DEFER | Vendor runtime agents cover deployment. The Vertex AI Agent Engine and ADK gap is noted and deferred | agents/aws/aws-agentcore-agent, agents/azure/azure-ai-foundry-ops-governor-agent, agents/databricks/databricks-genai-agent-engineering-agent, agents/nvidia/nvidia-agentic-ai-platform-review-agent |

### E: business value

E1 to E7 are rejected as agents because no baseline data is reachable from a marketplace repository. Re-entry for any of them: a sponsor runs a value case on measured data outside the repository, the record passes validate:agentic-board structurally, and the owner attests the measured evidence in the pull request. That attestation is a human decision; the gate cannot verify truth.

| Id | Candidate | Verdict | Reason | Existing assets (partial neighbors) |
|---|---|---|---|---|
| E1 | Revenue Leakage and Quote-to-Cash | REJECT as an agent | No baseline data reachable from a marketplace repository | agents/sap/sap-order-to-cash-agent, agents/salesforce/salesforce-sales-cloud-revenue-agent |
| E2 | Procure-to-Pay Exceptions | REJECT as an agent | No baseline data reachable from a marketplace repository | agents/accounting/accounting-procure-to-pay-advisor-agent, agents/sap/sap-procurement-ariba-value-leakage-agent |
| E3 | Operational Bottleneck Investigator | REJECT as an agent | No baseline data reachable from a marketplace repository | agents/sap/sap-signavio-process-mining-value-agent |
| E4 | Customer Retention and Service Recovery | REJECT as an agent | No baseline data reachable from a marketplace repository | skills/cross-functional/case-to-resolution-protocol |
| E5 | Regulatory Change Impact | REJECT as an agent | No baseline data reachable from a marketplace repository | agents/legal/legal-regulatory-compliance-agent, skills/cross-functional/audit-evidence-mapping-protocol |
| E6 | Incident Business-Impact Coordinator | REJECT as an agent | No baseline data reachable from a marketplace repository | agents/sap/sap-rise-sla-vendor-risk-agent, skills/sap/sap-rise-sla-vendor-risk-review |
| E7 | Contract Obligation and Commercial Risk | REJECT as an agent | No baseline data reachable from a marketplace repository | agents/legal/legal-contract-review-agent |
| E8 | Business Process Redesign and Automation Architect | COMPOSE | Technical rungs go to the architecture agent (agents/cross-functional/agentic-system-architecture-review-agent). Business rungs (eliminate, redesign, conventional automation), the value case and stop conditions go to agentic-board-protocol (skills/cross-functional/agentic-board-protocol). The Signavio agent has an automation-candidate finding code but no scoring method | agents/sap/sap-signavio-process-mining-value-agent, skills/sap/sap-signavio-process-mining-value |

## Authority matrix

Both agents carry the same thirteen-row authority card in their AGENT.md. The rows below are condensed; the agent files are authoritative.

### Shared by both agents

| Row | Value |
|---|---|
| Upstream authority | A human principal carried in the envelope as an opaque individual subject, acting under the deploying organization's policy. This repository cannot authenticate who that principal is |
| Tools | Read, Grep and Glob equivalents, plus read-only documentation lookup where the harness provides it; no execution, network or write tools. Declared as a grant in four adapters and prompt-level in three; whether a harness enforces a declaration is not established here except for the Claude Code `tools` field (see the per-harness table) |
| Risk tier | Own work: T0 static review. Subject decisions: any R0 to R5 as proposed for the decision under review and confirmed by a principal; escalated at R3+ or with any materiality flag. Flags never remap the tier |
| Human approval | Never self-approves. A decision request always goes to a principal. Only a principal-sourced decision record carries approved or rejected. R5 needs two distinct human subjects |
| Maximum budget | Carried in the envelope (tool calls, tokens, cost in integer minor units); never raised down a chain; the agent stops when any dimension is exhausted. Nothing here enforces a budget at runtime |
| Fail-closed behavior | Stop and escalate on missing principal identity, blocking missing evidence, live-guard intent, instruction bait, a routing loop, depth, budget or deadline exhaustion, challenge rounds beyond two, and conflicting instructions (with a stop note) |

### Where they differ

| Row | Architecture agent | Security agent |
|---|---|---|
| Permitted inputs | Sanitized designs, code, configuration, traces and documentation; handoff envelopes | The same plus tool descriptions, MCP server and agent card manifests and memory-store schemas; no credentials, tokens or signing keys |
| Prohibited actions | Executing, deploying, load-testing or contacting any system; recording any decision; widening authority down a chain; claiming certification | The same plus executing attacks or probes, calling or fetching a tool, server or endpoint under review, and recording a HARD gate as overruled or lowering it |
| Delegable tasks | Review questions only: security attack surface to the security agent; vendor interiors, MCP schema fidelity, evaluation, unit prices, value-case challenge and regulatory classification to named existing agents | Review questions only: architecture, containment and durability to the architecture agent; vendor interiors, MCP schema fidelity, evaluation and regulatory classification to named existing agents |
| Evidence returned | Verdict (pass, pass-with-conditions, block, simpler-pattern-sufficient), lowest sufficient pattern, rejected rungs, findings with failure scenario | Verdict (pass, pass-with-conditions, block), findings with ASI id and HARD-gate flag, plan-only verification tests, residual runtime-only risks |
| Designated challenger | agentic-security-review-agent for architecture decisions it proposes; finance-capital-allocation-advisor-agent for automation investment | agentic-system-architecture-review-agent, which may challenge feasibility and placement of a control but never downgrades a confirmed exploit path; finance agent for automation investment |
| Handoff and escalation | Hands off when a finding leaves what it owns; escalates for R3+, materiality flags, open HARD gates, unresolved dissent, live work | The same plus evidence that a compromise or credential exposure is already live, and any request to execute an attack |

## Topology and handoff contract

The envelope in [schemas/agentic-handoff.schema.json](../../schemas/agentic-handoff.schema.json) is the only channel between board agents and principals. The schema checks shape only; the cross-field rules live in tests/agentic-board-suite/agentic_board_rules.py, one predicate per rule id, so each defect fails with the id that names it. There are 67 rule ids in six families as of 2026-10-30: 29 envelope, record-set and chain rules (HO), 8 computed stop conditions (STOP), 1 challenge link rule (CH), 12 decision-request rules (DR), 7 decision-record rules (DRC) and 10 value-case rules (VC). The rule tables are in the protocol references: [handoff envelope](../../skills/cross-functional/agentic-board-protocol/references/handoff-envelope.md), [stop conditions](../../skills/cross-functional/agentic-board-protocol/references/stop-conditions.md), [decision record](../../skills/cross-functional/agentic-board-protocol/references/decision-record.md) and [value case](../../skills/cross-functional/agentic-board-protocol/references/value-case.md).

| Message type | Purpose |
|---|---|
| delegate | Ask an agent to perform analysis, phrased as a review question |
| return | Deliver a verdict to the requester |
| challenge | A designated challenger's adversarial finding on a proposal |
| escalate | Stop and hand to an accountable principal, with a route hint |
| decision-request | An agent assembles proposal, alternatives, challenges and dissent for a principal. Never a decision |
| decision-record | The principal's decision, bound to one decision-request by the sha256 of its canonical JSON |

A typical decision, taken from the synthetic fixture for an R3 architecture review with a sustained HARD-gate challenge:

| Hop | Message | From to to | What happens |
|---|---|---|---|
| 0 | delegate | principal to architecture agent | Root of the correlation; parent is null |
| 1 | delegate | architecture agent to security agent | Proposal sent to the designated challenger |
| 2 | challenge | security agent to architecture agent | Round 1, hard_gate true, outcome sustained |
| 3 | decision-request | architecture agent to principal | Recommends redesign; carries alternatives, challenge ids and dissent |
| 4 | decision-record | principal to architecture agent | The principal conditionally approves over the HARD gate with a risk acceptance reference, bound by request digest |

Four properties carry the design, and each is checked by a named rule family rather than asserted by prose.

- **Agents never decide.** Agent-sourced envelopes may carry only not-required or pending-human-review. Only a principal-sourced decision-record carries approved or rejected.
- **Monotonic attenuation.** Down a chain, tier, classification, materiality flags and prohibitions never shrink; deadline and max_hops never extend; authorized actions and budget limits only narrow; a missing parent is a violation.
- **Computed stops.** Depth, deadline, budget, blocking missing evidence, mutation intent, instruction bait, routing loops and challenge rounds are computed from message fields, never from the wall clock.
- **No execute verb.** Authorized actions are read, analyze, recommend, challenge and escalate. Live work escalates to a principal with a route hint to the owning provider's gated live-guard path; the hint is never a dispatch.

Principals are opaque individual subjects of the form `subject:corp-idp:<hex digest>` plus a separate role. Self-approval is detected by comparing subjects. The runtime, not this repository, authenticates senders; a subject in a record set proves nothing about who sent it. No field may carry credentials, tokens or secrets, and a committed invalid fixture shows an envelope with an unknown credential-shaped field failing the schema rule.

## Security, governance and escalation model

### Risk tiers

The envelope's subject_risk_tier is the canonical tier of the decision under review. The board's own work is static review. The canonical table, quoted from docs/compliance/applicability-engine.md:

| Tier | Meaning | Gate posture |
|---|---|---|
| R0 | Read-only, non-sensitive | Observe; audit event recommended |
| R1 | Read-only, sensitive data in scope | Read-only-runtime; runtime-evidence-gate; redaction |
| R2 | Reversible non-production change | Plan + policy gate; approval per policy |
| R3 | Production change, bounded, reversible | Independent approval + JIT + rollback + audit event; fail closed if audit unavailable |
| R4 | Production change, broad blast radius | R3 controls + blast-radius assessment + change window |
| R5 | Irreversible / high-blast-radius production action | R4 controls + second approver + explicit risk acceptance |

The owner's proposed scale maps onto it as follows. Owner R0 and R1 map to canonical R0 and R1 by data sensitivity; owner R2 maps to R2; owner R3 maps to R3. Owner R4 (financially material, security-critical, destructive, regulatory or hard-to-reverse) splits across canonical R4 for broad blast radius and R5 for irreversible or high-blast-radius actions, and also sets materiality flags. The flags (financial, regulatory, security-critical, external-communication, personal-data, irreversible) force full-depth review and principal escalation. They never remap the tier. The full crosswalk, the execution-tier crosswalk and the candidate NIST AI RMF mapping are in the [risk-tier crosswalk](../../skills/cross-functional/agentic-board-protocol/references/risk-tier-crosswalk.md). The AI RMF core page carries a banner that version 1.0 is being updated (https://airc.nist.gov/airmf-resources/airmf/5-sec-core/), so treat that mapping as provisional.

### Review depth and the HARD gate

Review depth is proportionate. R0 and R1 without materiality flags need a single specialist and an optional challenge. R2, any materiality flag, or an automation investment needs the designated challenger, at least one materially different alternative, and a dissent entry for every open challenge. R3 and above additionally make a principal decision-record the only way forward, and R5 needs a second approver distinct from the decider. The board never resolves a disagreement by the most confident agent, by voting or by averaging, and dissent is preserved, never deleted.

A security challenge with hard_gate true and outcome sustained or unresolved forbids the proposer from recommending approve or conditional-approve. Only a principal decision-record carrying a risk acceptance reference may approve over it, and agents can never record a gate as overruled.

### Candidate threat mapping

The security agent walks all ten items of the OWASP Top 10 for Agentic Applications (2026), ASI01 to ASI10 (names confirmed at https://genai.owasp.org/2025/12/09/owasp-top-10-for-agentic-applications-the-benchmark-for-agentic-security-in-the-age-of-autonomous-ai/). A repository rule is named for an item only where that rule operates on the threat's object; five items have such a rule (ASI01, ASI03, ASI07, ASI08 and ASI09) and the other five (ASI02, ASI04, ASI05, ASI06 and ASI10) are marked runtime-only (product B), because no manufactured coverage is acceptable. The row-by-row mapping, the review questions, the evidence to request, the plan-only test scenarios and the limit of each repository control are in [threat-model-owasp-asi.md](../../skills/cross-functional/agentic-security-review/references/threat-model-owasp-asi.md). That file is the one owner of the mapping and is authoritative; this record deliberately carries no copy of the table, so the two cannot drift apart unnoticed. The mapping is a candidate review aid that an accountable owner confirms, not a claim that any design satisfies an item.

### Per-harness restriction

The adapters declare the following. "Strength" says how far the declaration can be trusted as a restriction, and what that rests on. Only the Claude Code row rests on a source this record verified.

| Adapter | Declaration carried in this repository | Strength |
|---|---|---|
| copilot.agent.md | tools: read, search, search/codebase | Declared; treated as harness-level per docs/execution-tiers.md line 237 (repository documentation; not re-verified against GitHub's documentation in this wave) |
| codex.toml | sandbox_mode = "read-only" | Declared; treated as harness-level per docs/execution-tiers.md line 236 (repository documentation; not re-verified against OpenAI's documentation in this wave) |
| cursor.agent.md | readonly: true | Declared; verify against Cursor's official documentation for the installed version. No source for Cursor's behavior was verified in this wave, and docs/execution-tiers.md line 239 says cursor adapters are not a tool grant |
| claude-code.agent.md | tools: "Read, Grep, Glob" | Documented: per https://code.claude.com/docs/en/sub-agents the `tools` field lists the subagent's tools, and a subagent that omits it inherits every tool available to subagents, so the field is set here. Verify against that page for the installed version |
| gemini.agent.md | none | Prompt-level only |
| kiro-ide.agent.md | none | Prompt-level only |
| kiro-cli.agent.json | none | Prompt-level only |

A SKILL.md `allowed-tools` line pre-approves tools; it does not restrict them. Claude Code documents that it "does not restrict which tools are available: every tool remains callable" (https://code.claude.com/docs/en/skills). docs/execution-tiers.md lines 238, 239 and 242 describe the same surfaces differently (see the defect row under "Pre-existing defects found"); where they conflict, this record follows the verified Claude Code documentation and leaves that document unchanged.

The gate asserts that the first four declarations are present; it does not test how any harness behaves. A team that runs the board under Gemini or Kiro relies on the prompt for its read-only limit, and a team that runs it under Cursor, Copilot or Codex should confirm what the declared field does in the installed version before relying on it.

### Regulatory classification

Classification under the EU AI Act is routed to legal-regulatory-compliance-agent. This record states no obligations or dates; see the correction above.

## Evaluation and adversarial test plan

### What validate:agentic-board proves

The gate is deterministic and offline, reading committed files only (`python3 -m unittest discover -s tests/agentic-board-suite -p "test_*.py"`). It proves:

- Every one of the 67 rules has a sole-violation fixture, and each of the 76 invalid fixtures fails exactly its one declared rule.
- The 10 valid record sets pass and are all marked synthetic.
- The fixtures are current with their builder, and every agent id they use exists in the catalog.
- The protocol references document exactly the rule table, no more and no fewer ids.
- subject_risk_tier and data_classification reuse the audit-event vocabularies, the board's actions are a subset of the schema's, and designated challengers exist in the catalog.
- The board agents are generic, static-review and experimental, bundle the protocol, and carry the read-only declarations in four harness adapters; the board skills declare Read Grep Glob and link each reference.
- An AST denylist scan finds none of the listed wall-clock, network or randomness imports and calls in the rules or the fixture builder.
- The mutation-intent and instruction-bait patterns behave as pinned on listed positive and negative sentences.

### What it does not prove

- That any content is true, that any principal is who the record says, or that anyone approved anything.
- That any harness enforces its declared restriction, or that an agent follows its prompt. No model is run, so no routing or review quality is measured.
- That the rules and the fixture builder are free of ambient state. The scan is a denylist of specific imports and of calls written as a plain name followed by an attribute, so indirect forms pass it: `import datetime` with `datetime.datetime.now()`, `from time import time` with `time()`, and `os.urandom` were each replayed against the scan's logic on 2026-10-30 and none is caught. Determinism rests on code review as well as on the scan.
- That the mutation-intent and bait patterns are complete. They are fixture oracles, deliberately broad, and they are not runtime enforcement. A hostile objective can avoid them.
- That a value case is real. The arithmetic and ordering rules check bookkeeping. Repository fixtures are synthetic and can never claim more than pilot status.
- Anything about product B behavior.

### Mutation probe

The probe neutralizes each rule add-site in agentic_board_rules.py in memory and replays every fixture. It found 64 mutation sites (63 rule add-sites and the early return for an empty document). Fixtures killed 63. The one survivor is the empty-document return, which test_empty_document_fails_closed covers directly. The probe was re-run on 2026-10-30 and reproduced these numbers. Routing is not used: the board is wired into no routing fixture, and validate:maestro-routing does not exercise it.

### Negative probes the gate already carries

Each is a committed invalid fixture that must fail with exactly one rule id: DRC-PRINCIPAL-SOURCE--agent-writes-decision (an agent records a decision), DRC-REQUEST-BINDING--request-edited-after-decision (the request changed after the principal decided), DR-HARD-GATE--unresolved-hard-gate (approval recommended over an open HARD gate), STOP-LIVE-GUARD--mutation-dispatched (a mutation objective was dispatched) and HO-SCHEMA--unknown-credential-field (a credential-shaped field).

### Adversarial tests that remain plan-only

The security agent designs adversarial test plans and the repository never executes them. Before any adopter relies on the board, an adopter-run program should replay real handoff logs through the gate, run a red-team dataset against the deployed runtime, and measure the metrics each AGENT.md defines. None of that is done here, and no baseline exists in this repository.

## Implementation backlog

Ordered by priority. "Owner decision" marks items that need the repository owner rather than a contributor.

| Priority | Item | Why | Owner decision |
|---|---|---|---|
| 1 | Backfill execution_tier on the 344 untiered agents, and reconcile docs/execution-tiers.md ("must declare") with the optional schema field | Contradiction between doc and schema; three untiered targets of the rejected maestro hold execute tools | Yes: tier assignment is a per-agent judgment, and the gate's own notes call it a separate change |
| 1 | Fix the copilot-governance-maestro routing table and give it a routing fixture | Four named agents do not exist; the gate never checks the table | Yes: which real agents it should route to |
| 1 | Replace the hardcoded skill count at ROADMAP.md line 88 | Contradicts ROADMAP.md lines 31-34 | Trivial; no decision beyond approval |
| 1 | Reconcile docs/execution-tiers.md lines 236-242 (section "Harness tool grants must match the declared tier") with the Claude Code documentation and with the committed adapters, and verify what Cursor's `readonly` field does against Cursor's official documentation for the installed version | The stable-contract document says companion `allowed-tools` is enforced and that claude-code and cursor adapters carry no grant; Claude Code documents that `allowed-tools` restricts nothing, and committed adapters carry both fields | Yes: the document is a stable contract, so the owner decides the corrected wording |
| 1 | Decide where the TypeScript board's application-security, secrets, identity and credential-issuance questions should route, then repoint the remaining clauses and the generator template line listed under "Pre-existing defects found" (including the per-server MCP trust entry in the MCP tool-contract companion skill) | Those clauses name a "security board" or "application security board" that is not in the catalog | Yes: agentic-security-review-agent is scoped to agentic systems, so the target is an owner decision |
| 2 | Maestro re-entry review | Criteria are in ADR-0003 | Yes: the owner decides whether the criteria are met |
| 2 | Evaluate whether Claude Code offers a restricting counterpart to allowed-tools for skills | allowed-tools only pre-approves; a skill-level restriction would narrow the prompt-only gap. Verify against https://code.claude.com/docs/en/skills for the installed version before assuming one exists | No |
| 2 | Vertex AI Agent Engine and Google ADK gap | No asset exists; the framework table covers ADK only as a design-review input | Yes: a new asset must fit an existing provider (new providers wait for Milestone 3) |
| 3 | OpenTelemetry GenAI observability reference | Revisit when the conventions leave Development status (per https://github.com/open-telemetry/semantic-conventions-genai) | No |
| 3 | Business agents E1 to E7 | Only with a sponsor-measured value case that passes the gate and an owner attestation | Yes, per case |
| 3 | Red-team dataset and a tool-poisoning observer in mods/ | llm-ai-pipeline-test-review-agent flags a missing red-team dataset; no mod watches tool descriptions. A deterministic dataset fits the repository; LLM-graded evaluation is out of scope in ROADMAP.md section 4 | Yes: scope, and the declared reach level (L0 to L3) of any new mod |
| 4 | Fill each AGENT.md metric from adopter data | The metrics are defined and unmeasured | No; needs an adopter |

## Enterprise-readiness scorecard

This scorecard is the author's judgment against a rubric defined in this document, not an external assessment. The rubric scores the level of evidence reached, not how good the contract sounds.

| Score | Evidence level |
|---|---|
| 0 | None |
| 1 | Intent stated |
| 2 | Contract written and reviewable |
| 3 | Contract checked by a deterministic offline gate |
| 4 | Exercised against a real adopter's data or runtime |
| 5 | Sustained production evidence |

| Dimension | Score | Evidence | Why not higher |
|---|---|---|---|
| Security | 3 | No execute verb; principal-only approvals bound by request digest; HARD gate; 67 rules with sole-violation fixtures and a mutation probe; read-only declarations in four harness adapters | Sender authentication, authorization and audit storage are runtime-only; three adapters are prompt-level; enforcement of the declared grant is verified only for the Claude Code `tools` field, and the repository's own execution-tiers document disagrees; ASI02, ASI04, ASI05, ASI06 and ASI10 have no repository control; no red-team run |
| Operational burden | 2 | Nothing to operate in this repository; one added gate; two agents and three skills to install via two install roles | Adopters must build the runtime, record store and approver authentication; the cost of challenger, alternative and dissent for R2+ work is unmeasured |
| Interoperability | 2 | Envelope is a JSON Schema 2020-12 document; seven harness adapters; parity with the audit-event vocabularies is checked offline | No wire binding to MCP or A2A is implemented; framework and protocol facts drift; no other system has consumed the envelope |
| Economics | 1 | A value-case contract with interval arithmetic, counterfactuals and a promotion rule, with bookkeeping checked offline | No measured benefit, no baseline, and synthetic fixtures cannot exceed pilot. The arithmetic proves bookkeeping, not value |
| Documentation | 3 | This record, an ADR, five parity-checked protocol references, and per-agent authority cards; source claims are labeled by verification level | Parity checking covers rule tables, not prose accuracy; no external review |
| Adoption | 1 | Installable through the cloud-ai-platform-engineer and cloud-security-engineer roles | No adopter, no usage evidence, experimental lifecycle |

**Overall: Not Fortune 50-grade: evidence-ready contracts, no production evidence.** No dimension reaches level 4. The board can be adopted as a review contract and a record format today; nothing here shows it improves an outcome.

## Adversarial review outcome

Basis: two adversarial refuters probed the synthesized design in memory (including tests/validate-maestro-routing.py). The orchestrator's working decision record, board-decision.md, records the changes that were accepted; it is a session working file and is not committed to this repository. Later refuter passes reviewed the written assets, this record and ADR-0003, and their objections were applied. No refuter transcript is in the repository. board-decision.md records no per-asset APPROVE or APPROVE WITH CONDITIONS label from the first two refuters, and the verdicts of the later passes are not reproduced here. The rejections (A1, A3, A4 and E1 to E7) restate decisions that board-decision.md records. Every other label below is this record's own reading of the findings and the accepted changes. It is not a refuter verdict and must not be cited as independent assurance.

| Asset | This record's reading (not a refuter verdict) | Reason |
|---|---|---|
| Maestro (A1) | REJECT | Divergent second routing path; probe showed a live-guard bypass; untiered targets with execute tools |
| Architecture Review Chair (A3) | REJECT | Nothing for it to chair; the human owner resolves |
| Separate Decision Auditor agent (A4) | REJECT as an agent, COMPOSE as a contract | Same-model independence is procedural only |
| Business agents E1 to E7 | REJECT | No reachable baseline data |
| agentic-system-architecture-review-agent | APPROVE WITH CONDITIONS | Static review only; every verdict is a proposal subject to the designated security challenge and a human owner; per-harness limits stated honestly; metrics unmeasured |
| agentic-security-review-agent | APPROVE WITH CONDITIONS | Plan-only tests; ASI mapping only where a rule operates on the threat's object; the HARD gate can be overridden only by a named human with a risk acceptance reference |
| agentic-board-protocol | APPROVE WITH CONDITIONS | Crosswalk keeps the canonical tiers; framework mappings stay candidate-only; the EU AI Act is routed, not summarized |
| agentic-system-architecture-review | APPROVE WITH CONDITIONS | Framework facts are research-delegate findings cited "per" sources and must be re-verified for the installed version |
| agentic-security-review | APPROVE WITH CONDITIONS | MCP authorization and A2A claims are limited to the verified revisions and wording; verify against installed software |
| schemas/agentic-handoff.schema.json | APPROVE WITH CONDITIONS | Shape only; opaque subjects; no credential fields; useful only if the runtime authenticates senders |
| tests/agentic-board-suite | APPROVE WITH CONDITIONS | Oracles are not runtime enforcement; the determinism scan is a denylist; the one mutation survivor has its own covering test; it proves form, not truth |
| Kiro Power invariant text and install-role wiring | APPROVE | Each corrects a verified defect or wires the board into discovery |
| TypeScript per-server MCP-trust routing | APPROVE WITH CONDITIONS | Only the per-server MCP trust clauses were repointed, and one per-server entry in the MCP tool-contract companion skill still names the old board. Other clauses in the maestro and in the runtime-boundary, package-publication, MCP tool-contract and automation-governance specialists still route to a "security board" or "application security board" that does not exist; the backlog item holds the owner decision |

The accepted changes recorded in board-decision.md (not committed) are the conditions above in compressed form:

1. No agent ever writes a human decision; only a principal-sourced decision-record carries approved or rejected, bound to the request by sha256 of its canonical JSON.
2. Principals are opaque individual subjects plus a role; self-approval compares subjects; senders are authenticated by the runtime, not by this repository.
3. There is no execute verb; live work escalates with a route hint.
4. subject_risk_tier is the canonical tier of the decision under review; flags never remap it.
5. Tier, classification, materiality and prohibitions never shrink down a chain; deadline never extends; actions and budgets only attenuate; a missing parent is a violation.
6. Stop conditions are computed from message fields only.
7. The HARD gate forbids recommending approval over an open challenge, and only a principal with a risk acceptance reference can override it.
8. One owner per concept.
9. Every ASI row maps either to a repository rule on that threat's object or to runtime-only.
10. The honesty boundary on harness tool grants and on allowed-tools, with a per-harness strength column that separates the one verified declaration from the unverified ones.
11. The value case uses integer minor units, exact interval arithmetic, a benefit ceiling, an oversight floor, recomputed unit cost, mandatory counterfactuals and a promotion rule that synthetic records cannot reach.

## Owner-ordering exception

ROADMAP.md lines 23-25 order the growth axes as verification depth, then tooling and automation, then provider breadth, then distribution. Milestone 1 (verification depth) is still open: ROADMAP.md lines 62-76 list success criteria that are unchecked. This wave is breadth work, shipped while M1 is open, so it is an exception to that order.

Three things bound the exception. The board uses the existing generic provider, so no new provider is introduced and ROADMAP.md line 91 ("No new providers until Milestone 3") is respected; docs/taxonomy.md already says a topical board uses `provider: generic` until it ships a coherent set, with the qa board as the precedent. The repository owner explicitly requested the implementation. And the delivery is a draft pull request that the owner can decline or defer to Milestone 3, the provider and domain breadth milestone, without losing the design record or the ADR.
