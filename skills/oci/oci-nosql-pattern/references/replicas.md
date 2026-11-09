# Global Active Tables are availability, not backup

Use the NDCS Global Active Tables workflow, not on-prem multi-region agents or
DynamoDB global-table flags. Verify eligible table/schema state, supported region
set and exact SDK/API before planning a replica. Schema freezing constrains later
evolution; confirm regional write conflict semantics and application conflict
resolution rather than promising cross-region serializable transactions.

Replica initialization is asynchronous and must finish before traffic/failover.
DDL scope differs: index/TTL/storage changes can propagate, while read/write units
and capacity mode are regional. Absolute expiration is replicated. Dedicated hosted
environments currently document no Global Active Tables support: re-check eligibility
before a combined CMEK/replica plan. Measure lag and test regional client routing,
identity/key/network readiness and failback. Replication can propagate accidental
deletes/corruption; retain independent exports and prove restoration separately.

[Global Active Tables](https://docs.oracle.com/en/cloud/paas/nosql-cloud/gasnd/index.html),
[hosted environment limitation](https://docs.oracle.com/en/cloud/paas/nosql-cloud/mmmmq/), checked 2026-11-08.
