# Risk-Tier Crosswalk

Reference for [agentic-board-protocol](../SKILL.md). Load it when you assign subject_risk_tier, translate a requester's own risk scale, relate board work to the execution tiers, or draft a candidate framework mapping.

The board never defines risk labels of its own. The envelope's subject_risk_tier uses the canonical R0 to R5 scale already in this repository, and it describes the decision under review, not the board's own work. The board's own work is static review.

## Canonical action risk tiers

Quoted from docs/compliance/applicability-engine.md. That file is the authority; if the two ever differ, the file wins.

| Tier | Meaning | Gate posture |
|---|---|---|
| R0 | Read-only, non-sensitive | Observe; audit event recommended |
| R1 | Read-only, sensitive data in scope | Read-only-runtime; runtime-evidence-gate; redaction |
| R2 | Reversible non-production change | Plan + policy gate; approval per policy |
| R3 | Production change, bounded, reversible | Independent approval + JIT + rollback + audit event; fail closed if audit unavailable |
| R4 | Production change, broad blast radius | R3 controls + blast-radius assessment + change window |
| R5 | Irreversible / high-blast-radius production action | R4 controls + second approver + explicit risk acceptance |

The same file states that an unknown for any R3+-relevant input blocks until an owner resolves it. The board follows that rule: when the evidence does not settle whether the tier is R2 or R3 or above, or which of R3, R4 and R5 applies, the missing input is blocking missing evidence and the work stops and escalates. A doubt between two tiers is never settled by quietly choosing the lower one.

### Assigning the tier

- Classify the decision under review by what it would authorize or fund: the change it would make, its reversibility, its blast radius and the sensitivity of the data in scope.
- Set materiality flags separately, on their own criteria.
- Carry the tier unchanged or higher down a chain. If a reviewer thinks the tier was overstated, it says so in its verdict; a principal re-issues the work under a new correlation.
- Illustrations (synthetic): a read-only question-and-answer assistant over internal documents is R0, or R1 when sensitive data is in scope. A single bounded, reversible production configuration change is R3. Granting an agent standing write access to a production system is at least R3, and R4 when the write scope it opens has broad blast radius. Permanently purging an off-boarded tenant's long-term agent memory is R5 with the irreversible and regulatory flags.

## Crosswalk from the owner's proposed scale

The repository owner's brief proposed a five-level scale, R0 to R4, that reuses the labels of the canonical scale with different meanings. Envelopes always use the canonical scale. Translate as follows.

| Owner's level | Owner's meaning | Canonical tier |
|---|---|---|
| R0 | Analysis | R0 or R1, by data sensitivity: R0 when non-sensitive, R1 when sensitive data is in scope |
| R1 | Read-only | R0 or R1, by data sensitivity |
| R2 | Sandbox | R2, reversible non-production change |
| R3 | Reversible, controlled enterprise change | R3, production change, bounded and reversible |
| R4 | Financially material, security-critical, destructive, regulatory, or hard to reverse | R3, R4 or R5, set by the change's own blast radius and reversibility. The owner's reasons become materiality flags, which never remap the tier |

How the owner's R4 translates. The owner's R4 is defined by the nature of the change, while the canonical tier is defined by its blast radius and reversibility, so the owner's label does not set the canonical tier. The change does:

- By blast radius and reversibility. A bounded, reversible production change is canonical R3. A production change with broad blast radius that is still reversible is canonical R4. An irreversible or high-blast-radius production action is canonical R5.
- By nature. Financial, regulatory, security-critical, irreversible and external-communication concerns become materiality flags with those names. The schema also has a personal-data flag, which the owner's scale does not.
- Flags force full-depth review and principal escalation. They never remap the tier. A financially material change that is bounded and reversible stays R3 with the financial flag; an irreversible one is R5 with the irreversible flag.
- Destructive or hard-to-reverse work gets the irreversible flag. R3 means reversible, so hard-to-reverse work is not R3 until its reversibility is shown, and a destructive action that cannot be undone is R5.
- An unknown about blast radius or reversibility is blocking missing evidence under the rule above. It is not a reason to pick the lower tier.

## Crosswalk to execution tiers

The R scale and the execution tiers answer different questions. R describes the risk of the action a decision would authorize. The execution tiers in docs/execution-tiers.md describe what an agent or skill is allowed to do.

| Execution tier | What it means in docs/execution-tiers.md | How the board relates to it |
|---|---|---|
| T0 static-review | Files and content provided in the conversation; Read, Grep, Glob | The tier of the board's own agents and skills. A high subject_risk_tier never raises the board's own tier: the board reviews decisions about high-risk actions without taking them. |
| T1 read-only-runtime | Read-only operational access to a connected system | Not used by the board's two agents. A composed challenger, finance-capital-allocation-advisor-agent, declares read-only-runtime in the catalog. |
| T2 sandbox-mutating | Mutation only when reversible or never committed | Not dispatched by the board. |
| T3 production mutation | Prohibited for agents: "Production mutations are NEVER agent-callable." | The board's envelope has no execute verb. A request to mutate is a stop condition, not an objective. |
| mutating-runtime | Agent-level live guards, gate-only: an approval token naming the exact target, a PREFLIGHT dry-run, an idempotency key, a signed attestation and a ROLLBACK | The board never dispatches them. An escalate envelope may carry a route_hint pointing a human at the owning provider's live-guard path. |

An approved R3 or higher decision describes a production change. Making that change through an agent is outside this board and, in docs/execution-tiers.md, goes only through a gated live-guard path with human approval.

A tier is a declaration until a harness enforces it. For the board's two agents, claude-code `tools: "Read, Grep, Glob"` and gemini `tools:` are documented allowlists (an omitted list inherits every parent tool), cursor `readonly: true` blocks file edits and state-changing shell but not every tool, copilot `tools:` follows this repo's own gates, and the codex adapter sets `features.shell_tool = false` but cannot set its own sandbox (a Codex custom agent inherits the parent session's, per https://developers.openai.com/codex/subagents); under codex, kiro-ide and kiro-cli the no-write limit is prompt-level only. The per-harness table in [docs/execution-tiers.md](../../../../docs/execution-tiers.md) is the single source for these claims. A SKILL.md allowed-tools line pre-approves tools; it does not restrict them ([Claude Code skills documentation](https://code.claude.com/docs/en/skills)).

## NIST AI RMF candidate mapping

The mapping below uses the [NIST AI RMF core](https://airc.nist.gov/airmf-resources/airmf/5-sec-core/), which has the functions GOVERN, MAP, MEASURE and MANAGE. The page carries the banner "The AI RMF 1.0 is being updated. A revised version is in progress." Read each subcategory's exact wording on the NIST page for the version in force; the descriptors below are short paraphrases, except where quoted.

This is a candidate review aid. It says which board mechanism supports a subcategory. It never says the board satisfies one. An accountable owner decides what counts, in the manner of docs/compliance/framework-profiles.md, where a framework mapping stays a candidate until an owner confirms it and agents never set the confirmation.

| Subcategory and short descriptor | Board mechanism that supports it |
|---|---|
| GOVERN 2.1, roles and responsibilities | The one-owner-per-concept table in the skill, the authority cards in the two agent contracts, and a subject and role on every principal. |
| GOVERN 3.2, roles for human-AI configurations and oversight | Decision requests always go to a principal; agent messages cannot carry approved or rejected; the human decision is a separate message type. |
| GOVERN 6.1, third-party risk | The security review's provenance review of tools, skills and MCP servers. The board reviews; it does not manage the supplier. |
| MAP 1.4, business value defined | The value case: baseline, verified-benefit interval and measurement plan ([value-case.md](value-case.md)). |
| MAP 3.5, human oversight processes | The principal decision step, and the oversight cost carried in the value case. |
| MAP 4.1, third-party component risks | The agent component inventory in the security review. |
| MEASURE 1.3, assessors who were not front-line developers | The designated challenger, a separate contract from the proposer. Independence is procedural, not epistemic; whether it meets this subcategory is the owner's call. |
| MEASURE 2.7, security and resilience | The security review and the architecture review's reliability and containment reference; stop conditions as containment in the record. |
| MANAGE 1.1, whether development or deployment should proceed | The decision-request and the principal's decision-record: the proceed-or-not decision lands with a human. |
| MANAGE 2.1, "viable non-AI alternative systems, approaches, or methods" | The automation ladder and the required counterfactuals ([value-case.md](value-case.md)). |
| MANAGE 2.4, supersede, disengage or deactivate | The architecture review checks a design's kill switch and disengage path. The board has no runtime, so it has no disengage mechanism of its own. |
| MANAGE 4.1, appeal and override, incident response | An override of a HARD gate is a principal's decision with risk acceptance and acknowledged dissent, and dissent is preserved for later appeal. The board runs no incident response. |

## EU AI Act

Route regulatory classification, including EU AI Act risk class and the organization's role under the Act, to legal-regulatory-compliance-agent; the board does not classify. The regulatory materiality flag forces full-depth review and principal escalation, but it is a flag and not a legal classification. Per [EUR-Lex](https://eur-lex.europa.eu/eli/reg/2026/1744/oj/eng), a Digital Omnibus on AI was adopted as Regulation (EU) 2026/1744 and amends obligations and dates, so pre-Omnibus summaries may be stale. This reference states no dates. Check obligations and application dates on EUR-Lex for the version in force before using any of them, and have a qualified owner confirm classification and role.
