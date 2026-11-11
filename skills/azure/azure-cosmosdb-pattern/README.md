# azure-cosmosdb-pattern

Provider-native **Azure Cosmos DB for NoSQL** guidance, version **1.0.0**.
Entry: [SKILL.md](SKILL.md). Not a Cosmos MongoDB, vCore, PostgreSQL, Cassandra,
Table, Gremlin or Fabric compatibility layer. No cloud operation runs on load.

## Human triggers

These are agent requests, not newly installed host commands. Select one command
and only its feature/method guides. [Help](references/commands/help.md) is a stop
route, not an invitation to inspect your environment.

| I want to… | Request to type | Outcome / gate |
|---|---|---|
| Discover scope | `/azure-cosmosdb-pattern help` | Commands/features only |
| Focus on a bundle | `/azure-cosmosdb-pattern select --features iam,encryption` | Focused explanation, no activation |
| Design a model | `/azure-cosmosdb-pattern design --features core,indexes,capacity` | Access matrix, partition/id, costs/alternatives |
| Refactor an app | `/azure-cosmosdb-pattern refactor --features core,ttl` | Compatibility/ETag/migration diff, no reset |
| Review local code/IaC | `/azure-cosmosdb-pattern audit --features iam,encryption,backup` | Offline by default; sanitized local evidence |
| Inspect authorized metadata | `/azure-cosmosdb-pattern audit --mode live --features capacity,monitoring` | Target-confirmed read-only plan; approval still required |
| Protect continuously | `/azure-cosmosdb-pattern backup --method continuous` | Observed tier/window and change plan, no enablement |
| Review periodic policy | `/azure-cosmosdb-pattern backup --method periodic` | Interval/retention/redundancy and support readiness |
| Recover before corruption | `/azure-cosmosdb-pattern restore --method continuous-new-account` | Source instance/time/region, **new account**, validation |
| Recover deleted resources | `/azure-cosmosdb-pattern restore --method continuous-deleted-resource` | Deleted DB/container only, absent target name; no overwrite |
| Request periodic recovery | `/azure-cosmosdb-pattern restore --method periodic-support` | Azure Support case plan; no fictional self-service backup CLI |
| Plan bounded testing | `/azure-cosmosdb-pattern performance-test --features core,vectors --environment nonproduction --duration-seconds 30 --max-requests 500 --max-concurrency 2 --max-ru 1000` | Plan only; exact target/run/cost/stop approval required |
| Plan feature lifecycle | `/azure-cosmosdb-pattern feature-plan --features encryption,iam --action configure` | Reviewed desired state, not application |
| Plan TTL disable | `/azure-cosmosdb-pattern feature-plan --features ttl --action disable` | Retention/deletion implications; no mutation |

The real stdlib selector accepts identical syntax from this skill directory:

```bash
python3 scripts/select_mode.py /azure-cosmosdb-pattern audit --mode offline --features iam,encryption
```

It emits `execution: none` and local paths, reads no references and performs no
network/subprocess/writes. Selection is neither authorization nor an audit result.
Recovery defaults to continuous protection/new-account recovery; named/on-demand
snapshots and AWS-shaped `pitr`/`on-demand` methods are rejected, not translated.

## Feature bundles

| Focus | `--features` value | Native concern |
|---|---|---|
| Model/read path | `core,indexes,capacity` | Partition routing, index cost and hot keys |
| Security | `iam,encryption` | Entra control/data roles, account key/Key Vault boundaries |
| Business continuity | `backup,replication,ttl` | Backup vs replication, recoverability and expiry |
| Change processing | `change-feed,monitoring` | Lease/checkpoint, retry/deletion semantics and lag |
| Semantic search | `vectors,embeddings,iam` | Real vector policy, trusted tenant and model/source mapping |
| Deployable examples | `provisioning,iam,capacity` | ARM/CLI equivalence and least-privilege prerequisites |

Unselected features are not implicitly healthy. A bundle is one invocation's
scope, not persistent configuration. Generic feature toggles fail when native
lifecycle doesn't support the requested action (for example disabling backup,
encryption, or an enabled vector search container).

## Manual combinations — never an automatic pipeline

| Goal | Manual sequence | Gate between steps |
|---|---|---|
| New model | design → audit offline | Inspect actual generated model/IaC; no creation |
| Safe refactor | audit offline → refactor → performance-test | Separate compatibility/backfill and bounded-run approvals |
| Recovery drill | backup continuous → restore continuous-new-account → audit live | New target/time, restore approval, application/RPO/RTO proof before cutover |
| Deleted resource recovery | audit live → restore continuous-deleted-resource | Verify deletion event, timestamp and absent target; no live overwrite |
| CMK lifecycle | audit offline → feature-plan configure → audit live | Keys/identities preserved, approved delta and authorized metadata readback |
| Semantic retrieval | select vectors,embeddings,iam → design → performance-test | Model/version/dimensions and tenant/ground truth fixed first |

## Included, not automatically loaded

- [Provisioning](references/provisioning.md): complete ARM JSON and equivalent
  gated az CLI example, private-by-default networking, Session consistency,
  `/tenantId`, `id`, dedicated 400 RU/s container, Continuous7Days, service-managed
  encryption and no key-based auth. Existing private connectivity/data role setup
  is intentionally separate, not silently opened.
- [Language adapters](references/language-adapters.md): JS/TS, Java, Rust preview,
  PHP REST, Kotlin/JVM, Python/.NET/Go and a portable behavior contract. No claim
  of universal SDK parity/compilation.
- [Runtime](references/runtime-contract.md): ETag/412, same-logical-partition
  transactional batch, continuation/session tokens, bounded 429 handling and TTL.
- [Types/vectors](references/data-types-and-vectors.md), [embeddings](references/embedding-integrations.md):
  native vector policies, embedding generation and pinned Blob source references;
  no claim Cosmos itself generates an embedding from a source URI.
- [Recovery](references/recovery.md): distinct continuous/periodic/deleted-resource
  procedures with permissions, keys, async state and restored-settings checks.
- [Sources](references/sources.md): actual Context7, MicrosoftDocs and Azure
  documentation MCP retrieval dated 2026-11-08, including observed source drift.

## June / December content reminder

Next full review: **2026-12-01**. [Maintenance](references/maintenance.md) and the
read-only helper require an explicit client-local date; overdue remains overdue.
Targeted edits never constitute a full audit. No calendar, cron, memory or external
notification is configured.

```bash
python3 scripts/review_due.py --as-of 2026-11-08
```

See [eval contract](evals/eval.md), [adversarial prompts](evals/prompts.json) and
[changelog](CHANGELOG.md). Tests run only locally with injected clients/fake CLI;
SDK compilation, Azure acceptance, runtime security/performance and restore/cutover
remain unverified. Installation/host discovery is separate from repository creation.
