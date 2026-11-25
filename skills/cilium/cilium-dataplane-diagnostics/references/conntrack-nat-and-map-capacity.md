# Conntrack, NAT and map capacity

## Problem

Cilium keeps connection and translation state in fixed-size BPF maps. When a map fills, insertion fails and new flows are dropped while existing flows are expected to keep working (derived from first principles, not a ledger fact), which looks like intermittent timeouts. Capacity must be judged from supplied counts against configured maxima, including forward-looking headroom.

## Mechanism from first principles

- Conntrack (CT) records each tracked connection so replies and policy decisions are consistent. TCP and non-TCP (any) flows use separate maps. Entries are consumed on new connections and freed by garbage collection (GC) when they expire.
- NAT state is kept for translated flows (masquerade, NodePort SNAT). Derived from first principles (not a ledger fact): each translated flow consumes a NAT entry in addition to its CT entry.
- Policy maps are per endpoint; the policy map max applies to each endpoint, not cluster-wide.
- All maps have hard caps and "insertion beyond the limit will fail" [CIL-DP-E12].

## Defaults and flags (1.20.5 documentation)

| Map | Default | Flag | Ledger |
| --- | --- | --- | --- |
| CT TCP | 512k (524288) | `--bpf-ct-global-tcp-max` | [CIL-DP-E12] [CIL-DP-E13] |
| CT any (UDP and other) | 256k (262144) | `--bpf-ct-global-any-max` | [CIL-DP-E12] [CIL-DP-E13] |
| NAT | 512k (524288) | `--bpf-nat-global-max` | [CIL-DP-E12] [CIL-DP-E13] |
| Policy per endpoint | 16k (16384) | `--bpf-policy-map-max` | [CIL-DP-E12] [CIL-DP-E13] |
| LB services | 64k (65536) | `--bpf-lb-map-max` | [CIL-DP-E12] [CIL-DP-E13] |
| Neighbor | 512k (524288) | `--bpf-neigh-global-max` | [CIL-DP-E12] [CIL-DP-E13] |

Helm equivalents named in the chart: `bpf.ctTcpMax`, `bpf.policyMapMax`, `bpf.mapDynamicSizeRatio` [CIL-DP-E15].

## Dynamic sizing and the NAT rule

`--bpf-map-dynamic-size-ratio` (default 0.0025, i.e. 0.25 percent of system memory) sizes the large maps (CT, nodeport neighbor, SNAT external, reverse socket LB), with a minimum of 131072 CT entries [CIL-DP-E15]. Whether the chart default matches 0.0025 is documented as a comment with a null value, so the agent default applies [CIL-DP-E15].

Rule: when CT sizes are set, NAT must not exceed 2/3 of the combined CT size; it is auto-set when NAT is not explicit or dynamic sizing is used [CIL-DP-E14].

## Garbage collection

`--conntrack-gc-interval` defaults to 0, meaning a dynamic interval that rises when few entries are collected and falls when many are; the current value is in agent logs [CIL-DP-E17]. Source constants in 1.20.5: start 5m, minimum 10s, max LRU interval 12h [CIL-DP-E17]. Metrics `datapath_conntrack_gc_runs_total` and `datapath_conntrack_gc_entries` show GC activity [CIL-DP-E16].

## CT-full symptom and remedies

Symptom: `drop (CT: Map insertion failed)` in the drop stream [CIL-DP-E16]. Documented remedies: lower the GC interval, or raise the CT TCP and any maxima (trading CPU for GC, or memory) [CIL-DP-E16]. Resizing LB maps and restarting causes connection disruption [CIL-DP-E20]. Any such change goes through the human change process; no guard exists for cilium-config changes.

Policy map overflow: find the selector that covers many identities; raising the policy map size is a last resort. `--enable-endpoint-lockdown-on-policy-overflow` shuts all endpoint traffic on overflow, so verify its value before concluding on traffic loss [CIL-DP-E19].

## Map pressure metric

`cilium_bpf_map_pressure{map_name}` is a ratio of required size to configured size: below 1.0 is utilization, at or above 1.0 is full. Policy map pressure is emitted only above the threshold `policyMapPressureMetricsThreshold` (default 0.1). Metrics exist only when `prometheus.enabled=true` [CIL-DP-E18].

## Headroom arithmetic (pattern, supplied numbers only)

Inputs come from the user: `count_now` (entries in use per supplied listing or metric), `max_configured` (flag or Helm value, or derived dynamic size), `peak_observed` (highest count in the supplied window).

```text
utilization      = count_now / max_configured
peak_utilization = peak_observed / max_configured
headroom_entries = max_configured - peak_observed
growth_factor    = projected_load_multiplier        (user-supplied, never assumed)
projected_peak   = peak_observed * growth_factor
fits             = projected_peak < max_configured  (strictly below; at 1.0 is full)
nat_ok           = nat_max <= (2/3) * (ct_tcp_max + ct_any_max)
```

Rules:

- Compute only from numbers the user supplied; if `peak_observed` or `max_configured` is missing, the result is `unknown`.
- A point-in-time count is not the peak; label which it is.
- Cost of larger maps or nodes goes to `finops-maestro-agent`.

## What missing metrics mean

Missing is not healthy. Prometheus disabled, a scrape gap, or a metric filtered out each produce silence. A map with no pressure series is `unknown`, not low. Collecting a map listing is `cilium-dbg bpf ct list` [C2] (form varies by version; see the drop reasons reference).

## Contradictory results

- Pressure ratio below 1.0 while CT-insertion drops occur: check the window and the specific map name; the ratio may belong to a different map.
- Counts near max but no drops: GC may be keeping up; the dynamic interval changes the margin. Not proof of safety.

## Scenario

A retailer plans a sale that will triple concurrent connections through one node pool. Supplied: peak CT TCP count and the configured TCP max. Applying the pattern with growth_factor 3 as supplied gives projected_peak; if that is at or above the configured max, the forward-looking finding is "insufficient headroom", with remedies listed as change-process items and the memory cost handed to `finops-maestro-agent`.
