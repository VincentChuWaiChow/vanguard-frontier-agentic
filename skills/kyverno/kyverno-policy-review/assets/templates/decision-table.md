# Decision table worksheet

Records the policy-review decision: for one control, which legitimate operation must succeed, which in-scope violation must be caught, which adjacent out-of-scope object must be skipped, and whether the configured action matches intent and rollout stage. The reviewer fills it from supplied evidence only.

## How to use

- Fill each cell from supplied evidence (manifest, offline-test, installed-config); never paste secrets, tokens or customer identifiers.
- Write `Unknown` for any cell the evidence does not settle, and name the smallest item that would settle it.
- Keep one row per operation or object; keep the decision ID on the sheet.
- The engine column names where the result came from (offline CLI, admission, population report); an offline `fail` does not show whether admission denies or audits.
- The denial message must name the field to fix and who to ask.

## Decision table

Decision ID: `<KYV-DEC-nnnn>`. Control: `<control name and intent: mandatory preventive | detective | advisory>`.

| Legitimate operation | In-scope violation | Adjacent out-of-scope | Configured action vs intent and stage | Engine | Denial message | Evidence | Result |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `<operation that must succeed>` | `<operation that must be caught>` | `<object that must be skipped>` | `<Deny/Audit/Warn> vs <intent>, stage <stage>, owner <role>, deadline <date>` | `<offline CLI / admission / report>` | `<exact text>` | `<EV-n>` | `<supported / conditional / blocked / indeterminate>` |
| CREATE Pod `payments/legitimate-workload`, every container sets runAsNonRoot | CREATE Pod `payments/violating-workload`, init container has no runAsNonRoot | Pod `catalog/adjacent-workload` (outside the namespace condition) | Deny vs mandatory preventive, enforcing; owner payments platform owner | offline CLI v1.20.0: pass / fail / skip | "example-require-run-as-non-root: set securityContext.runAsNonRoot=true on every container and init container. If the image cannot run as non-root, ask the payments platform owner for a time-bound PolicyException." | EV-1 offline-test | `conditional`: logic shown offline; admission effect Unknown |

## Pod controllers

| Controller | Name used in autogen | Offline result present | Evidence | Result |
| --- | --- | --- | --- | --- |
| `<controller>` | `<lowercase plural resource name>` | `<yes/no>` | `<EV-n>` | `<result>` |
| Deployment `payments/violating-deployment` | deployments | yes: fail | EV-1 offline-test | `supported` for the tested object |

## Evidence status

| Item | Type | Locator | Captured at | Omits |
| --- | --- | --- | --- | --- |
| `<EV-n>` | `<manifest / offline-test / installed-config / ...>` | `<file or report>` | `<timestamp>` | `<what it does not show>` |
| EV-1 | offline-test | kyverno test output, pinned CLI v1.20.0 | `<timestamp>` | webhook behavior, installed config, existing resources |
