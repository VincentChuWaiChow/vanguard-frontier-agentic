# Task 1.6 evaluation: installation-free shared cases

## Authorization and boundaries

Defined before implementation for the user-authorized installation-free wave.
This is preparation only: the full task checkbox and dependencies remain unchanged.
No installs, dependency/toolchain changes, remote CI, pushes, global configuration
or unrelated edits. Parent owns module/manifest wiring, evals, serialized Cargo
and separate no-reply local commits. Preserve other workers' files.

## Observable checks

- Extract all five existing core behavioral cases without changing test names,
  independent expected payloads, negative mutants, schemas or fixture inventory.
- Both consumer wrappers call the same shared functions with explicit fixture
  roots derived from their manifest directories, not fixed host paths/CWD/home.
- Execute the real TUI core_contracts target with default and no-default features
  using locked offline dependencies. Missing or wrong fixtures must fail, not skip.
- Format both wrappers and shared source. Inspect Desktop wrapper wiring and
  actual consumer metadata closure, but label Desktop execution UNVERIFIED.
- Preserve frozen fixtures, approved requirements/design, dependencies and pins.
  Existing full-suite baseline failures cannot be relabeled as a green gate.

## Deferred full obligations

Desktop behavior under its own lock and native compilation require task 1.4;
shared source or TUI-only results do not prove both-consumer parity. Full task 1.6
stays unchecked. Status: checks defined; implementation not yet graded.

## Results: preparatory slice only

| Criterion | Result | Evidence |
| --- | --- | --- |
| Shared cases and fixtures | PASS | Parent diff against pre-extraction source: same five test names, assertions and independent expected values; all eight compatibility fixture hashes unchanged |
| TUI behavior | PASS | Locked offline core_contracts: five passed default and five passed no-default, no ignored/skipped tests |
| Formatting | PASS | Actual cargo +stable fmt --check for TUI and Desktop (including shared source traversal), both exit 0 |
| Engine boundary | PASS | Actual normal engine subtree checked under TUI and Desktop locks; 31 boundary mutants reject |
| Desktop execution/parity | UNVERIFIED | No native prerequisites installed; second consumer test execution deferred, full task 1.6 unchecked |
| Full-suite baseline | UNVERIFIED this slice | No production behavior changed; previous 17/11 failures not relabeled as success |

Commands (all local; CARGO_HOME and shared target selected from existing caches):

```bash
cargo +stable test --manifest-path tools/vfa-tui/Cargo.toml --locked --offline --test core_contracts
cargo +stable test --manifest-path tools/vfa-tui/Cargo.toml --locked --offline --no-default-features --test core_contracts
cargo +stable fmt --manifest-path tools/vfa-tui/Cargo.toml --check
cargo +stable fmt --manifest-path tools/vfa-desktop/Cargo.toml --check
python3 tools/vfa-tui/tests/check_feature_boundary.py --manifest-path tools/vfa-tui/Cargo.toml --self-test
python3 tools/vfa-tui/tests/check_feature_boundary.py --manifest-path tools/vfa-tui/Cargo.toml --consumer-manifest tools/vfa-desktop/Cargo.toml
```

Route: one bounded built-in worker (serving model/effort unobservable), parent
source comparison and independent graders. No assertion changes or repair needed.
No dependencies, frozen approvals, user stores, global files or Git remotes changed.
