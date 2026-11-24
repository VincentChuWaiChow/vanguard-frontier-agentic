# 🛡️ Kyverno Policy Engineering Board

<p align="center">
  <span style="font-size:3.5em">🛡️</span>
</p>

A board of accountable decision owners for Kyverno admission policy. Each seat owns one costly, recurring decision; the maestro routes the smallest set of seats that can resolve a case. Every seat is static-review: no cluster access, no shell, no mutation. Live changes go to the existing `kubernetes-live-admission-policy-guard-agent` as an approval-bound handoff.

Agents, harness adapters, shared evidence definitions and per-skill evidence ledgers are generated from `scripts/kyverno_data/` by `scripts/gen_kyverno_board.py`; edit the data, not the generated files.

## Seat inventory

| Seat (business customer) | Choice | Agent | Companion skill |
| --- | --- | --- | --- |
| Admission control architect (platform security) | Extended the existing agent | `kyverno-policy-review-agent` | `kyverno-policy-review` |
| Admission reliability engineer (SRE / on-call) | New agent | `kyverno-admission-reliability-agent` | `kyverno-admission-reliability` |
| Artifact trust reviewer (software supply chain) | Merged | `kyverno-policy-review-agent` (artifact admission reference) plus the existing `sigstore-cosign-supply-chain-review-agent` | `kyverno-policy-review` |
| Exception portfolio owner (service owner and risk delegate) | New agent | `kyverno-exception-governance-agent` | `kyverno-exception-governance` |
| Policy lifecycle engineer (GitOps / platform operations) | New agent | `kyverno-policy-lifecycle-agent` | `kyverno-policy-lifecycle` |
| Migration and fleet release owner (platform release manager) | New agent | `kyverno-migration-readiness-agent` | `kyverno-migration-readiness` |
| Control evidence analyst (assurance and product leadership) | New agent | `kyverno-control-evidence-agent` | `kyverno-control-evidence` |
| Kyverno maestro (requesting team) | New agent | `kyverno-maestro-agent` | `kyverno-review-routing` (+ `jira-ticket-authoring`, `confluence-document-authoring`) |

**Why artifact trust is merged.** Whether an ImageValidatingPolicy expresses the approved signer and attestation requirement is an admission-semantics question, answered in `skills/kyverno/kyverno-policy-review/references/artifact-admission.md`. Signer, trust-root and attestation design is already owned by `sigstore-cosign-supply-chain-review-agent`, whose scope covers Kyverno image-verify identity constraints. A separate Kyverno seat would duplicate one of the two and own no exclusive decision. Routing fixture `010-near-artifact-trust` pins the boundary.

## Overlap test for each new seat

| Agent | Accountable human | Costly recurring decision | Input that changes its answer | Gap in the existing owner | Misrouting fixture |
| --- | --- | --- | --- | --- | --- |
| `kyverno-admission-reliability-agent` | SRE / on-call for API write availability | Ship a policy change; diagnose failed writes | failurePolicy, timeout, replica count, measured latency | The reviewer judges rule semantics, not outages | `012-near-registry-incident` |
| `kyverno-exception-governance-agent` | Service owner and risk delegate | Grant, narrow, renew or expire a deviation | `expiresAt`, approval scope, adjacent workloads | The reviewer treated exceptions as a finding list, not a portfolio with removal evidence | `013-near-exception-expiry-annotation` |
| `kyverno-policy-lifecycle-agent` | Platform operations (GitOps) | Enable mutate-existing, generation or scheduled deletion | sync setting, source type, reconciler ownership | The GitOps owners review Argo CD/Flux, not Kyverno-generated state | `014-near-gitops-revert` |
| `kyverno-migration-readiness-agent` | Platform release manager | Move a cohort between policy families or releases | Controller version, parity on CREATE/UPDATE/DELETE | No owner covered the v1.20.0 legacy-engine removal | `015-near-update-parity` |
| `kyverno-control-evidence-agent` | Assurance analyst | State coverage or compliance to leadership | Inventory, freshness window, result breakdown | No owner held numerator/denominator discipline | `016-near-report-states` |
| `kyverno-maestro-agent` | Requesting team lead | Who owns which decision; consolidated answer | Number of decisions in the request; dissent | No Kyverno router existed | `020`/`021` mixed-intent, `031`/`032` Atlassian-only |

Not created as agents: CEL, YAML, documentation, metrics and individual CRDs. Policy testing and the developer-facing denial message are responsibilities of every seat.

## What the board borrowed from the Istio board

| Istio specialization | Question borrowed | Kyverno application (owner) |
| --- | --- | --- |
| Ambient enrollment | Which components actually see this workload? | Request matching, exclusions, admission versus background (`kyverno-policy-review-agent`) |
| Authorization | Which enforcement point can evaluate the required evidence? | Request identity and operation versus stored fields; native VAP versus Kyverno (`kyverno-policy-review-agent`) |
| Traffic resilience | What do dependencies do under stress? | Affected writes during webhook or registry degradation (`kyverno-admission-reliability-agent`) |
| Gateway API | Does declared intent have an accepted attachment? | Served policy type, webhook scope and a representative decision; object existence is weak evidence (`kyverno-policy-review-agent`) |
| Upgrade readiness | Which cohorts can move, and how is prior behavior restored? | Parity on a shared corpus; changes already made to workloads (`kyverno-migration-readiness-agent`) |
| Dataplane diagnostics | Which observation discriminates between causes? | Policy denial versus webhook error versus upstream rejection versus stale reports (`kyverno-admission-reliability-agent`) |
| Maestro | Who owns the next decision? | One lead, bounded referrals, existing live-guard handoff (`kyverno-maestro-agent`) |

Mixed mesh cases produce two claims: Kyverno can evaluate declared mesh configuration within its policy scope; effective traffic authorization needs Istio evidence and goes to `istio-maestro-agent` (fixture `017-near-mesh-label`).

## Verification layers

1. Structural: `npm run validate` (catalog, schemas, references, ledgers, routing) and `npm run validate:kyverno-board` (bindings, read-only tiers, generated drift, isolated export, stale-claim regressions).
2. Deterministic: `tests/fixtures/kyverno-maestro-routing/` (routing grader), `tests/fixtures/kyverno-board-decisions/` (decision-record validator, every rule proven to fire), `tests/fixtures/kyverno-policy-cli/` (pinned kyverno CLI v1.20.0; CI workflow `kyverno-policy-fixtures`).
3. Runtime: `tests/fixtures/kyverno-runtime-semantics/` specifies API-server/controller behavior and is marked not run.
4. Model behavior: `tests/fixtures/kyverno-board-evals/` is a rubric corpus; it stays not run until a recorded run is added to its manifest.

## 📦 Install

```bash
# The whole board (companion skills bundled; the Atlassian skills are not, because --provider scopes companions)
npx vfa-export-agents --platform claude-code --provider kyverno --repo .

# The maestro with its Atlassian companions (jira-ticket-authoring, confluence-document-authoring)
npx vfa-export-agents --platform claude-code --agents kyverno-maestro-agent --repo .
```
