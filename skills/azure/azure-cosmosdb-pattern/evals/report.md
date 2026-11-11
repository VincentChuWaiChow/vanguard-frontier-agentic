# Local evaluation report — 1.0.0, 2026-11-08

Scope: new azure-cosmosdb-pattern package only. One bounded implementation worker;
parent owns independent review/integration/publication. Model/effort inherited;
actual runtime identity and effective tool isolation UNVERIFIED. No nested agents.

| Criterion | Result | Evidence |
|---|---|---|
| 1. Exact identity, compact lazy entry, human tables | PASS | SKILL65 lines/585 words; metadata/README/link tests; creator validation |
| 2. Nine commands, isolated audit/recovery/features | PASS | Selector normal/slash namespace and every method; selected path existence and no sibling preload tests |
| 3. Safe real selector, bounded plans, fail-closed inputs | PASS | Stdlib import/no-read guard; actual subprocess negative tests; execution none |
| 4. NoSQL native correctness | PASS locally/docs | Partition/id/RU/index/consistency/ETag/TTL/migration guides; JS11 injected-client tests |
| 5. RBAC/encryption lifecycle | PASS locally/docs | Separate control/data roles and Entra; service-managed baseline; CMK/default identity/Key Vault/recovery dependencies |
| 6. Vector/embedding types and integrations | PASS locally/docs | Native policy/index fixtures, finite/dimension/range/sparse tests; generation/source refs; dated drift constraints |
| 7. Complete aligned examples and portable adapters | PASS locally | ARM account/DB/container, exact fakeCLI execution/equivalence/negative fixtures; language/REST matrix |
| 8. Native recovery methods and safety | PASS locally/docs | Continuous7/30/35 vs pinned subset, periodic Support, deleted-resource lifetime; guard scope/time/target/key/TTL negatives and actual restore-snippet fakeCLI |
| 9. June/December reminder | PASS | Explicit-date helper, initially2026-12-01, partial update doesn't clear overdue tests; no writes/scheduling |
| 10. Sources/MCP and executable evals | PASS locally | Actual Context7 + MicrosoftDocs + Azure docs code samples; source ledger; JSON/link/metadata/router/fakeCLI/runtime tests and semantic prompts |

Commands run from repository root (all final exit0):

```text
python3 -B -m unittest discover -s skills/azure-cosmosdb-pattern/evals -p 'test_*.py' -v
  25 tests passed (negative matrices also use subtests).
node skills/azure-cosmosdb-pattern/evals/runtime.test.mjs
  11 native node:test cases passed.
python3 -B skills/azure-cosmosdb-pattern/scripts/validate_assets.py
  PASS local invariants; Azure acceptance UNVERIFIED.
python3 -B "$HOME/.codex/skills/skill-creator/scripts/quick_validate.py" skills/azure-cosmosdb-pattern
  Skill is valid.
bash -n skills/azure-cosmosdb-pattern/assets/provision.sh
  PASS syntax.
git diff --check -- skills/azure-cosmosdb-pattern
  PASS tracked diff; parent owns staged-new-file whitespace gate.
```

Commands were invoked through the required RTK wrapper. Direct Node invocation
enumerated all11 tests; this environment's isolated `node --test` initially
reported only the file. A deliberate throwing canary failed as expected. Initial
JS test-fixture syntax errors were corrected before final passing run. Parent
independently verified Node with `--test --test-isolation=none`.

Parent/expert-identified defects fixed and regression-guarded: verbs-first Azure
CLI scope placement; sparse vector holes; no silent full-account recovery when
subset requested; same-account token/feed lifetime exception; native CLI partial
restore shorthand instead of ARM JSON. Expert/source review is evidence, not a
proof of all possible behavior.

UNVERIFIED: Azure CLI/Bicep/service schema acceptance (az absent, no install), SDK
compilation/all-language parity, live account/data access, effective RBAC/key/
network posture, measured performance/quality/cost, actual backup/restore/cutover,
host discovery/selective loading and savings. No live discovery/mutation, SDK
installation, credentials, load/drill, scheduling, memory update, commit or push
was performed by this worker. No pass@k or zero-RPO/bulletproof claims.
