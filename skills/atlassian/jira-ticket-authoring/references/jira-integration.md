# Jira integration

Default authoring artifacts to portable Markdown. Discover the actual connector and target site's capabilities at execution time. A template is not a REST request.

## Resolve before writing

1. Resolve the intended site and project from the user's instruction, supplied URL, or accessible context. Disambiguate only when necessary.
2. Read the existing issue for updates. For creation, search narrowly for existing work with the same outcome before adding another item; search visibility and pagination limit duplicate-detection confidence.
3. Obtain the project's available work types and applicable field metadata, including required fields and allowed values. Follow pagination. Discover optional fields when setting them. Map content patterns such as “spike” to a configured type; do not create a type implicitly.
4. Resolve assignees, versions, components, parent and links from real identifiers. Unknown values stay unset where allowed, or become focused questions when required.
5. Prepare the exact field change and preserve unrelated data. A request for a draft/review is not a request to publish. An explicit resolved create/update request supplies authorization; do not ask for confirmation again.

## Body formats and hierarchy

Direct Jira Cloud REST v3 uses ADF for `description`, `environment`, and multiline text custom fields; single-line text fields use strings. Connector tools may accept Markdown or HTML and convert them. Read the current schema instead of applying the REST body type to a connector. [REST v3 issues](https://developer.atlassian.com/cloud/jira/platform/rest/v3/api-group-issues/)

Available create fields and required fields depend on project/type metadata. A subtask needs an actual subtask type and a parent identifier. A parent mentioned in prose does not create hierarchy. [REST v3 issues](https://developer.atlassian.com/cloud/jira/platform/rest/v3/api-group-issues/)

For Data Center, verify the installed version's API and editor. Do not reuse Cloud v3 assumptions automatically.

## Service requests and communication audience

When publishing a service-management request, inspect the active connector's request-specific metadata and visibility/audience controls. A configured Jira work type alone may not provide the request type, required request fields, or customer context needed by that operation. Verify the supported request contract at execution time; leave unresolved required values explicit.

An internal note and a customer-visible update have different audiences. Check the intended visibility before writing. Draft customer communications remain unsent unless sending has been explicitly authorized; reuse that authorization when it already exists. Do not copy restricted diagnostic evidence into customer-facing text.

## Preserve existing content

Fetch the current body in a representation that can preserve tables, inline media, and embedded content. Connector conversion may be lossy; inspect its current edit contract before rewriting. Prefer a targeted section change when supported. If only replacement is available, merge against the latest full content and inspect the rendered/read-back result.

List-valued fields may replace their entire value. Read existing labels/components and merge the intended change instead of erasing the other values. A description edit does not authorize changing status, sprint, assignee, or priority.

## Relationships

Express the intended relationship first: `A blocks B`. Read the configured type's outward/inward wording and the active tool's argument contract before assigning arguments; parameter names alone are not enough. Read back the relation from both issues where possible. Hierarchy, Jira-to-Jira links, and external document links are different operations. [Issue linking model](https://developer.atlassian.com/cloud/jira/platform/issue-linking-model/)

Check for existing external links before adding a Confluence reference. Keep canonical URLs where available and avoid duplicates. An issue-link retry can still add another comment even when the relation already exists, so retrying the whole request is not always harmless. [Issue link API](https://developer.atlassian.com/cloud/jira/platform/rest/v3/api-group-issue-links/)

## Ambiguous and partial outcomes

- If creation returns an issue key but a secondary operation fails, retain the key and repair the failed operation. Do not create the ticket again.
- If a timeout leaves the write outcome unknown, inspect the target or search before retrying. Report unresolved ambiguity rather than promising success.
- If an edit succeeds but a move, link, or reorder fails, report those outcomes separately.
- For concurrent edits, re-read and reapply the intended delta; do not overwrite newly added content from a stale snapshot.
- After writing, verify the returned key, title, relevant body/criteria, field values, parent, and links. Use actual returned URLs. Report checks that access or tool limitations prevented.

These are engineering safeguards. Exact error shapes, representation conversion, and deferred-tool availability are connector-specific and can change. [Rovo MCP supported tools](https://developer.atlassian.com/cloud/rovo-mcp/guides/supported-tools/)
