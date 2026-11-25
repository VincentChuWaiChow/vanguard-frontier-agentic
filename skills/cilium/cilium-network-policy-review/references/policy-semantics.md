# Policy semantics: what is actually enforced

Reference for the `cilium-network-policy-review` skill. Load it for every policy review: no "allowed", "denied" or "default-deny" conclusion is valid until the five questions below have been answered for the endpoints in scope.

## Why this reference exists

A policy review that reads only allow rules reaches the wrong answer in both directions. Cilium combines three things before a packet is allowed: the agent-wide enforcement mode, the per-direction default-deny state of each endpoint, and the union of allow rules minus any deny rule. Miss the mode and a cluster with `never` looks segmented; miss a deny rule and a flow the allow rules permit is in fact blocked; miss `enableDefaultDeny: false` and a policy that "selects the pod" does not isolate it.

## Question 1: what is the enforcement mode?

The agent runs in one of three modes, set by the Helm value `policyEnforcementMode` or the agent flag `enable-policy` [CIL-NP-E24]:

| Mode | Behavior |
| --- | --- |
| `default` | Endpoints have unrestricted access until a policy selects them; once selected, only allowed traffic passes, per direction. |
| `always` | Enforcement is on for every endpoint, even when no rule selects it. |
| `never` | Enforcement is off for every endpoint, even when rules select it; all traffic is allowed. |

Review rules:

- An unknown mode makes every default-deny conclusion `needs-review`.
- `never` means the supplied policies enforce nothing. The documented migration procedure to Cilium sets enforcement to `never` for its duration (see the lifecycle skill), so a migrating cluster is unsegmented until the mode is changed back.
- `always` changes the meaning of "no policy selects this pod": it is denied, not open. Health endpoint and infrastructure traffic need explicit allows.

## Question 2: is each endpoint in default-deny, per direction?

In `default` mode an endpoint moves to default-deny for ingress when any selecting rule has an ingress section, and for egress when any selecting rule has an egress section [CIL-NP-E25]. Kubernetes NetworkPolicy uses the same per-direction model: a pod is isolated for egress only when a policy selecting it has `Egress` in its `policyTypes`, and for ingress likewise; when `policyTypes` is omitted, `Ingress` is always set and `Egress` only if the policy has egress rules [CIL-NP-E27].

Consequences that reviews get wrong:

- An ingress-only policy does not restrict egress. "Default-deny in both directions" needs a selecting rule in each direction.
- A rule with `enableDefaultDeny` set to false for a direction is ignored when deciding that direction's default mode, so a policy can select a pod without isolating it [CIL-NP-E25].
- `enableDefaultDeny` does not apply to L7 rules: adding an L7 rule without an L7 allow-all still causes drops, even with default-deny disabled [CIL-NP-E25].

## Question 3: what do the allow rules permit?

Allows are additive and never conflict: for a direction, the permitted connections are the union of what every applicable policy allows, so evaluation order does not matter [CIL-NP-E27]. A supplied policy set that is not claimed complete can therefore only establish a lower bound on what is allowed.

## Question 4: do any deny rules apply?

`ingressDeny` and `egressDeny` take precedence over allow rules, whether the allow comes from a CiliumNetworkPolicy, a CiliumClusterwideNetworkPolicy or a Kubernetes NetworkPolicy, and a deny rule selecting a pod also puts it in default-deny [CIL-NP-E22]. Deny rules do not support L7 rules or `toFQDNs` [CIL-NP-E23].

Review rules:

- Never conclude "this flow is allowed" from allow rules alone. Search every CNP and CCNP that selects the endpoint for `ingressDeny` and `egressDeny`.
- A cluster-wide deny (for example a CCNP denying `world` to every endpoint) overrides every namespace team's allow; its owner is the owner of that decision.
- A "deny" requirement that depends on L7 or FQDN matching cannot be met by a deny rule; it has to be expressed as the absence of an allow.

## Question 5: is anything in audit mode?

Policy audit mode allows all traffic and logs what policy would have dropped. It can be set for the whole agent (`--policy-audit-mode=true`) or per endpoint, covers L3 and L4 only, and while it is on no network policy is enforced; the upstream documentation does not recommend it for production [CIL-NP-E26].

- An endpoint in audit mode is unenforced, whatever its policies say. Ask for the agent setting and per-endpoint state.
- Audit verdicts are the right input for a default-deny rollout, because they show what would be dropped without dropping it. Turning audit on or off is a configuration change: per-endpoint it is set with `cilium-dbg endpoint config <id> PolicyAuditMode=Enabled` [C4], inside the agent pod, and a restart of the agent resets it [CIL-NP-E26]. It goes to the human change process; the board never sets it.

## DNS and default-deny egress

A default deny-all egress policy also blocks DNS; workloads that resolve names need a separate allow to the cluster DNS service [CIL-NP-E27]. An egress default-deny proposal without a DNS allow is a predicted outage for any workload that connects by name.

## Scenario

A team says namespace `orders` is "fully default-deny" and supplies two CNPs, both ingress-only, plus a CCNP from the platform team. The review answers each question in turn. Mode: not supplied, so `needs-review`. Direction: ingress is isolated by both CNPs; egress is not isolated by anything supplied. Allows: the two ingress lists, as a lower bound. Deny: the CCNP carries an `egressDeny` to `world`, which blocks internet egress for every endpoint despite egress being otherwise open. Audit: unknown. The finding is "ingress default-deny in `orders`; egress open except internet, which a cluster-wide deny owned by the platform team blocks; enforcement mode and audit state unconfirmed", not "fully default-deny".
