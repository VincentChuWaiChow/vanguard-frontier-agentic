# oci-nosql-pattern

Provider-correct Oracle NoSQL Database Cloud Service guidance, version **1.0.0**.
Start at [SKILL.md](SKILL.md). This is not Oracle AI Database/Autonomous SQL.
No installation, registration, live operation, background job or measured savings
is implied. Templates and local fake tests are not service/runtime acceptance.

## Triggers

Host-dependent agent requests; from this folder the real offline selector accepts
the same normal/slash-prefixed syntax and returns JSON `read` paths only.

| Intent | Trigger | Output / boundary |
|---|---|---|
| Help | `/oci-nosql-pattern help` | Commands/features only, then stop |
| Choose domains | `/oci-nosql-pattern select --features iam,encryption` | Selected guides, no OCI activation |
| Model access | `/oci-nosql-pattern design --features core,indexes,ttl` | Access matrix, shard choice, SQL/index alternatives, requested code/IaC |
| Change code/schema | `/oci-nosql-pattern refactor --features core,indexes` | Small compatible diff; backfill/cutover/rollback plan if needed |
| Offline audit | `/oci-nosql-pattern audit --mode offline --features iam,encryption,backup` | Sanitized/local evidence, no OCI inspection |
| Metadata audit | `/oci-nosql-pattern audit --mode live --features ttl,monitoring` | Authorized exact-target metadata only, no row scans/remediation |
| Performance plan | `/oci-nosql-pattern performance-test --environment nonproduction --duration-seconds 30 --max-requests 500 --max-concurrency 2` | Positive caps and further rate/cost/stop requirements; no run |
| Configure lifecycle | `/oci-nosql-pattern feature-plan --features ttl --action configure` | Proposed TTL change, not deployment |
| Security plan | `/oci-nosql-pattern feature-plan --features encryption,iam --action configure` | Managed/dedicated CMEK and rights prerequisites |
| Export plan | `/oci-nosql-pattern backup --method migrator --features backup,encryption` | Customer-managed JSON/Object Storage export, no snapshot/PITR promise |
| SDK export plan | `/oci-nosql-pattern backup --method sdk` | App-owned bounded page exporter contract, not a supplied backup service |
| Import plan | `/oci-nosql-pattern restore --method migrator` | Exact export/NEW table/schema/TTL approval and validation |
| SDK import plan | `/oci-nosql-pattern restore --method sdk` | App-owned conditional importer contract, no source overwrite |
| AI boundary | `/oci-nosql-pattern select --features vectors,embeddings,iam` | Numeric arrays/source provenance and external retrieval architecture |

Default features: `core`; backup/restore: `backup`. Default audit: offline; default
method: migrator. `--method pitr`, unknown features, empty bundles, unsafe action
combinations and nonpositive performance limits fail. Native ANN/vector enablement
is unsupported by this package: no verified NDCS native API in the reviewed docs.

```bash
python3 scripts/select_mode.py /oci-nosql-pattern audit --features iam,encryption
python3 scripts/review_due.py --as-of 2026-11-08 # replace with actual client date
```

## Feature combinations

Only named paths load; indirect `embeddings`/`backup` routes do not load sibling
methods or live audit instructions. Unselected scope is not implicitly healthy.

| Bundle | Features | Tradeoff |
|---|---|---|
| Query/correctness | `core,indexes,ttl` | Shard locality, continuation, row expiration |
| Security | `iam,encryption` | Compartment/table access and dedicated CMEK prerequisites |
| Capacity | `core,indexes,monitoring` | Hot shards, consumed RU/WU and throttles |
| Recovery | `backup,ttl,encryption` | Export integrity, original expiry and key retention |
| Regional | `replicas,monitoring` | Replication lag/conflicts, not historical recovery |
| Semantic retrieval | `vectors,embeddings,iam` | App generation plus separately approved vector service |

## Manual workflow combinations

One command at a time, not `combo` or an automatic pipeline. Every transition
retains its own authorization; selecting a plan does not apply it.

| Goal | Manual sequence | Gate |
|---|---|---|
| New model | design → offline audit | Review actual DDL/code; approve deploy separately |
| Safe refactor | offline audit → refactor → performance-test | Data compatibility, rollback and separately bounded approved test |
| Recovery drill | backup migrator → restore migrator → live audit | Export evidence/NEW target approval; application validation before cutover |
| CMEK review | offline audit → feature-plan configure → live audit | Dedicated environment/service-owner plan; no fake table `kms_key_id` |
| Retrieval | select vectors/embeddings → design → performance-test | Separate vector store IAM/model/dimension/tenant and recall ground truth |

## Included artifacts and verification

[Provisioning](references/provisioning.md) selects matching [DDL](assets/orders.ddl),
[Terraform](assets/orders.tf) and guarded [OCI CLI](assets/create-table.sh). Either
IaC or CLI owns a NEW table, never both. Customer recovery uses reviewed Migrator
JSON templates and an intentionally incomplete manifest that fails the offline
[safety checker](scripts/check_recovery.py). No export/import is executed here.
[Adapters](references/language-adapters.md) cover JS/TS, Java, Rust, Kotlin via
Java, PHP/other REST integration plus Python/Go/.NET. Only the dependency-free
injected JS runtime helper is executed by local tests; SDK snippets are uncompiled.

Review every June and December; next **2026-12-01**. [Maintenance](references/maintenance.md)
is read-only on load, overdue stays overdue, and partial edits are not full audits.
See [sources/evidence](references/sources.md), [evals](evals/eval.md) and
[adversarial prompts](evals/prompts.json). No all-language/runtime-proof claim.
