# Drop reasons and signals

## Problem

A drop line names a reason, not a root cause. This reference maps the important reasons to a hop and an owner, lists the signals around them, and version-gates the commands whose syntax or binary name differs across releases. Hubble and flow-telemetry completeness (ring buffer size, lost events) is owned by the companion skill `cilium-evidence-boundary`; do not restate it here.

## Canonical source

The canonical list of drop reasons is the `errors` map in `pkg/monitor/api/drop.go` of the Cilium source; its header says the values are shared with `bpf/lib/common.h` and `api/v1/flow/flow.proto` [CIL-DP-E09]. Cite the file at the cluster's version tag, because codes can change between releases (1.20.5 values below).

## Important reasons, interpretation and owner

| Code | Text | Interpretation (hop, read from the reason text) | Owner | Ledger |
| --- | --- | --- | --- | --- |
| 133 | Policy denied | Policy verdict at the endpoint | `cilium-network-policy-review-agent` | [CIL-DP-E09] |
| 181 | Policy denied by denylist | Deny rule matched | `cilium-network-policy-review-agent` | [CIL-DP-E09] |
| 189 | Authentication required | Policy requires authentication that is not established | `cilium-network-policy-review-agent` | [CIL-DP-E09] |
| 155 | CT: Map insertion failed | CT map full or insertion failed; capacity | this skill (capacity reference) | [CIL-DP-E09] [CIL-DP-E16] |
| 167 | No mapping for NAT masquerade | NAT state missing for the flow; NAT map or masquerade configuration | this skill | [CIL-DP-E09] |
| 151 | Stale or unroutable IP | Destination not routable or identity stale; routing hop | this skill | [CIL-DP-E09] |
| 158 | Service backend not found | Service translation has no backend | this skill (service load balancing reference) | [CIL-DP-E09] |
| 169 | FIB lookup failed | Host routing lookup failed | this skill (routing reference) | [CIL-DP-E09] |

Also present in 1.20.5: 163 Unknown connection tracking state, 160 No tunnel/encapsulation endpoint (labelled datapath BUG), 136 Fragmentation needed, 190 No conntrack map found, 191 No nat map found, 198 Rate limited, 196 TTL exceeded, and code 15 "MTU error message" [CIL-DP-E10]. Their causes are not explained on the pages reviewed; treat them as signals that need a discriminating check, not as diagnoses.

## Metrics

- `drop_count_total` is exported in the `cilium_` namespace (as `cilium_drop_count_total`) with labels `reason` and `direction`; `drop_bytes_total` has the same labels. Both are marked enabled by default, but the agent exports metrics only when `prometheus.enabled=true` [CIL-DP-E11].
- Reading it: a non-zero rate for one reason localizes the hop; a metric silent because Prometheus is off is unknown, not zero.
- Endpoint regeneration: `endpoint_regenerations_total` (labels `reason`, `outcome`, `error`), `endpoint_regeneration_time_stats_seconds` (`scope`, `status`), `endpoint_state`, `endpoint_restoration_*`, `endpoint_component_status`. Failures show as `outcome=fail`; the exact label values were not inspected [CIL-DP-E34].
- Endpoint states in source: waiting-for-identity, ready, waiting-to-regenerate, regenerating, disconnecting, disconnected, restoring, invalid [CIL-DP-E35]. `endpoint-regen-interval` (default 2m0s) periodically re-applies endpoint configuration [CIL-DP-E35]. Regeneration triggers were not verified from a docs page [CIL-DP-E36].

## Collection commands (version-gated)

Binary name: the agent debug binary is `cilium-dbg` in the stable docs. It is absent from the 1.14 troubleshooting page and present from the 1.15 page, so "documented since 1.15" is the verified bound; the release note announcing the rename was not found [CIL-DP-E01] [CIL-DP-E02] [CIL-DP-E40]. The bare name `cilium` is the separate host-side CLI, though the docs mix both inside the pod; prefer `cilium-dbg` in the agent pod [CIL-DP-E01].

Conntrack listing syntax differs by release:

| Version | Form | Class | Ledger |
| --- | --- | --- | --- |
| 1.18 | `cilium-dbg bpf ct list global` (synopsis is `( global or cluster <identifier> )`) | [C2] | [CIL-DP-E06] |
| 1.19, 1.20 | `cilium-dbg bpf ct list` (synopsis is `[cluster <identifier>]`); the argument `global` is still accepted for backwards compatibility in 1.20.5 source | [C2] | [CIL-DP-E06] |

If the version is unknown, state both forms and do not choose one.

Drop observation: `cilium-dbg monitor --type drop` [C2]; documented type values are agent, capture, debug, drop, l7, policy-verdict, trace, trace-sock, with `--from`, `--to`, `--related-to`, `-v`, `-vv`, `--json` [CIL-DP-E04]. Datapath debug messages in `monitor -v` are off by default until `datapath` is added to `debug-verbose`, which is a configuration change needing authorization [CIL-DP-E05].

Privilege: every documented example runs by exec into the agent pod and `bpf` commands read pinned maps on the node, so treat them as privileged host-level observation and per the companion skill `cilium-evidence-boundary` for sensitive collection [CIL-DP-E08].

## Contradictory results

- The metric shows reason X but the sampled monitor shows none: the monitor is a window, the metric is cumulative; compare windows before concluding.
- A drop reason with a "BUG" label: still collect the version and configuration; do not claim a Cilium defect from one line.

## Scenario

A telecom operator sees `drop_count_total` with reason `No mapping for NAT masquerade` rising only on two nodes after a node-pool replacement. The hop is NAT; the next observation is the NAT map pressure on those nodes versus the others, and whether masquerade settings differ per node pool. Policy-owned reasons on the same nodes are routed to `cilium-network-policy-review-agent`. A regression after an upgrade also co-routes to `cilium-lifecycle-readiness-agent`.
