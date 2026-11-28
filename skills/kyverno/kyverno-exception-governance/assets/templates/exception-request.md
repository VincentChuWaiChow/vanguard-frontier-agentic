# Exception request worksheet

Records one PolicyException request or renewal: exactly what is excepted, who authorized it, what compensates, when it ends and how it is removed. The requester fills the request columns; the reviewer fills disposition and adjacent checks. Approval authority stays with the named risk delegate.

## How to use

- Fill each cell from supplied evidence (manifest, installed-config, rendered-gitops); never paste secrets, tokens or customer identifiers.
- Write `Unknown` where evidence is missing and name the smallest item that would settle it.
- Name roles, not people; keep the decision ID.
- A date in an annotation is not a removal mechanism; record `expiresAt` and the reconciling process separately.
- Always fill the adjacent-workloads cell with the workloads tested and their results.

## Request

Decision ID: `<KYV-DEC-nnnn>`.

| Exact workload | Namespace | Policy / kind | Rule | Reason | Authorized creator | Compensating control |
| --- | --- | --- | --- | --- | --- | --- |
| `<name>` | `<namespace>` | `<policy name> / <kind>` | `<rule or validation>` | `<why the deviation is needed>` | `<role>` | `<control and how it is checked>` |
| `legacy-invoice-exporter` | `payments` | `payments-run-as-non-root / ValidatingPolicy` | run containers as non-root | legacy exporter image cannot run as non-root | payments platform owner (role) | label `network-isolation=strict` checked by an exception validation |

## Approval and removal

| Reviewer | expiresAt | Expiry owner | Removal mechanism | Adjacent workloads checked | Disposition |
| --- | --- | --- | --- | --- | --- |
| `<role>` | `<RFC3339 or none>` | `<role>` | `<reconciled deletion / none>` | `<workload: result>` | `<approve-as-scoped / narrow-scope / renew-with-conditions / reject / expire-now>` |
| risk delegate (role) | `<RFC3339, set>` | payments service owner (role) | Unknown: reconciliation evidence not supplied | `invoice-batch-root: fail` (offline) | `renew-with-conditions` pending removal evidence |

## Evidence status

| Item | Type | Locator | Captured at | Omits |
| --- | --- | --- | --- | --- |
| `<EV-n>` | `<manifest / installed-config / ...>` | `<file>` | `<timestamp>` | `<what it does not show>` |
| EV-1 | offline-test | kyverno test output, pinned CLI v1.20.0 | `<timestamp>` | expiry behavior, enablement flag, namespace allowance |
