# Task 3.1 closure: current catalog provider compatibility

## Objective, authority and scope

The user's explicit request to continue work toward full task3.1 admits this
bounded validation-closure wave, rather than another unrelated preparation task.
Current canonical agents/skills include `sap`; the shared Provider enum rejects
it, causing actual snapshots to label both collections unavailable and producing
the existing17/11 library failures. Accept exactly that catalog-defined value,
not arbitrary unknown providers. Earlier preparation-only provider-repair
exclusions remain historical; this new wave changes only the identified mismatch.

Production ownership: `tools/vfa-tui/src/models/provider.rs`. Regression ownership:
shared `tests/support/engine_cases.rs`, TUI `tests/core_contracts.rs`, Desktop
`tests/engine_contracts.rs`, and TUI `tests/session_contracts.rs`. Preserve all
existing fixtures, prior work, public paths/old values, manifests/locks, export
policy, native backends and lifecycle code. No installs, commits, pushes or remote
CI. Conditional publication still requires full task3.1 completion.

## Checks defined before editing

| Criterion | Grader / expected signal | Failure owner |
| --- | --- | --- |
| Runtime RED | Shared serde/display round-trip test actually executes and rejects known `sap`; real checkout snapshot is unavailable before fix | Parent |
| Minimal GREEN | Add only Sap enum variant; same test passes, old known values round-trip unchanged, arbitrary unknown strings still reject | Parent |
| Real catalog | Actual snapshot is CURRENT for agents/skills, exact IDs/counts match independently parsed catalog data, SAP sets are not dropped | Parent |
| Shared regressions | Same case wired into both consumer wrappers; TUI both modes run; Desktop runtime/lock-graph execution remains UNVERIFIED if native prerequisites prevent build | Parent |
| Full local gates | Full Cargo suites in both modes, strict clippy/fmt, spec/integrity/root npm validation; preserve and report every failed/ignored check | Parent |
| Independent review | One reused Rust reviewer after GREEN; one bounded repair maximum | Reviewer / parent |

Complexity A: closed known value, localized enum extension, exact executable
oracles. Routine local data-contract risk, not a cloud/live-platform mutation.
Parent keeps implementation/integration; one reviewer is sufficient, no writer
fan-out. Context7 verifies Serde's kebab-case/unknown-variant behavior. This fixes
real data discovery; it does not close Windows identity, native bootstrap,
prerequisites1.5/2.1 or the full task checkbox by inference.

## Actual RED/GREEN and review

Before production edits the shared regression executed and failed with unknown
variant `sap`, exit101. The real checkout snapshot independently failed because
agents were UNAVAILABLE instead of CURRENT, with the same named parse error.
The entire production fix is one appended `Sap` variant. The identical tests now
pass; unknown/case-mutated/suffix-mutated provider values still reject. Current
agents/skills retain every catalog ID and every SAP ID, not just nonempty counts.

One reused Rust reviewer found no blockers in the enum extension or independent
oracles/wrappers. Source review only; parent ran the graders. Configured/live role
gpt-6.1-sol/high, serving identity UNVERIFIED; read-only assignment behavioral
under inherited workspace-write. No writer fan-out or repair was needed.

## Commands and current results

Shell commands ran through `rtk proxy`. Cargo uses existing ignored caches:
`CARGO_HOME="$PWD/tools/vfa-tui/target/consumer-cargo-home"` and
`CARGO_TARGET_DIR="$PWD/tools/vfa-tui/target/consumer-build"`.

```bash
cargo test --manifest-path tools/vfa-tui/Cargo.toml --locked --offline --no-fail-fast
cargo test --manifest-path tools/vfa-tui/Cargo.toml --locked --offline --no-default-features --no-fail-fast
cargo clippy --manifest-path tools/vfa-tui/Cargo.toml --locked --offline --all-targets -- -D warnings
cargo clippy --manifest-path tools/vfa-tui/Cargo.toml --locked --offline --no-default-features --all-targets -- -D warnings
cargo fmt --manifest-path tools/vfa-tui/Cargo.toml --check
cargo fmt --manifest-path tools/vfa-desktop/Cargo.toml --check
npm run validate
python3 tools/vfa-tui/tests/check_feature_boundary.py --manifest-path tools/vfa-tui/Cargo.toml --self-test
python3 tools/vfa-tui/tests/check_feature_boundary.py --manifest-path tools/vfa-tui/Cargo.toml --consumer-manifest tools/vfa-desktop/Cargo.toml
python3 tools/vfa-desktop/tests/check_bootstrap.py --manifest-path tools/vfa-desktop/Cargo.toml --toolchain stable --self-test
python3 .kiro/specs/gpui-desktop/verify.py --tasks --self-test
python3 tests/validate-asset-integrity.py
graphify update .
git diff --check
```

| Criterion | Status | Actual evidence |
| --- | --- | --- |
| Intended runtime RED, then GREEN | PASS | Known-provider roundtrip and real-checkout snapshot failed before the enum edit;6 core and20 session contracts now pass |
| Full default suite | PASS | Exit0;1121 passed,0 failed,1 pre-existing ignored MockRunner doctest across13 targets;754 library tests pass |
| Full engine-only suite | PASS | Exit0;900 passed,0 failed,1 same pre-existing ignored doctest across12 targets;571 library tests pass |
| Previous17/11 failures | PASS resolved | All formerly failing loader/store/search/app/report library identities pass; no hidden test gating or baseline reset |
| Prior parallel profile miss | PASS current run, stability UNVERIFIED | Unchanged profile tests pass under ordinary parallel execution in both full suites; no claim that the historical intermittent root cause was repaired |
| Strict clippy/fmt and root npm validation | PASS | All exit0; no manifests/locks/catalog data changed |
| Engine dependency boundary | PASS both lock graphs | Engine checker plus31 self-tests passes; Desktop normal engine subtree passes after restoring the unchanged locked dependency cache |
| Pinned bootstrap provenance | PASS static/source | Actual compiler/archive/VCS/normal features and18 negative mutants pass against the restored SDK cache; not compilation/runtime proof |
| Desktop engine-contract execution | MISS prerequisite | Exact `cargo +stable test --manifest-path tools/vfa-desktop/Cargo.toml --locked --offline --test engine_contracts`, with Desktop target and2 build jobs, exits101 in yeslogic-fontconfig-sys: pkg-config is unavailable |
| Spec, integrity, graph and scope | PASS | Spec/task checker and self-tests, unchanged asset integrity, AST graph update and exact21-file cumulative allowlist; graph HTML is too large to generate |
| Full/native completion | UNVERIFIED | No native UI/foreground/Windows identity proof and no completion of prerequisites1.5/2.1; task3.1 remains unchecked |

The initial offline Desktop graph check failed because its GPUI registry cache
had disappeared. Parent restored only locked crate cache material under ignored
project storage, with no version/feature/lock change or global installation, then
reran the same graph and provenance graders successfully. The subsequent actual
Desktop test attempt reached the real host prerequisite failure above; no test
execution or successful native skip is claimed. pkg-config alternatives, native
headers, clang/cmake/ninja are also unavailable in current path/header checks.

Raw logs/source hashes: ignored
`tools/vfa-tui/target/session-preparation-wave/catalog-compatibility/`. Preserve
earlier identity/preparation/parallel-miss evidence as historical snapshots.
Context7 `/serde-rs/serde` confirms enum renaming and strict unknown-variant
handling; actual positive/hostile tests supply proof. Coverage remains UNVERIFIED.

User explicitly declined host installation: "skip we cant install". No
installation was attempted. The native execution gate remains unmet, not waived.

Status: READY FOR REVIEW for the reviewed compatibility fix, with host/native gates still
incomplete. Git commit/push remains held by the user's explicit full-task
completion condition, not reinterpreted as preparation publication.
