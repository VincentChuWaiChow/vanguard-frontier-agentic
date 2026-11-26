# Task 3.2: owned watcher continuation

Authority/scope: the approved three-slice continuation in
[the shared contract](no-install-continuation.md). Reuse already delivered
refresh/search/freshness behavior. Full native task dependencies remain unchanged.

Before implementation, require `watcher_lifecycle` and additive `catalog_session`
cases to prove:

- Overflow cannot silently lose the final change: a bounded dirty/rescan signal
  eventually reloads the current bytes without blocking notification callbacks.
- Old-source events cannot refresh or publish after selecting another source.
- Explicit shutdown observes retry-task exit and stops the owned watcher; receiver
  closure/source replacement cannot retain an unowned infinite retry worker.
- Actual Linux filesystem rename/delete/recreate events reach the existing session
  refresh path; deterministic lifecycle tests do not depend on arbitrary sleeps.
- Existing watcher consumers and manual refresh remain compatible. This does not
  prove all blocking-worker shutdown or macOS/Windows/native GUI behavior.

Parent owns integrated regression/lint/scope checks.

## Observed results

- PASS: original closed-receiver/synchronous-shutdown ownership regression failed
  at runtime. New explicit async shutdown retains its join across cancellation.
- PASS: portable lifecycle target has three tests; catalog session target has
  thirteen, including three new native-identity/source-event cases on Linux.
- PASS: eleven watcher unit tests, including deterministic bounded overflow while
  forwarding is blocked and a final-byte/revision oracle. Existing legacy event
  variants/API stay intact; a full-rescan invalidation reuses Workspace(root).
- PASS: real Linux rename/delete/recreate notifications feed the shared refresh
  path; old-generation events cannot publish after source replacement.
- PASS: explicit shutdown observes retry/forward worker exit, synchronous unwatch
  acknowledgments and debouncer callback-thread stop off the async executor.
- UNVERIFIED: notify's internal backend-thread exit has no public join. Legacy
  synchronous shutdown/drop requests finite cleanup but cannot acknowledge it and
  requires a live runtime. Other native hosts, UI and full 4.7 shutdown are unrun.

Raw logs under ignored `tools/vfa-tui/target/no-install-continuation/` include
`watcher-red-runtime.log`, `watcher-green-{default,engine}.log` and
`watcher-unit-green.log`. Parent full regressions/lint/packaging pass.
Status: READY FOR REVIEW for the owned-watcher continuation, not the full task.
