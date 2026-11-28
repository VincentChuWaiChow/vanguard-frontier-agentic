# Incident hypotheses worksheet

Records competing explanations for failed or slow writes so each is tested by evidence that separates it from the others. The incident lead fills it; a second reader challenges the status column. Use it with the incident discrimination reference.

## How to use

- Fill from supplied evidence only; keep one row per hypothesis and do not delete a contradicted row.
- A hypothesis needs a discriminating observation, not just a consistent one; write what would be seen if it were false.
- Mark `Unknown` with the smallest item that would settle it; never paste secrets, tokens or customer identifiers.
- Status values: open, supported, contradicted, indeterminate. Keep the decision ID.
- A denial is a policy outcome; an error or timeout is an availability outcome; a filtered write is never evaluated.

## Hypotheses

| Hypothesis | Predicts | Discriminating evidence | Contradicted by | Status |
| --- | --- | --- | --- | --- |
| `<policy denial / webhook call error or timeout / controller unavailable / registry dependency error / upstream API rejection / filtered by resourceFilters / stale or incomplete reports>` | `<what the requester should see>` | `<evidence type and exact object>` | `<observation that would refute it>` | `<status>` |
| Policy denial | The policy's own message in the API error; request counted with request_allowed false | `admission-trace` showing the webhook response text | An API error naming the webhook with no policy message | open |
| Webhook call error or timeout | Error naming the webhook; delayed write under Fail | `admission-trace`; latency histogram for Pod CREATE in `payments` | A policy message in the error | open |
| Controller unavailable | No admission pod ready; all matching writes fail under Fail | `installed-config`: Deployment readiness | Some matching writes succeeding with evaluation | open |
| Registry dependency error | Failures only for image-verified Pods | `admission-trace`; registry availability record | Failures for kinds with no image policy | open |
| Upstream API rejection | Error from the API server with no webhook named | `admission-trace` at the API server | A webhook name in the error | open |
| Filtered by resourceFilters | No Kyverno evaluation or counter series for the kind or namespace | `installed-config`: resource filters and webhook selector | A request-counter series for that kind | open |
| Stale or incomplete reports | Report results lag the cluster or omit objects | `population-report` timestamps against the object list | Reports with current timestamps covering all objects | open |

## Evidence status

| Item | Type | Locator | Captured at | Omits |
| --- | --- | --- | --- | --- |
| `<item>` | `<type>` | `<locator>` | `<timestamp>` | `<what it cannot show>` |
| API error text for `payments/invoice-api` rollout | `admission-trace` | `<ticket attachment ID>` | `<timestamp>` | Requests that succeeded |
