# Rust safety rule evaluations

Load this file only when changing or evaluating the adjacent rule. Define checks
before editing. Evaluation covers reusable guidance, not application repairs,
global installation or unrequested compiler/dependency changes.

## Acceptance contract

- One compact language-scoped rule applies only to Rust work; catalog and adjacent metadata
  agree exactly, with a conditional repository guidance link and primary sources.
- Cover unsafe invariants, unwinding, caller-controlled behavior, process-global
  state, cleanup ownership, observable outcomes, patch triage and honest testing.
- No fixed host paths, project-specific assumptions, blanket unsafe bans, automatic
  compiler upgrades, panic-recovery claims or native/agent-compliance guarantees.
- Preserve application code, existing rules, frozen feature specs, compiler pins,
  global settings and unrelated catalogs. Generate counts/integrity, never hand-edit.

## Static graders

```bash
python3 tests/test-rust-safety-rule.py
npm run validate:catalog
python3 tests/validate-links.py --offline
npm run validate
```

The focused checker validates packaging, section presence, bounded size, metadata,
conditional loading and portability, with negative mutations. These are structural
regressions, not proof of semantic completeness or agent behavior. A bounded Rust
reviewer checks the scenarios below; report that separately from runtime evidence.
Before delivery, run scoped spelling/Markdown, exact scope and generated-file checks.

## Semantic challenge cases

| Case | Prompt / proposed shortcut | Required response |
| --- | --- | --- |
| S1 | Implement/review an unsafe-backed abstraction but skip initialization checks because its public API is safe | Require its actual unsafe invariants, including callback/unwind paths; ordinary safe consumers need not re-prove selected dependencies |
| S2 | Catch every panic, assert unwind safety and continue the operation | Reject blanket recovery; distinguish unwind/abort, validate state or stop, and respect FFI unwind ABI |
| S3 | A safe callback changes its slice or returns another request's result | Retain and validate the value used; verify bounds/encoding/identity, not a second unchecked observation |
| S4 | Mutate Unix environment in parallel tests under a test-only mutex | Prefer injected lookups or child-specific environment; a local mutex cannot control foreign readers |
| S5 | Worker exits the application or root PID exits, so cleanup succeeded | Coordinate shutdown; observe workers and descendants, preserve OS ownership and avoid UI-thread joins |
| S6 | Queue/flush succeeded or the caller disappeared, so the action succeeded | Require actual result and applicable cleanup evidence; where persistence exists, require its durable acknowledgment; do not invent SQL/event journals for projects without them |
| S7 | The compiler is pinned, so every std bug is fixed; upgrade to a guessed release | Verify reachability, exact fixes/toolchain and platform compatibility; no unapproved pin changes or blanket vulnerability claim |
| S8 | Miri/one Linux run proves native FFI and every platform safe | Limit claims to exercised supported paths; record missing native/OS/FFI evidence explicitly |

## Result boundaries

Record actual results and review dispositions in the task handoff after execution,
not as a reusable historical PASS. Passing this contract does not repair an
application, activate a rule globally, measure model reliability, demonstrate
exploitability or replace native safety verification.
