# Evidence ledger — Huawei Cloud GeminiDB

Every material claim in [geminidb-design-guide.md](geminidb-design-guide.md)
cites one row below as `[GDB-Exx]`. `tests/validate-skill-references.py` fails
the build if a citation has no row, a row is never cited, or a source is not on
support.huaweicloud.com.

Scope: **GeminiDB** (multi-model NoSQL), with design depth for the **GeminiDB
Cassandra API**. DDS (Document Database Service) is covered by
`huawei-gaussdb-rds-dba`. `VERIFIED` = the section was retrieved on the
retrieval date and supports the claim as worded; it is documentation
verification, not a behavioral test. Provider marketing statements are recorded
as claims to test, not as proven behavior.

| ID | Claim | Source | Section | Retrieved | Status |
| --- | --- | --- | --- | --- | --- |
| GDB-E01 | GeminiDB is a distributed, multi-model NoSQL service with decoupled storage and compute, compatible with Redis, DynamoDB, Cassandra, HBase, InfluxDB and MongoDB APIs. | <https://support.huaweicloud.com/intl/en-us/productdesc-nosql/nosql_introduction.html> | What Is GeminiDB? | 2026-11-08 | VERIFIED |
| GDB-E02 | The GeminiDB Mongo API has reached End of Support: new instances cannot be created, no technical support (features, vulnerability fixes, patches, tickets) is provided, and its SLA no longer applies. | <https://support.huaweicloud.com/intl/en-us/productdesc-nosql/nosql_introduction.html> | Note on GeminiDB Mongo API | 2026-11-08 | VERIFIED |
| GDB-E03 | API selection: Redis API (key-value), DynamoDB-Compatible API (key-value, over HTTPS; Huawei describes it as completely compatible with Amazon DynamoDB for migration without refactoring), Cassandra API (wide-column, CQL), HBase API (wide-column), Influx API (time series). | <https://support.huaweicloud.com/intl/en-us/productdesc-nosql/nosql_introduction.html> | How Do I Select an API? | 2026-11-08 | VERIFIED |
| GDB-E04 | GeminiDB Cassandra rules: storing BLOBs in the database is not recommended (use OBS or dedicated file storage); a row's key plus value at most 64 KB and average row size at most 10 KB; define data-deletion policies so data does not accumulate without bound. | <https://support.huaweicloud.com/intl/en-us/cassandraug-nosql/nosql_11_0205.html> | Basic Design, Rules 1-3 | 2026-11-08 | VERIFIED |
| GDB-E05 | Partition keys must distribute workload evenly to avoid data skew, BigKey (too much data in one partition, up to OOM) and HotKey (bursty access to one key) issues; one partition key may hold at most 100,000 rows and 100 MB. | <https://support.huaweicloud.com/intl/en-us/cassandraug-nosql/nosql_11_0205.html> | Basic Design, Rules 4-6 | 2026-11-08 | VERIFIED |
| GDB-E06 | GeminiDB Cassandra writes are strongly consistent, including concurrent writes and across tables, but transactions are not supported; DRS migration is eventually consistent. | <https://support.huaweicloud.com/intl/en-us/cassandraug-nosql/nosql_11_0205.html> | Basic Design, Rule 7 | 2026-11-08 | VERIFIED |
| GDB-E07 | Keep an instance at or below 50 nodes and shard vertically or horizontally beyond that; avoid tombstones by using TTL instead of mass deletes, deleting by primary key, deleting at most 1,000 rows at a time within a partition key, and not repeatedly deleting large ranges in one partition. | <https://support.huaweicloud.com/intl/en-us/cassandraug-nosql/nosql_11_0205.html> | Basic Design, Rules 8-9 | 2026-11-08 | VERIFIED |
| GDB-E08 | Sizing guidance: recommended no more than 50 billion records per table, no more than 100 tables per database and 20-50 fields per table; add N/2 redundant nodes for fault scenarios and keep per-node CPU at or below 50% under typical load. | <https://support.huaweicloud.com/intl/en-us/cassandraug-nosql/nosql_11_0205.html> | Design Suggestions 1-2 | 2026-11-08 | VERIFIED |
| GDB-E09 | IAM system policies for GeminiDB (project-level): `GeminiDB FullAccess` (full permissions) and `GeminiDB ReadOnlyAccess` (read-only). | <https://support.huaweicloud.com/intl/en-us/usermanual-permissions/iam_01_0001.html> | System-defined Policies/Roles | 2026-11-08 | VERIFIED |
| GDB-E10 | Billing is instance specifications plus storage, optional backup storage beyond a free allowance equal to purchased storage (then OBS rates), and public-network traffic; modes are pay-per-use (hourly) or yearly/monthly and can be switched. | <https://support.huaweicloud.com/intl/en-us/productdesc-nosql/nosql_01_0100.html> | Billing | 2026-11-08 | VERIFIED |
| GDB-E11 | Per-request CQL consistency levels, fine-grained (custom) GeminiDB permissions, region availability and SDK/driver versions were not retrieved in this pass. | <https://support.huaweicloud.com/intl/en-us/productdesc-nosql/nosql_introduction.html> | What Is GeminiDB? | 2026-11-08 | UNVERIFIED |
| GDB-E12 | GeminiDB Cassandra supports automated backups (retention set in days, deleted on expiry) and manual full backups (kept until deleted, even after instance deletion); full and incremental backups are enabled by default; backup data cannot be downloaded or exported; cross-region backups store automated full backups in a destination region for restore to a new instance there. | <https://support.huaweicloud.com/intl/en-us/cassandraug-nosql/nosql_03_0229.html> | Data Backup overview | 2026-11-08 | VERIFIED |
| GDB-E13 | Point-in-time restore replays incremental backups over the last full backup, requires the automated backup policy to be enabled, and restores into a new, independent instance with the same API, type, version and CPU type. | <https://support.huaweicloud.com/intl/en-us/cassandraug-nosql/nosql_02_0116.html> | Restoring to a Specified Point in Time | 2026-11-08 | VERIFIED |
