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

## Amendment: operational coverage for every NoSQL service

The first version covered data modeling, consistency and capacity, but not the
operational dimensions the original requirement listed. This amendment adds them.

- **One operations reference per service**, with the same ten sections in the same
  order: backup and restore, disaster recovery and replication, cost, monitoring,
  performance, IAM, tagging, encryption, network isolation and audit logging. The
  shape is `templates/nosql-operations-template.md`.
- **Every section states a status** (VERIFIED, PARTIAL, UNSUPPORTED, UNVERIFIED or
  NOT APPLICABLE) and cites ledger rows. UNSUPPORTED needs an explicit provider
  statement; absence from documentation is UNVERIFIED.
- **Cost records billing dimensions only.** Unit prices and currency amounts are
  rejected, because they vary by Region and change over time.
- **Coverage**: DynamoDB, DocumentDB, Keyspaces, Cosmos DB for NoSQL, for MongoDB and
  for Apache Cassandra, Oracle NoSQL Database Cloud Service, Firestore in Native mode
  and with MongoDB compatibility, Bigtable, Tablestore, GeminiDB and DDS.
- **`tests/validate-nosql-operations.py`** enforces the sections, statuses,
  citations, ledger status of cited rows and the no-prices rule.
- **DynamoDB build and operate skill.** `aws-dynamodb-patterns` is ported from the
  canonical copy in agentic-vantage, with its own ledger. It complements the
  review-only DynamoDB skill.

### Smaller clouds and further pattern ports

- **Managed MongoDB on the smaller clouds** gets the same ten-section operations
  reference: Scaleway Managed MongoDB®, OVHcloud Public Cloud Databases for MongoDB and
  IONOS Cloud DBaaS MongoDB. Scaleway and OVHcloud get minimal new skills, because they
  had no database skill. IONOS uses its existing database lifecycle skill.
- **Hetzner and Contabo offer no managed NoSQL service**, so neither has an operations
  reference.
  - Hetzner's managed servers provide MySQL/MariaDB, PostgreSQL and Redis, and MongoDB is
    self-installed on bare metal
    (<https://docs.hetzner.com/managed/managed-server/managed-vs-bare-metal-server/>).
  - Contabo describes its database hosting as self-managed
    (<https://contabo.com/en/database-hosting/>).
  - Revisit if either launches a managed NoSQL product.
- **Redis/Valkey in-memory stores are out of scope.** They are caches, not the document,
  key-value or wide-column databases this ADR covers.
- **`oci-nosql-pattern` and `azure-cosmosdb-pattern`** are ported from agentic-vantage,
  which stays canonical. Each has its own ledger, port record and companion agent.
  Maestro routing sends build-or-operate requests to them, and review stays with the
  existing specialists. `validate:nosql-pattern-evals` runs all three pattern skills'
  evals.

DocumentDB and Keyspaces get minimal new skills that hold only the operations
reference. A shared AWS skill would have mixed three data models, and an extension
of a DynamoDB skill would have broken the "do not combine services" rule.

## Services evaluated and not built

These candidates were considered and deliberately left out. Each would add
maintenance cost without a verified requirement, and none is described as
supported by this work.

| Candidate | Reason it is deferred |
| --- | --- |
| Data-modeling depth for Amazon DocumentDB and Amazon Keyspaces | Operations are covered by their own minimal skills. Schema, index and query design are not. |
| Cosmos DB API for Gremlin and Table; design depth for the MongoDB and Cassandra APIs | Different consistency mapping and feature sets. The MongoDB and Cassandra APIs have operations references only. |
| Firestore in Datastore mode; design depth for Firestore with MongoDB compatibility | The Firestore reference covers Native mode. MongoDB compatibility has an operations reference only; Datastore mode is defined but not covered. |
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
