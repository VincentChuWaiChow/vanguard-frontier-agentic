# Task 3.1: engine-only session and immutable snapshot preparation

Current status: READY FOR REVIEW for verified engine preparation, with regression
limits. Full task3.1/native acceptance stays pending. Parallel-profile stability
and coverage remain explicit gaps.

Latest continuation: [catalog compatibility closure](task-3.1-catalog-compatibility.md)
fixes the known SAP mismatch. Full local suites now pass in both feature modes;
the17/11 failures in this report are historical pre-fix evidence, not current
failures. Native/prerequisite/coverage and historical profile-stability limits
remain; the full task and conditional Git delivery gate are still incomplete.

## Authorization and scope

User explicitly approved task3.1 engine preparation after the parser handoff.
Owned production paths are `tools/vfa-tui/src/engine/{session,snapshot,mod}.rs`;
tests are `tools/vfa-tui/tests/session_contracts.rs`. Preserve prior uncommitted
parser changes. No UI/runtime wiring, dependency/lock changes, installation or
remote CI. Commit/push authorization is now conditional on full verified task3.1
completion (see delivery gate below); that condition is not met.

Discovery found a correctness boundary: CatalogStore parsed files and later
reread them for hashes. User subsequently approved the two-file consistency
extension recorded below; it is not an unresolved approval boundary now.

## Journeys and eval contract defined before implementation

1. Operator selects an exact checkout independently of CWD; native identity and
   an epoch accompany immutable owned catalog data. No parent traversal/fallback.
2. Missing/malformed/tainted/duplicate collections have named independent quality
   states; successful zero is EMPTY, not an invented healthy failed load.
3. Every queued/running/cancelling operation blocks source replacement. Dropping
   an ID/observer never marks an operation terminal. A rejected switch preserves
   previous source/snapshot/epoch; a successful switch invalidates old tags while
   retaining old operation context.
4. Browsing invokes no checkout helper, target write or network action. Snapshot
   hashes must describe the exact bytes parsed, not a second filesystem read.

Parent owns all implementation/integration and exact graders. One reused
rust_reviewer performs bounded discovery and one final review follow-up, no
recursive delegation; maximum one parent repair. Native identity portability
and operation ownership made discovery provisional B; fixed invariants/tests
bound implementation. Profile remains gpt-6.1-sol/high with checked disk schema
and read-only defaults; serving identity UNVERIFIED, inherited workspace-write
means the read-only assignment is behavioral. Parent auth/model are UNVERIFIED.

## Required graders and pass signals

| Criterion | Grader / signal | Failure owner |
| --- | --- | --- |
| Intended RED | New session test target compiles to intended missing-module/API failure before production edits | Parent |
| Native source identity | Exact-root/alias/replacement/native-encoding cases; missing/wrong markers reject; unsupported identity backend fails explicitly | Parent |
| Honest immutable snapshots | Fixed fixtures, independent collection failures, empty/partial/unavailable, duplicate IDs/JSON keys, exact parsed-byte hashes and old snapshot retention | Parent |
| Operation admission | All nonterminal states block switching; explicit valid terminal transition releases block; invalid/foreign IDs/transitions reject; bounded retention | Parent |
| GREEN | Same session target in default and no-default modes, plus prior focused contracts | Parent |
| Regression | Both complete library failure identity sets unchanged from current17/11 baselines; strict clippy both modes, fmt, root npm validation | Parent |
| Scope | Approved input hashes/locks/previous parser bytes preserved; spec self-tests, integrity, links/JSON/whitespace and AST graph update | Parent |
| Independent challenge | Concrete findings independently reproduced and corrected within one repair | Reviewer / parent |
| Coverage | Existing tooling only; otherwise percentage/80-percent threshold explicitly UNVERIFIED | Parent |

Root validity uses expected package identity and the catalog directory, not the
health of any single collection, so missing agents does not hide valid peers.
Unix identity retains an open directory handle and device/inode; other backends
fail explicitly until a native implementation is verified. No cross-platform,
atomic multi-file transaction, malicious same-user race, durable-operation,
trust-grant invalidation or UI acceptance is claimed by isolated models.

## Identity milestone: actual RED/GREEN

Before production edits, the new session_contracts target failed at the intended
missing `engine::session` import (E0432, exit101). It was not a setup/dependency
failure. The additive SourceIdentity implementation then passed all5 native Unix
cases: exact-root selection, independent collection health, canonical aliases and
directory replacement, non-UTF8 paths, and invalid/oversized/symlinked markers.
Paths remain native PathBuf/OsStr; open directory ownership survives clones.
Marker reads are bounded through the opened regular file; O_NOFOLLOW/O_NONBLOCK
avoid following a substituted link or hanging on an observed FIFO substitution.
This is not a malicious same-user filesystem transaction guarantee.

Initial GREEN command (same target as RED):

```bash
CARGO_HOME="$PWD/tools/vfa-tui/target/consumer-cargo-home" \
CARGO_TARGET_DIR="$PWD/tools/vfa-tui/target/consumer-build" \
cargo test --manifest-path tools/vfa-tui/Cargo.toml --locked --offline --no-default-features --test session_contracts
```

Source identity was the first milestone, not complete task3.1 preparation.
Snapshot/operation coordination and final review were then pending the explicit
scope decision. The approved extension below continued the same wave. TDD
checkpoint commits remain forbidden.

Milestone regression checks: the combined session/parser/core/runtime/process/
profile/journal/report targets pass65 tests with default features and64 without;
strict all-target clippy in both modes and fmt exit0. Commands used the same
local Cargo home/target above and `--locked --offline`. Full-library baseline
comparison, final npm validation and final independent review are still owed for
the complete approved wave, not silently converted into passes at this milestone.

Context7 source: [Unix MetadataExt](https://doc.rust-lang.org/stable/std/os/unix/fs/trait.MetadataExt.html)
and [Windows MetadataExt](https://doc.rust-lang.org/stable/std/os/windows/fs/trait.MetadataExt.html).
Installed Rust1.99.1 primary std source confirms windows_by_handle remains
unstable. SourceIdentity returns IdentityUnavailable on unsupported backends;
only native Linux tests ran. Coverage tooling/percentage remains UNVERIFIED.

Identity-milestone housekeeping: spec structural self-tests, read-only asset
integrity, local links/state/whitespace and exact cumulative13-file scope pass.
`graphify update .` exits0 and refreshes the AST graph; HTML exceeds its node
limit and is not native visual evidence. Raw logs/source hashes are stored under
ignored `tools/vfa-tui/target/session-preparation-wave/identity-milestone.json`
and adjacent logs. Snapshot provenance and final aggregate checks stay pending.

## Approved loader/store consistency extension

User explicitly approved adding `src/catalog/loader.rs` and `src/catalog/store.rs`
to the same wave. Preserve existing public loader/store entry points, envelopes,
sorting, taint/error semantics and reload behavior. Extract shared parsers; add a
crate-private captured-byte reader constructor. Initial load hashes the exact
owned bytes sent to parsing, never a later read. Bounded default reads retain
the current20MiB source limit, including growth after metadata inspection.
No raw reread/hash-only scan or snapshot bracketing substitute.

Before edits, add a constructor contract with a stateful reader that replaces
each on-disk file immediately after capturing it. Assert one call per catalog,
the original parsed IDs and independent original-byte hashes. Intended missing
constructor RED precedes production changes. Then add snapshot/session contracts
for independent quality, duplicate array IDs and decoded JSON map keys, empty
versus failure, immutable old contexts, same-source no-op, failed-source atomicity,
all nonterminal guards, explicit terminal release, foreign IDs and bounded
operation retention. These are engine models, not actual job/journal authority.

## Final TDD and independent review evidence

- Capture constructor RED: exit101/E0599 for missing `load_with_reader` before
  production edits. GREEN: stateful reader replaces all six fixture files with
  `null` immediately after reading; original parsed IDs and independently hashed
  original bytes survive with exactly one reader call per catalog. A second
  contract rejects failed/oversized reader observations without healthy data.
- Snapshot/session RED: exit101 for missing snapshot/session APIs before their
  implementation. GREEN: immutable owned snapshots with existing domain models,
  per-collection quality, duplicate IDs/decoded JSON keys, source epochs, foreign
  tags/IDs, explicit operation state/release and retained original context.
- Reviewer found one blocker: blocking root open preceded the directory check,
  so selecting a FIFO could hang. Parent reproduced actual runtime RED in a
  bounded child; after three seconds the child was explicitly killed and reaped,
  then the test failed. One repair adds Unix O_DIRECTORY/O_NOFOLLOW/O_NONBLOCK.
  The identical regression is GREEN; all19 session cases pass in each mode.
- No independent re-review is claimed. Parent inspected the fix and reran all
  graders; one discovery turn, one final review turn, one integrated repair.

## Verified guarantees

| Guarantee | Test target | Result |
| --- | --- | --- |
| Exact native roots, aliases, replacement and non-UTF8 identity; invalid/FIFO roots cannot hang | Identity cases and `selecting_fifo_source_is_rejected_without_blocking` | PASS native Linux |
| Actual parsed bytes equal hashed observations despite immediate on-disk replacement | `catalog::store::tests::initial_load_hashes_exactly_the_captured_bytes_once_per_catalog` | PASS both modes |
| Reader failures/oversize never become healthy input | `catalog::store::tests::captured_reader_failure_and_oversize_are_not_healthy_empty_inputs` | PASS both modes |
| Missing/malformed/tainted/empty collections have independent honest quality | Snapshot quality cases in `tests/session_contracts.rs` | PASS |
| Duplicate IDs and decoded JSON object keys reject before map overwrite; different entity kinds can share an ID | Both duplicate cases | PASS |
| Same-source no-op and failed switch preserve snapshot/epoch; new source invalidates old tags but retains old observations | Switch/context cases | PASS |
| Every nonterminal operation blocks switching; observer loss cannot release it; transitions and retention are bounded | Operation guard cases | PASS |
| Published domain values cannot be mutated through the snapshot | New `CatalogSnapshot::catalog` compile-fail doctest | PASS |
| Browsing does not execute fixture helpers or change source bytes | Browse sentinel/content case and source inspection | PASS |

## Exact final commands and outcomes

Commands run through `rtk proxy`; Cargo home/target were the existing ignored
consumer caches shown above. Run the test/clippy commands both normally and
with `--no-default-features`:

```bash
cargo test --manifest-path tools/vfa-tui/Cargo.toml --locked --offline --test session_contracts
cargo test --manifest-path tools/vfa-tui/Cargo.toml --locked --offline --test session_contracts --test export_protocol --test core_contracts --test report_service_contracts --test runtime_bridge --test process_contracts --test profile_ownership --test journal_commits -- --test-threads=1
cargo test --manifest-path tools/vfa-tui/Cargo.toml --locked --offline --lib
cargo clippy --manifest-path tools/vfa-tui/Cargo.toml --locked --offline --all-targets -- -D warnings
cargo fmt --manifest-path tools/vfa-tui/Cargo.toml --check
cargo test --manifest-path tools/vfa-tui/Cargo.toml --locked --offline --no-default-features --doc
npm run validate
```

| Grader | Status | Evidence |
| --- | --- | --- |
| New session target, ordinary parallel harness | PASS | 19 tests per mode, exit0, zero ignores |
| Serial focused regressions | PASS | 79 default/78 engine-only tests, exit0; explicitly `--test-threads=1` |
| Full library | MISS baseline only | Exit101:737 pass/17 fail default,560 pass/11 fail engine-only; exact failure names match fresh pre-production baselines; two added unit cases pass |
| Strict clippy both modes and fmt | PASS | Exit0 |
| Immutability doctest | PASS scoped | New compile-fail case passes; one pre-existing ignored MockRunner doctest remains ignored |
| Repository validation | PASS | `npm run validate`, exit0; no native Codex marketplace installation or remote CI claim |
| Parallel focused attempt | MISS observed | Unchanged `profile_ownership::two_process_exclusion_and_normal_drop_preserve_owned_data` failed once with child Busy; not silently discarded |
| Profile isolated rerun | PASS scoped | Exact failing test passes alone; full serial runs also pass. Parallel stability/root cause remain UNVERIFIED, not proven baseline flakiness |
| Preservation | PASS | Reload method body byte-identical; legacy public entry points retained; parser bytes, manifests/locks, Node producer and frozen spec inputs unchanged |
| Spec/integrity/scope | PASS | Structural self-tests and read-only integrity verifier exit0; local links/state/whitespace and exact16-file cumulative source/evidence scope pass |
| Graph maintenance | PASS AST only | `graphify update .` exits0; HTML exceeds its node limit, not native visual evidence |

The parallel miss is outside the changed profile code/test paths, which remain
byte-identical to HEAD. No unapproved profile fix or retry-until-green claim.
Its failed log, isolated result and serial results are retained independently.

Raw evidence under ignored `tools/vfa-tui/target/session-preparation-wave/`:
capture RED/GREEN, snapshot RED/GREEN, review FIFO RED/GREEN, final mode-specific
contracts/library/clippy/new-target logs, doctest/root-validation logs and final
source/metadata hashes. Identity-milestone evidence remains historical, separate
from final wave evidence. Context7 `/serde-rs/serde` primary MapAccess/Visitor docs
informed duplicate-key validation; actual decoded-key regressions provide proof.

## Remaining limits and route discipline

No new dependencies, installs, commits, pushes or remote effects. Parent owns all
source integration; one reused reviewer, configured/live role gpt-6.1-sol/high,
serving identity/parent authentication UNVERIFIED; read-only delegation was
behavioral under inherited workspace-write. No additional writer was useful for
this coupled loader/snapshot/session change. Total token usage is unknown.

Windows native identity deliberately fails closed; macOS/Windows execution,
Desktop lock-graph behavior, native UI/runtime, freshness/reload/search, actual
selection/trust/preview adapter invalidation, job cleanup and durable journal
admission remain future/full-task work. OperationStatus is reused from the
journal, but model transitions are not cleanup or commit receipts. Hash/parse
consistency is per captured file, not an atomic multi-file filesystem transaction
or malicious same-user race defense. Coverage/80-percent threshold and parallel
profile stability remain UNVERIFIED. Task3.1 remains unchecked.

## Conditional Git delivery gate

User authorized: "git commit and git push when task 3.1 is marked and verified
as done". This does not accept engine preparation as full-task completion or
waive existing dependencies/verification. Current completion audit: task3.1 and
prerequisites1.5/2.1 remain unchecked; Windows native identity is unsupported;
full-library gates retain17/11 failures and parallel-profile stability/coverage
remain unresolved. Verified production/test hashes match the existing evidence.

No staging, commit or push occurred. When the full predicate is satisfied,
delivery is limited to task3.1-owned changes on the current feature branch;
preserve unrelated/uncommitted task5.2 work. Shared module/evidence diffs require
surgical staging and verification of the exact committed snapshot, not merely
the mixed working tree. Recheck branch, identity, outgoing commits and remote
before an ordinary fast-forward push; no force-push or publication scope expansion.
