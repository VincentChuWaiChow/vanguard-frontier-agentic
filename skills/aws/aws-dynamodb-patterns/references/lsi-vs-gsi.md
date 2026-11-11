# LSI vs GSI decision guide

Start from the request's partition scope and freshness requirement—not “local is
faster” or “global is unlimited.” First check whether the existing table key can
serve the query; every added index creates write/storage and operational costs.

## Decision sequence

1. **Different partition-key attribute or regrouping across customers?** An LSI
   cannot do it; consider a GSI, materialized view or a different primary model.
   A GSI may also reuse the base partition-key attribute—it need not differ.
2. **Same partition key, different ordering, strong read required?** An LSI is a
   candidate only at **new-table creation**, if the whole item collection is
   demonstrably bounded below 10 GB. A GSI is not a strong-read substitute.
3. **Already-deployed table without that LSI?** No retrofit. Use an eventual GSI
   if freshness permits; otherwise redesign the authoritative base-key path or
   plan an explicitly approved data migration. Do not silently recreate a table.
4. **Unbounded collection, unknown growth or throughput isolation needed?** Reject
   the LSI by default. Evaluate a GSI or re-key/bucket the base model, accounting
   for query fan-out. Do not promise strong alternate-key reads from an eventual GSI.
5. **Cross-partition lookup plus mandatory strong discovery?** Neither an LSI nor
   an ordinary GSI query solves it. An eventual index followed by strong GetItem
   can validate discovered rows but cannot discover rows still missing from it.

## Comparison

| Criterion | LSI | GSI |
|---|---|---|
| Partition key | Exactly the base-table partition-key attribute | Can reuse it or use a different key attribute |
| Sort key | Required alternate sort key; base table must have a sort key | Optional; partition-only or composite index |
| Query scope | One known base PK value | One known index PK value; can regroup data from different base PKs |
| Consistency | Eventual or strong via ConsistentRead | Eventual only; true is rejected |
| Lifecycle | Define at table creation; cannot add or delete later | Create with table or add/delete online, with backfill/impact review |
| Size boundary | 10 GB per item collection: base items **plus all LSI entries** sharing a PK value | No LSI-style 10 GB item-collection limit; ordinary service/item/partition limits remain |
| Provisioned capacity | Reads/writes share the table's allocation | Separate index read/write allocation; overload can still back-pressure base writes |
| On-demand mode | Table request accounting; no provisioned allocation to set | On-demand index requests still cost money and have limits/hot keys |
| Unprojected attributes | Can fetch from base table, with additional read/latency cost | Cannot fetch automatically; project fields or issue explicit base reads |
| Default index count | Up to 5 LSIs/table | Default 20 GSIs/table; verify current adjustable quota |
| Key uniqueness | Alternate sort values need not be unique | Index key combinations need not be unique |
| Encryption | Inherits the table key | Inherits the table key; not independently selectable |

“Global” does **not** mean global cross-Region strong consistency or an arbitrary
cross-partition query. Query still requires equality on one index partition key.
An LSI is not an independently scalable copy of the table and constrains how the
item collection can be split. Near-limit growth is a design problem, not something
TTL's delayed deletion or on-demand billing guarantees will solve.
Native vector indexes are a separate access path with SearchVectors, but they
also consume the table's GSI quota. See [data types and vectors](data-types-and-vectors.md)
before assuming the GSI quota is entirely available for ordinary secondary indexes.

## Worked choices in this skill's model

### New, bounded customer history; strongly read most recently updated orders

- Base PK: `TENANT#acme#CUSTOMER#c42`; base SK remains immutable order creation key.
- Alternate `LSI1SK`: `UPDATED#2026-01-03T09:00:00.000Z#o100` only on order items.
- Candidate: LSI1 with HASH `PK`, RANGE `LSI1SK`, INCLUDE `status`, `orderId`.
- It is valid only if PK-level base+all-LSI growth is bounded and strong updated
  ordering is worth permanent capacity/storage coupling. Include a stable tie-break
  ID in the alternate string, rather than relying on unspecified equal-key order.
- Maintain LSI1SK alongside each relevant update, including close/reopen; lexical
  time ordering requires fixed-width UTC. The example put/close in Commands targets
  the **default** table; adapt writes deliberately before using this alternative.

The [new-table CLI asset](../assets/create-table-with-lsi.json) contains the complete
base/GSI/LSI key definitions. For CloudFormation, the equivalent **property fragment**
below belongs in a new table's Properties along with the original definitions:

```yaml
# Append {AttributeName: LSI1SK, AttributeType: S} to AttributeDefinitions.
LocalSecondaryIndexes:
  - IndexName: LSI1
    KeySchema:
      - AttributeName: PK
        KeyType: HASH
      - AttributeName: LSI1SK
        KeyType: RANGE
    Projection:
      ProjectionType: INCLUDE
      NonKeyAttributes: [status, orderId]
```

This is not a standalone template, nor a permitted update to an existing table.
The default `table.cfn.yaml` is intentionally left without an LSI.

### All open orders for a tenant, across customers

Choose a sparse GSI keyed by `TENANT#acme#STATUS#OPEN`, not an LSI on the customer
PK. Accept index lag and atomically maintain/remove GSI keys. A dominant tenant
can still create a hot index key; shard only with fan-out/merge/cursor design.

### Existing large customer history; add a new sort order

Do not add an LSI—it is impossible online, and an unbounded collection violates
its design constraint. If eventual discovery is acceptable, evaluate a new GSI
(which may reuse the customer PK) and backfill/readiness checks. If strong alternate
reads are essential, redesign the base layout/migrate; state that cost explicitly.

## Decision record and adversarial probes

Record chosen/rejected index, request scope/freshness, projected fields, largest
collection estimate and growth, read/write coupling/skew, and rollout/rollback.
Reject these proposals explicitly:

- “Add an LSI tomorrow on the existing table.” No online operation exists.
- “Use ConsistentRead=true on the GSI.” It does not enable strong reads.
- “12 GB for this customer's base+LSIs is okay because billing is on-demand.” No.
- “Make the LSI partition key tenant-wide instead of customer.” That is a GSI-like
  regrouping, not an LSI on this base table.
- “Changing the index projection encrypts it with its own KMS key.” Both indexes
  share table encryption; see [Encryption](encryption.md).

Sources: [AWS comparison](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/SecondaryIndexes.html),
[LSI behavior/collections](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/LSI.html),
[GSI behavior](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/GSI.html),
[index quotas](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/ServiceQuotas.html).
