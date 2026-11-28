# Which specialist owns this decision, and what is the nearest wrong owner?

## Question

Given a request, which single board seat leads each decision, which near neighbour must not take it, and which routing fixture proves the boundary?

## Release scope

Routing does not depend on a Kyverno release. The specialists apply release scope; for example a legacy ClusterPolicy question on v1.20.0 or later is still routed to admission design, which then reports that the legacy engine is deleted [KYV-E01].

## Assumptions

- The repository grader for this table is keyword based (`tests/validate-maestro-routing.py` with `tests/fixtures/kyverno-maestro-routing/taxonomy.json`); it proves boundaries for the committed phrasings only, not model judgment.
- A request the grader leaves `unclassified` is not a failure: Jira-only and Confluence-only wording deliberately leaves the board.

## Evidence to request

- The request text and any decision already made (with its ID).
- Whether the request is advisory, runtime evidence collection, or an explicit execution request.

## Interpretation

| Owner | Decision it owns | Real trigger | Exclusion (goes elsewhere) | Evidence boundary | Routing fixture |
| --- | --- | --- | --- | --- | --- |
| `kyverno-policy-review-agent` | Does the rule separate legitimate from prohibited writes; does an image rule verify the approved signer and claims? | "Does this ValidatingPolicy still let invoice retries through?" | Webhook outage (reliability); signer trust design (`sigstore-cosign-supply-chain-review-agent`) | Manifests, installed config, offline tests; never claims observed traffic | `001-happy-admission-design`, `010-near-artifact-trust`, `011-near-pre-existing-resource`, `017-near-mesh-label` |
| `kyverno-admission-reliability-agent` | Can admission sustain the change and recover; which hypothesis explains a failure? | "What happens to pod writes if the controller is unreachable?" | Whether the denial was correct (policy review) | Webhook config, metrics, admission traces; no probes | `002-happy-admission-reliability`, `012-near-registry-incident` |
| `kyverno-exception-governance-agent` | Is the deviation necessary, bounded and verifiably removed? | "Is this PolicyException renewal still bounded?" | Estate coverage figures (control evidence); creating the exception live (guard) | Exception manifests, flags, post-expiry evidence; offline results never prove expiry | `003-happy-exception-governance`, `013-near-exception-expiry-annotation`, `051-adv-approval-in-ticket` |
| `kyverno-policy-lifecycle-agent` | Will mutate/generate/delete converge, and what remains after reversal? | "Will this GeneratingPolicy fight Argo CD over downstream objects?" | Argo CD or Flux configuration itself (`argocd-gitops-review-agent`, `fluxcd-kustomization-helmrelease-review-agent`) | Rendered GitOps output, object state; never runs syncs or cleanup | `004-happy-policy-lifecycle`, `014-near-gitops-revert` |
| `kyverno-migration-readiness-agent` | Which cohorts can move with behavior preserved, and what aborts? | "Which cohort can migrate from legacy ClusterPolicy before v1.20?" | Single-rule semantics (policy review); Istio upgrades (`istio-upgrade-readiness-agent`) | Versions, parity corpus, controlled tests; offline UPDATE parity is not proof [KYV-E24] | `005-happy-migration-readiness`, `015-near-update-parity` |
| `kyverno-control-evidence-agent` | What coverage and impact can the evidence support, for which population and window? | "What is the denominator behind our coverage percentage?" | Why a cohort has no results (policy review or reliability) | Inventories, reports, metrics, events; blocked creates are not objects [KYV-E09] | `006-happy-control-evidence`, `016-near-report-states` |

Mixed requests route to the owners of each decision (`020-mixed-exception-and-coverage`, `021-mixed-promotion-and-latency`); the maestro then names one lead per decision ID and turns the other owner into a referral with one return question.

Leaves the board: Jira-only wording goes to `jira-ticket-authoring` (`031-negative-jira-only`); Confluence-only pages go to `confluence-document-authoring` (`032-negative-confluence-only`); explicit live changes reach the live-guard gate (`040`-`043`), while "review whether applying this is safe" stays advisory (`030-negative-review-apply-safety`).

## Worked example (synthetic)

**Requester supplied**: "Can we promote cohort A to Deny in the next release without p99 admission latency exceeding its budget?"

**Agent derived**: Two decisions. KYV-DEC-0101 (promotion) is led by `kyverno-migration-readiness-agent`; the latency question becomes one referral to `kyverno-admission-reliability-agent` with the return question "What is the p99 admission latency for pod CREATE in payments during the canary window?". Promotion to enforcement needs an independent challenger: `kyverno-policy-review-agent`, naming invoice-retry-worker as the legitimate workload that must pass.

**Still needed**: the canary window metrics and a controlled test with real old/new UPDATE pairs.

**Decision today**: `conditional` / `needs-review` until the referral returns and the challenge runs.

**Cheapest discriminating check**: the p99 answer from the referral; a breach ends the decision without further work.

**Developer-facing denial message**: not applicable to routing; the lead's denial message travels with its verdict.

**Escalation owner**: platform release manager.

**Keep out of evidence**: tenant identifiers, kubeconfig content, tokens.

## Counterexample

Five specialists each say "cohort A is fine" because each read the same Audit report. The reliability engineer alone notes that the pre-canary latency baseline is unknown. That named missing evidence decides the latency point; agreement among the others does not.

## Escalation threshold

When two owners claim the same decision and no evidence would discriminate, record the ownership boundary as unresolved and name the human owner; do not pick by count.

## What would change the verdict

- A new decision appearing in the request (route it separately).
- A referral answer that contradicts the lead's provisional conclusion.
- An explicit execution request (switch to the live handoff).
