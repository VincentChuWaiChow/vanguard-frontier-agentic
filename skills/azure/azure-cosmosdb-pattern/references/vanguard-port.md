# Port record and evidence index

This skill is ported from the canonical copy in the private repository
`VincentChuWaiChow/agentic-vantage` (`skills/azure-cosmosdb-pattern`, commit `728a4aa`, version 1.0.0).
**agentic-vantage is the source of truth**: make content changes there first, then
re-sync this directory and re-run the gates here.

## Local differences from upstream

- `SKILL.md` frontmatter adds `allowed-tools` (Read, Grep, Glob and the two offline
  helper scripts only) and `metadata.author`, `updated` and `category`, which this
  repository's skill schema requires.
- `metadata.json`, this file and [the evidence ledger](evidence-ledger.md) exist only here.
- `scripts/recovery_guard.py` rejects an in-account restore whose target resource group
  differs from the source account's resource group (Codex review finding);
  `evals/test_package.py` covers it. Apply upstream too.
- Three corrections, to apply upstream too:
  - `commands/restore/continuous-deleted-resource.md` states Microsoft's limit of
    three active restore operations per account [CP-E354].
  - `runtime-contract.md` notes that Microsoft pages disagree on whether
    ReadConsistencyStrategy is direct-mode only [CP-E124].
  - `language-adapters.md` names the documented Python method `query_items`; the
    upstream text named `query_iterable`, a sample variable [CP-E174].

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

### Modeling, indexing, runtime contract, vectors, change feed and SDK adapters

Covers modeling.md, indexing-and-capacity.md, runtime-contract.md, data-types-and-vectors.md, embedding-integrations.md, change-feed-and-integrations.md, language-adapters.md, workflow.md, the design, refactor, performance-test, select and feature-plan commands and the runtime assets.

[CP-E101] [CP-E102] [CP-E103] [CP-E104] [CP-E105] [CP-E106] [CP-E107] [CP-E108]
[CP-E109] [CP-E110] [CP-E111] [CP-E112] [CP-E113] [CP-E114] [CP-E115] [CP-E116]
[CP-E117] [CP-E118] [CP-E119] [CP-E120] [CP-E121] [CP-E122] [CP-E123] [CP-E124]
[CP-E125] [CP-E126] [CP-E127] [CP-E128] [CP-E129] [CP-E130] [CP-E131] [CP-E132]
[CP-E133] [CP-E134] [CP-E135] [CP-E136] [CP-E137] [CP-E138] [CP-E139] [CP-E140]
[CP-E141] [CP-E142] [CP-E143] [CP-E144] [CP-E145] [CP-E146] [CP-E147] [CP-E148]
[CP-E149] [CP-E150] [CP-E151] [CP-E152] [CP-E153] [CP-E154] [CP-E155] [CP-E156]
[CP-E157] [CP-E158] [CP-E159] [CP-E160] [CP-E161] [CP-E162] [CP-E163] [CP-E164]
[CP-E165] [CP-E166] [CP-E167] [CP-E168] [CP-E169] [CP-E170] [CP-E171] [CP-E172]
[CP-E173] [CP-E174] [CP-E175] [CP-E176] [CP-E177] [CP-E178] [CP-E179] [CP-E180]
[CP-E181] [CP-E182] [CP-E183] [CP-E184] [CP-E185] [CP-E186] [CP-E187] [CP-E188]
[CP-E189] [CP-E190] [CP-E191]

### Access, encryption, monitoring, provisioning, recovery and replication

Covers access-control.md, encryption.md, monitoring.md, provisioning.md, recovery.md, replication.md, the Azure CLI, audit, backup, restore, recovery-target and help commands and the ARM and provisioning assets.

[CP-E301] [CP-E302] [CP-E303] [CP-E304] [CP-E305] [CP-E306] [CP-E307] [CP-E308]
[CP-E309] [CP-E310] [CP-E311] [CP-E312] [CP-E313] [CP-E314] [CP-E315] [CP-E316]
[CP-E317] [CP-E318] [CP-E319] [CP-E320] [CP-E321] [CP-E322] [CP-E323] [CP-E324]
[CP-E325] [CP-E326] [CP-E327] [CP-E328] [CP-E329] [CP-E330] [CP-E331] [CP-E332]
[CP-E333] [CP-E334] [CP-E335] [CP-E336] [CP-E337] [CP-E338] [CP-E339] [CP-E340]
[CP-E341] [CP-E342] [CP-E343] [CP-E344] [CP-E345] [CP-E346] [CP-E347] [CP-E348]
[CP-E349] [CP-E350] [CP-E351] [CP-E352] [CP-E353] [CP-E354] [CP-E355] [CP-E356]
[CP-E357] [CP-E358] [CP-E359] [CP-E360] [CP-E361] [CP-E362] [CP-E363] [CP-E364]
[CP-E365] [CP-E366] [CP-E367] [CP-E368]

## Unconfirmed and conflicting claims

| Evidence | Status | Claim |
| --- | --- | --- |
| [CP-E106] | UNVERIFIED | `id` is a string (references/modeling.md:3-4; data-types:8-9) |
| [CP-E124] | CONFLICT | ReadConsistencyStrategy is direct mode only (runtime-contract.md:6) |
| [CP-E125] | UNVERIFIED | Latest Committed / Global Strong offered by ReadConsistencyStrategy (runtime-contract.md:6) Note: the orchestrator's 2026-11-09 mechanical re-check could not confirm this quote on the fetched page (dynamic content, code block or bot protection); treat as unconfirmed. |
| [CP-E136] | UNVERIFIED | TTL relative to server `_ts` (runtime-contract.md:25) |
| [CP-E142] | UNVERIFIED | Not every Functions binding/processor has parity (all-versions-and-deletes) (change-feed.md:6) Note: the orchestrator's 2026-11-09 mechanical re-check could not confirm this quote on the fetched page (dynamic content, code block or bot protection); treat as unconfirmed. |
| [CP-E158] | CONFLICT | Vector policy/index mutability differs between pages (skill says docs vary) (data-types:27; feature-plan.md:10). Note: skill's "docs vary" statement is accurate |
| [CP-E161] | UNVERIFIED | If-Match used for conditional writes (replace/delete); patch via If-Match (runtime-contract.md:8; REST (language-adapters.md:94-95)). Note: patch part |
| [CP-E169] | UNVERIFIED | `response.headers?.["x-ms-session-token"]` read from a JS FeedResponse (assets/runtime.mjs:35). Note: likely returns undefined; see fix list |
| [CP-E170] | UNVERIFIED | `container.item(id, pk).replace(body, {accessCondition: {type: "IfMatch", condition: etag}})` (runtime.mjs:43-44). Note: shape ok, literal `"IfMatch"` unconfirmed |
| [CP-E172] | UNVERIFIED | 429 error carries `error.code` 429 and `error.headers["x-ms-retry-after-ms"]` in JS SDK (runtime.mjs:55). Note: JS error shape |
| [CP-E178] | UNVERIFIED | Java: CosmosQueryRequestOptions.setPartitionKey; container.queryItems(...).byPage(token, pageSize); CosmosItemRequestOptions.setIfMatchETag (language-adapters.md:39-46) Note: the orchestrator's 2026-11-09 mechanical re-check could not confirm this quote on the fetched page (dynamic content, code block or bot protection); treat as unconfirmed. |
| [CP-E179] | UNVERIFIED | No first-party NoSQL PHP SDK (language-adapters.md:14). Note: absence, consistent with skill's wording "not identified" Note: the orchestrator's 2026-11-09 mechanical re-check could not confirm this quote on the fetched page (dynamic content, code block or bot protection); treat as unconfirmed. |
| [CP-E312] | UNVERIFIED | Key Vault must be publicly accessible or allow trusted-service bypass; private-endpoint-only vault not sufficient (references/encryption.md:27-30) Note: the orchestrator's 2026-11-09 mechanical re-check could not confirm this quote on the fetched page (dynamic content, code block or bot protection); treat as unconfirmed. |
| [CP-E317] | UNVERIFIED | Names, indexed paths and partition-key values are not all hidden by CMK (references/encryption.md:47-48) Note: the orchestrator's 2026-11-09 mechanical re-check could not confirm this quote on the fetched page (dynamic content, code block or bot protection); treat as unconfirmed. |
| [CP-E320] | UNVERIFIED | ARM API version 2025-04-15 exists for databaseAccounts, sqlDatabases, containers (references/provisioning.md:7; assets/account.arm.json:11,29,36) Note: the orchestrator's 2026-11-09 mechanical re-check could not confirm this quote on the fetched page (dynamic content, code block or bot protection); treat as unconfirmed. |
| [CP-E324] | CONFLICT | Account name 3-44 chars, lowercase letters/numbers/hyphens (assets/account.arm.json:5 (maxLength 44); assets/provision.sh:10) |
| [CP-E326] | UNVERIFIED | `az cosmosdb create` flags `--kind GlobalDocumentDB`, `--locations`, `--default-consistency-level`, `--backup-policy-type`, `--continuous-tier`, `--enable-automatic-failover`, `--enable-multiple-write-locations`, `--disable-local-auth`, `--public-network-access DISABLED`, `--minimal-tls-version Tls12` (assets/provision.sh:21-25) Note: the orchestrator's 2026-11-09 mechanical re-check could not confirm this quote on the fetched page (dynamic content, code block or bot protection); treat as unconfirmed. |
| [CP-E332] | UNVERIFIED | Strong consistency and multi-region writes not simultaneously supported (references/replication.md:6-7) Note: the orchestrator's 2026-11-09 mechanical re-check could not confirm this quote on the fetched page (dynamic content, code block or bot protection); treat as unconfirmed. |
| [CP-E340] | UNVERIFIED | Deleted container/database snapshots retained 30 days; shared-throughput backup/restore is database-scope (references/commands/backup/periodic.md:21-25) Note: the orchestrator's 2026-11-09 mechanical re-check could not confirm this quote on the fetched page (dynamic content, code block or bot protection); treat as unconfirmed. |
| [CP-E343] | UNVERIFIED | Tier change/migration shortens usable history; shorter tier loses older recovery points; new tier does not create older backups (references/commands/backup/continuous.md:9-10,13) Note: the orchestrator's 2026-11-09 mechanical re-check could not confirm this quote on the fetched page (dynamic content, code block or bot protection); treat as unconfirmed. |
| [CP-E352] | UNVERIFIED | Change feed restarts from beginning; only post-restore lifetime events; pre-deletion events not propagated (references/commands/restore/continuous-deleted-resource.md:42-45) Note: the orchestrator's 2026-11-09 mechanical re-check could not confirm this quote on the fetched page (dynamic content, code block or bot protection); treat as unconfirmed. |
| [CP-E355] | UNVERIFIED | `az cosmosdb restore` flags: `--account-name` (source), `--target-database-account-name`, `--location`, `--restore-timestamp`, `--resource-group`, `--disable-ttl`, `--disable-local-auth`, `--public-network-access DISABLED` (references/commands/restore/continuous-new-account.md:12-16) Note: the orchestrator's 2026-11-09 mechanical re-check could not confirm this quote on the fetched page (dynamic content, code block or bot protection); treat as unconfirmed. |
