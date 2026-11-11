# OCI NoSQL pattern acceptance contract — 2026-11-08

Defined before implementation; parent contract has ten requirements. Owned surface:
this new skill only. No cloud calls, credentials, installations, schedules or data.

| Check | Observable pass condition | Grader |
|---|---|---|
| Entry | exact singular name, version1.0.0, <=90 lines /900 words, Markdown targets exist | test_package.py |
| Lazy commands | nine command files; help only help; selected audit/recovery method only; no feature indirect leakage | test_package.py selector tests |
| Selector | normal/slash commands; rejects unsupported PITR/vector/streams activation, bad options and nonpositive caps; no file reads/network/subprocess | test_package.py imported and CLI tests |
| Provider model | shard prefix, same-shard ACID only, ABSOLUTE/EVENTUAL, opaque continuation/versions, TTL, no AWS semantics | adversarial prompts + review |
| Security | data/control rights distinct; tenancy/table/region; default managed encryption versus dedicated CMEK | adversarial prompts + review |
| Types/AI | typed numeric arrays are not native vector indexes; model/dimension/tenant contract; honest other-service integration | test_runtime.mjs + review |
| Provisioning | complete matching DDL/Terraform/CLI; no IF NOT EXISTS/adopt/delete fallback; fake CLI target/error gates | test_package.py |
| Recovery | export/import manifest, new destination, no PITR promise, TTL/deletes/delta limits; readiness not recovery proof | test_package.py + prompts |
| Review | explicit date; June/December; next 2026-12-01; overdue never rolls; read-only | test_package.py |
| Evidence | dated primary sources + actual Context7; JSON/link/resource tests, actual shipped snippet fakeCLI/runtime tests | package suite + quick_validate |

Run from repo root:
`python3 -m unittest discover -s skills/oci-nosql-pattern/evals -p 'test_*.py' -v`
`node --test --test-isolation=none skills/oci-nosql-pattern/evals/test_runtime.mjs`
`python3 ~/.codex/skills/skill-creator/scripts/quick_validate.py skills/oci-nosql-pattern`

Local pass does not establish service acceptance, SDK compilation, IAM, backup,
restore, load performance, host registration or selective host loading. Report those
UNVERIFIED. Do not manufacture pass@k from one deterministic run.
