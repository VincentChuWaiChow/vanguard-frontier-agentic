# Task 4.1 evaluation contract

## Scope and route

User authorized the ready wave (4.1 and 5.1), then 4.2 after 4.1, with separate
frequent local commits. No push or native-verification waiver. Preserve pending
task 1.4 as a labelled incomplete checkpoint, not a completed prerequisite.
Ultracode remains parent coordinator; Codex Agent Teams supplies bounded workers.

Parent owns module/dependency wiring, shared records, Cargo execution and Git.
One implementation worker owns only `tools/vfa-tui/src/engine/profile.rs` and
`tools/vfa-tui/tests/profile_ownership.rs`. It is not alone and must preserve
other edits. A separate Rust reviewer gets one bounded review, followed by one
integrated repair pass. No recursive delegation, real user-profile mutation,
system install, task 4.3 recovery/writer integration or implementation beyond
the authorized ready wave. Generic implementation role is used because exact
disk implementation profiles have incomplete metadata; inherited serving model/
effort is unobservable, not relabelled as a configured specialist.

## Observable checks defined before implementation

1. Explicit native profile roots or OS-appropriate per-user discovery resolve one
   canonical profile identity. Paths are caller-selected/discovered, never fixed
   to a home, checkout, skill or temporary fallback. Missing/relative/unusable
   private roots fail explicitly; no silent in-memory or HOME/tmp substitution.
2. Private profile creation and lock opening reject symlink/non-file surprises.
   Unix ownership/permissions are checked before mutation of existing content;
   create private directories/files without broadly chmodding existing parents.
   Platform-specific assumptions and unobserved hosts remain explicit.
3. An owned, non-clonable guard retains an OS-backed exclusive file lock for its
   lifetime. The lock file is never truncated/deleted/recreated during ownership.
   Contention is a named busy result, not PID-file liveness inference. Accessors
   expose native canonical paths without exposing a second lock handle.
4. Two actual test processes prove a second owner cannot mutate sentinel journal/
   preferences or recover jobs while the first lives; ownership becomes available
   after normal release and forced owner death. Use temporary test profiles and
   a re-executed test binary, no published helper pointing into excluded tests.
5. Preserve source/lock identity with Unicode/spaces and path aliases; reject an
   occupied/invalid/unwritable profile without falling back or removing the lock.
   Directory creation is not proof of lock ownership, and a busy acquisition must
   not alter existing protected data.
6. Default and no-default `profile_ownership` tests, fmt, strict clippy, existing
   core/report regressions and actual normal engine-boundary checks pass. Existing
   full-suite SAP failures remain recorded, not hidden or repaired. Do not claim
   Windows/macOS native execution from Linux test results.
7. Before its own commit, run repository validation, scoped spelling/Markdown,
   frozen approvals/fixtures, exact scope and no-reply author/committer checks.
   Commit only this task's verified scope; no push. Update AST after code changes.

## Graders

Use discovered manifests and session-selected Cargo caches. Required target:
`cargo test --manifest-path tools/vfa-tui/Cargo.toml --locked --test profile_ownership`,
repeated with `--no-default-features`; use explicit temp roots in tests.
Independent reviewer challenges filesystem/lock lifetime and false-positive tests.
Full cross-platform release/native admission remains a later mandatory gate.

## Results and review

Scoped capability is implemented. Full baseline gates remain non-green, and
Windows/macOS native behavior plus Windows ACL inheritance are UNVERIFIED.

| Criterion | Result | Evidence |
| --- | --- | --- |
| Native paths and private roots | PASS Linux | Explicit Unicode/space roots, canonical lexical alias, absolute XDG discovery, no HOME/tmp substitution, private user-owned Unix modes and invalid-root negatives |
| Session-long owner | PASS Linux | Retained non-clonable File; Send/Sync assertion; real contender reports Busy while holder lives; no truncation/unlink or changed sentinel/lock inode |
| Normal release and crash | PASS Linux | Two actual test processes; reacquisition after guard drop while first process still lives and after forced owner death |
| Profile contract tests | PASS | Eight default/eight no-default harness cases, including the re-executed child dispatcher; all seven substantive parent cases pass |
| Existing engine contracts | PASS | Five core cases in each mode; eleven default/ten no-default report contracts |
| Full regression | MISS existing baseline | Exact same seventeen default / eleven no-default library failures as task 1.3; no new failures; one existing ignored doctest retained |
| Build/format/boundary | PASS | Both all-target strict clippy modes and fmt; actual engine normal closure under TUI and Desktop locks plus 31 negative fixtures |
| Consumer lock/provenance | PASS metadata | Only profile dependencies added; all pinned toolkit package identities unchanged, recorded lock hash updated; eighteen bootstrap provenance mutants pass, not native compile proof |
| Package | PASS candidate snapshot | Normal packaging/verification, original manifest and packaged source bytes compared, packaged no-default library builds; no dirty/no-verify bypass or publication |
| Independent review | PASS process | Rust source review found a Windows test assumption, repaired in tests; guard was not weakened; no unobserved Windows runtime result claimed |
| Root validation | PASS | Run on an isolated tracked-checkpoint plus exact task 4.1 candidate bytes; concurrent task 5.1 exporter edits excluded |
| Other native/legacy MSRV | UNVERIFIED | No macOS/Windows run or declared Rust-minimum verification; current compiler was used; no blanket release waiver |

The independent review identified that Windows byte-range locks intentionally
deny reads through a second file handle. Sentinel journal/preferences and inode
metadata remain inspected during ownership, but lock bytes are now inspected
only after release/death. Every re-executed contender/discovery probe also has a
kill-and-reap deadline, so replacing nonblocking locking with a blocking call
cannot hang the suite indefinitely. This is a test repair, not a weaker lock.

Context7 verified the fs2 locking API and advisory boundary against
[fs2's primary API documentation](https://docs.rs/fs2/latest/fs2/trait.FileExt.html).
The per-user resolver uses the platform-local directory API documented by
[dirs-next](https://docs.rs/dirs-next/latest/dirs_next/fn.data_local_dir.html),
while Linux deliberately requires explicit absolute XDG input rather than its
HOME fallback. Two generic implementation worker profiles were available in the
ready wave; only this task's worker touched its two owned files. Model/effort were
inherited and unobservable. The independent checked Rust reviewer is configured
Sol/high with read-only defaults; serving identity is not attested. Parent owns
all Cargo, integration and Git actions. This was the smallest disjoint route.

### Commands and evidence

```bash
manifest=tools/vfa-tui/Cargo.toml
# Repeat with --no-default-features.
cargo +stable test --manifest-path "$manifest" --locked --offline \
  --test profile_ownership --test core_contracts --test report_service_contracts
cargo +stable clippy --manifest-path "$manifest" --locked --offline --all-targets -- -D warnings
cargo +stable test --manifest-path "$manifest" --locked --offline --no-fail-fast
cargo +stable fmt --manifest-path "$manifest" --check
python3 tools/vfa-tui/tests/check_feature_boundary.py --manifest-path "$manifest" --self-test
python3 tools/vfa-tui/tests/check_feature_boundary.py --manifest-path "$manifest" \
  --consumer-manifest tools/vfa-desktop/Cargo.toml
python3 tools/vfa-desktop/tests/check_bootstrap.py \
  --manifest-path tools/vfa-desktop/Cargo.toml --toolchain stable --self-test
```

Both full suites exit 101 solely for the recorded failure sets, compared by
actual failing names in `vfa-task-4-1-baseline-comparison.json`. Other completed
Cargo/contract/boundary commands exit zero. Logs are
`vfa-task-4-1-{default,no-default}-{contracts,clippy,full}.log`, `boundaries.log`,
`package.log` and `package-evidence.json` in the selected temporary run location.
Packaging compares `Cargo.toml.orig` because Cargo normalizes its published
manifest; the archive's source/module/lock bytes match the candidate.

The first isolated root-validation attempt hit the known sandbox Node child-pipe
EPERM failure, not product assertions with working I/O. The same candidate is
rerun passed with approved local-test execution (exit 0); no network, install or Git mutation.
Its source hashes and actual outcome are in
`vfa-task-4-1-validation-snapshot.json` and `npm-validate-approved.log`. Do not use
an in-flight task 5.1 root tree to claim this commit's validation.

### Task-specific commit scope

- TUI manifest/lock, `src/lib.rs`, new `src/engine/mod.rs` and `profile.rs`.
- `tools/vfa-tui/tests/profile_ownership.rs`.
- Desktop lock and provenance lock hash, required by shared engine dependencies.
- This eval, shared index/task status and existing workflow records.

Keep task 5.1 script/tests/fixtures and task 4.2 files out of this task's commit.
Task 1.4 remains unchecked; no native package setup or push is authorized here.
