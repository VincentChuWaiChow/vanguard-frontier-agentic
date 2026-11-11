# Acceptance contract — azure-cosmosdb-pattern 1.0.0

Defined before implementation, 2026-11-08. Scope: Azure Cosmos DB **for NoSQL**,
documentation, local selectors/examples/tests; no provider deployment or data access.

| Criterion | Deterministic grader / semantic evidence |
|---|---|
| Exact name/version, <=90 lines/900 words, tabular lazy entry | package metadata/link tests; quick_validate.py |
| Nine isolated commands and feature/method-specific routing | selector subprocess/negative/import-side-effect tests |
| Safe offline defaults, positive performance caps, no execution | selector tests; no network/subprocess/filesystem imports |
| Native partition/id/RU/ETag/token/batch/TTL semantics | runtime injected examples and adversarial tests; domain review |
| Control/data RBAC and key dependencies | access/encryption guides; ARM/CLI invariants |
| Current numeric/vector/embedding constraints | vector validator negative tests and dated primary sources |
| Complete aligned native IaC/CLI + adapters | JSON parse, ARM resource/CLI fake-command equivalence; SDK support matrix |
| Distinct continuous/periodic/deleted-resource recovery | isolated routes, recovery guard tests; documentation evidence |
| June/December due ledger never self-advances | explicit-date helper and overdue/partial-review tests |
| Sources/MCP and meaningful regressions | source ledger, prompts, fakeCLI/snippet/runtime suite |

Run from repository root:

```bash
python3 -m unittest discover -s skills/azure-cosmosdb-pattern/evals -p 'test_*.py' -v
node skills/azure-cosmosdb-pattern/evals/runtime.test.mjs
python3 "$HOME/.codex/skills/skill-creator/scripts/quick_validate.py" skills/azure-cosmosdb-pattern
git diff --check -- skills/azure-cosmosdb-pattern
```

All-language compilation, CLI/service acceptance, actual RBAC/network/key access,
load/latency/cost results, restore/cutover and effective host lazy loading remain
UNVERIFIED. Never infer these from local tests. A waiter is not workload proof.
Semantic grader uses evals/prompts.json; report each case PASS/MISS/UNVERIFIED,
not invented pass@k statistics. Parent owns preservation/integration grading.
