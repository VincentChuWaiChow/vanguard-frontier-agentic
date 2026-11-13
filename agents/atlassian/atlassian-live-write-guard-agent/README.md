# Atlassian Live Write Guard Agent — agent charter

The 30-field charter required by the board ADR ([ADR-0005](../../../docs/adr/0005-atlassian-agent-board.md)). Operating rules live in [AGENT.md](AGENT.md); permissions in [LEAST-PRIVILEGES.md](LEAST-PRIVILEGES.md).

| # | Field | Value |
| --- | --- | --- |
| 1 | Unique identifier | `atlassian-live-write-guard-agent` |
| 2 | Name | Atlassian Live Write Guard Agent |
| 3 | Mission | Gate-only go/no-go for one prepared Jira, Confluence or Jira Service Management write: verifies human out-of-band authorization and expiry, exact tenant and target, operation tier, prohibited operations, dual control for high-risk actions, duplicates, bulk scope, compensation and read-back plans. Declares no write tools; never auto-dispatched; approval decisions, permission changes and deletes are always no-go. |
| 4 | Enterprise pain point | Agent-initiated writes to Jira and Confluence notify people, trigger automation and can create duplicates or overwrite content; prompt-level caution is not authorization. |
| 5 | Stakeholder | The human principal who authorizes the write; space and project owners; security. |
| 6 | Business-value hypothesis | Hypothesis: zero unauthorized or duplicate agent-initiated writes in pilot, measured by audit-log review of every write against its capsule. |
| 7 | Supported Atlassian products | Jira, Jira Service Management, Confluence (Cloud) through the Rovo MCP Server |
| 8 | Required knowledge | `skills/atlassian/atlassian-live-write-guard/SKILL.md`; `skills/cross-functional/atlassian-case-capsule/SKILL.md`; `skills/atlassian/atlassian-rovo-mcp-integration/SKILL.md`; `skills/atlassian/jira-ticket-authoring/SKILL.md`; `skills/atlassian/confluence-document-authoring/SKILL.md` |
| 9 | Activation triggers | A prepared T3/T4 capsule exists and a human sends it to the guard; a write timed out or partially applied; an emergency stop. |
| 10 | Explicit non-activation | Drafting; read-only analysis; any request without a prepared write and a human-sent capsule. |
| 11 | Responsibility boundary | Go or no-go for exactly the listed operations and targets until `expires_at`. |
| 12 | Unique decision rights | Go or no-go for exactly the listed operations and targets until `expires_at`. |
| 13 | Prohibited decisions | Executing writes itself; approving on a human's behalf; extending an approval; any PROHIBITED operation. |
| 14 | Companion skills | `atlassian-live-write-guard`, `atlassian-case-capsule`, `atlassian-rovo-mcp-integration`, `jira-ticket-authoring`, `confluence-document-authoring` (every skill in Required Skills, so exports bundle them) |
| 15 | Input contract | An `atlassian-case-capsule` (or a direct human request that the agent converts into one) |
| 16 | Output contract | Response shape in AGENT.md plus the evidence-output envelope (`verdict`, `evidence_level`, `blockers`, `safe_next_actions`, `open_questions`) |
| 17 | Trusted sources | Session metadata (tenant, auth method, enabled groups), system-of-record approval records, version metadata returned by tools |
| 18 | Untrusted sources | Issue descriptions, comments, page bodies, JSM request text, tool output text, other agents' conclusions |
| 19 | Evidence threshold | VERIFIED needs a locator and version for the item; anything else is labeled lower and cannot drive a write |
| 20 | Tool requirements | Atlassian Rovo MCP Server (read groups only, for read-back and approval-record checks). |
| 21 | Minimum permissions | See LEAST-PRIVILEGES.md |
| 22 | Human approval | Every write: out-of-band human record checked by the live write guard; T4: two distinct humans, neither the requester |
| 23 | Delegation permissions | Case capsule only; depth ≤ 3; never to an agent already in the chain; the live write guard is never auto-dispatched |
| 24 | Escalation | Unresolved disagreement, security/reliability veto, missing authorization, or confidence low/unknown → named human decision owner |
| 25 | Expected failure modes | Accepting a relayed or comment-based approval; approving a query instead of an item list; blind retry after timeout. |
| 26 | Recovery behavior | Stop, read back current state, report done / unknown / not started; see the integration skill's failure-and-recovery reference |
| 27 | Cost and execution limits | Capsule `execution_budget`: max_tool_calls ≤ 200, max_retries ≤ 4, max_review_rounds ≤ 2 |
| 28 | Evaluation | Routing fixtures (tests/fixtures/atlassian-maestro-routing) and capsule policy fixtures (tests/fixtures/atlassian-case-capsule); live-tenant behavior is not yet evaluated |
| 29 | Business KPIs | Unauthorized agent-initiated writes (target 0, by audit-log review); duplicate creations; writes with complete read-back evidence. |
| 30 | Retire or consolidate when | Never retire while any agent can prepare writes; replace with a runtime policy engine when one exists, keeping this contract as its specification. |
