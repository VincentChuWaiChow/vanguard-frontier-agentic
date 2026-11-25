# Conflicts and aggregation

Reference for the `cilium-review-routing` skill. Load it when specialists disagree, or when their evidence differs in scope or time.

## Why agreement is not evidence

Two specialists that read the same manifest and agree have produced one observation, not two. Agreement between agents is independent evidence only when the agents used independent evidence. So the router never treats a count of agreeing verdicts as a reason, and never averages severities or confidences into a combined score.

## Combining verdicts

1. A `blocked` stands if its condition is confirmed within its stated applicability. Another specialist's `approved` on a different decision does not cancel it.
2. Otherwise, a `needs-review` stands. The combined verdict cannot be stronger than the weakest owner's verdict on a required decision.
3. `approved` is combined only when every required decision is `approved` and no required input is missing.
4. Carry `evidence_level` per decision. Do not compute a combined evidence level.

## Resolving a disagreement

When two specialists make incompatible claims, work through these steps in order and stop at the first that resolves it.

1. **Name the exact claim in conflict.** "The drop is caused by policy" against "the drop is caused by conntrack exhaustion", not "they disagree about the outage".
2. **Check the scope.** Same cluster, nodes, namespace, workload and time window? Different scopes can both be right.
3. **Check the version.** Same Cilium version and enabled features? A rule that changed between minors can make both readings correct for their own version.
4. **Compare provenance and capture time.** Newer, in-scope, directly observed evidence outranks older or derived evidence. Record why.
5. **Expose hidden assumptions.** One specialist may have assumed a routing mode or a feature flag the other did not.
6. **Name the smallest discriminating observation.** For the example above: the drop reason counter for the affected node and window, which separates `Policy denied` from `CT: Map insertion failed`. Give it a command class.
7. **Preserve what remains.** If nothing supplied resolves it, the disagreement is part of the result, with both claims, both evidence sets, and the observation that would settle it.

## Independent review

Request an independent review when a conclusion would authorize a high-blast-radius change (cluster-wide policy, BGP, upgrade, migration). The reviewer must use evidence the proposer did not produce, or say that it could not. A specialist never reviews its own recommendation.

## What the combined response must keep

- Every specialist's evidence IDs and scope.
- Every untested assumption.
- Every unresolved disagreement, with the observation that would resolve it.
- The specialist that owns each open question.
