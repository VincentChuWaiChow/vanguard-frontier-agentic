# Primary sources and lookup evidence — 2026-11-08

Facts below were checked through official Oracle documentation/Oracle-owned SDK
references. Search availability is not proof of universal feature absence. These
URLs are re-checkable, not a claim of live service acceptance in every realm.

| Topic | Primary source | Evidence / boundary |
|---|---|---|
| Service scope, managed/customer responsibility | [Features/responsibility](https://docs.oracle.com/en/cloud/paas/nosql-cloud/dtddt/features-of-oracle-nosql-database-cloud-service.html) | Customer backup/restore via Migrator; no customer PITR workflow established |
| Native features/hosted eligibility | [NDCS features](https://docs.oracle.com/en/cloud/paas/nosql-cloud/mmmmq/) | No verified native ANN in reviewed list; hosted CMEK/Global Active eligibility separate |
| SDKs/limits/capacity | [Plan service](https://docs.oracle.com/en/cloud/paas/nosql-cloud/dtddt/plan-your-service.html) | Java/Python/Go/Node/.NET/Rust/Spring catalog; REST has fewer SQL features |
| Sharding/consistency | [Cloud concepts](https://docs.oracle.com/en/cloud/paas/nosql-cloud/dtddt/cloud-concepts.html) | Native shard/primary keys, EVENTUAL/ABSOLUTE |
| Numeric/complex types | [Types](https://docs.oracle.com/en/cloud/paas/nosql-cloud/rnpxl/) | NUMBER vs FLOAT/DOUBLE, arrays/JSON; no invented vector type |
| Indexes | [Create index](https://docs.oracle.com/en/cloud/paas/nosql-cloud/uapto/) | Fixed/JSON/composite indexes, async state, type constraints |
| Version/write/batch behavior | [Write APIs](https://docs.oracle.com/en/cloud/paas/nosql-cloud/pdhxw/index.html) | Node writeMany, same shard; conditions are not cross-shard ACID |
| Node query/versions | [SDK reference](https://oracle.github.io/nosql-node-sdk/) and [QueryResult](https://oracle.github.io/nosql-node-sdk/interfaces/QueryResult.html) | v5.5.4 page empty+non-null continuation; SDK types not REST token |
| Java query lifecycle | [QueryRequest](https://docs.oracle.com/en/cloud/paas/nosql-cloud/csnjv/oracle/nosql/driver/ops/QueryRequest.html) | Request completion, prepared SQL, read budget |
| Rust SDK/signatures | [Oracle Rust repository](https://github.com/oracle/nosql-rust-sdk) and [published API](https://docs.rs/oracle-nosql-rust-sdk/latest/oracle_nosql_rust_sdk/struct.QueryRequest.html) | Native SDK exists;0.1.4 batch/full completion difference |
| IAM | [Policy reference](https://docs.oracle.com/en/cloud/paas/nosql-cloud/odsql/) | Table/index/row rights distinct |
| CMEK | [Dedicated CMEK](https://docs.oracle.com/en/cloud/paas/nosql-cloud/icxer/) | Dedicated environment prerequisite, default Oracle-managed; not per-table switch |
| Expiration | [TTL management](https://docs.oracle.com/en/cloud/paas/nosql-cloud/mtiuc/) | Default changes do not rewrite existing expiration |
| Regions | [Global Active Tables](https://docs.oracle.com/en/cloud/paas/nosql-cloud/gasnd/index.html) | Regional capacity/global DDL and replicated absolute expiry |
| Metrics | [Monitoring](https://docs.oracle.com/en/cloud/paas/nosql-cloud/mgygg/) | oci_nosql namespace and resource/table dimensions |
| Export/import capability | [Migrator workflow](https://docs.oracle.com/en/cloud/paas/nosql-cloud/dtddt/using-oracle-nosql-database-migrator.html) | Cloud and JSON Object Storage supported; no locking/snapshot isolation |
| Config/TTL details | [Migrator reference](https://docs.oracle.com/en/cloud/paas/nosql-cloud/dtddt/oracle-nosql-database-migrator-reference.html) | nosqldb_cloud/object_storage_oci, schemaPath, overwrite, includeTTL/rounding |
| Native IaC | [Terraform table](https://docs.oracle.com/en-us/iaas/tools/terraform-provider-oci/latest/docs/r/nosql_table.html) and [index](https://docs.oracle.com/en-us/iaas/tools/terraform-provider-oci/latest/docs/r/nosql_index.html) | Matching table limits/DDL/index keys; no unsupported table CMEK |
| CLI | [NoSQL reference](https://docs.oracle.com/en-us/iaas/tools/oci-cli/latest/oci_cli_docs/cmdref/nosql.html) and [row get](https://docs.oracle.com/en-us/iaas/tools/oci-cli/latest/oci_cli_docs/cmdref/nosql/row/get.html) | Table/index/query/row/work requests; no invented backup/PITR subcommands |
| REST query | [QueryDetails](https://docs.oracle.com/en-us/iaas/tools/python/latest/api/nosql/models/oci.nosql.models.QueryDetails.html) | Prepared statement, variables, consistency, read limit |
| External vector service | [Oracle AI Database scope](https://docs.oracle.com/en/cloud/paas/autonomous-database/vcapi/concepts/oracle-ai-database-vector-search.html) and [vector tables](https://docs.oracle.com/en/cloud/paas/autonomous-database/vcapi/concepts/vector-table.html) | AI Database, not NDCS; own model/store/ANN limits |
| OCI retrieval concepts | [Vector stores](https://docs.oracle.com/en-us/iaas/Content/generative-ai/vector-stores.htm) | Generation/retrieval distinct from NoSQL storage |

## Actual tool retrieval

- Context7 `resolve_library_id` for `Oracle NoSQL Database Cloud Service` returned
  Oracle SQL Database 23ai/AI Vector Search/Autonomous/JDBC libraries, NOT an exact
  NDCS match. They were not substituted as NoSQL evidence.
- A second `resolve_library_id` for `oracle nosql-node-sdk` returned broad
  `/websites/oracle_en`. `query_docs` on that library with NoSQL continuation/
  consistency/WriteMultiple returned actual cloud `using-query-api-to-fetch-data`
  excerpts for `NoSQLClient.query/queryIterable` and empty continuation handling.
  This is successful limited provider retrieval, not complete Context7 coverage.
- Oracle docs were independently searched/opened for every domain above. The old
  IAM URL under `iaas/Content/Identity/Reference/nosqlpolicyreference.htm` failed
  retrieval; the official NoSQL policy-reference page was used instead.
- Local `oci --version`: 3.95.0. Only offline `--help`/JSON-input generation ran
  for table create/get, index create, row get/update, query prepare/execute. Offline
  installed OCI SDK2.188.0 model inspection confirmed PreparedStatement.statement
  and repeated row-get key flags. No auth/config/credentials or OCI service call.
- `terraform version` failed before execution with snap confinement refusal:
  `snap-confine has elevated permissions and is not confined but should be.`
  No init/download/escalation used; Terraform format/provider validation remains
  UNVERIFIED. The local controlled-fixture checks are not HCL/provider validation.

No native PITR or native NDCS ANN endpoint was established. Accordingly those
activation paths fail closed; revisit sources if the user's target release claims
support. Exact IAM effectiveness, SDK builds, key access, deployed index choice,
export integrity, RPO/RTO and application restore all remain runtime-UNVERIFIED.
