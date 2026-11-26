# Task 4.3: evidence-only admission and recovery preparation

Authority/scope: the approved three-slice continuation in
[the shared contract](no-install-continuation.md), including the narrow additive
journal-query extension identified in the assessment. No real helper execution,
native lifecycle completion, schema migration or full-task completion.

Before implementation, require `evidence_recovery` to prove:

- Actual SQL/COMMIT failure yields no start receipt and zero inert action calls.
- Known action success survives terminal persistence failure as success plus audit
  failure; pending evidence remains bounded and blocks further admission.
- Evidence retry preserves identical IDs/sequence/payload, deduplicates committed
  events and never re-invokes the action; conflicts/permanent errors stop retries.
- Bounded nonterminal recovery queries run through the owned journal worker.
  A new exclusive owner classifies old nonterminal records interrupted with
  unknown effects, never successful or replayed; live owners cannot be recovered.
- Real temp SQLite transactions, two-process ownership and owner-death fixtures
  exercise behavior, not fabricated receipts or mock-only persistence assertions.

Keep existing `journal_commits` and `profile_ownership` contracts unchanged.
Parent owns integrated regression/lint/scope checks.

## Observed results

- PASS: initial compiled runtime RED invoked a probe despite start SQL failure;
  the implemented receipt gate leaves the probe count zero. Expanded tests were
  added afterward; no claim that every case had preimplementation runtime RED.
- PASS: sixteen evidence tests in each feature mode, alongside unchanged thirteen
  journal and eight ownership tests; actual SQL and deferred COMMIT failures,
  known outcome/audit independence, retry identity/conflicts and fixed bounds.
- PASS: keyset pages do not skip rows when statuses change. Orphan/corrupt current
  projections fail closed; recovery records unknown effects, never action replay.
- PASS: real process exclusion/owner death and duplicate controllers sharing one
  profile owner. The minimal registry guard remains held through journal join.
- PASS: callback unwind retains bounded unknown evidence without production
  catch_unwind. Strict lint passes after removing a needless test-only borrow.
- UNVERIFIED: real helper/process-tree/native lifecycle integration and full task
  dependencies. Raw journal edits and same-user database editors are not fenced.

`close()` acknowledges worker closure, not an evidence flush; choosing to close
with pending data discards that in-memory observation. Restart can recover only
interrupted/unknown effects. Raw logs: `evidence-red.log`,
`evidence-green-{default,engine}.log` and parent `postreview-*.log` under ignored
`tools/vfa-tui/target/no-install-continuation/`.
Status: READY FOR REVIEW for evidence-only preparation, not the full task.
