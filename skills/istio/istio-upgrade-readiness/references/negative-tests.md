# Upgrade counterexamples

| Case | Required behavior |
| --- | --- |
| UPGRADE-01 | Missing component inventory is unresolved readiness |
| UPGRADE-02A | Control plane one minor ahead is a different case from data plane ahead |
| UPGRADE-02B | Numeric skew within bounds is not full support certification |
| UPGRADE-03 | No universal gateways-first sequence |
| UPGRADE-04 | No patch-only claim for all in-place upgrades |
| UPGRADE-05 | A changed storage flag does not migrate persisted objects |
| UPGRADE-06 | Canary revisions do not isolate shared CRDs or all extensions |

Use synthetic versions in helper tests only to verify comparison arithmetic. Select actual supported releases explicitly for live integration tests; no installed-version support is asserted by the synthetic cases.
