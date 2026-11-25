# Flow telemetry as evidence

Reference for the `cilium-evidence-boundary` skill. Load it whenever a conclusion rests on Hubble flows, drop metrics, or the claim that something was not observed. This is the board's single owner of telemetry-completeness rules; specialists reference it instead of restating it.

## The problem this solves

Hubble is the most persuasive evidence a Cilium user can supply, because it shows individual flows with identities, verdicts and drop reasons. That persuasiveness is the risk. Flow records are a sample of what the datapath emitted, held in a bounded buffer, filtered by whoever exported them. Three conclusions are routinely drawn from them that the data cannot support: "this dependency does not exist", "no malicious traffic occurred", and "the policy is the cause".

## How Hubble holds flows

- Hubble runs inside each Cilium agent. It reads events from the agent's monitor and stores them in a user-space ring buffer on that node; once the buffer is full, new events overwrite the oldest [CIL-EV-E05].
- The buffer capacity is one less than a power of two, configured with `--hubble-event-buffer-capacity` (Helm `hubble.eventBufferCapacity`), default 4095 flows [CIL-EV-E06].
- Hubble Relay connects to every node's Hubble server and answers cluster-wide queries [CIL-EV-E07]. A Relay query is a union of per-node buffers, each with its own retention horizon.
- `hubble observe --last N` [C1] returns the last N flows per node when queried through Relay, not N cluster-wide [CIL-EV-E08].

The consequence is first-principles arithmetic, not a fixed number: on a node that emits events at rate R per second, the buffer covers roughly capacity / R seconds. A busy node can cover seconds; a quiet node can cover hours. A flow older than the busiest relevant node's horizon is gone, whatever the query asks for.

## Where events are lost

The flow API names three loss points [CIL-EV-E09]:

| Lost-event source | Meaning |
| --- | --- |
| `PERF_EVENT_RING_BUFFER` | The agent did not keep up with events produced by the datapath. |
| `OBSERVER_EVENTS_QUEUE` | The Hubble event queue was full. |
| `HUBBLE_RING_BUFFER` | An event was overwritten before a reader got it. |

When the event queue fills, events are dropped from Hubble and its metrics, but traffic is not affected [CIL-EV-E10]. So a gap in telemetry is not a gap in traffic, and the absence of lost-event reports in an export is only meaningful if the export was set up to include them.

## What Hubble cannot see

- Pods must be managed by Cilium to be observable by Hubble [CIL-EV-E11]. A workload outside Cilium management is invisible, not quiet.
- Hubble reports only events the datapath emits. Monitor aggregation settings reduce what is emitted.
- Host-network traffic visibility is not stated on the pages reviewed for this skill; treat it as `unknown` unless the user supplies evidence for their configuration.
- L7 fields appear only where L7 visibility or policy redirects the flow through the proxy.

## Rules for using flows as proof

1. **Absence is not evidence of absence.** "No flow from A to B was observed" supports only "none was retained in the exported buffers, for the exported filter, in the exported window". It never proves that the dependency does not exist, that a policy blocked it, or that no malicious communication occurred.
2. **State the coverage.** Every flow-based conclusion names the nodes queried, the time window, the filter, whether Relay was used, and whether lost events were reported.
3. **A drop reason is a datapath fact, not a root cause.** `Policy denied` says which program dropped the packet; it does not say whether the policy is wrong or the traffic is unwanted. Other drop reasons are interpreted by `cilium-dataplane-diagnostics`.
4. **Observed allow does not identify the allowing rule.** Successful traffic under several overlapping policies does not prove which one allowed it.
5. **Correlate clocks before correlating events.** Flow timestamps, application logs and change records must be on comparable clocks before ordering is claimed.

## Building a dependency inventory from flows

Default-deny rollout and migration planning both need a list of real dependencies. Build it as a lower bound:

- Collect flows over a window that covers the workload's business cycle (batch jobs, month-end, failover tests), not a convenient hour.
- Record per-node coverage and lost events for that window.
- Mark every dependency the business says exists but the flows did not show as `unknown`, not as removed.
- Feed the inventory to the owning specialist; this skill decides only whether the inventory is complete enough to rely on.

## Metric cardinality

Hubble metrics and flow exports multiply by labels such as namespace, workload and destination. High label cardinality raises monitoring cost and can be the reason a team disabled the very metric a review needs. When a metric is missing, record it as missing visibility and hand cost questions to `finops-maestro-agent`; never conclude that the thing it would have measured is healthy.

## Scenario

After a suspected intrusion, a team exports one hour of Hubble flows through Relay and finds no connection from the compromised pod to an external address. The board's answer: the export shows no retained flow in that window for that filter. It asks for per-node buffer capacity and event rates to establish how much of the hour each node actually retained, whether lost events were reported, and whether the pod was Cilium-managed. Until then, "no exfiltration occurred" is `unknown`, and the incident stays open.
