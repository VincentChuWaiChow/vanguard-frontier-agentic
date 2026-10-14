# Approved anchored repair result

User approval: one additional focused repair and commits; no push requested.
One additional native read-only reviewer turn, inherited model/effort UNVERIFIED.

## Evidence

- RED: `node --test --test-name-pattern=anchored tests/mods.test.mjs`: 2 fail,
  0 pass, exit 1. Native redactor test: 17 pass, 2 fail, exit 1.
- Fix: keep original context-first matching; recheck both sliced standalone leaves.
  Only the two return paths and their comment changed in the algorithm.
- GREEN: production regression suite 60 pass, 0 fail, exit 0; native redactor
  19 pass, 0 fail, exit 0. Full native suites total 50 passing tests.
- Seeded anchored nested/argv property passes 200 runs; existing P1-P5 pass.
- Independent production-export probes and injected Codex runtime pass, exit 0.

## Rejected alternative

Raw-first replacement can consume an AWS secret's prefix before its name/value
context is seen, leaving part of the secret. Tests retain both an anchored-prefix
extension case and an overlapping AWS/access-ID builtin case to prevent that regression.

## Remaining limits

No new blocker in focused review. Live installed Kiro/Codex sessions remain unverified.
Existing documented output-schema/key masking and heuristic limitations are unchanged.
