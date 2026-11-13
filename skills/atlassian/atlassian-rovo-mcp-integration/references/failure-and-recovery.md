# Failure and recovery

## Rate limits

When any Jira Cloud rate limit is exceeded the response is HTTP 429; clients respect `Retry-After` and back off. `RateLimit-Reason` values include `jira-quota-tenant-based`, `jira-burst-based` and `jira-per-issue-on-write` [ATL-E15]. Per-issue write limiting allows 20 writes per 2 seconds and 100 writes per 30 seconds on one issue [ATL-E16]. Retry only if the API is idempotent and the response includes `Retry-After` [ATL-E17]. Record the `RateLimit-Reason`; a per-issue reason means batching writes to one issue should be reduced, not retried harder. Retries count against the capsule's `max_retries`.

## Failure table

| Failure | Detection | Response |
| --- | --- | --- |
| 429 on a read | HTTP 429, `Retry-After` present [ATL-E15] | Wait the stated time, retry once within `max_retries` [ATL-E17]. |
| 429 on a write | 429, `RateLimit-Reason` `jira-per-issue-on-write` or other [ATL-E15] [ATL-E16] | Do not retry unless idempotent and `Retry-After` present [ATL-E17]. Read back first: the write may have landed. |
| 429 without `Retry-After` | Header absent | Do not retry [ATL-E17]. Report and ask the human. |
| Expired or revoked OAuth | Auth error on a call that worked before | Stop. Do not fall back to an API token unless the admin enabled it [ATL-E11]. Ask the human to re-authenticate; re-establish the session contract. |
| Tool missing from `discover` | Expected name absent from `discover` output [ATL-E03] | The group is likely not enabled or the scope is missing [ATL-E04] [ATL-E05]. Report UNAVAILABLE; never substitute a different tool of unknown tier. |
| Permission denied | 403 or tool-level denial | Do not retry or try other identities. Acting as the user means the human lacks access [ATL-E12]; report and escalate. |
| Timeout, write outcome unknown | No response after a create, edit, or transition | Read back or search first (`searchJiraIssuesUsingJql` [ATL-E06], or `getConfluenceContent` and `diffConfluenceContentVersions` [ATL-E08]). Never blind-retry a create. |
| Duplicate creation | Search finds two items with the same summary or key content | Stop; do not delete (delete is PROHIBITED). Report both, propose which to keep, and hand any closing action to the live write guard. |
| Confluence version conflict | Update rejected on a stale version | Re-read and diff [ATL-E08]; follow [publishing rules](../../confluence-document-authoring/references/publishing.md). Do not restate or bypass them. |
| Partial cross-product execution | Jira step done, Confluence step failed (or reverse) | No atomicity across products. Record completed steps with read-back evidence, then compensate through the live write guard or report the partial state. |
| Budget exhaustion | `max_tool_calls`, `max_retries` or review rounds reached | Stop. Return what is verified, mark the rest PROPOSED or UNAVAILABLE, and hand back. |
| Interrupted workflow | Session lost or context reset | Resume from the last confirmed read-back, not from memory. Re-run `discover`, re-check tenant, re-read target state. |

## Idempotency notes

- `createJiraIssue` is not idempotent [ATL-E06]: search by a unique marker before any second attempt.
- `editJiraIssue` and `transitionJiraIssue` can be re-applied only after a read shows the current state differs from the target [ATL-E06].
- Confluence writes in `write_confluence` [ATL-E07] and JSM writes in `write_jsm` [ATL-E09] follow the same rule: read the live state, then decide. Customer-visible JSM actions (`createJsmRequest`, `transitionJsmRequest`) are never repeated without a read-back, because a duplicate is seen by the customer.

## Resume checklist

1. Re-establish the session contract (server, auth, groups, tenant).
2. Re-run `discover` and re-classify tools.
3. Read the target state; compare with the last recorded read-back.
4. Check approval expiry against the capsule's `as_of`; an expired approval is not valid.
5. Continue only the steps not confirmed done.
