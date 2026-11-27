# Verification transcript excerpts

The repository ignores `*.log`; raw logs remain local under `results/`. These
verbatim excerpts preserve key passing/failing assertions in versioned evidence.
Full commands, observed exit codes and source identities are recorded in
`state.json`; actual backend observations and hashes are separate versioned JSON.
Excerpts are not complete logs and do not widen the verification scope.

## Compiler failure before UI trait repair

Local source: `desktop-check-prerequisites.log` (last 28 lines).

```text
    Checking gpui-pre-platform v0.3.12
    Checking nucleo-matcher v0.3.1
    Checking fs2 v0.4.3
    Checking vfa-tui v0.1.0 (/home/raishin/projects/vanguard-frontier-agentic/tools/vfa-tui)
    Checking gpui-kit v0.9.1
    Checking vfa-desktop v0.1.0 (/home/raishin/projects/vanguard-frontier-agentic/tools/vfa-desktop)
error[E0599]: no method named `update_window` found for mutable reference `&mut TestAppContext` in the current scope
   --> tests/ui.rs:22:8
    |
 22 |     cx.update_window(handle, |_, window, cx| {
    |     ---^^^^^^^^^^^^^
    |
    = help: items from traits can only be used if the trait is in scope
help: there is a method `add_window` with a similar name, but with different arguments
   --> /home/raishin/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/gpui-pre-0.3.12/src/app/test_context.rs:284:5
    |
284 | /     pub fn add_window<F, V>(&mut self, build_window: F) -> WindowHandle<V>
285 | |     where
286 | |         F: FnOnce(&mut Window, &mut Context<V>) -> V,
287 | |         V: 'static + Render,
    | |____________________________^
help: trait `AppContext` which provides `update_window` is implemented but not in scope; perhaps you want to import it
    |
  3 + use gpui_kit::AppContext;
    |

For more information about this error, try `rustc --explain E0599`.
error: could not compile `vfa-desktop` (test "ui") due to 1 previous error
```

## Explicit native target failure before approved SDK API repair

Local source: `native-x11-command.log` (last 18 lines).

```text
   Compiling vfa-desktop v0.1.0 (/home/raishin/projects/vanguard-frontier-agentic/tools/vfa-desktop)
error[E0599]: no method named `rendered_scene` found for mutable reference `&mut gpui_kit::Window` in the current scope
   --> tests/native_smoke.rs:103:55
    |
103 |                         let scene_primitives = window.rendered_scene().len();
    |                                                       ^^^^^^^^^^^^^^ method not found in `&mut gpui_kit::Window`

warning: unused import: `AppContext`
 --> tests/native_smoke.rs:3:16
  |
3 | use gpui_kit::{AppContext as _, QuitMode, WindowOptions};
  |                ^^^^^^^^^^
  |
  = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

For more information about this error, try `rustc --explain E0599`.
warning: `vfa-desktop` (test "native_smoke") generated 1 warning
error: could not compile `vfa-desktop` (test "native_smoke") due to 1 previous error; 1 warning emitted
```

## Desktop suite final GREEN: six engine cases and one UI case

Local source: `desktop-tests-final.log` (last 38 lines).

```text
    Finished `test` profile [unoptimized] target(s) in 0.28s
     Running unittests src/lib.rs (tools/vfa-desktop/target/debug/deps/vfa_desktop-a876d20df907d866)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/main.rs (tools/vfa-desktop/target/debug/deps/vfa_desktop-584a0f6eb20b0d06)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/engine_contracts.rs (tools/vfa-desktop/target/debug/deps/engine_contracts-557c556d5b31e910)

running 6 tests
test provider_catalog_sap_roundtrip_preserves_strict_unknown_rejection ... ok
test catalog_schema_rejects_unknown_fields_and_wrong_discriminator ... ok
test missing_catalog_file_is_an_error_not_a_healthy_empty_collection ... ok
test public_domain_imports_and_fixed_catalog_contract ... ok
test same_oracle_rejects_payload_schema_order_and_missing_fixture_mutants ... ok
test malformed_collection_preserves_other_collections_and_reload_retains_last_good ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/ui.rs (tools/vfa-desktop/target/debug/deps/ui-163a95d440155a1a)

running 1 test
test bootstrap_click_changes_application_state ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

   Doc-tests vfa_desktop

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

```

## Disconnected callback negative oracle: expected test exit 101

Local source: `callback-mutant.log` (last 25 lines).

```text
   Compiling vfa-tui v0.1.0 (/tmp/vfa GPUI callback mutant rt31e9u3/checkout space-é/tools/vfa-tui)
   Compiling vfa-desktop v0.1.0 (/tmp/vfa GPUI callback mutant rt31e9u3/checkout space-é/tools/vfa-desktop)
    Finished `test` profile [unoptimized] target(s) in 6.76s
     Running tests/ui.rs (/home/raishin/projects/vanguard-frontier-agentic/tools/vfa-desktop/target/debug/deps/ui-a89312bfef2f63dd)

running 1 test
test bootstrap_click_changes_application_state ... FAILED

failures:

---- bootstrap_click_changes_application_state stdout ----

thread 'bootstrap_click_changes_application_state' (397) panicked at tests/ui.rs:32:9:
assertion `left == right` failed
  left: 0
 right: 1
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    bootstrap_click_changes_application_state

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

error: test failed, to rerun pass `--test ui`
```

## Explicit native strict clippy

Local source: `native-clippy.log` (last 2 lines).

```text
    Checking vfa-desktop v0.1.0 (/home/raishin/projects/vanguard-frontier-agentic/tools/vfa-desktop)
    Finished `dev` profile [unoptimized] target(s) in 0.57s
```

## TUI default shared cases

Local source: `tui-core-default.log` (last 14 lines).

```text
   Compiling proptest v1.11.0
    Finished `test` profile [unoptimized + debuginfo] target(s) in 24.40s
     Running tests/core_contracts.rs (tools/vfa-tui/target/debug/deps/core_contracts-6ffcae53e71ec19e)

running 6 tests
test catalog_schema_rejects_unknown_fields_and_wrong_discriminator ... ok
test provider_catalog_sap_roundtrip_preserves_strict_unknown_rejection ... ok
test missing_catalog_file_is_an_error_not_a_healthy_empty_collection ... ok
test public_domain_imports_and_fixed_catalog_contract ... ok
test same_oracle_rejects_payload_schema_order_and_missing_fixture_mutants ... ok
test malformed_collection_preserves_other_collections_and_reload_retains_last_good ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

```

## TUI engine-only shared cases

Local source: `tui-core-engine.log` (last 14 lines).

```text
   Compiling vfa-tui v0.1.0 (/home/raishin/projects/vanguard-frontier-agentic/tools/vfa-tui)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 9.64s
     Running tests/core_contracts.rs (tools/vfa-tui/target/debug/deps/core_contracts-0405e22eeae4db93)

running 6 tests
test catalog_schema_rejects_unknown_fields_and_wrong_discriminator ... ok
test provider_catalog_sap_roundtrip_preserves_strict_unknown_rejection ... ok
test missing_catalog_file_is_an_error_not_a_healthy_empty_collection ... ok
test public_domain_imports_and_fixed_catalog_contract ... ok
test same_oracle_rejects_payload_schema_order_and_missing_fixture_mutants ... ok
test malformed_collection_preserves_other_collections_and_reload_retains_last_good ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

```

## Complete repository validation: host rerun recorded exit 0

Local source: `repository-validate-escalated.log` (last 18 lines).

```text
✔ fixed semantic and canonical-digest oracles reject negative mutations even with recomputed digests (62.061926ms)
✔ provider, role, fallback and no-pair companion rules also cover --all and no-skills (230.345868ms)
✔ unknown/malformed/ambiguous selections and unavailable apply mode fail before stdout or writes (420.857849ms)
✔ unsafe source metadata, traversal and duplicate destination definitions fail closed (394.631073ms)
✔ symlink ancestors/leaves and non-regular destinations are rejected without target effects (789.995292ms)
✔ PLAN_LIMIT rejects oversized artifacts before output; never emits truncated success (44.4923ms)
✔ existing hard-link destination aliases cannot mutate sources or collide with other artifacts (108.489867ms)
✔ serialized protocol overflow fails explicitly with empty stdout and no target directory (1839.061806ms)
✔ hard-linked metadata, catalogs, executable and outside-target sentinels are never writable effects (377.738551ms)
✔ JSON plan and help cannot report a successful human-only response (81.00065ms)
ℹ tests 29
ℹ suites 0
ℹ pass 29
ℹ fail 0
ℹ cancelled 0
ℹ skipped 0
ℹ todo 0
ℹ duration_ms 9894.904334
```

## Spec structural checks, not native acceptance

Local source: `spec-final.log` (last 5 lines).

```text
PASS: rejects fixed Unix skill path
PASS: rejects fixed Windows output path
PASS: rejects fixed relative skill path
PASS: rejects missing final visual gate
Scope: structural checks only; no semantic, SMT or runtime proof
```
