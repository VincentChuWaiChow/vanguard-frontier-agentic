# Approved engine-only continuation: 3.4, 4.3 and 3.2 watcher

## Goal and authority

The user approved the three assessed slices with "go ahead for these". Implement
only governance scan quality, evidence-only admission/recovery, and owned watcher
refresh/lifecycle integration. This is local engine preparation, not native task
completion. Existing full-task dependencies and frozen requirements/design remain
unchanged. No installs/downloads, dependency changes, real helper execution, real
operator-target writes, UI work, external CI, commits, pushes or publication.

## Success criteria and graders

Parent owns failure disposition and independent integrated verification. Each
criterion is PASS, MISS or UNVERIFIED; unrun/native checks cannot become PASS.

1. Governance: report results derive from the same scan observations as their
   quality metadata. Named failures, expected/successful counts and exclusions
   distinguish successful empty, not configured, unavailable and partial inputs.
   Retained results preserve original provenance and are explicitly stale.
   Grader: `governance_contracts` with fixed independent workspace/report oracles,
   missing/invalid/unreadable/mixed inputs, suppressions and failed refresh.
2. Evidence: actual committed start receipt gates an inert action probe. Known
   action outcome and audit failure remain independent. Retained evidence is
   bounded; retries preserve identity/payload and never replay an action.
   Exclusive-owner restart recovery classifies nonterminal records as interrupted
   with unknown effects, without inventing success. Grader: `evidence_recovery`,
   real SQL/COMMIT failure, conflicting retries, bounded recovery enumeration,
   permanent-error stop and existing ownership/journal contracts.
   A minimal process-local guard also rejects duplicate evidence controllers
   sharing one `Arc<ProfileOwner>`; retain it through actual journal-worker closure.
   This complements OS ownership and does not prevent arbitrary raw-journal edits.
3. Watcher: notification overflow coalesces into a reliable dirty/rescan signal.
   Session replacement rejects obsolete events; explicit shutdown observes retry
   worker exit and stops owned notification resources. Grader: `watcher_lifecycle`
   and `catalog_session`, deterministic overflow/source-switch/shutdown cases and
   real Linux rename/delete/recreate smoke. No cross-platform acceptance inferred.
4. Compatibility: legacy scanner vector API and report payload/CLI behavior remain
   unchanged. No new normal presentation dependencies. Graders: full default and
   no-default Cargo suites, report/core golden contracts, strict all-target clippy,
   rustfmt, existing feature-boundary checks under both metadata lock graphs,
   ordinary package verification and packaged no-default library check.
5. Repository/scope: spec structural mutants, affected Node contracts, asset
   integrity, source-only repository validation, unchanged manifests/locks and
   approved input hashes; exact allowed diff and empty Git index. Graphify AST
   refresh follows code changes. Keep known cache-scanner failures separate.

## Integration surfaces and ownership

- Governance writer: `src/engine/governance.rs`, `src/federation/scanner.rs`,
  `src/headless/reporter.rs`, `src/models/report.rs`, `tests/governance_contracts.rs`
  and new fixtures only under `tests/fixtures/governance/`, all in `tools/vfa-tui`.
- Evidence writer: `src/engine/evidence.rs`, additive
  `src/persistence/operations.rs` queries/controller-authority guard and
  `tests/evidence_recovery.rs` only.
  No schema/migration/durability change; journal ownership stays authoritative.
- Watcher writer: `src/catalog/watcher.rs`, `src/engine/session.rs`,
  `tests/watcher_lifecycle.rs` and additive `tests/catalog_session.rs` cases only.
- Parent: shared module exports, these evaluation/progress documents, task
  preparation bookkeeping, integration, formatting and repository graders.

Paths above are relative to the explicit selected checkout, not operator paths.
The three writers do not consume one another's new APIs. Parent serializes shared
Cargo cache/target commands with `flock` and alone runs whole-crate formatting.
Tests use temporary fixtures and existing cached crates/toolchain, locked/offline.

## Route, bounds and blockers

One implementation wave with three exact runtime-exposed `rust_beast_mode`
specialists, configured Sol/medium/workspace-write. The offline profile lacks
name/description, but native tool metadata explicitly names the available role;
dispatch must succeed before treating it as live. One reused `rust_reviewer`,
configured Sol/high, provides independent read-only review. Serving identities
and effective read-only isolation remain unverified; assignments do not enforce
isolation. No recursive delegation. Parent owns all integration decisions.

At most three implementation runs, one integrated review run and one task-wide
post-review repair batch. Initial implementation timebox 60 minutes, total 90
minutes, not a completion promise. A failed safety gate, need for forbidden files,
new dependencies, uncontrolled scope or ambiguous outcome stops the affected
packet. Do not weaken old tests or add successful skips. Record runtime TDD RED
separately from an initial missing-API compile error.

## Evidence

Raw logs and frozen hashes: ignored `tools/vfa-tui/target/no-install-continuation/`.
Per-slice contracts: `task-3.4.md`, `task-4.3.md`, `task-3.2-watcher.md` beside this
file. Existing `.workflow/ultracode/gpui-desktop/` remains the sole parent workflow.
Initial status: defined before source implementation; results pending at that point.

## Integrated engine results

- PASS: actual initial runtime RED for all three slices, plus separate missing-API
  compile failures where applicable; no setup failure counted as behavioral RED.
- PASS: one independent source review identified two governance counterexamples.
  Both were reproduced by parent runtime regressions, then repaired in one batch
  with the strict-clippy test-borrow miss. No second independent review claimed.
- PASS: post-review full default suite 1,204 and engine-only suite 982, zero
  failures, one pre-existing ignored doctest in each. Both all-target strict
  clippy modes and crate formatting pass with locked/offline cached dependencies.
- PASS: unchanged core/report/export oracles, 56 Node exporter/static-CI tests,
  both metadata lock-graph boundaries, spec mutants and asset-integrity checks.
- PASS: ordinary Cargo package verification from an exact source-only snapshot;
  packaged no-default library check and all 100 Rust source bytes match. Tests
  and their helper fixtures remain excluded from the package.
- PASS: AST graph refresh; oversized optional HTML visualization is unavailable,
  not native UI evidence. Seven frozen approval/manifest/lock inputs are unchanged.
- MISS environment: checkout npm validation reproduces its ignored Cargo cache
  secret-pattern failure before source implementation. Snapshot sandbox execution
  independently reproduces Node spawnSync EPERM; a harmless approved outside-
  sandbox probe works. The unchanged pipeline then passes on the exact source-only
  snapshot in the separately tool-approved execution context; raw failures remain
  recorded alongside that pass in the parent final report.
- UNVERIFIED: native Desktop/second-lock behavior, other hosts, notify backend
  internal thread exit, coverage instrumentation and unavailable Markdown/spelling
  linters. No installation is performed to turn an unavailable check green.

Status: READY FOR REVIEW for engine preparations only; full tasks stay unchecked.
