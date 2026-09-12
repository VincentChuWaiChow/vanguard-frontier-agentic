# Mirroring and fault-experiment boundaries

## TRAFFIC-06: duplicated requests have effects

Mirroring sends real requests to the destination; ignoring its response does not make its processing a simulation. Treat possible emails, payments, bookings, inventory changes, audit events and external calls as risks to investigate. Require isolation of data, credentials, outbound dependencies, retention and customer payloads, not only a separate Service name.

Determine exactly which requests are eligible, their rate/percentage, who approved the experiment, how success/abort is measured and how rollback stops new mirrored work. Account for requests already in flight and asynchronous side effects. A header flag alone is not proof that the application suppresses writes.

## TRAFFIC-07: fault scope

Specify caller population, destination, protocol, host/path/headers, duration, blast radius and stop criteria. Reject broad production fault proposals without an approved experiment boundary. Check compatibility of fault injection with retries/timeouts for the target API and release; do not stack examples from unrelated tutorials.

Generate a plan only. A review request never authorizes traffic injection, curl probes, workload creation, traffic mirroring activation or production load testing.

Sources: MIRROR, VS and NETWORK. Tests: TRAFFIC-06, TRAFFIC-07.
