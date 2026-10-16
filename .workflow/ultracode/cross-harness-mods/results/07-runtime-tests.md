# Runtime tests result

Ownership: `tests/mods.test.mjs` only. Native Codex settings inherited; actual
model/effort and token usage unverified. Parent inspected the tests and ran them.

- Sandbox subprocess checks were invalidated by `EPERM`; no pass inferred.
- RED, reviewed escalation: `rtk proxy node --test tests/mods.test.mjs`:
  58 tests, 51 pass, 7 fail, exit 1.
- Parent's one bounded repair addressed unsupported events, parser diagnostic
  disclosure, audit session locking/framing, and generator symlink access.
- GREEN, same command: 58 tests, 58 pass, 0 fail, exit 0, about 8.5 seconds.
- Cases use production exports and actual command adapters for all supported
  harness/events, disposable fixtures, concurrency, malformed input and drift.

This historical 58-test run did not cover anchored extensions. The approved
follow-up adds focused coverage; the final suite has 60 passing tests, exit 0.
