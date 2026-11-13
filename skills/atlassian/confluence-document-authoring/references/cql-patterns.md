# CQL and search patterns

Reference edition: Confluence Cloud. Reviewed 2026-11-11. These are query patterns, not proof of successful execution on a tenant. CQL searches Confluence; JQL searches Jira.

## Query selection

Replace illustrative spaces and labels with the target's actual values. Build a simple scoped query, run it through the available search surface, then refine. Check field-specific operators; not every field supports every operator. Escape user input with the target grammar or a structured client.

| Field | Supported operators relevant here |
| --- | --- |
| `ancestor`, `parent`, `space`, `label`, `creator`, `type` | `=`, `!=`, `IN`, `NOT IN` |
| `created`, `lastmodified` | `=`, `!=`, `>`, `>=`, `<`, `<=` |
| `title` | `=`, `!=`, `~`, `!~` |
| `text` | `~`, `!~` |

The field's operator support does not automatically extend to every function used with it; `currentUser()` supports `=` and `!=`. [CQL fields](https://developer.atlassian.com/cloud/confluence/cql-fields/) · [CQL functions](https://developer.atlassian.com/cloud/confluence/cql-functions/)

| Goal | Illustrative CQL |
| --- | --- |
| Technical designs in a space | `type = page AND space = "ENG" AND label = "tdd" ORDER BY lastmodified DESC` |
| Accepted-ADR candidates under a maintained label convention | `type = page AND space = "ENG" AND label = "adr" AND label = "adr-accepted"` |
| API pages mentioning authentication | `type = page AND space = "ENG" AND label = "api-docs" AND text ~ "authentication"` |
| Production runbooks | `type = page AND space = "OPS" AND label = "runbook" AND label = "production"` |
| Draft-labeled pages created by the connected identity | `type = page AND label = "draft" AND creator = currentUser()` |
| Pages modified since a calendar-day boundary | `type = page AND space = "ENG" AND lastmodified >= startOfDay("-7d")` |

A label such as `draft` or `adr-accepted` is a maintained team convention, not a native publication state or approval record. Confirm the authoritative page metadata after searching. If the site uses native content status, discover its values and the current `pageStatus` field support; a status macro or table cell is not automatically that field. [CQL fields](https://developer.atlassian.com/cloud/confluence/cql-fields/)

`ancestor = 12345` searches descendants at all depths; `parent = 12345` selects direct children. Use a verified content ID, never an example number. The `space` field uses a space key, not the REST v2 numeric space ID. `currentUser()` requires an authenticated connection and refers to that identity, which may differ from the human requester. [CQL fields](https://developer.atlassian.com/cloud/confluence/cql-fields/) · [CQL functions](https://developer.atlassian.com/cloud/confluence/cql-functions/)

## Date semantics and a documentation discrepancy

`startOfDay("-7d")` denotes the start of the day seven days ago. It is not exactly the preceding 168 hours. For an exact rolling interval, calculate an explicit cutoff using the target's configured timezone and a documented date/time format such as `"yyyy-MM-dd HH:mm"`; account for the supported precision. Record the evaluation time and timezone.

Do not declare `now("-1w")` invalid solely because it is absent from the function catalogue. Official operator and created-field examples use `now("-4w")`, while the dedicated functions page and supported-functions lists omit it. Treat this as inconsistent documentation: validate offset queries against the target, or use an explicit cutoff. [Operators](https://developer.atlassian.com/cloud/confluence/cql-operators/) · [Fields](https://developer.atlassian.com/cloud/confluence/cql-fields/) · [Functions](https://developer.atlassian.com/cloud/confluence/cql-functions/)

## Text is not structured decision state

`text ~ "Status: Approved"` searches indexed text; it does not establish that the ADR is accepted, nor that the status field has that value. It may match discussion, old decisions, or examples. Use governed labels or verified native status to identify candidates, then inspect the decision record.

The `~` operator uses Confluence text-search semantics, not an arbitrary substring contract. Exact phrase queries need the documented nested quoting/escaping. [Text search](https://developer.atlassian.com/cloud/confluence/performing-text-searches-using-cql/)

```cql
text ~ "\"technical design\""
```

That searches for the phrase; `text ~ "technical design"` searches the terms without requiring that phrase. If the CQL is sent inside JSON, use a serializer to escape the CQL string again; URL encoding is a separate transport layer. Do not concatenate unescaped user text into the query.

## Search completeness

Follow returned cursors/next links when the requested scope spans pages. Keep the original filters while paginating. Report unreadable pages, unvisited pages, indexing delay, and permission constraints relevant to the conclusion. Fetch a just-created page by its returned ID when search has not indexed it. Do not interpret no accessible matches as proof of absence.

Retain the query and retrieval time for a report. A live Jira/Confluence macro is a changing view; use a dated evidence snapshot for historical decisions or release claims.
