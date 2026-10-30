# Tools, memory and data reach

Reference for [agentic-security-review](../SKILL.md). Load it when you compute what an agent can reach, review tools as privilege grants, trace injection and exfiltration paths, review memory write paths, or review how model output reaches a downstream interpreter.

Everything below is a static review of supplied material. No repository rule computes data reach, detects exfiltration, or filters content at runtime; those are product B concerns. Where a test is suggested it is a plan for the adopter's authorized owner to run in a non-production environment, with synthetic and visibly marked values.

## Effective data reach

Effective data reach is the union, over every tool, retrieval source, memory store and peer agent the agent can use, of the data each can read, change or send under the identity it runs as, tagged with a data classification. Use the vocabulary of the handoff envelope: public, internal, confidential, restricted.

Compute it from grants and configuration, never from the prompt. Instructions are not a boundary: a prompt that says only use the ticket tool for this customer limits what a cooperative model chooses to do, not what the identity can reach.

1. List tools, retrieval sources, memory stores and peer agents.
2. For each, record the identity it runs as and what that identity can reach: datasets, APIs, tenants, files. Include indirect reach: any tool that takes a URL, a path, query text or an identifier that the model chooses.
3. Mark each as read, write, send or execute; note egress; assign a classification.
4. Merge the rows into one reach table.
5. Set the table against what the use case needs. The gap is the finding.

A synthetic example. The use case is: answer customer questions about open tickets for one tenant.

| Tool or source | Runs as | Can reach | Classification | Access | Egress |
|---|---|---|---|---|---|
| ticket_search | agent service identity | Tickets of every tenant | confidential | read | none |
| kb_retrieval | agent service identity | Wiki, including a space for personnel matters | restricted | read | none |
| send_email | agent service identity | Any external address | not applicable | send | yes |

Need: one tenant's tickets, read only. Gap: tickets of other tenants, the personnel space, and an outbound channel. Each gap row is a candidate finding; the combination is the exfiltration triad described below, which is a HARD-gate candidate.

## Tools as privilege grants

Treat each tool as a grant of authority, not a convenience.

- Classify what the tool can do: read, write, send (anything that leaves the trust boundary), execute, or act with a credential.
- Ask whether the tool enforces its own argument bounds and authorization, or relies on the model to behave. Bounds that exist only in the prompt are not bounds.
- Ask whether the call runs with the caller's entitlement or the agent's; see [identity and delegated authorization](identity-and-delegated-authorization.md).
- Ask whether a human sees the exact arguments and target before a consequential call, and whether what the human sees is what runs.
- Ask who can change the tool, its description or its schema, and how a change is noticed; see [supply chain and provenance](supply-chain-and-provenance.md).

Exfiltration triad (judgment, not an external source). An agent that combines all three of these is a HARD-gate candidate:

1. reach to confidential or restricted data,
2. ingestion of untrusted content,
3. an outward channel.

Ask which leg is broken, by what, and where that control is enforced. A leg is broken only by a control outside the model: tool-side authorization, a network egress policy, a human confirmation that shows the exact arguments, or data that is not retrievable at all. A prompt instruction is not a broken leg.

## Indirect prompt injection

Indirect prompt injection is instruction text that reaches the model through data the agent reads instead of through the user's own message. Treat every one of these channels as attacker-controlled until shown otherwise:

- web pages, documents, tickets, emails and chat messages the agent retrieves,
- tool output and error messages, including output from a trusted tool that echoes untrusted data,
- file names, metadata and image text,
- messages from other agents, and Agent Card text,
- memory entries written earlier,
- tool names, descriptions and schemas, which are text the model reads as part of its context.

Tool descriptions deserve explicit attention. A server author, a compromised server, or a description that changes after approval can put directive text where the model treats it as guidance. Review descriptions as you review prompts. `typescript-mcp-tool-contract-agent` flags directive-shaped text in a tool description as a possible injection vector for servers it reviews; this skill asks the wider question of what a steered call could do.

Review checks:

- Which privileged actions are reachable in the same session after untrusted content has been read?
- Does a consequential call need a confirmation that shows exact arguments, a target and a reason?
- Can tool access be narrowed once untrusted content enters the context, for example by splitting a read-untrusted phase from an act phase? The pattern choice belongs to `agentic-system-architecture-review-agent`; the security question is whether the narrowing is enforced outside the model.
- Is any control described as the model noticing and refusing? That is detection, not prevention, and detection is probabilistic. A demonstration that an attack failed once is not evidence; a reviewable control is.

## Exfiltration paths

| Path | How data leaves | Evidence to request | Control direction |
|---|---|---|---|
| Tool chaining | The result of a read tool becomes an argument to a send or write tool | The tool graph: which tools can be called after which, and with what identity | Different identities and a confirmation at the boundary; a policy outside the model that blocks confidential data in send arguments |
| Egress-capable tools | HTTP fetch, webhooks, email, chat, ticket comments, file upload | The egress allowlist and the destinations each tool can reach | Default-deny egress, allowlisted per tool |
| Rendering | Output rendered by a client that auto-fetches images or follows links, with data placed in a URL | How the client renders model output; the image and link policy | Disable auto-fetch, allowlist origins, strip or rewrite links in model output |
| Write-capable tools to shared locations | The agent writes data where another party can read it: a shared document, a public repository, a ticket the requester can see | Writable locations and who can read each | Write only where the audience is no wider than the data's audience |
| Logs and telemetry | Prompts and tool results copied into logs, traces, analytics or a third-party observability service | Log sinks, retention and redaction | Redact before export; classify trace stores like the data they hold |
| Memory | Data stored in one context is recalled in a context with a wider audience | The memory section below | Partition by user, tenant and task |
| Errors | Detail in error text or tool output shown to the requester | Sample error paths | Return minimal errors; log detail where only an owner can read it |

## Memory and context poisoning

This skill owns memory and context poisoning as a threat (ASI06). The architecture review owns the memory lifecycle: state scope, retention, and deletion that reaches checkpoints and replicas. See [state, memory and durability](../../agentic-system-architecture-review/references/state-memory-and-durability.md) and hand lifecycle findings there.

Every place the system can store something that a later model call reads is a write path:

- the conversation window and any summary or compaction of it,
- long-term memory in key-value, document or vector stores,
- shared or team memory,
- caches of tool results,
- notes or instructions the agent writes for itself.

For each write path, ask:

1. Who or what causes a write: the user, a rule, or the model's own decision? Can untrusted content lead to a write?
2. Does each entry carry provenance: source, the caller who caused it, time, a trust level (user-stated, tool-derived, document-derived), tenant, and expiry?
3. How is an entry recalled: as data with its provenance visible, or concatenated into the prompt as if it were an instruction?
4. Is it partitioned by user, tenant and task, so that one party's write cannot reach another party's reads?
5. Is there a retention and expiry review, so stale or unreviewed entries age out?
6. Can an entry be found by provenance and quarantined, and can you tell which later outputs recalled it? Does a purge reach summaries derived from the entry and replicas?

Poisoning tests (plans for the adopter's owner, synthetic values only):

| Test | Steps | Safe result |
|---|---|---|
| Instruction persistence | Put a marked synthetic instruction in an untrusted document and ask the agent to remember it. Start a later session as the same user. | The text is not recalled or followed as an instruction |
| Cross-user recall | Store a private entry as one user. Query as another user of the same tenant. | No recall |
| Cross-tenant recall | Repeat as a user of another tenant. | No recall |
| Summary laundering | Run a long conversation containing marked text through compaction. | The summary does not promote the marked text to a trusted statement |
| Expiry | Query after an entry's expiry. | The entry is not recalled |
| Purge | Delete an entry, then query for it and for summaries derived from it. | Neither is recalled |

## Output handling

Model output is untrusted input to whatever consumes it. List every sink: SQL, shell, code interpreters, HTML and markdown renderers, templates, API calls with model-built URLs or bodies, configuration files, and the prompts of other agents.

- Validate output against a schema before use, and encode it for the sink. Parameterize queries; do not splice model text into them.
- Do not execute generated code outside a sandbox with no credentials, bounded resources and no unneeded egress (ASI05).
- Do not auto-follow URLs the model built, and do not render links or images from model output without a policy.
- Treat another agent's output as untrusted data, not as instruction (ASI07).
- Redact and flag credential-shaped strings in output; do not echo them into the review. `mods/secret-redactor` is an optional harness hook for this, with documented limits: regex redaction is not complete DLP, and Kiro and Codex cannot redact tool output (see mods/secret-redactor/mod.toml and mods/README.md).

## What the repository can and cannot show

- The offline gate scans a delegate objective for a leading mutation verb, and any objective for instruction-override or claimed-approval phrasing. It does not scan retrieved content, tool output, memory or evidence.
- Nothing here computes reach, detects exfiltration, filters output, or partitions memory. The deploying organization's runtime does, or nothing does.
- A review of static material can state that a path exists in the design as supplied. It cannot show that the deployed system has no other path.
