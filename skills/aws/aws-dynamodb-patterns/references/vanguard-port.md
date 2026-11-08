# Port record and evidence index

This skill is ported from the canonical copy in the private repository
`VincentChuWaiChow/agentic-vantage` (`skills/aws-dynamodb-patterns`, commit
`a317609`, version 1.8.0). **agentic-vantage is the source of truth**: make content
changes there first, then re-sync this directory and re-run the gates here.

## Local differences from upstream

- `SKILL.md` frontmatter adds `allowed-tools` (Read, Grep, Glob and the two offline
  helper scripts only) and `metadata.author`, `updated` and `category`, which this
  repository's skill schema requires.
- `metadata.json`, this file, [the evidence ledger](evidence-ledger.md) and three
  repository adapters (`workflow-and-output.md`, `official-sources.md`,
  `safety-checklist.md`, linked from a short "Repository references" section at the
  end of `SKILL.md`) exist only here. The AWS skill gates require them.
- Three inline evidence citations ([DP-E214] in `data-types-and-vectors.md`, [DP-E410] in
  `commands/restore.md`, [DP-E453] in `streams-and-dax.md`) ground this repository's
  behavioural scenarios. Keep them when re-syncing.
- Two CodeQL fixes, to apply upstream too: `scripts/review_due.py` prints its result
  in the `try` statement's `else` branch, so `result` is never read uninitialised, and
  `evals/operations.test.py` uses `assertLessEqual` for its metric-subset check.
- `commands/restore/on-demand.md` matches the backup's source `TableId` as well as its
  ARN [DP-E701] [DP-E702], and `evals/backup-restore.test.py` covers a backup from an
  earlier incarnation of the table and a missing `TableId`. Apply upstream too: a table
  deleted and recreated under the same name keeps its ARN, so the ARN check alone could
  approve restoring the wrong data.
- `streams-and-dax.md` adds .NET to the documented DAX clients [DP-E453]. The upstream
  text listed Java, Node.js, Python and Go; AWS also documents a .NET client. Apply the
  same fix upstream.

## Sync procedure

1. Copy the upstream directory over this one, keeping the local differences above.
2. Re-check every changed claim against AWS documentation and update the ledger rows.
3. Run the skill's evals (`npm run validate:aws-dynamodb-patterns`, which needs
   `cfn-lint`), then the repository gates.

## Verification summary

Each claim was checked against the AWS page it cites, or the closest official page,
on 2026-11-08. A mechanical re-check then confirmed that the quoted sentence appears on
the cited page for every VERIFIED row whose page could be fetched as text. One claim
was contradicted and fixed (DAX .NET client). UNVERIFIED rows are listed below; treat
those statements as unconfirmed until re-checked.

## Evidence by topic

### Modeling, indexes, runtime contract, design and refactor

Covers modeling.md, lsi-vs-gsi.md, runtime-contract.md, workflow.md, commands/design.md, commands/refactor.md.

[DP-E101] [DP-E102] [DP-E103] [DP-E104] [DP-E105] [DP-E106] [DP-E107] [DP-E108]
[DP-E109] [DP-E110] [DP-E111] [DP-E112] [DP-E113] [DP-E114] [DP-E115] [DP-E116]
[DP-E117] [DP-E118] [DP-E119] [DP-E120] [DP-E121] [DP-E122] [DP-E123] [DP-E124]
[DP-E125] [DP-E126] [DP-E127] [DP-E128] [DP-E129] [DP-E130] [DP-E131] [DP-E132]
[DP-E133] [DP-E134] [DP-E135] [DP-E136] [DP-E137] [DP-E138] [DP-E139] [DP-E140]
[DP-E141] [DP-E142] [DP-E143] [DP-E144] [DP-E145] [DP-E146] [DP-E147] [DP-E148]
[DP-E149] [DP-E150] [DP-E151] [DP-E152] [DP-E153] [DP-E154] [DP-E155] [DP-E156]
[DP-E157] [DP-E158] [DP-E159] [DP-E160]

### Data types, native vectors and embeddings

Covers data-types-and-vectors.md, embedding-integrations.md, commands/embedding.md and the vector assets.

[DP-E201] [DP-E202] [DP-E203] [DP-E204] [DP-E205] [DP-E206] [DP-E207] [DP-E208]
[DP-E209] [DP-E210] [DP-E211] [DP-E212] [DP-E213] [DP-E214] [DP-E215] [DP-E216]
[DP-E217] [DP-E218] [DP-E219] [DP-E220] [DP-E221] [DP-E222] [DP-E223] [DP-E224]
[DP-E225] [DP-E226] [DP-E227] [DP-E228] [DP-E229] [DP-E230] [DP-E231] [DP-E232]
[DP-E233] [DP-E234] [DP-E235] [DP-E236] [DP-E237] [DP-E238] [DP-E239] [DP-E240]
[DP-E241] [DP-E242] [DP-E243] [DP-E244] [DP-E245] [DP-E246] [DP-E248] [DP-E249]
[DP-E250] [DP-E251] [DP-E252] [DP-E253] [DP-E254] [DP-E255] [DP-E256]

### IAM, encryption, provisioning and CLI

Covers iam-and-access.md, encryption.md, provisioning.md, commands/aws-cli.md and the policy, CloudFormation and CLI assets.

[DP-E301] [DP-E302] [DP-E303] [DP-E304] [DP-E305] [DP-E306] [DP-E307] [DP-E308]
[DP-E309] [DP-E310] [DP-E311] [DP-E312] [DP-E313] [DP-E314] [DP-E315] [DP-E316]
[DP-E317] [DP-E318] [DP-E319] [DP-E320] [DP-E321] [DP-E322] [DP-E323] [DP-E324]
[DP-E325] [DP-E326] [DP-E327] [DP-E328] [DP-E329] [DP-E330] [DP-E331] [DP-E332]
[DP-E333] [DP-E334] [DP-E335] [DP-E336] [DP-E337] [DP-E338] [DP-E339] [DP-E340]
[DP-E341] [DP-E342] [DP-E343] [DP-E344] [DP-E345] [DP-E346] [DP-E347] [DP-E348]
[DP-E349] [DP-E350] [DP-E351] [DP-E353] [DP-E354] [DP-E355] [DP-E356] [DP-E357]
[DP-E358] [DP-E359] [DP-E360] [DP-E361] [DP-E362] [DP-E363] [DP-E364] [DP-E365]
[DP-E366] [DP-E367] [DP-E368] [DP-E369] [DP-E370] [DP-E371] [DP-E372] [DP-E373]

### Backup, restore, replicas, Streams and DAX

Covers recovery-and-replicas.md, streams-and-dax.md, commands/backup*, commands/restore*, commands/recovery-target.md.

[DP-E401] [DP-E402] [DP-E403] [DP-E404] [DP-E405] [DP-E406] [DP-E407] [DP-E408]
[DP-E409] [DP-E410] [DP-E411] [DP-E412] [DP-E413] [DP-E414] [DP-E415] [DP-E416]
[DP-E417] [DP-E418] [DP-E419] [DP-E420] [DP-E421] [DP-E422] [DP-E423] [DP-E424]
[DP-E425] [DP-E426] [DP-E427] [DP-E428] [DP-E429] [DP-E430] [DP-E431] [DP-E432]
[DP-E433] [DP-E434] [DP-E435] [DP-E436] [DP-E437] [DP-E438] [DP-E439] [DP-E440]
[DP-E441] [DP-E442] [DP-E443] [DP-E444] [DP-E445] [DP-E446] [DP-E447] [DP-E448]
[DP-E449] [DP-E450] [DP-E451] [DP-E452] [DP-E453] [DP-E454] [DP-E455] [DP-E456]
[DP-E457] [DP-E458]

### Monitoring, SDK adapters, audit and performance tests

Covers monitoring.md, language-adapters.md, commands/performance-test.md, commands/audit*, commands/feature-plan.md, commands/select.md and the SDK assets.

[DP-E501] [DP-E502] [DP-E503] [DP-E504] [DP-E505] [DP-E506] [DP-E507] [DP-E508]
[DP-E509] [DP-E510] [DP-E511] [DP-E512] [DP-E513] [DP-E514] [DP-E515] [DP-E516]
[DP-E517] [DP-E518] [DP-E519] [DP-E520] [DP-E521] [DP-E522] [DP-E523] [DP-E524]
[DP-E525] [DP-E526] [DP-E527] [DP-E528] [DP-E544] [DP-E545] [DP-E546] [DP-E547]
[DP-E530] [DP-E531] [DP-E532] [DP-E533] [DP-E534] [DP-E535] [DP-E536] [DP-E537]
[DP-E538] [DP-E539] [DP-E540] [DP-E541] [DP-E542] [DP-E543]

## Unconfirmed and conflicting claims

| Evidence | Status | Claim |
| --- | --- | --- |
| [DP-E112] | UNVERIFIED | "base table must have a sort key" for an LSI (lsi-vs-gsi.md:30 (second half)) |
| [DP-E125] | CONFLICT | LSI cannot be added to an existing table (CFN update of LocalSecondaryIndexes) (lsi-vs-gsi.md:33,164,214 vs CFN property docs) |
| [DP-E133] | UNVERIFIED | AWS CLI --starting-token/NextToken are not interchangeable with service ExclusiveStartKey/LastEvaluatedKey (runtime-contract.md:33-34) |
| [DP-E150] | UNVERIFIED | Base-table key schema changes cannot be done in place via UpdateTable (need new table/migration) (runtime-contract.md:110; refactor.md:29) |
| [DP-E153] | UNVERIFIED | Vector index SearchSchema is separate from generic AttributeDefinitions; vector indexes use SearchSchema (design.md:35-38; workflow.md:62) |
| [DP-E158] | UNVERIFIED | Key attributes cannot be changed by an UpdateExpression; moving a primary-key value needs a new item (refactor.md:32-34) |
| [DP-E204] | UNVERIFIED | Key scalar types are S/N/B (not L/M/sets) (D:23) |
| [DP-E253] | UNVERIFIED | PutVectors can overwrite an existing key (C:330-333) |
| [DP-E313] | UNVERIFIED | CLI (inspected 2.37.4) has no `dynamodb get-abac-status`; IAM action name is not evidence a CLI op exists (iam-and-access.md:114-117). Note: absence consistent with claim; version-specific |
| [DP-E320] | UNVERIFIED | Tag on create uses CreateTable `--tags`; CreateTable with tags needs TagResource permission (iam-and-access.md:102-103). Note: flag exists; permission requirement not located |
| [DP-E322] | UNVERIFIED | `aws:ResourceTag`, `aws:RequestTag`, `aws:TagKeys` support varies by action/resource (iam-and-access.md:110-111). Note: SAR page is JavaScript-rendered; condition keys not readable in this pass |
| [DP-E345] | UNVERIFIED | AWS-managed key `alias/aws/dynamodb` may not exist until first use (encryption.md:125) |
| [DP-E348] | UNVERIFIED | Auto rotation of same KMS key is not switching ARN (encryption.md:143-145). Note: partially supported |
| [DP-E355] | UNVERIFIED | `DeletionPolicy: Retain` / `UpdateReplacePolicy: Retain` keep resources (provisioning.md:64). Note: not fetched |
| [DP-E361] | UNVERIFIED | TTL may report ENABLING before ENABLED (provisioning.md:97-98; aws-cli.md:306). Note: not fetched |
| [DP-E367] | UNVERIFIED | put-item `--condition-expression`; update-item with `SET ... REMOVE`; `--expression-attribute-names/values` (aws-cli.md:131-137 (file 241-247)). Note: not fetched |
| [DP-E368] | UNVERIFIED | `create-table` alt with LSI asset, then `wait table-exists` (aws-cli.md:157-164 (file 267-275)). Note: not fetched |
| [DP-E371] | UNVERIFIED | TTL is delayed cleanup, not an exact expiry (aws-cli.md:195-200 (file 305-310)). Note: not fetched |
| [DP-E372] | UNVERIFIED | CLI flags `--cli-input-json file://`, `describe-time-to-live`, `describe-continuous-backups`, existing name yields ResourceInUseException (create-table.sh:15,26-37). Note: cited page is PutResourcePolicy errors, not CreateTable/CLI; flags not confirmed |
| [DP-E409] | UNVERIFIED | Timestamp and latest flags are mutually exclusive (restore/pitr.md:32,38). Note: no sentence found that says combining is rejected; searched Tutorial and API_RestoreTableToPointInTime |
| [DP-E412] | UNVERIFIED | Restored table is independent, incl. from a global-table replica; replica topology is not restored (recovery-and-replicas.md:52-54; restore.md:67). Note: searched pointintimerecovery_restores, PointInTimeRecovery.Tutorial, Howitworks, API_RestoreTableToPointInTime, API_RestoreTableFromBackup |
| [DP-E418] | UNVERIFIED | Native on-demand backup does not require PITR to be enabled (backup.md:21-22). Note: searched CreateBackup, Restore.Tutorial |
| [DP-E444] | UNVERIFIED | Iterator types TRIM_HORIZON oldest / LATEST newest; dynamodbstreams CLI flags (streams-and-dax.md:50-51 (TRIM_HORIZON/LATEST), 52-55 (CLI get-shard-iterator, describe-stream --limit/--no-paginate, get-records --limit)). Note: CLI examples page not fetched; only API-level wording found |
| [DP-E445] | UNVERIFIED | IAM actions are dynamodb:DescribeStream/GetShardIterator/GetRecords on stream ARN; ListStreams has no resource-level scope; UpdateTable enables streams (streams-and-dax.md:67-69). Note: API action names confirmed, but IAM action names/resource types not: Service Authorization Reference pages are JS-rendered and returned no text; iam-policy-example-data-stream pages returned empty |
| [DP-E446] | UNVERIFIED | Mapping setup needs lambda:CreateEventSourceMapping; iam:PassRole only for creating/changing the function execution role (streams-and-dax.md:74-77). Note: lambda-api-permissions-ref and with-ddb fetched; no PassRole / CreateEventSourceMapping text extracted |
| [DP-E451] | UNVERIFIED | A GSI cannot become strongly consistent through DAX (streams-and-dax.md:105-106) |
| [DP-E457] | UNVERIFIED | Setup needs dax:CreateCluster and iam:PassRole on the exact role (streams-and-dax.md:143-144). Note: SAR pages JS-rendered |
| [DP-E510] | UNVERIFIED | ReplicationLatency is "queried in source Region" (references/monitoring.md:69) |
| [DP-E519] | CONFLICT | Primary sources disagree: user guide says AWS/DynamoDB default is ignore with override; PutMetricAlarm API reference says always ignore (references/monitoring.md:113-121) |
| [DP-E521] | UNVERIFIED | cloudwatch:ListMetrics/GetMetricStatistics/GetMetricData lack table-level ARN scoping (use Resource *) (references/monitoring.md:123-125) |
| [DP-E522] | UNVERIFIED | PutMetricAlarm/DescribeAlarms use alarm resources (references/monitoring.md:126-127) |
| [DP-E523] | UNVERIFIED | Contributor Insights is opt-in (enabled per table or GSI) and has a cost impact; the page does not state that it exposes key values (monitoring.md:62-63; commands/performance-test.md:56) |
| [DP-E546] | UNVERIFIED | Rust builder methods `.consistent_read`, `.scan_index_forward`, `.limit`, `.set_exclusive_start_key`, `last_evaluated_key` (references/language-adapters.md:176-177) |
| [DP-E547] | UNVERIFIED | Java 2.x `software.amazon.awssdk:dynamodb`, `QueryRequest.builder()...exclusiveStartKey`, `queryPaginator`, `legacy com.amazonaws.services.dynamodbv2`; PHP `Aws\DynamoDb\DynamoDbClient`, `Marshaler` (references/language-adapters.md:91-120,188-213) |
