# Sources and evidence boundaries

## 1.8.0 design/refactor commands (2026-11-08)

Context7 returned current GSI online backfill/readiness and secondary-index lifecycle
semantics. AWS documentation specialist research verified UpdateItem upsert/key
boundaries and CloudFormation named replacement/online-index caveats; exact primary
links are in the refactor route. CFN summary interruption labels are not treated
as permission for in-place key/LSI redesign. No table, item, index or schema changed.
User added backup/restore before publication: Context7 returned current PITR and
restore contracts; the backup specialist verified asynchronous on-demand backup
boundaries, source/readiness and restored-setting/RPO/RTO gates through AWS docs.
Exact primary links live with the method-specific procedures. Tests use fake AWS
only; no backup was created and no actual recovery/business-continuity test ran.

## 1.7.0 command organization (2026-11-08)

Current Brownfield SKILL.md and references/commands/ procedures supplied the local
per-command selective-loading pattern, not copied domain content. Command guides
were moved with relative links updated; audit now has separate offline/live
transports. The real local selector accepts the slash prefix; actual host command
discovery/registration is not asserted. Live identity/table examples are exercised
only with a fake AWS binary; no live environment was audited during this refactor.
Context7 independently returned the STS Account/Arn and DescribeTable metadata
contracts; exact primary URLs are linked in the live audit route.

## 1.6.0 scoped audit/performance modes (2026-11-08)

Context7 resolved the DynamoDB Developer Guide and returned current internal versus
client latency and metric aggregation semantics. AWS documentation MCP specialist
research checked hot-key, partial-batch/throttle and vector monitoring distinctions;
primary URLs are in the lazy performance guide. Selector results are local workflow
routes, not audit/performance measurements, persistent state or host commands.
No live audit/load run or feature activation was performed.

## 1.5.0 lazy CLI companion (2026-11-08)

Context7 resolved the AWS CLI reference and returned InvokeModel file/binary/output
and KB job command contracts. AWS documentation MCP specialist research verified
native/S3 vector and KB retrieval shapes; the [CLI companion](commands/embedding.md)
links exact command sources. S3 metadata/filter queries also need GetVectors; S3/KB
pagination must not inherit DynamoDB SearchVectors' no-pagination assumption.
Actual guide snippets are exercised only through a fake AWS binary. Static selective
routes do not prove host loading, measured token savings or live service acceptance.

## 1.4.0 embedding meaning/precision and integrations (2026-11-08)

Context7 resolved the DynamoDB Developer Guide and returned current f32 versus
base-item precision, same-model generation/query and Bedrock tutorial text. AWS
Knowledge MCP independently returned DynamoDB requirements/Number limits and S3
PutVectors float32/API separation. Official Bedrock StorageConfiguration and Titan
V2 request/response docs establish the managed-store/model boundaries. Sources are
linked beside the claims in [embedding integrations](embedding-integrations.md).
Binary32 format/constant checks use primary Python struct/Microsoft documentation
and local IEEE packing; numeric format limits are not live AWS acceptance evidence.
The configured DynamoDB specialist independently checked KB/S3 Vectors integration.
No installation, AWS invocation, relevance/cost benchmark or full content audit.

Research date: 2026-11-08. This package is an original synthesis. Source availability
and APIs can change; verify version-sensitive details at use time. No upstream
prose/code was vendored and no license compatibility claim is made.

## Requested inspiration

The repeated access-patterns URL was deduplicated (six unique inputs).

| Source | Observed access | Treatment |
|---|---|---|
| [AWS amazon-dynamodb](https://www.skills.sh/aws/agent-toolkit-for-aws/amazon-dynamodb) / [repository skill](https://github.com/aws/agent-toolkit-for-aws/blob/main/skills/specialized-skills/database-skills/amazon-dynamodb/SKILL.md) | Listing and indexed repository material reviewed | Design axioms/workload-first inspiration; no imported deployment/benchmark workflow |
| [Toolbox patterns](https://www.skills.sh/giuseppe-trisciuoglio/developer-kit/dynamodb-toolbox-patterns) | Listing/overview reviewed; complete upstream file not verified | v2 TypeScript-specific inspiration, isolated optional adapter; primary Toolbox docs checked |
| [Access patterns](https://www.skills.sh/juancarestre/dynamodb-skills/dynamodb-access-patterns) | Page and bounded repository lookup unavailable | Name/topic only; contents not inspected |
| [Single-table](https://www.skills.sh/juancarestre/dynamodb-skills/dynamodb-single-table) | Unavailable | Name/topic only; no attributed technical claims |
| [Table design](https://www.skills.sh/juancarestre/dynamodb-skills/dynamodb-table-design) | Unavailable | Name/topic only; independently sourced AWS guidance |
| [Query interfaces](https://www.skills.sh/juancarestre/dynamodb-skills/dynamodb-query-interfaces) | Unavailable | Name/topic only; independently designed API contract |

Unavailable source contents must not be described as merged or verified. Revisit
them if accessible later, but do not install an upstream skill or run its scripts
merely to inspect it. Exact upstream license files were not verified.

## Context7 MCP actually used

Resolved Amazon DynamoDB to
`/websites/aws_amazon_amazondynamodb_developerguide` and queried:

1. Query pagination, filtering, LastEvaluatedKey and consistency.
2. CloudFormation/CLI table creation, on-demand, TTL and PITR split operations.

Resolved DynamoDB Toolbox to `/dynamodb-toolbox/dynamodb-toolbox` and queried:

3. Installed-major/version migration, command/schema API and JS/TS-only scope.

These retrievals supplemented direct AWS primary docs; they do not attest SDK
compilation or service acceptance. Do not copy malformed example fragments from
retrievals without validating them. Context7 snippets may reflect unversioned main.

For version 1.1.0, queried the same resolved DynamoDB library for the LSI/GSI
comparison (lifecycle, consistency, 10 GB collection limit, capacity/projections)
and encryption choices/CLI-CFN key selection. Direct AWS CLI, secondary-index,
encryption usage and CloudFormation SSE pages were also reviewed on 2026-11-08.
Links are adjacent to the three new guides; no live requests were used as evidence.

For version 1.2.0, **both MCPs were used again**, not inferred from configuration:

- Context7: three queries to the resolved DynamoDB developer guide for IAM/index/
  resource/tag rights, DAX cache consistency, and PITR/replica recovery boundaries.
- AWS Knowledge MCP: search and justified focused reads of the current service
  authorization reference, table/index examples, resource policies, stream ABAC,
  Streams/Lambda, DAX, PITR, global-table security and CloudWatch definitions. Exact
  primary links are adjacent to each domain guide, not guessed tool/API names.
- Significant findings: Streams now has independent tags/ABAC; transaction IAM
  rights use component actions; AWS-managed-key cross-account access is limited;
  core MRSC docs permit eligible Regions across continents while older guidance
  retains Region sets. Use current core docs and reverify intended Regions.
- The inspected CLI 2.37.4 rejected the offline `get-abac-status` operation name;
  the guides do not invent a public CLI command from the documented IAM action.
  Offline DAX input-skeleton inspection confirmed supported creation parameters.
- CloudWatch's user guide allows a missing-data override, but its current
  PutMetricAlarm API reference says AWS/DynamoDB always ignores missing data.
  Both were retrieved and the conflict is explicit; the example uses `ignore`
  rather than promising override/recovery behavior. Live transition/delivery
  tests remain required, not inferred.

Research is documentation evidence, not a test of an effective role, cache,
restored data, replication, failover or alarm delivery. No schedules, memory writes,
remote dispatch, new MCP installation or live AWS control/data calls were made.

For version 1.3.0 (2026-11-08), AWS Knowledge MCP research plus Context7 retrieval
verified the supported scalar/document/set families, native vector-index schema,
SearchVectors request/IAM/constraints and the distinction from OpenSearch. All
ten AttributeValue tags remain the existing wire types; vector storage is L of N.
The CLI 2.37.4 exposes the SearchVectors input skeleton (no service request).
Strict vector lint emitted E3039 on 1.57.2 and on the user-upgraded 1.60.0. The
installed rule's inspected implementation ignores VectorIndexes.SearchSchema;
the guide and tests expose this compatibility finding instead of deleting required
declarations or claiming strict lint passed. Primary links are in the new vector
guide. The content-review ledger schedules June/December and records targeted
research separately from a completed full audit.

## Primary documentation by claim

- Key conditions, filters, limit, ordering, consistency and pagination:
  [Query API](https://docs.aws.amazon.com/amazondynamodb/latest/APIReference/API_Query.html),
  [pagination](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/Query.Pagination.html).
- Schema/provisioning: [CloudFormation table](https://docs.aws.amazon.com/AWSCloudFormation/latest/TemplateReference/aws-resource-dynamodb-table.html),
  [CreateTable CLI](https://docs.aws.amazon.com/cli/latest/reference/dynamodb/create-table.html).
- Key/index and item/operation limits: [constraints](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/Constraints.html),
  [LSIs](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/LSI.html),
  [GSI back-pressure](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/gsi-throttling.html).
- Concurrency/batches: [transactions](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/transaction-apis.html),
  [TransactWriteItems token](https://docs.aws.amazon.com/amazondynamodb/latest/APIReference/API_TransactWriteItems.html),
  [BatchGetItem](https://docs.aws.amazon.com/amazondynamodb/latest/APIReference/API_BatchGetItem.html),
  [BatchWriteItem](https://docs.aws.amazon.com/amazondynamodb/latest/APIReference/API_BatchWriteItem.html).
- Lifecycle: [TTL](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/TTL.html),
  [global table modes](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/V2globaltables_HowItWorks.html),
  [PITR](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/PointInTimeRecovery.html).
- Language-specific links are adjacent to their examples in
  [Language adapters](language-adapters.md). Verify exact installed SDK versions
  and wrapper import paths in the target repository before adoption.

## Evidence classes

- **Documentation-based**: service semantics and inspected API examples.
- **Local syntax/static evidence**: skill metadata, link targets, CLI/CFN schema
  checks and asset equivalence. Not AWS deployment success.
- **Mocked behavior evidence**: injected query callback and adversarial pagination
  tests. Not an SDK, IAM, network, load or integration test.
- **Model-review evidence**: pilot responses and independent challenge review.
  Not proof of reliable skill triggering or causal improvement.
- **Unverified here**: live table creation, real concurrency/latency/capacity,
  workload authorization and compilation against every language's dependency graph.
