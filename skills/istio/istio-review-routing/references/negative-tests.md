# Routing counterexamples

| Case | Required behavior |
| --- | --- |
| ROUTE-01 | Simple authorization review selects its owner, not all specialists |
| ROUTE-02 | Ambient L7 access question resolves path before access semantics |
| ROUTE-03 | Missing specialist does not become a fabricated completed review |
| ROUTE-04 | An unresolved required decision prevents aggregate approval |
| ROUTE-05 | A mutation request returns a live-guard handoff, not auto-dispatch |

The included routing cases are a portable test corpus. Adapt them to VFA's actual taxonomy schema and native `evaluate(task, taxonomy)` grader before registering the maestro. Do not claim the portable cases already satisfy that unavailable native contract.
