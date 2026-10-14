# Final report

## Outcome

Every residual finding from the two triage passes is now either closed with
paired probes or explicitly left open with a recorded reason. Fourteen findings
were closed across this session; four items remain open, three of them requiring
an author decision rather than an implementation.

## Closed in this run

| Finding | Change | Decisive probe |
| --- | --- | --- |
| N1 | Export marker emitted for `.toml` and `.md` | export to require_asset passes; stripping the marker fails it again |
| P1 | Catalog carries `trust_matrix`; undeclared posture violates | undeclared MCP no longer satisfies the strictest boundary |
| T2 | Reviewed expectations frozen; compare before write | changed answer refused, tree untouched |
| SEC1, SEC2 | New `validate:tracked-secrets` blocking gate | canary in `docs/` fails; `<FAKE>` line passes |

## Open

- kiro-cli `.json` export confirmation.
- `accounting` and `dotnet` fixture drift.
- Advisory root scanner baseline before it can block.
- The parked `kind-rbac-preflight.yaml` workflow, deferred for CI cost and for
  needing an action digest from a repository outside session scope.

## Note on evidence

Two separate probe failures during this run were caused by the parent's own test
setup rather than by the code under test: a malformed canary one character short
of the AWS key pattern, and a fixture regeneration that dirtied the tree twice
before the compare-before-write restructure. Both are recorded because a probe
that fails for the wrong reason is indistinguishable from a passing fix unless
the cause is chased down.
