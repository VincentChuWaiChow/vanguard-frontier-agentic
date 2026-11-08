# Evidence ledger — Huawei Cloud DDS (Document Database Service)

Material DDS claims in [SKILL.md](../SKILL.md) cite one row below as
`[HW-DDS-Exx]`. `tests/validate-skill-references.py` fails the build if a
citation has no row, a row is never cited, or a source is not on
support.huaweicloud.com. GaussDB and RDS claims in this skill are not yet
ledgered.

Scope: **DDS**, Huawei Cloud's MongoDB-compatible service. The GeminiDB Mongo
API has reached End of Support; see `huawei-geminidb-architect`. `VERIFIED` =
the section was retrieved on the retrieval date and supports the claim as
worded; it is documentation verification, not a behavioral test.

| ID | Claim | Source | Section | Retrieved | Status |
| --- | --- | --- | --- | --- | --- |
| HW-DDS-E01 | Once a DDS shard key is set it cannot be changed; ranged sharding suits range queries, but a monotonically increasing key such as auto-generated `_id` concentrates inserts; hashed sharding distributes data more evenly. | <https://support.huaweicloud.com/intl/en-us/usermanual-dds/dds_01_0061.html> | Sharding; Selecting a Shard Key | 2026-11-08 | VERIFIED |
| HW-DDS-E02 | DDS system policies: `DDS FullAccess`, `DDS ReadOnlyAccess`, `DDS ManageAccess` (all operations except deleting DDS resources) and the `DDS Administrator` role (used with Tenant Guest and Server Administrator). | <https://support.huaweicloud.com/intl/en-us/usermanual-permissions/iam_01_0001.html> | System-defined Policies/Roles | 2026-11-08 | VERIFIED |
