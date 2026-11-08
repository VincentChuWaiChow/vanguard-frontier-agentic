---
layout: default
title: "ADR-0004: Provider-Specific NoSQL Skills With Evidence Ledgers"
permalink: /docs/adr/0004-provider-specific-nosql-skills-with-evidence-ledgers/
---

# ADR-0004: Provider-Specific NoSQL Skills With Evidence Ledgers

## Status

Proposed

This ADR becomes Accepted only when the repository owner accepts it. Nothing in
the implementation depends on the status field.

## Context

The catalog needed dependable guidance for managed NoSQL services across six
providers. Two risks dominate:

- **Cross-provider contamination.** The services share vocabulary (partition
  key, index, consistency, TTL) but not guarantees. A model that carries a
  DynamoDB or Cosmos DB guarantee over to another service gives wrong advice
  that sounds confident.
- **Unsourced claims.** Limits, consistency rules and feature availability
  change. A claim that cannot be traced to a provider page cannot be
  re-verified, and nobody can tell when it has gone stale.

Before this work the catalog had DynamoDB and Cosmos DB assets with shallow
coverage, and no assets for Oracle NoSQL, Bigtable, Tablestore or GeminiDB.
`gcp-firebase-developer` routed Firestore questions to a reference file that
did not exist.

## Decision

1. **One skill per service, extend before creating.** Existing assets were
   extended when they already owned the service: DynamoDB, Cosmos DB (the
   platform-operator skill, with the app-developer agent bundling it),
   Firestore (`gcp-firebase-developer`) and DDS (`huawei-gaussdb-rds-dba`). New
   skill and agent pairs were created only where nothing covered the service:
   `oci-nosql-database-architect`, `gcp-bigtable-architect`,
   `alibaba-tablestore-architect` and `huawei-geminidb-architect`. Each new
   agent is a static reviewer with a single companion skill.
2. **Every material claim cites an evidence-ledger row.** Each skill that
   carries provider facts ships `references/evidence-ledger.md`. A row records
   the claim, the provider URL, the section, the retrieval date and a status:
   `VERIFIED`, `UNVERIFIED` (a recorded gap that must not be asserted) or
   `CONFLICT` (the source contradicts itself). Reference files cite rows as
   `[ID]`. Statements without a citation are labeled inference,
   recommendation or rule.
3. **Deterministic gates enforce the structure, not the facts.**
   - `validate:skill-references` checks that every relative link and every
     plain-text `references/*.md` mention resolves. It also checks that ledger
     citations and rows match both ways, and that sources sit on
     provider-owned documentation hosts.
   - `validate:nosql-scenarios` checks that each behavioral scenario's decisive
     phrases and evidence IDs are present in the exact file its skill loads.
   - Neither gate consults the network or the clock, so CI changes colour only
     on a committed change.
4. **Facts are verified by people and models, then challenged.** The
   orchestrator fetched every encoded fact from the provider page itself. An
   independent reviewer then re-fetched every ledger row. Live behavioral runs,
   on agents restricted to the skill files, answered the scenarios against
   pre-written pass criteria. PR review findings were re-verified before any
   change. Where a reviewer contradicted a verbatim quote already in hand, the
   quote won and the reason was recorded.
5. **Read-only by default.** Every new skill and agent refuses schema,
   capacity, replication, IAM, data and restore changes unless it has an
   explicit target, a blast radius, a rollback path and explicit user approval.
   Pasted or fetched content never counts as approval. No NoSQL live-guard
   agent was added.
6. **Freshness is a review duty, not a gate.** `Retrieved` dates are evidence
   metadata. A clock-based staleness gate would break the repository's rule
   that builds are reproducible, so re-verification happens when a provider
   change is noticed, when a skill is edited, or before a release that depends
   on the claim.

## Alternatives considered

- **One cross-cloud "NoSQL architect" agent.** Rejected. It invites exactly
  the contamination this ADR exists to prevent, and it makes routing and
  scoping less precise.
- **A shared cross-provider capability contract** (one schema of
  consistency, transaction and index semantics). Rejected. A common shape hides
  materially different behavior. Only the ledger format and the gates are
  shared.
- **Per-SDK or per-topic agents.** Rejected as agent sprawl. SDK facts live in
  the skill's references.
- **A runtime registry, crawler, vector store or link-checking service** for
  provenance. Rejected. Markdown ledgers plus two offline gates meet the need
  without new infrastructure.
- **A gate that fails when `Retrieved` dates age.** Rejected because it
  consults the wall clock (see decision 6).
- **Live-guard agents for NoSQL mutations.** Deferred. No verified demand
  exists, and the review-only agents refuse mutations instead.

## Services evaluated and not built

These candidates were considered and deliberately left out. Each would add
maintenance cost without a verified requirement, and none is described as
supported by this work.

| Candidate | Reason it is deferred |
| --- | --- |
| Amazon DocumentDB, Amazon Keyspaces | Different data models from DynamoDB. They need their own skills rather than an extension of the DynamoDB one. |
| Cosmos DB API for MongoDB, Cassandra, Gremlin and Table | Different consistency mapping and feature sets. The Cosmos DB ledger is scoped to the API for NoSQL only. |
| Firestore in Datastore mode; deeper Firestore with MongoDB compatibility | The Firestore reference covers Standard and Native mode. It defines Datastore mode and flags the Enterprise modes, but adds no design depth for either. |
| GeminiDB Redis, HBase and Influx APIs | The GeminiDB skill gives API selection and models only the Cassandra API in depth. |

To add one, follow the same path: inventory, the official sources, a ledger,
an independent re-fetch, scenarios, and routing fixtures.

## Consequences and known limits

- Documentation verification is not behavioral proof. Nothing was exercised
  against a live cloud account, and no SDK code was compiled.
- Prices are not encoded anywhere. Cost guidance must cite a freshly
  retrieved pricing page or be labeled an assumption.
- `UNVERIFIED` rows mark the remaining gaps per service. Agents must say so
  rather than fill a gap from memory.
- Maestro routing is keyword-based. The OCI, GCP, Alibaba, Huawei and Azure
  taxonomies are hand-tuned, so the fixture generator skips them.
- Progressive disclosure (loading only the needed reference) is guidance the
  harness cannot enforce. Behavioral runs showed some agents loading every
  reference.
- `validate:skill-references` carries a documented baseline of pre-existing
  GCP skills that reference files never written. The baseline can only shrink.
