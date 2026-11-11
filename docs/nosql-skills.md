# NoSQL Skills

> **Status:** Provider-specific NoSQL skills and companion agents, designed in
> [ADR-0004](adr/0004-provider-specific-nosql-skills-with-evidence-ledgers/)
> (status: Proposed; the owner decides acceptance). Every material claim cites a row
> in the skill's evidence ledger. No guidance here has been tested against a live
> cloud account.

---

## What is covered

Each managed NoSQL service has one skill that owns it. Skills load their references
lazily: `SKILL.md` names the reference files, and an agent reads only the one the
task needs.

| Service | Skill | Companion agent |
| --- | --- | --- |
| Amazon DynamoDB | [`aws-dynamodb-patterns`][ddb-p] (build and operate), [`aws-dynamodb-data-modeling-performance-review`][ddb-r] (review only) | `aws-dynamodb-patterns-agent` |
| Amazon DocumentDB | [`aws-documentdb-architect`][docdb] | none |
| Amazon Keyspaces | [`aws-keyspaces-architect`][ks] | none |
| Azure Cosmos DB for NoSQL, for MongoDB (RU) and for Apache Cassandra | [`azure-cosmosdb-platform-operator`][cosmos-op]; [`azure-cosmosdb-pattern`][cosmos-p] (build and operate, NoSQL API) | `azure-cosmosdb-application-developer-agent`, `azure-cosmosdb-pattern-agent` |
| Oracle NoSQL Database Cloud Service | [`oci-nosql-database-architect`][ndcs]; [`oci-nosql-pattern`][ndcs-p] (build and operate) | `oci-nosql-database-architect-agent`, `oci-nosql-pattern-agent` |
| Firestore (Native mode) and Firestore with MongoDB compatibility | [`gcp-firebase-developer`][fs] | `gcp-firebase-developer-agent` |
| Cloud Bigtable | [`gcp-bigtable-architect`][bt] | `gcp-bigtable-architect-agent` |
| Alibaba Cloud Tablestore | [`alibaba-tablestore-architect`][ts] | `alibaba-tablestore-architect-agent` |
| Huawei Cloud GeminiDB (Cassandra API) | [`huawei-geminidb-architect`][gdb] | `huawei-geminidb-architect-agent` |
| Huawei Cloud DDS | [`huawei-gaussdb-rds-dba`][dds] | `huawei-gaussdb-rds-dba-agent` |
| Scaleway Managed MongoDB | [`scaleway-managed-mongodb-architect`][scw] | none |
| OVHcloud Public Cloud Databases for MongoDB | [`ovhcloud-mongodb-architect`][ovh] | none |
| IONOS Cloud DBaaS MongoDB | [`ionos-live-database-lifecycle-guard`][ionos] | `ionos-live-database-lifecycle-guard-agent` |

Hetzner and Contabo offer no managed NoSQL service; ADR-0004's amendment records the
sources. Redis and Valkey in-memory stores are out of scope.

The three **pattern skills** are copied from the private `agentic-vantage` repository,
which stays their source of truth. Each records its provenance and every local
difference in `references/vanguard-port.md`. Their agents are static reviewers: they
propose changes and never run them.

## How evidence works

Each skill has a `references/evidence-ledger.md`. A row records the claim, the
official source URL, the section with a verbatim quote, the retrieval date and a
status:

- **VERIFIED**: the quote was re-checked on the cited page;
- **UNVERIFIED**: the page could not be confirmed (dynamic pages, bot protection), so
  the claim is hedged, never asserted;
- **CONFLICT**: official pages disagree, and both sides are recorded instead of
  picking one.

Only provider-owned hosts are accepted as sources. `validate:skill-references`
enforces the host allowlist and that every cited ID exists.

## Operations references

Every service has an operations reference with the same ten sections, in this order:
backup and restore, disaster recovery and replication, cost, monitoring, performance,
IAM, tagging, encryption, network isolation, and audit logging. Each section carries
one status (VERIFIED, PARTIAL, UNSUPPORTED, UNVERIFIED or NOT
APPLICABLE) and the citations behind it.
Cost lists billing dimensions only; no prices are recorded anywhere.
`templates/nosql-operations-template.md` defines the shape.

## Audits

Every NoSQL skill has a lazy-loaded audit in three files:

- `references/commands/audit.md`, a router linked from `SKILL.md`, which loads
  exactly one variant;
- `audit/offline.md`, which checks the repository and sanitized exports with local
  tools only (`rg`, `jq`, `find`, `git`, `terraform show` on a saved plan);
- `audit/live.md`, which holds read-only control-plane commands for an operator with
  read-only authorization. The skill never runs them.

Live setup blocks run under `set -euo pipefail` and stop when the active account,
subscription, project, tenancy, organization or Region differs from the approved
one. Credential searches list file names only. A section with no documented read
command says `**No live command:**` and gives the reason; Huawei GeminiDB and DDS are
the main gap, because no official page names their KooCLI operations.

`validate:nosql-audit` checks every fenced block with a small shell lexer against a
per-mode allowlist: read verbs only for cloud CLIs, no secret-reading or data-plane
calls, GET-only `curl`, no data shells such as `mongosh`, and nothing but local
tools offline. It is a model of bash, not a sandbox: a shell construct it does not
model could still pass, so treat the live commands as reviewed examples, not as
proof of safety. `templates/nosql-audit-template.md` lists the rules.

## Gates

| Gate | What it checks |
| --- | --- |
| `validate:skill-references` | links resolve, sources are on allowlisted official hosts, ledgers are traceable |
| `validate:nosql-scenarios` | behavioural scenarios are answerable from the exact reference file each skill loads |
| `validate:nosql-pattern-evals` | the pattern skills' offline evals pass (fake cloud binaries; `cfn-lint` pinned in `tests/requirements-ci.txt`) |
| `validate:nosql-operations` | the ten sections, one status each, valid citations, no prices |
| `validate:nosql-audit` | lazy-load links, section order, command allowlist, VERIFIED citations for live commands |

All of them run inside `npm run validate`; see the
[configuration reference](configuration/) for the full gate list.

## What is not verified

- No guidance was tested against a live account, and no SDK code was compiled. Evals
  use fake cloud binaries.
- No audit command was run against a cloud. Live commands were checked against their
  reference pages, not executed.
- The gates are structural. They cannot show that a page supports a VERIFIED claim;
  the quote checks were a separate one-off pass.
- PARTIAL sections spell out their gaps, such as missing RPO or RTO figures or no
  documented switch to disable public endpoints.

[ddb-p]: https://github.com/VincentChuWaiChow/vanguard-frontier-agentic/tree/master/skills/aws/aws-dynamodb-patterns
[ddb-r]: https://github.com/VincentChuWaiChow/vanguard-frontier-agentic/tree/master/skills/aws/aws-dynamodb-data-modeling-performance-review
[docdb]: https://github.com/VincentChuWaiChow/vanguard-frontier-agentic/tree/master/skills/aws/aws-documentdb-architect
[ks]: https://github.com/VincentChuWaiChow/vanguard-frontier-agentic/tree/master/skills/aws/aws-keyspaces-architect
[cosmos-op]: https://github.com/VincentChuWaiChow/vanguard-frontier-agentic/tree/master/skills/azure/azure-cosmosdb-platform-operator
[cosmos-p]: https://github.com/VincentChuWaiChow/vanguard-frontier-agentic/tree/master/skills/azure/azure-cosmosdb-pattern
[ndcs]: https://github.com/VincentChuWaiChow/vanguard-frontier-agentic/tree/master/skills/oci/oci-nosql-database-architect
[ndcs-p]: https://github.com/VincentChuWaiChow/vanguard-frontier-agentic/tree/master/skills/oci/oci-nosql-pattern
[fs]: https://github.com/VincentChuWaiChow/vanguard-frontier-agentic/tree/master/skills/gcp/gcp-firebase-developer
[bt]: https://github.com/VincentChuWaiChow/vanguard-frontier-agentic/tree/master/skills/gcp/gcp-bigtable-architect
[ts]: https://github.com/VincentChuWaiChow/vanguard-frontier-agentic/tree/master/skills/alibaba/alibaba-tablestore-architect
[gdb]: https://github.com/VincentChuWaiChow/vanguard-frontier-agentic/tree/master/skills/huawei/huawei-geminidb-architect
[dds]: https://github.com/VincentChuWaiChow/vanguard-frontier-agentic/tree/master/skills/huawei/huawei-gaussdb-rds-dba
[scw]: https://github.com/VincentChuWaiChow/vanguard-frontier-agentic/tree/master/skills/scaleway/scaleway-managed-mongodb-architect
[ovh]: https://github.com/VincentChuWaiChow/vanguard-frontier-agentic/tree/master/skills/ovhcloud/ovhcloud-mongodb-architect
[ionos]: https://github.com/VincentChuWaiChow/vanguard-frontier-agentic/tree/master/skills/ionos/ionos-live-database-lifecycle-guard
