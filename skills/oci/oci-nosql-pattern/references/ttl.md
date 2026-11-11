# TTL is expiration, not backup

Table default TTL uses hours/days in NoSQL DDL/SDK; the example uses 30 DAYS.
Default 0 means no expiration. Individual puts can override expiry. Changing the
table default does not rewrite existing rows' expiration; expired rows are no
longer available. Do not claim a DynamoDB epoch attribute or delayed-delete read
semantics. Check the chosen SDK options for whether a put preserves or updates
existing TTL; test insert/update/default0 and boundary rounding explicitly.

Migrations/recovery preserve original absolute expiry where intended. Re-applying
30 days at restore resurrects/re-retains data; explicitly skip rows already expired.
Migrator `includeTTL` and `ttlRelativeDate` need release-specific review; do not
shift the TTL reference date silently. Migrator may round imported expiry to
hour/day boundaries; prove retention tolerances rather than claim byte-exact expiry. Retention/legal deletion policy owns any
exception. Global Active Tables replicate absolute expiration; replica lag is not
an opportunity to extend retention. TTL is neither CDC nor historical recovery.

[Table TTL](https://docs.oracle.com/en/cloud/paas/nosql-cloud/mtiuc/) and
[replica expiration](https://docs.oracle.com/en/cloud/paas/nosql-cloud/gasnd/index.html), checked 2026-11-08.
