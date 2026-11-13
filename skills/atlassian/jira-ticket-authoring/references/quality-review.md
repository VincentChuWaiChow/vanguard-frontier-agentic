# Ticket quality review

Use an evidence-backed verdict. This review method is a skill convention, not an Atlassian certification or numerical guarantee.

## Readiness verdicts

| Verdict | Meaning |
| --- | --- |
| Draft | Useful initial content; readiness has not yet been established |
| Needs clarification | A business rule, target, access boundary, or acceptance choice materially changes the work |
| Needs decomposition | Multiple independently deliverable outcomes or shared interfaces need coordination |
| Ready for triage | The reported need or symptom, affected context, impact as known, and next routing question are sufficient for intake; implementation may remain unresolved |
| Ready for fulfillment | The service request has the required target, eligible requester, applicable authorization and verification plan for the bounded fulfillment step; this verdict does not supply authorization |
| Ready for review | A handoff contains enough actual evidence and explicit gaps for its requested review; this does not imply acceptance or successful completion |
| Ready for investigation | A bounded question, available inputs, investigation method, deliverable, and stopping condition are sufficient for that investigation |
| Ready for implementation | The bounded behavior, relevant context, dependencies, and feasible verification are sufficiently settled for implementation |

Apply the verdict to the activity being handed off. A support case or feature request can be ready for triage while implementation details remain unknown. Use the selected pattern's specific prerequisites; fulfillment authorization and actual completion evidence are separate.

An investigation can be scoped yet still blocked on access or missing evidence. Name those prerequisites instead of calling it ready. A draft may be saved to Jira if requested; creating it does not require pretending it is ready. A well-specified child can be ready even when unrelated epic questions remain open.

## Review in this order

1. **Truth:** Can the reviewer distinguish observed facts, user reports, assumptions, and proposed choices? Are source versions and conflicts visible? Is unavailable evidence honestly marked?
2. **Business behavior:** Can someone explain the desired outcome and why it matters? Are cross-service rules and external data dependencies covered to the extent supported by evidence?
3. **Boundary:** Can the next assignment finish without inventing a business or design decision? For triage or discovery, are the unresolved decisions and expected output clear? For implementation, can one change be reviewed and verified independently? Are exclusions meaningful?
4. **Acceptance:** Is every `AC-nn` observable? Does each state input/precondition and expected output or invariant? Check applicable failure and boundary behavior.
   Keep story-specific behavior separate from an applicable, source-backed team Definition of Done. Checkbox counts, assigned fields, or a structural quality score do not establish readiness or completion; see [template adaptation](template-adaptation.md).
5. **Verification:** Does every criterion have a test, inspection, measurement, or operational check with an expected result? Are unknown commands marked? Is planned evidence distinct from observed evidence?
6. **Execution:** Are the prerequisites for the actual next activity available? Repository instructions, deployment context, and implementation proof apply when that stage is in scope, not merely because the work is recorded in Jira. Check required access, interfaces and dependencies for that activity. For an operational change, are applicable success, abort, and recovery conditions actionable?

Record a blocker as: `Missing fact or decision → consequence → source/person needed → affected AC or scope`. Use owner roles when a person has not been identified; do not assign someone by guess.

## Examples of consequential defects

| Draft wording | Problem | Repair |
| --- | --- | --- |
| Improve performance | No measurable behavior or agreed target | Obtain the workload and measurement target; label any proposed target |
| All tests pass | Does not identify the required change | Add observable criteria and map them to checks; retain the test-suite gate |
| The organization policy caused it | Evidence may establish only timing | Separate the hypothesis from the denied operation and causal evidence |
| See the design link | Missing relevant context; link may be inaccessible | Extract the controlling rule and cite its location/version |
| Add authentication and migrate all customers | Multiple outcomes and rollout risks | Decompose around independently verifiable outcomes and dependencies |
| Run pytest | Command may not exist in this repository | Inspect test configuration or mark the command unresolved |
| AC-01: implement the new helper | Implementation detail is not behavior | State what the caller or system must observe |
| Ready, except nobody has decided duplicate handling | Unsettled behavior changes acceptance | Needs clarification; name the decision and affected criteria |

## Completion handoff

Capture criterion → result → evidence, relevant change/PR identifiers, checks not run and reasons, remaining limitations, and follow-up findings. A successful template review does not prove the implementation works. Do not report tests, deployment, acceptance, or business approval as completed without the corresponding evidence.
