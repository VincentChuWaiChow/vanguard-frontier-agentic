# Default-Deny Rollout Readiness

Decision this reference supports: can a namespace move to default-deny, and can a policy change
ship, without breaking dependencies that nobody has listed? Flow-telemetry completeness, command
classes and verdict vocabulary are owned by the companion skill `cilium-evidence-boundary`.

## Core rule

Absence of observed flows does not prove a dependency does not exist; apply the flow-telemetry rules of the companion skill `cilium-evidence-boundary`, and keep a verdict that depends on "no flow seen" at `needs-review`.

## Mechanism

- A pod is deny-by-default in a direction only once at least one policy selects it for that
  direction; an unselected pod is allow-all in that direction. Effective policy is the union of
  all selecting allows. This is Kubernetes NetworkPolicy semantics as carried into the existing
  skill rules; it is derived here, not re-verified against a fetched page in this review.
- Adding the first egress policy to a pod therefore flips its whole egress to deny, including
  traffic nobody thought of as a dependency (cluster DNS, node-local services, the Kubernetes
  API, monitoring scrapes in the ingress direction).
- DNS egress to the cluster DNS service must be explicitly allowed once egress default-deny
  applies, otherwise name resolution fails first and masks every other dependency. Label: derived
  from the allow-union semantics above; the Kubernetes network-policies page was not fetched for
  this skill, so do not present it as documentation-verified.
- Cilium behavior that depends on its version is listed in the next section; read the target
  cluster's version first.

## Version notes that change policy meaning

| Version | Change | Review consequence | Source |
| --- | --- | --- | --- |
| 1.19 | ClusterMesh `policy-default-local-cluster` set by default (introduced as an option in 1.18); Helm `clustermesh.policyDefaultLocalCluster` defaults to true; selectors match only local-cluster endpoints unless a cluster is targeted | Policies that relied on cross-cluster matching stop matching; setting false keeps old behavior but is slated for deprecation | [CIL-NP-E14] |
| 1.19 | `FromRequires` and `ToRequires` are rejected | A manifest using them cannot be assumed to be enforcing anything | [CIL-NP-E17] |
| 1.19 | The `**` DNS wildcard matches multiple subdomains | A `toFQDNs` or DNS rule using `**` is broader than it was before upgrade | [CIL-NP-E17] |
| 1.20 | Egress rules combining `toServices` and `toPorts` deny when the Service is missing or selector-less without backends | A rule that looked like an allow becomes a deny for a missing Service | [CIL-NP-E18] |
| 1.20 | Kafka-aware policy and `l7proto` removed; remove such `rules` before upgrading. Empty CNP/CCNP rejected at admission via CEL | Policies carrying `kafka` or `l7proto` rules are an upgrade blocker, owned by lifecycle readiness | [CIL-NP-E19] |

ClusterMesh details: policies are not distributed across clusters, so the same policy must be
applied in every cluster; the cross-cluster selector label is `io.cilium.k8s.policy.cluster`.
The audit command before any flip is below. Egress gateway is not compatible with ClusterMesh, so that
interaction belongs to `cilium-external-connectivity-review-agent`.

Class: [C1] reads cluster state through the operator's authorized client.

```text
cilium clustermesh inspect-policy-default-local-cluster --all-namespaces   [C1]
```

The command and flag are documented on the ClusterMesh policy page [CIL-NP-E16].

## Building the required-flow inventory

1. Start from the supplied Hubble flows for the namespace: for each source workload, list
   destination identity or FQDN, port, protocol and verdict over the longest window supplied.
2. Add what flows cannot show: DNS (cluster DNS, and the FQDNs behind `toFQDNs`), the API server,
   node and kubelet probes, ingress controllers, monitoring and logging agents, and scheduled or
   failover paths named by the service owner.
3. Mark every inventory row `observed`, `derived` or `assumption`; rows from owner interviews
   are `assumption` until a flow or manifest confirms them.
4. Compare the inventory with the proposed allow set. Unmatched inventory rows are the predicted
   breakage; unmatched allow rows are over-permission to challenge.

Class: [C2] observational, via Relay; flows may carry sensitive metadata the operator must sanitize.

```text
hubble observe --from-namespace <ns> --verdict FORWARDED --since 24h   [C2]
hubble observe --to-namespace <ns> --verdict DROPPED --since 24h   [C2]
```

The flags `--verdict`, `--since` and `--from-namespace` are documented for the hubble CLI;
`--to-namespace` is assumed to mirror the documented `--from-*` filters and should be confirmed
with the operator's CLI help [CIL-NP-E13]. Running the CLI inside the agent pod is the documented
troubleshooting form and is [C2] [CIL-NP-E20]. Policy-denied drops can be isolated with
`--drop-reason-desc POLICY_DENIED` [C1] [CIL-NP-E13].

## Observe before enforce, and blast-radius ordering

- Audit-style observation first: the documented verdict set includes `AUDIT` [CIL-NP-E13], and
  policy audit mode (agent-wide or per endpoint) shows what policy would drop without dropping it;
  while it is on nothing is enforced, so it is a rollout tool, not a production state. Turning it on
  or off is a change for the human change process; details and limits are in
  [policy-semantics.md](policy-semantics.md).
- Order changes from smallest to largest blast radius: a single non-critical namespace, then
  namespaces by dependency depth (leaf services before shared ones), and last any
  `CiliumClusterwideNetworkPolicy`, which has cluster-wide scope.
- Apply allow rules before the isolating policy within one change, as a precaution (derived/unverified; not
  re-verified in this skill's ledger, confirm against the policy language page for the installed
  version): eBPF program reload may be asynchronous, so a misordered apply could briefly break traffic.
- Policy changes while an upgrade runs: a new policy implementation is postponed on a node until
  that node's upgrade completes and L7-proxy traffic is disrupted during upgrade; avoid
  combining both changes [CIL-NP-E21].

## Rollback

Rollback is re-applying the previous policy set, as a new change, through
`kubernetes-live-network-policy-guard-agent`, which guards CNP, CCNP, NetworkPolicy and
CiliumEgressGatewayPolicy. The reviewing agent never executes it. A rollback plan is credible only
if the previous manifests are captured in the evidence, the verification flow query is named, and
the abort trigger is stated (for example any `POLICY_DENIED` drop from a listed dependency).
Removing a default-deny is itself a critical finding; rolling back to a state that had none needs
the same review as removing one. Helm and cilium-config rollbacks have no guard: human change
process.

## Failure modes and contradictions

| Observation | Interpretation |
| --- | --- |
| No drops in a quiet window, then breakage at month end | Window shorter than the dependency period; inventory incomplete |
| DNS drops after the egress policy applies | Cluster DNS egress missing from the allow set |
| `toServices` rule exists but the flow is dropped (1.20) | Service missing or has no backends; check the Service before the rule |
| Cross-cluster flow dropped after upgrade to 1.19 | Local-cluster-only selector default; target the peer cluster explicitly |
| Policy YAML accepted but L7 field not enforced | Proxy not enabled for that path; confirm before trusting L7 |

## Scenario: payments namespace moves to default-deny

The platform team proposes ingress and egress default-deny for `payments` plus allow rules from
two weeks of Hubble flows. Review: the flows show the API tier and a database but no DNS rule, no
allow for the monthly reconciliation job, and a ClusterMesh peer calls a global service from
another cluster. Findings: egress DNS missing (derived), reconciliation job unobserved
(assumption, window too short), cross-cluster caller at risk if the mesh is on 1.19 or later
defaults [CIL-NP-E14]. Verdict `needs-review`, `evidence_level: partial`; next evidence is a
longer flow window, the job owner's schedule and the cluster-mesh audit output. Rollback is the
captured previous policy set applied via the live guard.
