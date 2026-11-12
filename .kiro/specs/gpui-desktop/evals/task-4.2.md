# Task 4.2 evaluation contract

## Scope and activation

User authorized the ready-wave sequence: 4.1/5.1, then 4.2 after 4.1's gates.
This contract is defined before journal implementation; activation waits for
verified profile ownership. Native task 1.4 remains pending. Parent owns shared
module wiring, execution, records and per-task Git commits; one worker owns only
`src/persistence/operations.rs`, `src/persistence/operation_schema.rs` and
`tests/journal_commits.rs` under `tools/vfa-tui`. Preserve other workers' edits.
No task 4.3 action admission/recovery/replay, UI, remote execution, user-profile
mutation, system installs, pushes, legacy migration or arbitrary SQL API.

## Required capability and regression checks

1. A dedicated single writer holds its SQLite connection and an
   `Arc<ProfileOwner>` for the entire connection/task lifetime. Paths alone are
   not ownership. Only the profile's separate `operations.sqlite` is opened;
   no legacy index import/reset/fallback, including after open failures.
2. Distinct application/schema identity and a transactional initial/forward
   schema are verified before mutating existing files. Refuse foreign, legacy,
   corrupt and newer stores without resetting or changing their bytes/sidecars.
   Never set an application ID merely to relabel an unrecognized existing DB.
3. Event insert and operation projection update commit atomically. Return a
   `Result<CommitReceipt, JournalError>` only after successful COMMIT, not enqueue,
   flush or ignored SQL result. Query real persisted events/projections and reopen
   the store after acknowledged commits. Duplicate operation/sequence with the
   same canonical payload is idempotent; conflicting payload is an error.
4. Operation IDs, session IDs, kind/status, sequence and bounded caller-redacted
   payload are validated. Do not infer unexecuted actions, immutable/tamper-proof
   history, privacy filtering or a new retention/deletion policy. Unknown persisted
   state must not silently become success or restart execution.
5. Inject actual SQL execution failure and COMMIT failure using temporary database
   fixtures, not a fake successful sender. Failed transactions leave neither new
   event nor changed projection. Verify actual receipt/row identity on retries.
   Writer/connection errors are explicit and worker/owner cleanup is bounded.
6. Construct a populated legacy-v4 fixture, assert its identity, hash the fixture
   and sidecars before/after refused journal open. A foreign/newer store is not
   modified by WAL/pragma/migration setup. No missing-fixture skips.
7. Test that journal/worker retention keeps the profile busy after the caller
   drops its owner reference; normal close releases it. Test fail-closed busy/open
   errors without weakening the task 4.1 OS lock. No real user database access.
8. Default/no-default `journal_commits`, profile/core/report contracts, fmt and
   strict all-target clippy pass. Compare full regression failures to the existing
   17/11 baseline; verify normal engine closure under both locked consumers and
   package bytes. No GPUI/native success inferred; task 1.6 behavioral parity and
   task 4.7 real worker-shutdown integration remain later obligations.
9. Before its separate local commit, parent runs repository validation, scoped
   spelling/Markdown, exact owned paths/frozen approvals, AST maintenance and
   explicit GitHub no-reply author/committer guards. No push. Keep each other
   task's changes out of this commit.

## Route and graders

One bounded implementation objective, one independent Rust source review and
one parent repair pass. No recursive agents or parallel Cargo commands. Required
test target: `cargo test --manifest-path tools/vfa-tui/Cargo.toml --locked --test
journal_commits`, repeated with `--no-default-features`. Use explicit temporary
profiles and independent row/hash expectations; never actual operator stores.

Status: activated after verified task 4.1 commit
`5bbe9bf921f9799e9e6a62fa2403a1a45845de91`. Scoped Linux journal checks now
pass; full known baseline gates remain non-green and native/lifecycle integration
remains unverified. Use the existing rusqlite dependency, no manifest change.

The initial schema may include a fixture-defined v1-to-v2 session lookup index
migration to exercise genuine transactional forward changes. This is not a claim
that a historical journal version was deployed. Existing rollback-journal stores
are inspected read-only before writable open; unrecognized/WAL-format stores
fail closed rather than initializing sidecars. Use a bounded 32-request channel
and finite SQLite busy timeout. No general SQL/fault-injection API is allowed.

Context7's `/rusqlite/rusqlite` documentation identifies read-only OpenFlags,
fallible consuming `Transaction::commit` and default rollback-on-Drop behavior.
Parent also inspected the installed 0.32.1 `src/transaction.rs`: `commit` executes
COMMIT and propagates failure; Drop calls `finish_`, which rolls back unless the
connection is already in autocommit. These are source facts, not passing tests.
The actual SQL execution/COMMIT/migration failure cases must still demonstrate
real persisted-state rollback on the selected dependency version.


## Results and bounded review

| Criterion | Result | Evidence |
| --- | --- | --- |
| Isolation and ownership | PASS Linux | Dedicated thread retains Connection and Arc owner through actual close; caller drop keeps profile Busy; close/open-error paths release ownership |
| Real commit receipts | PASS | Independently queried committed row IDs and projection, actual reopen, canonical UUID/time/JSON retry and stable receipt; gaps/conflicting duplicates reject |
| SQL and COMMIT failure | PASS | Real projection-abort trigger and deferred-FK COMMIT failure leave neither new event nor advanced projection; retry succeeds only after fault removal |
| Store identity and preservation | PASS | Distinct application/meta/version, private file checks, populated legacy-v4/WAL sidecar hashes, foreign/newer/corrupt/unsupported-WAL/schema refusals without reset |
| Forward migration | PASS fixture-defined | v1 core tables to v2 session index preserve actual rows; injected metadata-update failure rolls DDL/version changes back byte-identically |
| Persisted consistency | PASS | Parent RED reproduced valid-looking false success; eight actual SQL mutation cases now reject mismatched, missing, cross-operation or stale projection links on reads, retries and advances without repair |
| Journal/profile/core/report tests | PASS | Thirteen journal, eight profile and five core tests per mode; eleven default/ten no-default report tests; no missing-fixture skips |
| Rust regression | MISS existing baseline | Full suites exit 101 for exactly the same seventeen default/eleven no-default failures; actual names compared, not only totals; one existing ignored doctest |
| Format/clippy/dependency/package | PASS | Both strict all-target clippy modes, fmt, actual normal dependency closures under both locks and 31 negative fixtures; verified normal package/source bytes and packaged no-default build |
| Repository validation | PASS | Covered Node/catalog/config inputs unchanged from task 5.1; actual full repository gate exits 0 with functioning subprocess pipes |
| Native and lifecycle integration | UNVERIFIED | Other OS/ACL behavior, GPUI/native task 1.4, action admission/recovery, caller redaction, real shutdown integration and legacy MSRV remain separate obligations |

The configured Rust reviewer (`gpt-6.1-sol/high`, read-only default) found that an existing
projection with a valid but false SUCCEEDED status could disagree with its event
and still be returned successfully. Duplicate retries also bypassed projection
validation. Parent reproduced the flaw with the actual Rust API before repair.
The integrated fix validates a projection against its latest referenced event
before reads/retries/advances, without recursive decoders or history repair.
Eight SQL mutation classes exercise status/payload mismatch, wrong/missing or
cross-operation links, missing projection/event and a consistent-looking stale
projection. Old retries against an intact newer projection still pass unchanged.
This detects logical inconsistencies; it is not a tamper-proof-history claim.

Parent also made database/sidecar byte snapshots avoid a second read handle to
the actively locked owner file, preserving the task 4.1 Windows lock semantics.
Native Windows execution remains unverified, not inferred from this test repair.

### Exact commands and evidence

```bash
manifest=tools/vfa-tui/Cargo.toml
# Repeat test/clippy/full commands with --no-default-features.
cargo +stable test --manifest-path "$manifest" --locked --offline \
  --test journal_commits --test profile_ownership \
  --test core_contracts --test report_service_contracts
cargo +stable clippy --manifest-path "$manifest" --locked --offline --all-targets -- -D warnings
cargo +stable test --manifest-path "$manifest" --locked --offline --no-fail-fast
cargo +stable fmt --manifest-path "$manifest" --check
python3 tools/vfa-tui/tests/check_feature_boundary.py --manifest-path "$manifest" --self-test
python3 tools/vfa-tui/tests/check_feature_boundary.py --manifest-path "$manifest" \
  --consumer-manifest tools/vfa-desktop/Cargo.toml
npm run validate
```

Parent logs: `vfa-task-4-2-review-red.log` (one expected failing runtime regression),
`vfa-task-4-2-{default,no-default}-{contracts,clippy,full}.log`, `fmt.log`,
`boundary-self.log`, `boundary-desktop.log`, `package.log`,
`package-no-default.log`, `grade.log`, `grade-evidence.json` and
`npm-validate.log` in the session-selected evidence location. The JSON binds final
source/test hashes, exact commands, exit codes, full failure identities and the
package hash. Packaging uses a tracked-HEAD source snapshot plus only this task's
three production files, with normal Cargo verification; new package bytes match.
There is no dirty/no-verify/publication bypass or altered fixture expectation.

The temporary build filesystem filled during the first compile (no-space error,
not a test result). A fresh compact ignored target on the larger checkout
filesystem, debug info disabled and incremental compilation off, completed all
Rust graders. This changes only local build artifacts, not manifests, compiler,
feature flags or baseline expectations. The original cache was not deleted.

A diagnostic in-checkout temporary directory made workspace-detection negatives
find the real enclosing checkout. The original task 4.1 binary reproduced both
extra unit failures under that same bad test context. Parent preserved the
failed-run log and generated seed as ignored evidence, then restored the
outside-checkout temporary root used by the baseline full suites. All tests ran
again, including the workspace negatives/property test; no production/test
assertion was changed or waived for that environmental issue. The final failure identities are exactly 17/11.

### Scope and route

One reused builtin implementation worker owned the two new production modules
and new journal test; inherited serving model/effort are unobservable. One reused
configured Rust reviewer supplied the bounded source challenge. Parent owns
shared wiring, one integrated repair, actual serialized graders and Git. This is
the smallest dependency-respecting route after the profile task, with no extra
worker/review/research wave and no parallel Cargo invocations.

Commit only `src/persistence/mod.rs`, new `operations.rs`/`operation_schema.rs`,
`tests/journal_commits.rs`, this eval and shared task/index/workflow records.
Manifests, both lockfiles, profile/export code, legacy persistence, frozen
approvals and compatibility goldens remain unchanged. No integrity regeneration
is needed for tools/spec/workflow-only changes. Task 1.4 stays unchecked; no push.

Final spec structure/negative mutants, scoped spelling/Markdown, frozen inputs,
exact scope and AST update pass. The generated HTML graph exceeds its size limit;
this does not affect the successful AST update or provide native visual proof.
