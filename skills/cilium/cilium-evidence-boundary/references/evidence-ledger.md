# Evidence ledger — Cilium Evidence Boundary

Every material upstream claim in this skill cites one row below as `[CIL-EV-Exx]`.
`tests/validate-skill-references.py` fails the build if a citation has no row, a row is never
cited, or a source is not an allow-listed provider host. `VERIFIED` = the section was retrieved
on the retrieval date and supports the claim as worded; it is documentation verification, not a
behavioral test. Re-fetch before relying on a version-sensitive value.

Scope: Cilium 1.20.5, the stable documentation on the retrieval date, unless a row names another
version. Rows marked from `main` describe the development branch and may differ from a release.

| ID | Claim | Source | Section | Retrieved | Status |
| --- | --- | --- | --- | --- | --- |
| CIL-EV-E01 | `cilium connectivity test` creates a test namespace (default `cilium-test`, suffixed with a sequence number) with client and echo Deployments, Services and policies; a cleanup flag removes namespaces, deployments and services. | <https://raw.githubusercontent.com/cilium/cilium/main/cilium-cli/cli/connectivity.go> | `--test-namespace` and `--cleanup` flag help (main branch) | 2026-11-25 | VERIFIED |
| CIL-EV-E02 | The upgrade pre-flight check is marked Required; it pre-pulls the new image and is applied with `kubectl create -f cilium-preflight.yaml`, creating a `cilium-pre-flight-check` DaemonSet and Deployment in kube-system. | <https://docs.cilium.io/en/stable/operations/upgrade/> | Running pre-flight check (Required); 1.20.5 | 2026-11-25 | VERIFIED |
| CIL-EV-E03 | `cilium sysdump` collects as many logs as possible from all nodes by default, plus Kubernetes and Cilium objects, the Cilium ConfigMap, Helm values, bugtool output, kvstore data and Hubble flows; no redaction code was found by text search of the collection code on main, which is not an exhaustive audit and may differ from the released CLI. | <https://raw.githubusercontent.com/cilium/cilium/main/cilium-cli/sysdump/sysdump.go> | Task list in sysdump.go (main branch); docs troubleshooting page for defaults | 2026-11-25 | UNVERIFIED |
| CIL-EV-E04 | GHSA-gj49-89wh-h4gj: cilium-bugtool output contained the WireGuard private key; fixed in 1.19.3, 1.18.9 and 1.17.15; rotate keys if bugtool or sysdump archives were shared. | <https://github.com/cilium/cilium/security/advisories/GHSA-gj49-89wh-h4gj> | Advisory summary, affected and patched versions | 2026-11-25 | VERIFIED |
| CIL-EV-E05 | Hubble stores monitor events in a per-agent user-space ring buffer and overwrites older events once it is full; capacity is one less than a power of two. | <https://docs.cilium.io/en/stable/internals/hubble/> | Hubble server, The Observer service; 1.20.5 | 2026-11-25 | VERIFIED |
| CIL-EV-E06 | `--hubble-event-buffer-capacity` defaults to 4095 and must be one less than a power of two, no larger than 65535; Helm key `hubble.eventBufferCapacity`. | <https://docs.cilium.io/en/stable/cmdref/cilium-agent/> | Options; 1.20.5 | 2026-11-25 | VERIFIED |
| CIL-EV-E07 | Hubble Relay connects to each node's Hubble server and provides cluster-wide, multi-node queries. | <https://docs.cilium.io/en/stable/internals/hubble/> | Hubble Relay; 1.20.5 | 2026-11-25 | VERIFIED |
| CIL-EV-E08 | `hubble observe --last N` returns the last N flows in the buffer (default 20), and through Relay that is N per node. | <https://github.com/cilium/hubble> | Hubble CLI observe help (main branch) | 2026-11-25 | VERIFIED |
| CIL-EV-E09 | The flow API defines lost-event sources PERF_EVENT_RING_BUFFER, OBSERVER_EVENTS_QUEUE and HUBBLE_RING_BUFFER with the meanings used in this skill. | <https://github.com/cilium/cilium/blob/1.20.4/api/v1/flow/README.md> | LostEventSource (1.20.4 tag, cross-checked via Context7) | 2026-11-25 | VERIFIED |
| CIL-EV-E10 | When the Hubble event queue is full, events are dropped from Hubble flows and metrics; this does not impact traffic. | <https://docs.cilium.io/en/stable/operations/performance/tuning/> | Hubble, Increase Hubble Event Queue Size; 1.20.5 | 2026-11-25 | VERIFIED |
| CIL-EV-E11 | Pods need to be managed by Cilium in order to be observable by Hubble. | <https://docs.cilium.io/en/stable/operations/troubleshooting/> | Observing flows with Hubble; 1.20.5 | 2026-11-25 | VERIFIED |
