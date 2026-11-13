# Jira integration and traceability

Use verified issue/page URLs as the baseline link mechanism. Refer to an unresolved issue key as a reference, not a fabricated hyperlink. Add native remote links or macro embeds only through a supported surface.

## Choose an integration

| Need | Cloud approach |
| --- | --- |
| Link one work item | Insert its actual URL; choose supported Smart Link display if available |
| Display matching work | Configure the Jira work items macro with validated JQL and intended columns |
| Show a count | Use the macro's count display, with the query retained |
| Plot issue statistics | Use supported Jira Chart options |
| Show a timeline | Use the separate Jira timeline capability where the project/space is supported |

The documented work-items macro supports individual items, query results, selected items, and counts. Pasting an actual issue URL is a supported editor workflow. Viewer permissions and the Jira connection still matter. [Jira work items macro](https://support.atlassian.com/confluence-cloud/docs/insert-the-jira-issues-macro/)

Editor URL conversion does not establish that a connector's Markdown conversion creates the same Smart Link, macro, or embedded view. A Markdown link is the portable baseline; verify the rendered object before claiming dynamic Jira content. Link display can also depend on supported views and preferences. [Links and Smart Links](https://support.atlassian.com/confluence-cloud/docs/insert-links-and-anchors/)

Do not present `{jira:...}`, `{jiraissues:...|count}`, or a handwritten macro parameter string as universal Cloud syntax. Keep query content separate from its renderer. For API authoring, inspect a supported macro representation or use a verified ordinary link.

Jira Chart documents Pie, Created vs Resolved, and Two Dimensional charts. Timeline is a separate macro, documented for a supported team-managed software space's Timeline URL. `{jira-chart:type=timeline|project=PROJ}` is not the documented model. [Jira Chart](https://support.atlassian.com/confluence-cloud/docs/insert-the-jira-chart-macro/) · [Timeline](https://support.atlassian.com/confluence-cloud/docs/insert-the-jira-timeline-macro/)

## Query example

```jql
project = "ENG" AND statusCategory != Done ORDER BY updated DESC
```

`ENG` is illustrative. Validate actual project, fields, work types, and query access. Use JQL for Jira and CQL for Confluence; do not interchange them. Treat external query values as data and escape them correctly.

## Document-to-work trace

| Source requirement or decision | Document section/version | Jira reference | Criterion or delivery outcome | Evidence |
| --- | --- | --- | --- | --- |
| Local requirement ID or actual decision | Exact relevant anchor and revision | Verified issue URL/key or pending reference | AC ID or outcome | Planned check / observed result and locator |

Explain the controlling rule next to its link so readers do not have to infer why a source matters. Preserve source IDs across revisions when meaning is unchanged. If meaning changes, record the change and affected work.

If both writes are authorized, create/update each artifact and add reciprocal references using the returned identifiers. Check existing links before adding new ones. Creating a hyperlink in Confluence does not prove a Jira remote link was created. Verify each side and report partial outcomes separately.

Use Jira tickets for bounded work and acceptance criteria; use Confluence for design, decision rationale, operational knowledge, and dated release/meeting records. Use the companion `jira-ticket-authoring` skill when available.

## Evidence and audience

A live issue macro shows current state. For release notes or a historical decision, record the release artifact, relevant issue snapshot/query, and retrieval time. A closed ticket or merged change alone does not prove production deployment, and a deployment does not prove enablement for every customer or rollout cohort.

Preserve confidentiality across both systems. A reader may have access to a page but not its linked issue, or vice versa. Identify a material access gap; do not broaden permissions or copy restricted details to a wider audience as a workaround.
