# EXC-04 pilot answer (condensed)

Condensed by the orchestrator from the model's full answer; content preserved, wording shortened.

Model: claude-sonnet (subagent), harness: Claude Code subagent reading exported .claude/ install only, repetition 1.

Answer: the annotation date has passed but that does not mean the exception has expired. Only an annotation date, no expiresAt; nothing shows the controller stops honoring ledger-sync-root; treat it as probably still in force.
Outcome: conditional / needs-review; becomes blocked if the approval record requires enforced expiry.
Scope: policies.kyverno.io/v1 PolicyException, Kyverno v1.20.0 (assumed, not supplied).
Observed: annotation "Temporary until 2026-03-31" is an assertion, not a mechanism; invoice-batch-root fails offline.
Derived: only a set expiresAt makes the controller stop applying an exception [KYV-E13] [KYV-E14]; without it and without reconciliation evidence the exception stays in force. Exceptions disabled by default and exceptionNamespace restricts where they apply [KYV-E15]; flags not supplied so whether honored is unknown. Adjacent-workload result is a narrow negative probe on supplied inputs; does not prove match conditions bounded. Offline results cannot prove expiry (fixture 03: CLI still applies expired exception, controller skips it).
Disposition: expire-now; alternative renew-with-conditions with explicit RFC3339 expiresAt and expiry owner if risk delegate confirms need; live delete/renewal to kubernetes-live-admission-policy-guard-agent as approval-bound handoff; invoice-batch-root keep covered.
Residual exposure: unbounded bypass for ledger-sync-root ("probably running as root, going by the sibling policy name"; policy/rule not supplied).
Expiry verification method: installed-config with expiresAt; post-expiry admission-trace or background fail; object deletion unverified; or rendered-gitops reconciler removal.
Counterexample, falsifier, easiest way wrong (expiresAt omitted from summary, or exceptions disabled) stated.
Unknowns with settling items: expiresAt set; flags; approval record; match conditions and policyRefs; object live; expiry/removal owners.
Escalation: service owner and risk delegate; estate counts to control-evidence; policy change to policy-review; no self-approval.
Developer-facing note: borrowed wording from worked example, flagged as not from supplied evidence.
Files read: agent file; SKILL.md; evidence-vocabulary.md; disposition-and-expiry.md; exception-mechanics.md
