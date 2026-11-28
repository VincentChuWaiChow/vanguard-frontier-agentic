# Release envelope worksheet

Records whether the admission path can absorb one policy or configuration change, which writes are affected if it degrades, what aborts the release and how recovery works. The service owner and the release engineer fill it from supplied evidence; the reviewer reads it. Use it with the release envelope reference.

## How to use

- Fill every cell from supplied evidence only; write `Unknown` plus the smallest item that would settle it.
- Keep baselines at `unknown` until measured from a supplied series; never estimate a latency or a rate.
- The webhook error series is caller-supplied; name it, do not assume a metric.
- Never paste secrets, tokens, registry credentials or customer identifiers; keep the decision ID on every row.
- Recovery counts only if every dependency it needs is itself available during the degradation.

## Envelope

| Change | Matching operations | Measured write rate | Window | p95 / p99 by request class | Webhook error series (caller-supplied) | failurePolicy per policy | Replicas and PDB | Abort signal | Recovery path and its dependencies |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `<change ID>` | `<kind + operation>` | `<writes per minute, with series>` | `<planning window>` | `<per class, or unknown>` | `<series name>` | `<policy: Fail or Ignore>` | `<replicas, PDB on or off>` | `<signal, threshold, owner>` | `<rollback action; registry, GitOps controller, API access>` |
| `KYV-DEC-0000`: add image policy to invoice release | Pod CREATE in `payments` | Unknown: needs a 7 day request-counter export | Unknown: stated by service owner | Baseline unknown until measured | Unknown: caller to name the series | Image policy value unknown: read from installed-config | Two replicas, PDB unknown | Sustained p99 above the owner's budget; threshold supplied by the owner | Revert in Git; needs the GitOps controller and API access; recovery workloads in `payments` are matched by the policy |

## Evidence status

| Item | Type | Locator | Captured at | Omits |
| --- | --- | --- | --- | --- |
| `<item>` | `<manifest / installed-config / runtime-metric / admission-trace>` | `<path@revision or export ID>` | `<timestamp>` | `<what it cannot show>` |
| Admission latency histogram export | `runtime-metric` | `<export ID>` | `<timestamp>` | Registry latency under degradation |
