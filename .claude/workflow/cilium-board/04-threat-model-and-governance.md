# Threat Model and Governance

> Status: **BUILT — design record.** Previous: [03-board-contracts-and-orchestration.md](./03-board-contracts-and-orchestration.md) ·
> Next: [05-value-model-and-kpis.md](./05-value-model-and-kpis.md)

## 1. Default posture

Every Cilium agent is `static-review`: it reads supplied artifacts and nothing else (gate rule CB06).

> **Correction (refuter finding 1).** The first build kept the retained policy agent at `read-only-runtime`, allowing live reads "through an authorized read-only tool". That claim was false in practice: the rendered adapters grant only Read, Grep and Glob, so the agent could never reach such a tool, and the shared evidence-boundary skill forbade it anyway. The policy agent is now `static-review` like the rest of the board, and CB06 requires `static-review` for every Cilium agent. This removes a live-read ability the agent's earlier prose described; its adapters never granted it.

Rendered adapters grant only read tools where the harness supports a restriction: Claude Code `tools: "Read, Grep, Glob"` (gate rule CB10), Cursor `readonly: true`, Gemini a read-only allowlist, Copilot read and search aliases, Codex `features.shell_tool = false`. Kiro IDE and Kiro CLI have no verified restricting field, so on those harnesses the limits are prompt-level only. That is a real residual risk, stated in the adapter list of every `AGENT.md`, not a guarantee.

One correction to the previous state: the retained policy agent's old Codex adapter declared `sandbox_mode = "read-only"`. The shared renderer's docstring states, citing the Codex subagent documentation, that a custom agent file cannot set its own sandbox and inherits the parent session's; if so, that line implied a protection it did not provide. The re-rendered adapter uses `features.shell_tool = false` instead. This change did not re-verify the Codex documentation itself, and the Istio board's adapters still carry `sandbox_mode`; reconciling the two is a follow-up, not part of this change.

## 2. Separation of duties

Recommendation (specialist), independent review (a different reviewer on evidence it did not produce), deterministic validation (schemas, gates), human authorization (principal only; the handoff envelope reserves `approved` and `rejected` for a principal-sourced decision record), and execution (an existing guard or the human change process) are separate. No board agent holds more than the first two.

## 3. Mutation boundary

| Surface | Guard | Board behavior |
| --- | --- | --- |
| CNP, CCNP, NetworkPolicy, CiliumEgressGatewayPolicy | `kubernetes-live-network-policy-guard-agent` | names it, never dispatches it |
| ClusterMesh peer Secret, Gateway API resources | `kubernetes-live-network-architecture-mutation-guard-agent` | names it, never dispatches it |
| BGP, LB-IPAM, L2 announcements, Helm values, `cilium-config`, upgrades, CNI migration, key rotation | **none** | change proposal for the human change process |

The board deliberately does not add a Cilium live operator. A broad operator would be the most convenient asset on the board and the most dangerous; there is no guard design here strong enough to justify one, and the gap is recorded rather than filled with an unguarded tool.

## 4. Threat register

| ID | Threat | Attack surface and preconditions | Impact | Preventive controls | Detective controls | Residual risk | Test |
| --- | --- | --- | --- | --- | --- | --- | --- |
| T1 | Prompt injection through retrieved content | Ticket, page, log or manifest text addressed to the agent | Wrong route, false approval | Every skill treats retrieved text as data; routing ignores embedded instructions | Fixtures `adv-instruction-injection`, `adv-jira-injection` | Model-level compliance is not provable by a fixture | Routing fixtures; refuter review |
| T2 | Confused deputy | A user asks the board to "just apply" through a tool the host happens to grant | Unauthorized mutation | Read-only adapters; no execution authority in any contract; live guards never auto-dispatched | CB06, CB10; `live-*` fixtures | Kiro adapters are prompt-only | Negative probes in this change |
| T3 | Tool escalation through delegation | Maestro passes broader tools to a specialist | Privilege widening | Routing grants no tools; envelope `authorized_actions` cannot widen down a chain | `agentic-board-protocol` gate | Runtime enforcement is the host's | — |
| T4 | Wrong-target collection | Live read against an ambient kubeconfig | Cross-cluster or cross-tenant leakage | No agent reads live state; collection plans name the exact cluster, context and namespace for the operator | Output must name cluster scope | User may confirm the wrong cluster | — |
| T5 | Unsafe command generation | Agent names a command whose side effects the reader does not expect | Accidental mutation or data exposure | Six-class command classification, by behavior | CB01, CB02 | A misclassified command passes the gate | Refuter checked classes |
| T6 | Fabricated or stale evidence | Delegated writers or stale docs | Wrong conclusion presented as fact | Every upstream claim cites a ledger row with date and status; fact sheets from primary sources; Context7 cross-check | `validate:skill-references` | Ledger rows are documentation checks, not behavioral tests | Ledger negative probe in this change |
| T7 | Version drift | `/en/stable/` moves at each release | Rules applied to the wrong version | Version-gated statements; unknown version withholds conclusions | Ledger dates | Requires manual refresh each minor | — |
| T8 | Poisoned handoff | A forged "approved" in an envelope | Execution on a fake approval | Envelope reserves approval for principals; board never treats `approved` as consent | agentic-board gate | Off-repo runtimes may ignore it | — |
| T9 | Circular delegation or token exhaustion | Maestro re-routing loops | Cost, no answer | At most four owners; envelope `max_hops` | — | No runtime budget enforcement here | — |
| T10 | False incident closure | Ticket closed without reassessment | Recurrence | Closure is not proof; reassess against the same discriminating observation | Handoff rules | Depends on human process | — |
| T11 | Telemetry blind spots treated as proof | "No flows observed" | Missed intrusion, bad default-deny | Flow-telemetry rules; absence is never evidence | Response must state coverage | — | Scenario in flow-telemetry reference |
| T12 | Secret leakage through bundles | Sysdump or bugtool attached to a ticket | Credential exposure | Board never requests bundles; asks for excerpts; advisory cited | — | A user may attach one anyway | — |
| T13 | Audit evidence manipulation | Bounded review presented as certification | Failed or misleading audit | `approved` defined as bounded, never certification | Output contract | Readers can still misquote | — |

## 5. MCP and external-service boundaries

Research for this change used Context7 (`resolve-library-id`, `query-docs` against `/cilium/cilium` 1.20.4 and the docs index) and direct page retrieval of docs.cilium.io and github.com/cilium. No proprietary configuration, credentials or identifiers were sent to either service. The board itself declares no MCP dependency and its adapters grant no MCP tools.
