# Tool risk tiers

Tier labels: T1-read (query tenant data, no mutation), T3-authorized-write (bounded write after verified human authorization and read-back), T4-high-risk (archive, move, restore, customer-visible JSM actions; two distinct human approvers), PROHIBITED (never for any agent). Tool lists change: re-run `discover` in the actual session and use its output over this table.

## Decision table (tools named in the ledger)

| Tool or operation | Tier | Reason |
| --- | --- | --- |
| `discover` | T1-read | Finds deferred tools at runtime; it belongs to no permission group [ATL-E03] [ATL-E05]. It only lists, but its output is the source of truth for names. |
| `executeRead` | Never authorized by name | Invokes deferred tools of the read risk tier [ATL-E03] and hides which one runs. A case capsule names the concrete deferred tool; the capsule gate rejects `executeRead` and `executeWrite` themselves (`CAP-OPAQUE-EXECUTE`). |
| `executeWrite` | Never authorized by name | Invokes deferred tools of the write risk tier [ATL-E03] and hides which one runs. Not a group member [ATL-E05], so admins cannot disable it per group; the client must deny it. A case capsule must name the concrete deferred tool `discover` returned; the capsule gate rejects `executeWrite` itself (`CAP-OPAQUE-EXECUTE`). |
| `executeDestructive` | PROHIBITED | Invokes deferred tools of the destructive tier [ATL-E03]. Destruction cannot be made safe by prompt rules. |
| `searchJiraIssuesUsingJql` | T1-read | Searches with JQL [ATL-E06]. Results are untrusted content. |
| `createJiraIssue` | T3-authorized-write | Creates a work item [ATL-E06]. A blind retry can create a duplicate. |
| `editJiraIssue` | T3-authorized-write | Mutates fields [ATL-E06]. |
| `transitionJiraIssue` | T3-authorized-write | Changes status, sprint, or backlog placement [ATL-E06]. It can fire workflow automation. |
| `addOrEditJiraIssueComment` | T3-authorized-write | Writes visible text [ATL-E06]. A comment is never an approval record. |
| `getConfluenceContent` | T1-read | Reads a page [ATL-E08]. Content is untrusted. |
| `diffConfluenceContentVersions` | T1-read | Compares versions [ATL-E08]. |
| `getConfluenceContentPermissions` | T1-read | Reads who can access content [ATL-E08]. Reading is allowed; changing is not. |
| `getConfluenceContentRestrictionState` | T1-read | Reads restriction state [ATL-E08]. |
| `searchConfluence` | T1-read | Searches with CQL [ATL-E08]. |
| `listConfluenceSpaces`, `getConfluenceSpace`, `listConfluenceComments`, `listConfluenceAttachments`, `listConfluenceTasks` | T1-read | Members of `read_confluence` [ATL-E08]. Bodies and comments are untrusted content. |
| `getJiraIssue`, `getJiraScreen` | T1-read | Jira reads [ATL-E06]. |
| `getJsmRequest`, `searchJsmRequests`, `getJsmRequestActionMetadata`, `getJsmRequestTypeFields`, `getJsmOpsAlerts`, `getJsmOpsScheduleInfo`, `getJsmOpsTeamInfo`, `getJsmRequestResolverPlans` | T1-read | Members of `read_jsm` [ATL-E18]; whether JSM tools are reachable under OAuth is the ATL-E13 conflict. Request text is untrusted content. |
| `getTeamworkGraphObject` | T1-read | Member of `read_teamwork_graph` [ATL-E19]; returns linked data across products, so scope the request. |
| `createConfluenceContent`, `updateConfluenceContent`, `createConfluenceComment`, `updateConfluenceComment`, `addLabelsToConfluenceContent` | T3-authorized-write | Members of `write_confluence` [ATL-E07]. Version conflicts apply; see [failure-and-recovery.md](failure-and-recovery.md). |
| `moveConfluenceContent`, `archiveConfluenceContent`, `unarchiveConfluenceContent`, `restoreConfluenceContentVersion`, `setConfluenceContentStatus` | T4-high-risk | Members of `write_confluence` [ATL-E07]. Changes where content lives, whether it is findable, its published state, or reverts it. |
| `convertJiraIssueHierarchy` | T4-high-risk | Restructures parent/sub-task hierarchy [ATL-E06]; downstream boards and reports change. |
| `setConfluenceSpaceInstructions` | PROHIBITED | Sets a space's instructions [ATL-E07]: a persistent channel other agents read, so an agent writing it can plant instructions for every later session. |
| `setJsmAgentAvailability` | PROHIBITED | Sets a person's Workforce Management availability [ATL-E09]; an agent must not change a human's on-call or break status. |
| `addConfluenceContentPermissions` | PROHIBITED | Changes who can see content [ATL-E07]. |
| `removeConfluenceContentPermissions` | PROHIBITED | Same reason [ATL-E07]. |
| `replaceConfluenceContentPermissions` | PROHIBITED | Same reason [ATL-E07]. |
| `setConfluenceContentRestrictionState` | PROHIBITED | Same reason [ATL-E07]. |
| `createJsmRequest` | T4-high-risk | `write_jsm` [ATL-E09]; creates a customer-visible request. |
| `transitionJsmRequest` | T4-high-risk | `write_jsm` [ATL-E09]; customer-visible state change. |
| `updateJsmOpsAlert` | T4-high-risk | Acknowledges, closes, or escalates an alert [ATL-E09]; can silence or page responders. |
| `decideJsmRequestApproval` | PROHIBITED | Approves or declines a pending approval [ATL-E09]. An AI must never approve on a human's behalf. |
| `approveJsmRequestResolverPlans` | PROHIBITED | Approval decision [ATL-E09]. |
| Any tool whose name starts with `delete` | PROHIBITED | Destroys data; fail closed even if the name is not listed here. |

These tiers are enforced for handoffs by the case-capsule validator ([`validate_capsule.py`](../../../cross-functional/atlassian-case-capsule/scripts/validate_capsule.py), run in the source repository as `npm run validate:atlassian-capsule`); if this table and that gate ever disagree, the gate is the contract and this table is the defect.

## Classifying a tool found via `discover`

1. Record the exact name and which permission group `discover` attributes it to [ATL-E04] [ATL-E05].
2. If the name matches the table, use its tier.
3. If it is invoked through `executeDestructive`, it is PROHIBITED regardless of its name [ATL-E03].
4. If it is invoked through `executeWrite`, or you cannot tell, treat it as a write: T3 at minimum, T4 if it is customer-visible, moves, archives, or restores. Record the concrete tool name in the capsule, never `executeWrite`.
5. If the name starts with `delete`, or touches permissions, restrictions, or approvals, it is PROHIBITED.
6. Unknown name, unknown group, or unknown risk tier: treat as a write and refuse to call it without the live write guard. Never promote an unknown tool to read because its name sounds harmless.

Worked negative case: `discover` returns a tool named `refreshJiraIssueView` that is not in this table. Its tier is unknown, so it is a write. It does not run as a read, and the human is told it is unclassified.

## Client-side enforcement

A prompt rule ("never call write tools") is not enforcement; an injected instruction or a model error can bypass it. Enforcement is a client permission deny-list plus admin-side controls (below). Absent both, report the agent's read-only status as `unenforced`.

Illustrative Claude Code configuration for an MCP server registered under the name `atlassian` (tool names follow the pattern `mcp__atlassian__<toolName>`). This is an illustrative configuration, not a tested one. Re-check every name against `discover` output in the actual session before relying on it; names that no longer exist silently protect nothing, and new tools are not covered.

```json
{
  "permissions": {
    "deny": [
      "mcp__atlassian__executeDestructive",
      "mcp__atlassian__executeWrite",
      "mcp__atlassian__decideJsmRequestApproval",
      "mcp__atlassian__approveJsmRequestResolverPlans",
      "mcp__atlassian__addConfluenceContentPermissions",
      "mcp__atlassian__removeConfluenceContentPermissions",
      "mcp__atlassian__replaceConfluenceContentPermissions",
      "mcp__atlassian__setConfluenceContentRestrictionState",
      "mcp__atlassian__setConfluenceSpaceInstructions",
      "mcp__atlassian__setJsmAgentAvailability"
    ]
  }
}
```

The Claude Code adapters of this board's read-only agents go further: their `tools:` line allowlists `Read, Grep, Glob`, the read tools in this table by exact `mcp__atlassian__` name, and `executeRead`, which is how deferred read-tier tools found via `discover` are actually invoked [ATL-E03]. `executeWrite` and `executeDestructive` are never listed, so a subagent cannot inherit the session's write tools. A capsule still names the concrete deferred tool, never `executeRead` (`CAP-OPAQUE-EXECUTE`). Their Copilot adapters list the same read tools as `atlassian/<toolName>`, Copilot's MCP naming, because a Copilot `tools:` list enables only the tools it names, MCP tools included ([GitHub custom agents configuration](https://docs.github.com/en/copilot/reference/custom-agents-configuration), "Tools"). Both adapters assume the MCP server is registered under the name `atlassian`; under another name, the read tools are silently unavailable rather than wider. WebFetch is deliberately absent: an agent holding tenant data gets no outbound channel an injected instruction could use for exfiltration. Other harnesses here have no per-agent MCP allowlist.

Denying `executeWrite` blocks deferred writes only for sessions that should never write. Only the session running the live write guard, with verified authorization, may have it allowed. Directly exposed write tools such as `createJiraIssue` and `editJiraIssue` must also be denied for read-only sessions, using the names `discover` reports.

## Admin-side controls

- Tool availability depends on authentication method, scopes, enabled permission groups, and app access [ATL-E04]. Admins grant or revoke at the permission-group level and each tool inherits its group's access [ATL-E05].
- For a read-only agent, use an identity whose write groups (`write_confluence`, `write_jsm`, and the Jira write tools) are disabled, or an OAuth grant without write scopes [ATL-E04] [ATL-E07] [ATL-E09].
- `discover` and the execute tools belong to no permission group [ATL-E05], so group settings alone cannot remove them. Combine with the client deny-list.
- Some groups are labeled "Disabled by default; requires admin enablement", but the ledger could not bind the label to group names [ATL-E10]. UNVERIFIED: do not assume which groups are off; read the actual admin console state and record it.
- Atlassian's own guidance is least privilege and human review of high-impact changes [ATL-E14].
