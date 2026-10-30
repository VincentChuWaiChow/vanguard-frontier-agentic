# Identity and delegated authorization

Reference for [agentic-security-review](../SKILL.md). Load it when you review who an agent acts as, how authority is delegated to it and onward, how MCP or A2A calls are authorized, or whether a design leaks or invents authority in its messages.

The question this reference answers is: whose authority does each call carry, and who can show it afterward? A finding arises when the authority exercised is wider than the authority of the person or task that caused the call, or when nobody can tell who caused it.

Nothing in this repository authenticates a sender or enforces authorization. The material below is what to ask of a design and of the runtime that deploys it (product B).

## Caller identity and effective identity

Use the vocabulary of schemas/audit-event.schema.json, which this repository uses for its own live control plane:

- caller_identity: "Verified human/requester identity. Must be an identified individual principal — never a shared or unidentified identity for R3+ actions."
- effective_identity: "Service/JIT identity the action ran as."

Build an identity map with one row per hop and mark what survives.

| Identity | What it is | Review question |
|---|---|---|
| Caller | The individual on whose behalf the task runs | Is it carried to every tool boundary, or does it disappear at the agent? |
| Agent | The identity of the agent itself (a workload identity) | Is it distinct per agent and per environment, or shared with other agents? |
| Tool or service (effective) | The credential the tool uses against a downstream system | Is it scoped to the tool's purpose, short-lived, and issued just in time? |
| Peer agent | Another agent in the topology | How is it authenticated, and what may it ask for? |

Mark each hop as caller carried, caller lost, or caller replaced by a shared identity. A hop where only the effective identity is known means the audit trail cannot show who asked for the action. At R3 and above the canonical gate posture asks for independent approval, just-in-time access and an audit event (see the [risk tier crosswalk](../../agentic-board-protocol/references/risk-tier-crosswalk.md)); a design that cannot name the caller at the point of mutation cannot meet that posture. That is judgment, not a quotation from a framework.

## Confused deputy

A confused deputy is a component with its own authority that a less-privileged party induces to use that authority on the party's behalf. Agentic systems build it by default unless the design prevents it:

- The agent holds a broad service identity, and any user, or any text the agent reads, can cause calls the user is not entitled to make.
- An MCP server holds upstream credentials and makes calls for whichever client reaches it.
- An agent relays a user's token to a service for which the token was never issued.

For MCP-specific guidance on this class of problem, verify against the security best practices page for the revision in use, reached from https://modelcontextprotocol.io/specification/versioning; see the note under MCP authorization below.

Review checks:

- Does the downstream system authorize against the caller's entitlement, through a delegated credential issued for that caller, or only against the agent's identity?
- If only the agent's, who checks the caller's entitlement before each call, and is that check enforced outside the model? A prompt that says only do what the user may do is not enforcement.
- Can injected text change which resource, tenant or account a call targets while the identity stays the same? Pair this check with ASI01 and ASI03 in the [threat model](threat-model-owasp-asi.md).

## MCP authorization

The current MCP revision is 2026-07-28, per the [versioning page](https://modelcontextprotocol.io/specification/versioning): "The current protocol version is 2026-07-28". Older revisions are still deployed. Establish which revision each installed client and server implements and read that revision's authorization page; do not assume the lines below hold for another revision.

From the [2026-07-28 authorization page](https://modelcontextprotocol.io/specification/2026-07-28/basic/authorization), quoted exactly:

- "MCP clients MUST implement Resource Indicators for OAuth 2.0 as defined in RFC 8707"
- "MCP clients MUST send this parameter regardless of whether authorization servers support it."
- "MCP servers MUST validate that access tokens were issued specifically for them as the intended audience"
- "MCP clients MUST NOT send tokens to the MCP server other than ones issued by the MCP server's authorization server."
- "MCP servers MUST only accept tokens that are valid for use with their own resources."
- "MCP servers MUST NOT accept or transit any other tokens."

For MCP security guidance beyond the authorization page, verify against the security best practices page for the revision in use, reached from https://modelcontextprotocol.io/specification/versioning. This reference does not state what that page contains; read it for the installed revision before citing its wording.

What the lines mean for a review (judgment, not quotation):

- The resource indicator is how a client tells the authorization server which server a token is for, so the audience can be bound.
- Audience validation closes cross-service replay: a token issued for another resource is refused.
- The last two quoted lines concern tokens that are not valid for the server's own resources: a server accepts only tokens valid for its own resources and does not accept or transit any other tokens. They do not, on their own, state a rule about forwarding the token the server received.
- Forwarding the inbound token to a downstream API is the token-passthrough anti-pattern (judgment). A downstream API that validates audience refuses a token that was issued for the MCP server, which is why audience validation defeats it. Verify whether the revision in use states an explicit passthrough requirement, and in which words, on its authorization page; do not rely on this reference for that wording.
- A server needs its own credential for each downstream system, issued to the server and scoped to what the server's tools do.

| Check | Evidence | Finding if it fails |
|---|---|---|
| The client sends the resource parameter for each server | Sanitized client OAuth configuration or a request trace with values redacted | Tokens are not audience-bound |
| The server validates audience on every request | Server token-validation code or configuration | Any token from the same issuer unlocks the server |
| The server never forwards an inbound token downstream | The code path from the tool handler to the downstream client | Token passthrough |
| Downstream credentials are issued to the server and scoped | Identity provider registrations by name and scope, no secrets | Shared or over-broad downstream identity; confused deputy |
| Scopes are the minimum per tool | The scope list set against the tool list | Scope inflation |
| Token lifetime is short and refresh is handled safely | Configuration | Long-lived bearer credential |
| The caller reaches downstream audit | Sanitized log sample | Downstream audit shows only the server identity |

A mutation-capable server that fails the audience check or passes tokens through is a HARD-gate candidate; see the criteria in the [workflow and output reference](workflow-and-output.md).

For a server launched locally by the client, state how it obtains credentials (environment variable, file, prompt) and ask the same least-privilege and no-sharing questions. Verify what the revision in use says about authorization for that deployment style.

Tool-schema fidelity, protocol-version negotiation and error classification belong to `typescript-mcp-tool-contract-agent`, not to this review.

## A2A authentication, interrupted states and signed cards

The latest released A2A specification is 1.0.0, per the [specification page](https://a2a-protocol.org/latest/specification/). Verify against the version the peer implements.

Authentication. Per https://a2a-protocol.org/latest/specification/ the protocol delegates authentication to standard web security schemes that the Agent Card declares (API key, HTTP authentication, OAuth 2.0, OpenID Connect, mutual TLS), and the scope of authorization is implementation-defined. Per the same page the card declares its schemes and requirements in fields such as `securitySchemes` and `securityRequirements`; check the exact field names against the version in use. The review question is therefore not whether A2A authenticates. It is which scheme this deployment chose, who enforces scopes, and whether what the card promises matches what the server enforces.

AUTH_REQUIRED is not authorization. From the specification page:

> Agents MUST NOT treat the TASK_STATE_AUTH_REQUIRED state transition, by itself, as authorization for any particular operation.

A credential obtained while a task is in that state "MUST NOT be assumed to authorize subsequent messages on the Task unless that behavior is explicitly defined by the implementation, credential issuer, or extension". The interrupted states are INPUT_REQUIRED and AUTH_REQUIRED; the terminal states include FAILED, CANCELED and REJECTED, plus COMPLETED.

Review checks:

- Does any code path resume or execute after AUTH_REQUIRED on the strength of the state transition alone?
- Is a credential obtained in that state assumed to cover later messages on the task without a rule that says so?
- Hand the resumability and state-lifecycle aspects to `agentic-system-architecture-review-agent`; they are covered in its [delegation and handoff review](../../agentic-system-architecture-review/references/delegation-and-handoff-review.md).

Signed Agent Cards. From the specification page: "Agent Cards MAY be digitally signed using JSON Web Signature (JWS) as defined in RFC 7515", and the signed content "MUST be canonicalized using the JSON Canonicalization Scheme (JCS) as defined in RFC 8785". Signing is optional, so an unsigned card is not a specification violation; it is an unauthenticated claim, and any privileged use of what it says needs another trust anchor.

- A card is attacker-controlled text from a remote party until verified. Its skill descriptions reach a model the same way tool descriptions do; see [tools, memory and data reach](tools-memory-and-data-reach.md).
- When a card is signed, ask which key verifies it, how the deployer obtained that key, and what happens on rotation. A key delivered through the same channel as the card proves nothing unless it is pinned out of band.
- A valid signature shows that the card content was signed by the key holder. It does not show that the remote agent behaves as described or is entitled to the data you send it.

## Least privilege, lifetime and sharing

| Check | Evidence | Finding if it fails |
|---|---|---|
| Scopes and roles are the minimum for each tool and task | Scope list against the tool list and use case | Over-privileged identity |
| Credentials are short-lived or issued just in time | Issuer configuration, lifetimes | Long-lived static secret that outlives its task |
| No shared identity for R3 and above | Identity map | A shared or unidentified identity acts on a consequential change |
| Separate identities per agent and per environment | Registrations | A test agent can reach production, or agents are indistinguishable in logs |
| No human credential embedded in agent configuration | Sanitized configuration | A person's authority is borrowed by automation |
| A revocation path exists and is exercised | Runbook, drill record | A compromised agent keeps its access |

For MCP scope guidance, verify against the security best practices page for the revision in use, reached from https://modelcontextprotocol.io/specification/versioning.

## What the handoff envelope does not carry

The handoff envelope in [schemas/agentic-handoff.schema.json](../../../../schemas/agentic-handoff.schema.json) is a data contract that authorizes nothing, and it deliberately carries no credentials:

- The schema rejects unknown fields and has no field for a token, key or secret; its description states that no field may carry credentials, tokens or secrets. It cannot see a secret pasted into a free-text field. The sender redacts; a reviewer who finds one redacts and flags it and does not echo it.
- A message that carried a token would become a bearer artifact that spreads into logs, records and review transcripts. Because the envelope authorizes nothing, never treat a field such as approval_state as a credential or as proof of authority.
- Principals are opaque pseudonymous subjects of the form `subject:<issuer>:<hex>`, never names, emails, or shared identities. Records then carry no names or emails, and the mapping from subject to person stays with the organization's identity provider. That is pseudonymisation, not anonymisation: a subject the organization can map back to a person is still personal data under the GDPR (Recital 26), so the records need retention and data-protection handling. Self-approval and second-approver checks compare subjects, not roles. The [handoff envelope reference](../../agentic-board-protocol/references/handoff-envelope.md) explains the principal fields.
- Issuing individual subjects and authenticating the sender of a message are runtime duties (product B). A design that uses envelope-like messages as bearer capabilities, or lets an approved flag in a message act as authorization to a tool, is a finding.

## Typical findings

These are starting points. Whether a finding is a HARD gate depends on the criteria in the [workflow and output reference](workflow-and-output.md), not on this table.

| Finding | Typical severity | HARD-gate candidate |
|---|---|---|
| Token passthrough to a downstream API | blocking | Yes, when the server can mutate state or reach confidential or restricted data |
| No audience validation on a mutation-capable MCP server | blocking | Yes |
| Shared service identity for many users with no check of the caller's entitlement | blocking | Yes, when untrusted text reaches the agent and the identity can reach confidential or restricted data |
| AUTH_REQUIRED treated as authorization for a privileged operation | blocking | Yes |
| Caller identity lost at the tool boundary of a consequential action | condition, blocking at R3 and above | Only with a reachable exploit path |
| Long-lived static secret in agent configuration | condition | No, unless it is exposed to an attacker |
| Unsigned or unverified Agent Card used to route confidential data | condition | Only with a reachable exploit path |

## Evidence to request

Ask for sanitized evidence only: identity provider application registrations by name and scope, token claim samples with values redacted (audience, scope, subject shape, expiry), the server code path from handler to downstream client, and the redacted configuration. Never request a token, key or secret. Label each item with the envelope evidence vocabulary: configuration read from a repository is `repo-evidence`, a description supplied by the owner is `user-provided-sanitized-evidence`, and a statement about how a live system behaves is `live-evidence` only when the supplied material is a captured observation of that system.
