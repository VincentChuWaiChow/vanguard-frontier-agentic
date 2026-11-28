# Upgrade path and rollback

## Problem

A cluster owner wants to move Cilium from one version to another and expects to be able to go back. The documented path is narrow, and most upgrade incidents come from stepping outside it.

Scenario: a platform team on 1.18.x plans to jump straight to 1.20.5 over one weekend, with an old pipeline that reuses prior Helm values. This reference is how to review that plan.

## Mechanism and documented rules

- The only tested rollback and upgrade path is between consecutive minor releases: "going from (a hypothetical) 1.1 to 1.2 and back is supported while going from 1.1 to 1.3 and back is not" [CIL-LC-E01].
- Update to the latest patch of the current minor before attempting the upgrade [CIL-LC-E02].
- The pre-flight check is labelled Required. It pre-pulls the new image to reduce agent downtime and avoid image-pull errors [CIL-LC-E03].
- Set `upgradeCompatibility` to the initially installed version. Do not use `--reuse-values` across minor upgrades because the Helm template may render incorrectly. Save the current Helm values to a file and check for renamed or deprecated values. All Cilium components must run the same version [CIL-LC-E04].
- Rollback reverts the latest changes to the Cilium DaemonSet; only consecutive-minor rollback is tested. After features of the new minor have been consumed, the version-specific notes must be read for incompatible feature use before rolling back [CIL-LC-E05].
- L7 proxy traffic (Ingress, Gateway, L7 policy) is disrupted during the upgrade; L3/L4-only traffic is expected to see minimal or no impact; new policy implementation on a node is postponed until that node's upgrade completes [CIL-LC-E06].

## Preflight is a mutation

The documented preflight renders a chart with preflight enabled and applies it. This creates a DaemonSet and a Deployment named `cilium-pre-flight-check` in kube-system and also validates existing CiliumNetworkPolicies; cleanup deletes them [CIL-LC-E03]. It is not read-only.

| Step | Class | Who must authorize |
| --- | --- | --- |
| `helm template` render with preflight enabled, writing a local file [C0] | observational, no cluster change | none beyond normal access |
| `kubectl create -f` of that rendered file [C4] | mutation: creates DaemonSet and Deployment in kube-system | change approver for the cluster |
| `kubectl delete -f` of the same file [C5] | removes the preflight resources | same change approver |

This skill never runs these. A plan that says "run preflight to see if we are ready" has no evidence yet; readiness cannot depend on an output that has not been collected.

## Review checks

1. Current and target differ by at most one minor; otherwise `blocked` with a stepwise plan requested.
2. Current version is the latest patch of its minor; otherwise the first change is the patch bump (a separate change).
3. Values diff is produced from saved values, not `--reuse-values`; `upgradeCompatibility` is named in the plan.
4. Every component (agent, operator, Envoy and others deployed) is at one target version.
5. Preflight is in the plan with an authorizer, a cleanup step and a named owner for the policy-validation findings.
6. L7 disruption is stated: which Ingress, Gateway or L7-policy workloads, and what the business window is.
7. Rollback section names the revision to return to and the point after which it is no longer valid.

## Canary and fleet sequencing (derived)

These follow from the facts above; the reviewed pages do not prescribe node ordering [CIL-LC-E06].

- Because upgrade is per node and L7 traffic is disrupted per node, start with a node pool carrying no L7-dependent workloads, then a pool that does, then the rest. Derived.
- Because new policy is postponed on a node until its upgrade completes, a mixed-version window has nodes enforcing the old policy engine; keep that window short and do not combine it with policy changes. Derived.
- Because rollback is only tested before new-minor features are consumed, do not enable new-minor features in the same window as the upgrade. Derived.
- Hold a go/no-go gate after the canary pool: agents ready, no new drops, L7 workloads healthy. The evidence classes belong to the companion skill `cilium-evidence-boundary`.

## Discriminating evidence

| Question | Evidence | Class |
| --- | --- | --- |
| Which versions run? | `kubectl get ds cilium -n kube-system -o yaml` image field [C1] | observational |
| Which Helm revision is live? | `helm history cilium -n kube-system` [C1] | observational |
| Saved values at current revision | `helm get values cilium -n kube-system` [C2] | privileged observation, may contain sensitive values |

## Contradictions

- The pipeline uses `--reuse-values` but a values file also exists: treat the render as unverified until a diff from the saved values is supplied.
- Agents report different versions: all-components-same-version is violated; `blocked` until explained.

## Handoff

Execution of upgrade, preflight, rollback: human change process (no guard exists). Policy changes surfaced by preflight: cilium-network-policy-review-agent. Target-version architecture questions: kubernetes-network-architecture-review-agent.
