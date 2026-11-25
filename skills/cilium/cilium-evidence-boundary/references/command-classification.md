# Command classification

Reference for the `cilium-evidence-boundary` skill. Load it whenever a response names a command, a collection step, or a diagnostic bundle.

## Why commands are classified

The board never runs a command. It does name commands, because a diagnosis is only useful when it says which observation would discriminate between hypotheses. The risk is that a reader treats a named command as harmless because it contains `get`, `status`, `observe`, `debug` or `test`. Command names describe intent; they do not describe side effects, the privilege they need, or what they collect. So every command the board names carries a class that describes its actual behavior, and the class decides who must authorize it.

## The six classes

| Tag | Class | What it does | Who may run it |
| --- | --- | --- | --- |
| [C0] | Offline analysis | Reads only supplied artifacts: files, exports, saved command output. | The board's own static review. |
| [C1] | Observational | Reads cluster or API state with an authorized, read-only identity. | The user or an operator, against a confirmed target. |
| [C2] | Privileged observation | Needs exec into the Cilium agent pod, node access, or collects data that can be sensitive. | An authorized operator, with a named scope, time window and redaction plan. |
| [C3] | Active diagnostic | Generates traffic, creates resources, starts processes or changes diagnostic state. | An operator under change authorization for the named cluster. |
| [C4] | Mutation | Changes configuration or runtime state. | Only through an existing guard or the human change process. |
| [C5] | Destructive | Removes resources, interrupts service, or risks irreversible loss. | Only through the human change process, with rollback criteria. |

When a command fits more than one class, the higher class wins.

## Commands whose class is easy to get wrong

These are the cases where the name suggests a lower class than the behavior.

| Command | Looks like | Actual class | Why |
| --- | --- | --- | --- |
| `cilium connectivity test` | [C1] check | [C3] | It creates a test namespace (default `cilium-test`, suffixed with a sequence number) with client and echo workloads, Services and policies [CIL-EV-E01]. |
| Upgrade preflight (`kubectl create -f cilium-preflight.yaml`) | [C1] check | [C4] | It creates a `cilium-pre-flight-check` DaemonSet and Deployment in `kube-system` to pre-pull images [CIL-EV-E02]. |
| `cilium sysdump` | [C1] status | [C2] | It collects logs, ConfigMaps, Helm values, bugtool output, kvstore data and Hubble flows from every node by default [CIL-EV-E03]. |
| Any `cilium-dbg ...` subcommand | [C1] status | at least [C2] | `cilium-dbg` runs inside the Cilium agent pod, so collecting it requires exec into a kube-system workload; the agent pod's privileges are not restated here, but exec into it is never routine read access. |
| `hubble observe` through Hubble Relay | [C1] read | [C2] | It is read-only, but flows carry pod names, IP addresses, DNS names and, with L7 visibility, request metadata, which is data that can be sensitive; collection needs a named scope, window and redaction plan. |
| `tcpdump` on a node interface | [C1] read | [C2] | Requires node access and captures payload bytes unless filtered. |

## Diagnostic bundles are sensitive by default

A diagnostic bundle is a copy of the cluster's configuration and logs, so it inherits everything sensitive that configuration and logs contain. No redaction step was found in a text search of the `cilium sysdump` [C2] collection code on the development branch [CIL-EV-E03]; that is not an exhaustive audit, so treat any bundle as unredacted. One published advisory, GHSA-gj49-89wh-h4gj, records that bugtool output contained the WireGuard private key in releases before 1.19.3, 1.18.9 and 1.17.15, and advises rotating keys where such archives were shared [CIL-EV-E04].

Rules that follow from this:

- Never ask a user to attach a full sysdump or bugtool archive to a conversation, a ticket or a page.
- Ask instead for the specific excerpt that discriminates the hypothesis: one status section, one counter, one drop line, one policy object.
- When a bundle has already been shared, say whether the installed version is in the advisory's affected range, and hand key rotation to the human change process.

## How to write a classed command

Inline, on the same line: `hubble observe --verdict DROPPED --since 5m` [C2], via Hubble Relay against the confirmed cluster.

As a block, with a class line immediately before the fence:

Class: [C2] runs inside the Cilium agent pod; operator-authorized, one node, one time window.

```text
cilium-dbg monitor --type drop
```

For [C3], [C4] and [C5], also name the authorization path, for example "requires change authorization for the named cluster" or "route to `kubernetes-live-network-policy-guard-agent`". A higher-class command is never framed as a step the board takes.

## Guarded and unguarded mutations

| Mutation surface | Path |
| --- | --- |
| CiliumNetworkPolicy, CiliumClusterwideNetworkPolicy, NetworkPolicy, CiliumEgressGatewayPolicy | `kubernetes-live-network-policy-guard-agent` |
| ClusterMesh peer Secret, Gateway API resource instances (not their CRDs) | `kubernetes-live-network-architecture-mutation-guard-agent` |
| BGP CRDs, LB-IPAM pools, L2 announcement policies, Helm values, `cilium-config`, upgrades, CNI migration, any CustomResourceDefinition install or change (Gateway API CRDs included) | No guard exists. Hand off to the human change process; never propose execution. |

## Scenario

An incident commander asks the board to "just run the connectivity test in prod to see what's broken". The board answers that `cilium connectivity test` is [C3] because it creates workloads and policies in the production cluster, names the cluster and namespace it would touch, and proposes [C2] evidence first: drop counters and a bounded `hubble observe` window for the affected workloads. If the operator still wants the test, it is their authorized action, not the board's.
