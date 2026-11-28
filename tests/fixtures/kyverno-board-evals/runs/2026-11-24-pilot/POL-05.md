# POL-05 pilot answer (condensed)

Condensed by the orchestrator from the model's full answer; content preserved, wording shortened.

Model: claude-sonnet (subagent), harness: Claude Code subagent reading exported .claude/ install only, repetition 1.

Outcome: The cohort A canary is conditional (needs-review), low severity. The mandatory non-root control is blocked (blocked) for the prevention requirement, high severity. Both policies use the same validationActions: [Audit]. The ratings differ because of intent, owner and deadline, not the field value.

Scope: static review of two ValidatingPolicies. Supplied evidence omits apiVersion, releases, rule bodies and matchConstraints; group policies.kyverno.io treated as an assumption. Today 2026-11-24.

Evidence: manifest shows [Audit] (cannot show installed/effective). Approval text is an assertion, not a ticket or schedule. No installed-config or offline-test supplied.

Decision table: Canary — mandatory preventive in staged canary; legitimate op pod runAsNonRoot true (illustrative: invoice-retry-worker); violation root pod (illustrative); Audit; owner payments platform lead, deadline 2026-12-15 (21 days away), promotion criteria; violations recorded not blocked; Low, conditional/needs-review. Mandatory control — deadline passed about 2026-10-15 (derived, no record supplied); violations admitted; High, blocked/blocked for prevention. Engine: native VAP only if pure request check with binding, else Kyverno CEL.

Reasoning: canary is a bounded step; remaining risk is promotion never happening; conditional on owner/deadline/criteria recorded in a ticket (condition owner payments platform lead). Mandatory control: window ended, violations admitted, demonstrated defect against a prevention requirement; fix: promote to Deny after passing legitimate-operation test. Offline CLI v1.20.0 gives fail for both Audit and Deny, so only installed-config or admission-trace shows the enforcing action.

Denial message: "payments-run-as-non-root: set securityContext.runAsNonRoot=true on every container. Escalate to the payments platform owner if the image cannot run as non-root."

Falsifier: reopen if legitimate op denied or violation admitted under installed config (e.g. installed as Deny, or manifest edited during incident).
Counterexample: manifest vs installed copy; failurePolicy governs errors not violations.
Unknowns: deadline real? (ticket/schedule; signed extension would downgrade to conditional); installed action (installed-config); releases and group (controller tag, CRD served versions; legacy kinds not evaluated at v1.20.0+); rules match intent (kyverno test of pass/fail against Deny form).
Suggested action vs authority: promote mandatory control after tests; record canary owner/criteria; not authorization; live change to kubernetes-live-admission-policy-guard-agent with target, diff, blast radius, rollback, verification. Rollback: revert to Audit, Deny can block legitimate writes meanwhile. Escalation owner: payments platform lead.

Files read: .claude/agents/kyverno-policy-review-agent.md; .claude/skills/kyverno-policy-review/SKILL.md; references/evidence-vocabulary.md; references/policy-semantics-and-engine-selection.md
