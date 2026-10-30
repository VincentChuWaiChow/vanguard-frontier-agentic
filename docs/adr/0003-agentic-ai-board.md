---
layout: default
title: "ADR-0003: Agentic AI Board Without a Maestro"
permalink: /docs/adr/0003-agentic-ai-board/
---

# ADR-0003: Agentic AI Board Without a Maestro

## Status

Proposed

This ADR becomes Accepted only when the repository owner accepts it. Nothing in the implementation depends on the status field, and the draft pull request that carries it can be declined or deferred.

## Corrections after independent review

An independent adversarial review (chair verdict REDESIGN, weighted score 63.2/100, no hard gate failed) found that this ADR and the design record contradicted the pull request's own diff and overstated several controls. Most defects have since been fixed in code, and this ADR now describes the code as it is. The most important corrections:

- **docs/execution-tiers.md was amended in this PR.** This ADR described its conflict with the harness findings as a pre-existing defect that this wave does not fix, and called Cursor's behavior unverified. This PR rewrites the enforcement table (a SKILL.md `allowed-tools` line is marked "No" in Claude Code, with rows for claude-code `tools:`, cursor `readonly` and gemini), and Cursor `readonly` is verified as a partial restriction. The owner must explicitly accept or revert this change to a stable-contract document.
- **The gemini adapters carried a maximum grant, not "no tool grant".** An omitted `tools` field gives the agent every parent tool. They now carry a read-only allowlist (read_file, grep_search, glob, list_directory), asserted by the suite, so five adapters declare a grant and kiro-ide and kiro-cli stay prompt-level.
- **Counts and the HARD gate.** The gate has 85 rules, not 68, with 12 valid and 108 invalid fixtures. The HARD-gate guarantee now holds over every challenge on the proposal, because DR-CHALLENGE-COMPLETE requires every challenge sent no later than the request, or than a decision-record bound to it, to be attached; before, it held only for attached challenges and could be bypassed.
- **Mutation evidence and oracles.** The earlier 65-site probe used deletion operators only. The suite now commits 15 targeted mutants and kills all 15, and a full mutation probe is still open (RB-08). The mutation-intent and instruction-bait patterns are low-recall denylist canaries that fail open; they were wrongly described as broad and fail-closed.
- **Entering the board.** This ADR said an existing vendor maestro could choose a board agent. No maestro routing taxonomy maps to either board agent, so only a user can enter the board, by naming an agent, until inbound routes exist (RB-19).
- **Alternative 5.** The split into two agents is now justified on domain and context grounds, not on a rule this PR introduced, and a seeded-defect comparison of one merged agent against two has not been run.

The remaining corrections and the open items are in docs/architecture/agentic-ai-board.md, section "Corrections after independent review" (cited by path, as elsewhere in this ADR).

## Date

2026-10-30

## Context

The repository owner asked for the smallest coherent set of specialized agents that can solve consequential enterprise problems around agentic AI systems, coordinate under controlled authority, survive hostile inputs and prove measurable value. The owner supplied a discovery roster of 29 candidates (A1 to A5, B1 to B6, C1 to C5, D1 to D5, E1 to E8) and asked that each be classified EXISTS, EXTEND, COMPOSE, NEW or REJECT, with no new asset where an existing one should be extended.

Facts that shaped the decision, all verified in the repository on 2026-10-30:

- **This repository is product A, not product B.** It ships role contracts, skills, JSON Schemas and offline gates. It is not an orchestration runtime. ADR-0001 already says the architecture "does not natively support multi-agent orchestration at the catalog level", and that the system is "a content package (npm), not a runtime".
- **Much of the brief already exists.** The canonical R0-R5 action risk scale (docs/compliance/applicability-engine.md and schemas/audit-event.schema.json) collides with the brief's R0-R4. Execution tiers (docs/execution-tiers.md), a governance control plane (the python-live agents), domain handoff capsules, an adversarial frontend board and a set of vendor agentic governors are all present.
- **Real gaps exist.** No vendor-neutral review of agentic system architecture, no generic state and durable-execution design, no A2A or agent-identity coverage, no OWASP agentic mapping, no consumer-side provenance review and no automation decision method with a value model.
- **Routing is deterministic keyword scoring** (tests/validate-maestro-routing.py), and 12 of the 13 targets a board maestro would route to are already routed by their own provider maestro.
- **Roadmap constraints.** ROADMAP.md orders verification depth before provider breadth and its first milestone is still open. It says no new providers until Milestone 3. The generic provider already hosts topical boards (docs/taxonomy.md, with the qa board as the precedent).
- **Gates stay deterministic.** They never consult the wall clock or the network, and LLM-graded evaluation is out of scope.

Evaluation details, the full inventory and the per-candidate table are in docs/architecture/agentic-ai-board.md (the architecture directory is not published on the docs site, so it is cited by path rather than linked).

## Decision

1. **Ship two static-review agents** in agents/cross-functional under the generic provider: agentic-system-architecture-review-agent (pattern selection, delegation and handoff review, state and durability, cascading-failure containment) and agentic-security-review-agent (composed attack surface mapped to the OWASP Top 10 for Agentic Applications, identity and delegated authorization, tool and memory risk, provenance). Both are experimental. Each is a proposal-maker; neither decides.
2. **Ship three skills:** a companion skill per agent and agentic-board-protocol, the layer-3 protocol that owns the handoff envelope, computed stop conditions, the adversarial decision process, the risk-tier crosswalk and the value case. Skills declare `allowed-tools: Read Grep Glob`, which pre-approves tools and does not restrict them.
3. **Ship one JSON Schema and one offline gate.** schemas/agentic-handoff.schema.json defines the only channel between board agents and principals (shape only). tests/agentic-board-suite checks every cross-field rule with its own rule id and a sole-violation fixture, wired into `npm run validate` as validate:agentic-board.
4. **Fix the authority model in rules, not prose.** Agents never decide; only a principal-sourced decision record carries approved or rejected, bound to the request by the sha256 of its canonical JSON. The envelope has no execute verb. subject_risk_tier is the canonical R0-R5 tier of the decision under review; the owner's scale is crosswalked, never redefined, and materiality flags force full-depth review without remapping the tier. Stop conditions are computed from message fields only. Designated challengers differ by decision class and a proposer never challenges its own proposal. A security challenge marked as a HARD gate forbids recommending approval, and only a principal with a recorded risk acceptance can approve over it. This holds over every challenge on the proposal, not only attached ones: every challenge sent no later than the decision request (or a decision-record binding it) must be attached, and a HARD gate clears only when the same challenger sends a later round that is itself a HARD-gate withdrawal.
5. **Do not ship a maestro in this wave.** Re-entry criteria are below.
6. **Do not ship a chair, a standalone decision-auditor agent, or business-value agents (E1 to E7).** The decision auditor is composed as a contract: designated challengers, deterministic rules and the finance challenger.
7. **Compose rather than duplicate.** Single-vendor work goes to that vendor's agent or maestro; MCP tool-schema fidelity to typescript-mcp-tool-contract-agent; evaluation design to llm-ai-pipeline-test-review-agent; model unit prices to finops-ai-economist-agent; value-case challenge to finance-capital-allocation-advisor-agent; regulatory classification, including the EU AI Act, to legal-regulatory-compliance-agent; runtime authorization to the deploying runtime or the python-live control-plane pattern; live mutation to the owning provider's gated live-guard path.
8. **Make three small extensions** that keep neighbors truthful: the generic Kiro Power invariant text and agent directories (scripts/generate-kiro-powers.mjs), the install-role wiring (catalog/install-roles.json), and the TypeScript board's per-server MCP trust routing, which pointed at a "security board" that does not exist. The TypeScript fix is partial: other clauses, listed in the design record, still name a board that does not exist.
9. **State the honesty boundary everywhere it applies.** Nothing here authenticates senders, enforces authorization or stores audit records. Tool restrictions are declared in the copilot, codex, cursor, claude-code and gemini adapters; the kiro-ide and kiro-cli adapters carry no verified restricting field here, so their limits are prompt-level only. The gemini adapters first omitted `tools`, which per https://geminicli.com/docs/core/subagents/ gives the agent every parent tool, so they now carry a read-only allowlist (read_file, grep_search, glob, list_directory; names and kinds per https://geminicli.com/docs/reference/tools/), asserted by the suite. Whether a harness enforces its declaration is vendor behavior. It is verified here for the Claude Code subagent `tools` field (per https://code.claude.com/docs/en/sub-agents) and, as a partial restriction, for the Cursor `readonly` field, which gives restricted write permissions (no file edits, no state-changing shell commands) per https://cursor.com/docs/subagents. A SKILL.md `allowed-tools` line does not restrict tools; it only pre-approves them, and `disallowed-tools` is the restricting field (per https://code.claude.com/docs/en/skills). This PR amends the enforcement table in docs/execution-tiers.md to match; that is a change to a stable-contract document, and the owner must explicitly accept or revert it. Framework mappings are candidate review aids that an accountable owner confirms, never certification.
10. **Bound the value case.** Synthetic repository records can never exceed pilot status. A real promotion needs measured evidence held outside the repository and a human attestation.

## Consequences

### What becomes easier

- **Reviewing an agentic system without a vendor lens.** Pattern selection, durability, containment and the composed attack surface now have an owner each, and a "should this be multi-agent at all" question has a defined answer path.
- **Auditing a handoff.** A record set can be checked offline against 85 rules, each failing under its own id, including the rule that only a principal records a decision.
- **Adding a rule.** A new rule needs a predicate, a sole-violation fixture and a documented row; the gate fails if any of the three is missing.
- **Keeping one owner per concept.** The protocol's ownership table prevents a second asset from quietly restating the first.

### What becomes harder

- **Entering the board.** With no maestro, nothing classifies a task and picks the agent. No maestro routing taxonomy maps to either board agent, so only a user can enter the board, by naming an agent, until inbound routes exist (open item RB-19). Cross-vendor tasks are routed by hand.
- **Using the record format.** The envelope has to be produced and stored by a runtime or a harness that the adopter provides. Without it the format is documentation.
- **Review cost at R2 and above.** A designated challenger, at least one alternative and a dissent entry for each open challenge add work. That cost is unmeasured.
- **Keeping facts current.** MCP, A2A, the OpenTelemetry GenAI conventions and every agent framework change faster than this repository's release cadence. The assets mark such facts "verify against the official documentation for the installed version" rather than asserting them.
- **Prompt-only limits on two harnesses.** Anyone running the board under Kiro (kiro-ide or kiro-cli) relies on the prompt for the read-only boundary, because those adapters carry no verified restricting field.
- **Unverified enforcement on three more.** The Copilot and Codex declarations are present, but this wave verified no vendor documentation for them. The Gemini allowlist rests on vendor documentation for why it is set (an omitted `tools` field inherits every parent tool), not on a test of how Gemini enforces it. Anyone relying on one of these should confirm its behavior for the installed version. Cursor's `readonly` is verified as a partial restriction, not as a tool allowlist.

### Risks

- **False assurance.** A green gate or a valid envelope can be read as approval or as safety. It proves form and internal consistency, not truth, identity or approval.
- **Oracle evasion.** The mutation-intent and instruction-bait patterns (LIVE_GUARD_INTENT and INSTRUCTION_BAIT) are fixture oracles in an offline gate. They are low-recall denylist canaries that fail open, not runtime enforcement, and a hostile objective can avoid them. The suite pins known misses, known false stops and bait negatives, and the agents are told to stop on meaning, not on the canary.
- **Procedural, not epistemic, independence.** Designated challengers have separate contracts and contexts, but the same underlying model family may run both. Agreement between them is weaker evidence than agreement between independent humans.
- **Ordering exception.** This wave is breadth work while the verification-depth milestone is open. The cost is attention that M1 would have used.
- **Pre-existing defects remain.** 344 agents have no declared execution tier, one Microsoft maestro names four agent ids that do not exist, ROADMAP.md hardcodes a skill count, and several TypeScript specialists still route to a security board that does not exist. This wave records them in the design record and fixes none of them, except the TypeScript per-server MCP trust clauses, which it fixes in part. It also amends the enforcement table in docs/execution-tiers.md, which described harness tool grants differently from the Claude Code documentation and from the committed adapters; that amendment awaits the owner's explicit acceptance or reversal.

### Mitigations

- The honesty boundary is repeated in the schema description, the protocol, both agent contracts and the design record, so a reader of any one of them meets it.
- Every rule has a sole-violation fixture, and every invalid fixture fails exactly its one declared rule. An earlier probe that deleted rule add-sites found 65 sites and killed 64 by fixtures, but it used deletion operators only, and the independent chair found 5 of 5 targeted comparison, boolean and constant mutants surviving. The suite now commits 15 targeted mutants (TargetedMutantTests) and the corpus kills all 15. A full mutation probe with an enumerated list of equivalent mutants is still open (RB-08, partial).
- The gate runs an AST denylist scan for listed wall-clock, network and randomness imports and calls. A denylist does not catch indirect forms, which the design record lists, so determinism also rests on review. The gate is meant to change result only when committed data changes.
- Facts are labeled by verification level, and anything not confirmed from a primary source is left out or marked for verification.
- The pull request is a draft, the ordering exception is stated in it, and the owner can defer the whole wave to Milestone 3 without losing the design record.

## Alternatives considered

1. **Ship the full roster of 29.** Rejected. Twenty of the 29 candidates are composed from existing assets, split across owners, deferred or rejected rather than built, the seven business agents have no reachable baseline data, and the owner asked for the smallest coherent set.
2. **Add a board maestro (A1).** Rejected for this wave. It would own two leaves, duplicate routing that vendor maestros already do, and add a divergent second path. A probe showed its live-guard pattern routing a destruction request to a single agent that its own provider taxonomy gates. Four of its targets declare no execution tier and three hold execute tools. Adding any generic maestro would also change the generic Kiro Power's router.
3. **Add a chair agent (A3).** Rejected. The human decision owner resolves every decision request. The frontend board chair is a domain chair over ten workflows, which two specialists do not justify.
4. **Add a standalone decision-auditor agent (A4).** Rejected as an agent and composed as a contract. A same-model auditor adds procedural independence only.
5. **Merge architecture and security into one agent.** Rejected. Architecture review and security review are separate knowledge domains with separate context budgets, so two agents let each keep its own context. This is a domain and context judgment, not a consequence of any rule this PR introduced. A seeded-defect comparison of one merged agent against the two would test it, and that comparison has not been run.
6. **Reuse a domain capsule or extend the python-live control plane for handoffs.** Rejected. The existing capsules are domain-specific and carry no decision request, challenge or computed stop condition. The python-live plane is scoped to Python live-change governance, so the board points to it as the runtime authorization pattern.
7. **Adopt the owner's R0-R4 scale.** Rejected. It collides with the canonical R0-R5 scale that the audit-event schema already enforces. The board crosswalks onto the canonical scale.
8. **Create a dedicated provider for the board.** Rejected. ROADMAP.md forbids new providers until Milestone 3, and the generic provider already hosts topical boards.
9. **Build an orchestration runtime.** Out of scope. That is product B.
10. **Ship nothing and defer to Milestone 3.** Considered. The owner explicitly requested implementation, and the draft pull request keeps this option open.

## Re-entry criteria for the maestro

A board maestro may be reconsidered, by a new ADR that supersedes this decision, only when all of these hold:

1. At least four leaves are owned by the board and are not routed by any other maestro.
2. Vendor-specific terms hand off to the owning vendor maestro instead of routing to a board agent, so there is no divergent second path.
3. The maestro's live-guard intent pattern is a tested superset of the live-guard intent pattern of every board it routes to.
4. Every routing target declares a static-review or read-only-runtime tier and holds no execute tool.
5. A routing fixture exists under tests/fixtures with a taxonomy, inputs and expected files generated from the grader, with guarded-mutating-live agents listed as live guards (see CLAUDE.md, "Adding a maestro / router agent").
6. The generic Kiro Power's router selection is checked, so that the new maestro does not silently become the router for unrelated generic agents.

The repository owner decides whether the criteria are met.
