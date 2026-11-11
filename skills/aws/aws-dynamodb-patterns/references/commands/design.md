# design

Design a new table/model or a proposed access path. Read this command and only
selected feature references. Do not load audit/refactor/performance siblings.
This mode generates design/IaC/code; it does not create a table or query AWS.

```text
/aws-dynamodb-patterns design --features core,indexes,encryption
```

```bash
python3 scripts/select_mode.py design --features core,indexes,encryption
```

Provide the workload or relevant repo files in the surrounding agent request;
the selector accepts flags, not arbitrary free-text positional arguments. Core is
the default. Ask only for missing inputs that change correctness; label remaining
assumptions rather than inventing a proven production workload.

| Step | Required result |
|---|---|
| Define access patterns | Operation, inputs, complete key/filter, ordering, consistency, result/cursor bound and tenant/authorization context |
| Bound workload | Item bytes/growth, average and peak rates, cardinality, largest tenant/hottest key, retention, latency objectives and regional requirements |
| Compare models | Single vs multiple tables or another datastore; explicit chosen/rejected alternative, no automatic single-table or ORM |
| Choose keys/indexes | Actual PK/SK values, item/entity examples, sparse-index writes/removals, justified LSI/GSI/vector choice, skew/sharding/fan-out costs |
| Define correctness | Conditional/versioned/idempotent writes, uniqueness, retries/unknown outcomes, pagination/typed cursors and tenant boundary |
| Generate requested artifacts | CloudFormation OR AWS CLI and selected-language SDK/request examples, with owner/version context; no deployment |
| Prove or label | Deterministic key/request/negative fixtures and local schema checks; runtime IAM, load, recall or restore evidence stays UNVERIFIED until tested |

Before claiming an access pattern works, show its exact GetItem/Query/search
request. Do not invent an order-ID-only GetItem without all key attributes, use a
scan to conceal a missing hot-path index, or treat filter expressions as cheap reads.
An LSI/new-table decision, GSI eventual reads and vector-specific constraints must
match the selected guide. Generic AttributeDefinitions advice is not a substitute
for the vector SearchSchema rules.

Use the selected [modeling](../modeling.md), [runtime contract](../runtime-contract.md)
and [index](../lsi-vs-gsi.md) guidance as needed, not as an eager bundle. For a full
end-to-end delivery sequence, load [workflow](../workflow.md); for requested table
artifacts load [provisioning](../provisioning.md) and only the selected asset. Match
SDK syntax through current primary docs/Context7, not guessed cross-language APIs.

Return the access-pattern matrix, item/key/index examples, chosen tradeoffs,
requested code/IaC, skew/cost estimates and tests/unknowns proportionately. If the
request changes an existing persisted model, identify that boundary and use the
refactor workflow only when that additional scope is requested. A new design is
not evidence that existing data can be replaced or migrated safely.
