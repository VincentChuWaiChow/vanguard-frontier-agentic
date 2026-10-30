# Supply chain and provenance

Reference for [agentic-security-review](../SKILL.md). Load it when you review where an agent's tools, MCP servers, skills, prompts and models come from, how they are pinned and verified, how changes are noticed, and how a component is switched off.

This is consumer-side provenance: what an adopter should require of the components it loads. The OWASP item for it is ASI04 Agentic Supply Chain Vulnerabilities (see the [threat model](threat-model-owasp-asi.md)). No repository rule operates on an adopter's components, so ASI04 is runtime-only (product B) there. This repository contributes a review method and one existing pattern, the MCP trust matrix, described below.

## What counts as a component

| Class | Examples | What can go wrong |
|---|---|---|
| MCP server | A server the agent connects to | It changes behavior or tools after approval, holds credentials, or reaches egress |
| Tool | A function or API the agent can call, with its name, description and schema | The description or schema changes and steers the model; the implementation gains capability |
| Skill | A SKILL.md and its references loaded into an agent | Its text instructs the agent to call tools, and it is read as trusted guidance |
| Prompt and template | System prompts, templates, rules files | An edit changes behavior with no code review |
| Model | A hosted model version or downloaded weights | A version alias moves, or weights come from an unverified source |
| Peer agent | A remote agent reached by A2A or similar | Its Agent Card or behavior changes; see [identity and delegated authorization](identity-and-delegated-authorization.md) |
| Retrieval corpus | A document store the agent reads | Writers outside your control add text the model will read; see [tools, memory and data reach](tools-memory-and-data-reach.md) |

## The agent component inventory

Keep one inventory of every component an agent can load or call. This skill calls it an agent component inventory and names no specific bill-of-materials standard. An adopter can map the fields onto whichever format its organization already uses.

| Field | Why it is there |
|---|---|
| Component id and class | Unambiguous reference in findings |
| Publisher and source location | Who can change it, and where it is fetched from |
| Version, and the pin used (digest, version or tag) | What exactly was approved |
| Signature status and the identity that signed | Whether the publisher's claim was verified |
| Tool surface: tool names, a hash of each description and schema | Notices a post-approval change that no version bump announces (judgment) |
| Privileges: mutation capable, needs egress, needs credentials | Reach if the component is hijacked |
| Owner, reviewed-on date, next review | Someone answers when it drifts |
| Update channel and revocation contact | How a change arrives and how it is stopped |

An agent that can load components not in the inventory is a finding in itself: nothing was reviewed.

## Pinning

The repository's own MCP reference schema defines the pin vocabulary (schemas/mcp-reference.schema.json, field pin_strategy):

- digest: pin by image or commit digest.
- version: pin by published version string.
- tag: pin by version tag, which is mutable.
- none: no pinning advised by upstream.

Prefer a digest where the publisher offers one; a tag is mutable and can be moved to different content. A version string is only as stable as the registry's rules about republishing it, so verify that policy for the registry in use. When upstream advises none, the adopter's compensating controls are a mirror it controls, a review of the diff on every update, and least privilege with no credentials.

Pin the tool surface as well as the code (judgment). A tool's description and schema can change without a version bump you notice. Record a hash of each tool's name, description and input schema when the component is approved, and alert when it changes.

## Signature verification

The same schema describes how a release is signed (field signed_release): cosign means signed with Sigstore cosign, gh-attestation means a GitHub artifact attestation, unsigned means no verifiable signature, and unknown means the maintainers of this repository did not verify it.

- Verify against an expected signing identity. A signature that exists but was made by an identity you did not expect proves little.
- Treat unsigned and unknown as unverified, not as safe. The references under mcp/official currently declare signed_release unknown (check mcp/official/*.metadata.json); that records that nobody verified it.
- Where a component ships as a container image admitted to a Kubernetes cluster, the existing [Sigstore Cosign supply chain review skill](../../../sigstore/sigstore-cosign-supply-chain-review/SKILL.md) and `sigstore-cosign-supply-chain-review-agent` cover signing, admission policy and attestations. Their scope is Kubernetes workloads; for any other artifact type, state that no repository coverage exists.
- For NVIDIA NGC and NIM models and containers, use `nvidia-ngc-nim-supply-chain-governor-agent`.

## The MCP trust matrix as a pattern

This repository records a structured trust posture for every MCP reference it lists. The fields, verified in schemas/mcp-reference.schema.json, are:

| Field | Values | Review use |
|---|---|---|
| mutation_capable | true or false | Can any exposed tool change state, send messages, or write files |
| requires_egress | true or false | Does the server need outbound network access |
| requires_credentials | true or false | Does it need keys, tokens or cloud authentication |
| signed_release | cosign, gh-attestation, unsigned, unknown | Release signing posture |
| pin_strategy | digest, tag, version, none | How a consumer should pin it |

A server that is mutation capable, needs egress and holds credentials has the widest reach if hijacked. It deserves the strictest pinning, the narrowest credential scope, and a place in the effective data reach table.

The pattern, for an adopter: record the same five fields for every MCP server and tool an agent can reach, in the component inventory above. The repository enforces the pattern for its own references: tests/validate-mcp-trust-matrix.py requires the block on every MCP metadata file under mcp/, checks the allowed values, and requires catalog/mcp-references.json to carry the same matrix as the source file. The gate validates the shape and parity of a declared posture. It does not verify that a declaration is true.

## Skills, prompts and install paths

A skill is text an agent follows. Review a third-party skill like a dependency: read it, pin it by commit or release, and record it in the inventory. A SKILL.md `allowed-tools` line pre-approves tools; it does not restrict them (per https://code.claude.com/docs/en/skills), so it is not a boundary against a skill that asks for more.

How content is installed changes its trust. docs/integrations/skills-cli.md compares this repository's install paths: a registry package that a lockfile can pin, and a path that fetches the latest default-branch commit with no integrity check at install time. Apply the same distinction to any skill or prompt source.

## Models

Record the provider, the exact model version, and whether the provider lets you pin one. Where a hosted alias can move to a different model, the behavior of the agent can change with no change on your side. Model names, version policies and retirement dates come from the provider's official documentation; verify against the provider's official page for the version in use and do not copy them from memory or press coverage. For downloaded weights, record the source and checksum and verify them as for any artifact.

## Update and drift review

Define what triggers a review, who approves, and what is compared:

- a version or digest change,
- a change to the tool list, a tool description or a schema,
- a new permission, scope or egress destination,
- a change of publisher, repository owner or signing identity,
- a change in the component's own dependencies.

Compare the tool surface before and after, not only the version number. Stage the update where the agent cannot reach production data first. Set a named review cadence for each pin so that an old pin does not become an unreviewed one.

## Revocation

- Name the owner who can disable a component, and the path: remove it from the inventory and the agent's tool list, revoke the credentials issued to it, and purge caches it filled.
- If the component wrote to memory, find and quarantine those entries by provenance; see [tools, memory and data reach](tools-memory-and-data-reach.md).
- Keep a record of what the component touched while it was loaded, so an incident review can scope the impact.
- Exercise the path. A revocation that has never been tried is a claim. Plan a drill and record the result as sanitized evidence. The kill switch and disengage design belongs to the architecture review; see [reliability and containment](../../agentic-system-architecture-review/references/reliability-and-containment.md).

## Candidate alignment

NIST AI RMF GOVERN 6.1 and 6.2 concern third parties, and MAP 4.1 and 4.2 concern the risks and controls of third-party components (see the [AI RMF core](https://airc.nist.gov/airmf-resources/airmf/5-sec-core/); the page carries a banner that AI RMF 1.0 is being updated). Following this reference can support an owner's work against those subcategories. It does not satisfy them, and an accountable owner confirms any mapping.

## What this repository does not do

- It does not fetch, scan or verify any component of an adopter's system.
- It does not check that a trust matrix declaration is accurate, or that a signature verifies.
- It does not provide a runtime allowlist of tools or servers. The inventory is a review artifact; enforcement is product B.
