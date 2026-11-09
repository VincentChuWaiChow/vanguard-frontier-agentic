# Port record and evidence index

This skill is ported from the canonical copy in the private repository
`VincentChuWaiChow/agentic-vantage` (`skills/oci-nosql-pattern`, commit `728a4aa`, version 1.0.0).
**agentic-vantage is the source of truth**: make content changes there first, then
re-sync this directory and re-run the gates here.

## Local differences from upstream

- `SKILL.md` frontmatter adds `allowed-tools` (Read, Grep, Glob and the two offline
  helper scripts only) and `metadata.author`, `updated` and `category`, which this
  repository's skill schema requires.
- `metadata.json`, this file and [the evidence ledger](evidence-ledger.md) exist only here.

## Sync procedure

1. Copy the upstream directory over this one, keeping the local differences above.
2. Re-check every changed claim against the provider documentation and update the
   ledger rows.
3. Run the skill's evals (`npm run validate:nosql-pattern-evals`), then the
   repository gates.

## Verification summary

Each material claim was checked against the provider page it cites, or the closest
official page, on 2026-11-09. No claim was contradicted. The orchestrator then re-checked every
VERIFIED quote mechanically on the cited page; rows it could not confirm were
downgraded to UNVERIFIED. UNVERIFIED and CONFLICT rows are listed below; treat those
statements as unconfirmed until re-checked.

## Evidence by topic

### All references and assets

Covers every file under references/ and the Terraform, DDL, JSON, shell and Node assets.

[OCP-E101] [OCP-E102] [OCP-E103] [OCP-E104] [OCP-E105] [OCP-E106] [OCP-E107]
[OCP-E108] [OCP-E109] [OCP-E110] [OCP-E111] [OCP-E112] [OCP-E113] [OCP-E114]
[OCP-E115] [OCP-E116] [OCP-E117] [OCP-E118] [OCP-E119] [OCP-E120] [OCP-E121]
[OCP-E122] [OCP-E123] [OCP-E124] [OCP-E125] [OCP-E126] [OCP-E127] [OCP-E128]
[OCP-E129] [OCP-E130] [OCP-E131] [OCP-E132] [OCP-E133] [OCP-E134] [OCP-E135]
[OCP-E136] [OCP-E137] [OCP-E138] [OCP-E139] [OCP-E140] [OCP-E141] [OCP-E142]
[OCP-E143] [OCP-E144] [OCP-E145] [OCP-E146] [OCP-E147] [OCP-E148] [OCP-E149]
[OCP-E150] [OCP-E151] [OCP-E152] [OCP-E153] [OCP-E154] [OCP-E155] [OCP-E156]
[OCP-E157] [OCP-E158]

## Unconfirmed and conflicting claims

| Evidence | Status | Claim |
| --- | --- | --- |
| [OCP-E102] | UNVERIFIED | capacity_mode / capacityMode value "PROVISIONED" is valid (assets/orders.tf:29, assets/table-limits.json:2) |
| [OCP-E104] | UNVERIFIED | No per-table `kms_key_id`/SSE argument exists in `oci_nosql_table` (references/encryption.md:8; references/provisioning.md:15; commands/feature-plan.md:9) Note: the orchestrator's 2026-11-09 mechanical re-check could not confirm this quote on the fetched page (dynamic content, code block or bot protection); treat as unconfirmed. |
| [OCP-E108] | UNVERIFIED | `oci iam tenancy get --tenancy-id` and `oci iam compartment get --compartment-id` exist (assets/create-table.sh:14,16) Note: the orchestrator's 2026-11-09 mechanical re-check could not confirm this quote on the fetched page (dynamic content, code block or bot protection); treat as unconfirmed. |
| [OCP-E134] | UNVERIFIED | NDCS source params readUnitsPercent, includeTTL, requestTimeoutMs; top-level abortOnError (assets/migrator-export.json:8-10) Note: the orchestrator's 2026-11-09 mechanical re-check could not confirm this quote on the fetched page (dynamic content, code block or bot protection); treat as unconfirmed. |
| [OCP-E139] | UNVERIFIED | INTEGER is signed 32-bit, LONG signed 64-bit (references/types-and-vectors.md:3-4) |
| [OCP-E141] | UNVERIFIED | No native vector type / ANN / HNSW / IVF in NDCS (references/types-and-vectors.md:15; embedding-integrations.md:5-8; sources.md:54; commands/select.md:7) |
| [OCP-E143] | UNVERIFIED | Indexes consume the table's storage/write budget; no separate GSI capacity (references/indexes.md:12-14) |
| [OCP-E145] | UNVERIFIED | Parent/child tables share top parent's limits and shard key (references/modeling.md:14-16; replicas) Note: the orchestrator's 2026-11-09 mechanical re-check could not confirm this quote on the fetched page (dynamic content, code block or bot protection); treat as unconfirmed. |
| [OCP-E149] | UNVERIFIED | Node package `oracle-nosqldb` exports Consistency / NoSQLClient (language-adapters.md:66,89; sources.md) Note: the orchestrator's 2026-11-09 mechanical re-check could not confirm this quote on the fetched page (dynamic content, code block or bot protection); treat as unconfirmed. |
| [OCP-E155] | UNVERIFIED | REST has fewer SQL features than native SDKs (language-adapters.md:79-80,199-203; sources.md:11) |
| [OCP-E157] | UNVERIFIED | Table name regex caps at 64 chars (references/limits-in-assets: assets/orders.tf:14, create-table.sh:10) |
