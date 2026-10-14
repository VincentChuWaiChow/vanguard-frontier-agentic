# Property tests result

Ownership: `tests/fuzz-properties.test.mjs` only. Existing tests preserved.

Parent verification: `rtk proxy node tests/fuzz-properties.test.mjs`, reviewed
escalation because the sandbox blocks child processes: exit 0, all PASS lines.
P1-P5 each use 200 runs with seed `20261014`, production core/parser/render
exports, and actual audit appends. P3 samples small event sequences for bounded
subprocess cost; it is not exhaustive proof of all sequence lengths.

The approved follow-up adds an anchored nested/argv property: 200 seeded runs
against production exports, passing with the rest of the final full suite. No savings or
unmeasured pass-at-k metrics are claimed.
