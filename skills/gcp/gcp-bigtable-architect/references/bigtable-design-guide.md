# Cloud Bigtable design guide

Load this for row-key, column-family, consistency, app-profile, replication,
garbage-collection, capacity, IAM, backup or client-library questions. Every
bracketed ID resolves in [evidence-ledger.md](evidence-ledger.md). Text
without an ID is labeled inference or recommendation.

Bigtable is a wide-column store whose base table has **one index, the row key**
[BT-E01], and no multi-row transactions [BT-E05]. Alternate access paths come
from **asynchronous secondary indexes** (continuous materialized views), which
are eventually consistent [BT-E17]. Do not transfer Firestore, DynamoDB or
Cosmos DB concepts.

## Capability matrix

| Area | Status | Verified behavior | Evidence |
| --- | --- | --- | --- |
| Data model | SUPPORTED | Sorted key-value map; row key is the only index; column families; timestamped cell versions; sparse | [BT-E01] |
| Transactions | CONDITIONAL | Single-row atomicity only; no multi-row transactions | [BT-E05] |
| Secondary indexes | CONDITIONAL | Asynchronous secondary indexes via continuous materialized views; eventually consistent; index row key must include the source row key | [BT-E17] |
| Conditional writes / increments | CONDITIONAL | Single-row transactions; not allowed under multi-cluster routing | [BT-E05] [BT-E06] |
| Consistency | CONDITIONAL | Strong only via single-cluster routing with no use of other clusters; replication is eventual | [BT-E08] |
| TTL / versions | CONDITIONAL | Garbage collection by age or versions; asynchronous, up to a week | [BT-E09] |
| Storage type | SUPPORTED | SSD or HDD, permanent | [BT-E11] |
| Autoscaling | SUPPORTED | CPU and storage targets with min/max node bounds | [BT-E12] |
| IAM | SUPPORTED | Predefined roles down to table level | [BT-E13] |
| Backup / restore | SUPPORTED | Automated backups (7-day default); restore creates a new table | [BT-E14] |
| Pricing, per-node throughput | UNVERIFIED | Not retrieved this pass | [BT-E16] |

## Row key design

The row key is the whole query model. Design it from the access patterns before
anything else. Keys sort lexicographically as bytes, so pad numbers [BT-E02].

Reject [BT-E03] [BT-E04]:

- a timestamp first in the key, or alone (it causes hotspots);
- sequential numeric IDs (use reversed IDs instead);
- hashed keys (they lose the sort order that range scans and Key Visualizer
  depend on).

Prefer an entity identifier before the timestamp for time series (for example
`entity#timestamp`; the separator is a convention *(recommendation)*), and reversed timestamps after the
entity for newest-first reads [BT-E04]. Well-designed prefixes keep related rows
contiguous for prefix and range scans [BT-E02].

Hard limits [BT-E10]: 4 KB row keys, 100 MB cells, 256 MB rows and 200 MB per
mutation. Recommended limits [BT-E10]: about 100 column families per table,
16 KB column qualifiers, cells at or below 10 MB and rows at or below 100 MB. Rows that grow without bound, such as one
row per tenant holding all events, will cross those limits. Move growth into
the key. *(recommendation)*

## Alternate access paths

When the row key cannot serve a query, use an asynchronous secondary index: a
continuous materialized view with a different row key that must include the
source row key. Applications don't write to it, and it is **eventually
consistent** with the source table [BT-E17]. Never use it for read-after-write
lookups. *(recommendation)*

## Atomicity

Only **one row** is atomic [BT-E05]. Data that must change together belongs in
one row, which pulls against row-size limits [BT-E10]. Name that trade-off.
Read-modify-write (increments, appends) and check-and-mutate are single-row
transactions [BT-E05], and they **cannot be used with multi-cluster routing**
[BT-E06].

## App profiles, replication and consistency

| Goal | Configuration | Cost | Evidence |
| --- | --- | --- | --- |
| Strong consistency | Single-cluster routing, all app profiles on the same cluster, other clusters used only for failover | Gives up multi-cluster availability for normal traffic | [BT-E08] |
| Read-your-writes | Single-cluster routing, all profiles on the same cluster | Same as above | [BT-E08] |
| High rate of read-your-writes with HA | Multi-cluster routing with row-affinity routing, or clusters each in a different region (read-your-writes within the source region unless failover) | Not guaranteed; no single-row transactions | [BT-E06] [BT-E08] |
| High availability | Multi-cluster routing | Eventual consistency; no single-row transactions | [BT-E06] [BT-E08] |

Watch the defaults [BT-E07]:

- An instance created with one cluster has a single-cluster default profile
  with transactions on.
- An instance created with two or more clusters has a multi-cluster default
  profile with transactions off.
- Adding clusters later does not change the default profile.

Always ask how the instance was created before inferring behavior.
*(recommendation)*

Replication lag is typically seconds to minutes and can be longer [BT-E08].
"Wait a few seconds then read" is not a consistency strategy.

## Garbage collection is not deletion

Garbage collection is asynchronous, can take **up to a week**, and its timing
cannot be changed. Expired data still appears in reads until it is collected
[BT-E09]. Every read must apply a filter matching the GC rule [BT-E09], and GC
must never be used for compliance-deadline deletion. *(recommendation)*

## Capacity and storage

- SSD vs HDD is **permanent** per instance. Moving a table means a backup and a
  restore into another instance [BT-E11]. HDD is wrong for
  latency-sensitive serving. Choose deliberately. *(inference)*
- Autoscaling bounds [BT-E12]:
  - CPU target 10-80%
  - minimum nodes at least 10% of maximum, and maximum at most 10x minimum
  - storage target defaults to 50% per node
  - with 2x node scaling (the Enterprise Plus default), node counts must be even
- Storage per node is 5 TB on SSD and 16 TB on HDD; use at most 70% [BT-E10].
- Per-node throughput and pricing were not verified [BT-E16]. A capacity
  estimate must come from a load test or a freshly fetched official page.

## IAM

Applications get `roles/bigtable.user` (read-write) or `roles/bigtable.reader`
(read-only), granted at the **table** where possible [BT-E13].
`roles/bigtable.admin` includes data access and instance creation [BT-E13], so
keep it off application identities. `roles/bigtable.viewer` has no data access
[BT-E13]. *(recommendation)*

## Backup and restore

Automated backups keep 7 days by default. A restore always creates a **new
table**, and clients must be re-routed to it. Restores and cross-region copies
Cross-region backup copies, and restores into replicated or other-region
instances, incur network charges [BT-E14]. Retention can be set from 6 hours to
90 days (Enterprise) or 365 days (Enterprise Plus) [BT-E10]. A DR plan must include the re-routing step and
the restore-time performance. *(recommendation)*

## Client libraries

| Language | Package | Evidence |
| --- | --- | --- |
| Go | `cloud.google.com/go/bigtable` | [BT-E15] |
| Java | `com.google.cloud:google-cloud-bigtable` | [BT-E15] |
| Node.js | `@google-cloud/bigtable` | [BT-E15] |
| Python | `google-cloud-bigtable` | [BT-E15] |
| C# | `Google.Cloud.Bigtable.V2` (prerelease, `-Pre`) | [BT-E15] |
| HBase API (Java) | `com.google.cloud.bigtable:bigtable-hbase-1.x` (or `-1.x-hadoop`, `-1.x-shaded`; 2.x versions add an async client) | [BT-E15] |

Authenticate with Application Default Credentials [BT-E15], never embedded
keys. Look up method names in the library reference before writing code. They
are not in this ledger. *(rule)*

## When to push back

- A row key that starts with a timestamp, a sequential ID or a hash.
- A design that needs multi-row atomicity, or read-after-write through an
  asynchronous secondary index [BT-E17].
- Increments or check-and-mutate on a multi-cluster-routing profile.
- "Strong consistency" claimed while traffic uses multiple clusters.
- GC rules used as the only filter for expired data, or as legal deletion.
- HDD for latency-sensitive serving, or a storage-type change planned "later".
- Throughput or cost numbers without a load test or fresh pricing page.
