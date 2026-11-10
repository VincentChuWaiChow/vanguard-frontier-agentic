# Implementation Plan: GPUI Desktop

Status: TASK PLAN REVISED; AWAITING IMPLEMENTATION APPROVAL. The user approved the
design, requested parallel task planning, then requested visible checkboxes and
Impeccable visual audit/refinement with portable paths. All boxes are future work.
Architecture and scope come from these frozen approved inputs:

- Requirements SHA-256: `490b95ba21dcf9d6ec6ca803586afd785f1803287c38f0193bed053701e31160`.
- Design SHA-256: `33a22ec84e2c376dd1e6aa627b72bf1a209a1773832c7fc3e417e2aa583699d0`.

## Tasks

- [ ] 1. Compatibility foundation and native bootstrap

- [ ] 1.1 Capture executable compatibility fixtures
  - Depends: none
  - Paths: tools/vfa-tui/tests/core_contracts.rs; tests/integration/headless_reports.rs; tests/fixtures/compatibility/ under tools/vfa-tui
  - Design: D1, D9, D12
  - Requirements: R11.1, R11.3-R11.6
  - Verification: V1, V2
  - Checks: C1
  - Positive: Capture reviewed public imports, actual data envelopes, caller version, stdout/stderr, exit codes and legacy data effects.
  - Negative: Missing fixtures and wrong-envelope/exit/schema mutants fail; normalize only declared volatile fields.
  - Estimate: 2-4 hours; timebox, not a completion guarantee.

- [ ] 1.2 Introduce the default-terminal feature boundary
  - Depends: 1.1
  - Paths: tools/vfa-tui/Cargo.toml; tools/vfa-tui/Cargo.lock; tools/vfa-tui/src/lib.rs; tools/vfa-tui/tests/check_feature_boundary.py; terminal-dependent module/test gates
  - Design: D1, D12, D13
  - Requirements: R11.1-R11.4, R12.6
  - Verification: V1, V2
  - Checks: C1, C2
  - Positive: Default builds preserve public paths and CLI/TUI; no-default library excludes presentation dependencies and binary requires terminal.
  - Negative: Closure assertions detect terminal/GPUI leakage; do not hide domain tests or invalidate package contents to compile.
  - Estimate: 2-4 hours; timebox, not a completion guarantee.

- [ ] 1.3 Separate pure report requests from the CLI wrapper
  - Depends: 1.1, 1.2
  - Paths: tools/vfa-tui/src/headless/; tools/vfa-tui/src/models/report.rs; tools/vfa-tui/tests/report_service_contracts.rs
  - Design: D1, D9, D12
  - Requirements: R10.3, R11.1-R11.6
  - Verification: V1, V2, V4
  - Checks: C1, C2, C3
  - Positive: ReportRequest/service returns owned results with explicit producer identity and preserves executable legacy format/persistence behavior.
  - Negative: Detect service stdout/global-subscriber side effects, changed producer/envelope, exit-code drift or silent data-policy changes.
  - Estimate: 2-4 hours; timebox, not a completion guarantee.

- [ ] 1.4 Pin the GPUI bootstrap and native test entry points
  - Depends: 1.2
  - Paths: tools/vfa-desktop/Cargo.toml; Cargo.lock; rust-toolchain.toml; src/main.rs; tests/ui.rs; tests/native_smoke.rs; provenance/toolkit.json under tools/vfa-desktop
  - Design: D1, D3, D4, D12, D13
  - Requirements: R12.1-R12.2, R12.4-R12.6, R16.1-R16.2, R16.8-R16.9
  - Verification: V1, V8, V10
  - Checks: C2, C4, C5
  - Positive: Record exact Kit/snapshot/source/checksum and verified toolchain graph; compile Root/assets and same-version test imports with actual backend metadata.
  - Negative: Normal graph excludes Fast/WebView/shell/test-support; renderer/prerequisite absence is diagnostic, not a successful skipped native launch.
  - Estimate: 2-4 hours; timebox, not a completion guarantee.

- [ ] 1.5 Implement the persistent engine host and foreground bridge
  - Depends: 1.2, 1.3, 1.4
  - Paths: tools/vfa-tui/src/engine/contracts.rs; tools/vfa-tui/src/engine/runtime.rs; tools/vfa-tui/tests/runtime_bridge.rs; tools/vfa-desktop/src/engine_bridge.rs; tools/vfa-desktop/tests/bridge.rs
  - Design: D2, D3, D10
  - Requirements: R9.1, R9.4, R9.7, R9.9, R11.2
  - Verification: V6, V8
  - Checks: C2, C3, C4
  - Positive: Define shared D10 IDs/events and typed commands/results; one long-lived Tokio host serves bounded commands and priority cancellation with actual GPUI foreground round trip.
  - Negative: Saturated/disconnected queues cannot starve cancel; no per-click runtime, UI-thread join or dropped-future-as-cleanup assertion.
  - Estimate: 2-4 hours; timebox, not a completion guarantee.

- [ ] 1.6 Run shared behavioral contracts under both consumer lock graphs
  - Depends: 1.1, 1.3, 1.4
  - Paths: tools/vfa-tui/tests/support/engine_cases.rs; tools/vfa-tui/tests/core_contracts.rs; tools/vfa-desktop/tests/engine_contracts.rs
  - Design: D1, D9, D12, D13
  - Requirements: R11.1, R11.3, R11.6, R16.2
  - Verification: V1, V2, V3, V4
  - Checks: C1, C2, C3, C4
  - Positive: Reuse fixed independent engine cases under TUI and Desktop lockfiles with explicit fixture roots; each subsequent shared portability fix adds its regression to both runs.
  - Negative: Compile-only graph checks or only-TUI test results cannot pass parity; missing fixtures, different producer/schema semantics and missing second-graph regressions fail.
  - Estimate: 2-4 hours; timebox, not a completion guarantee.

- [ ] 2. Native process and runtime feasibility

- [ ] 2.1 Define portable process contracts and the controlled helper
  - Depends: 1.2, 1.5, 1.6
  - Paths: tools/vfa-tui/src/subprocess/; tools/vfa-tui/tests/support/process_helper.rs; tools/vfa-tui/tests/process_contracts.rs
  - Design: D5, D7, D10
  - Requirements: R9.1-R9.3, R9.8, R12.3, R15.3-R15.5
  - Verification: V6, V11
  - Checks: C2, C3, C6
  - Positive: Typed ProgramSpec and owned handles/control use a portable std-only helper for descendants, delayed exits and output modes.
  - Negative: Injection-like argv stays literal and invalid encoding fails; temporary helper compilation cannot create publish-invalid binary targets.
  - Estimate: 2-4 hours; timebox, not a completion guarantee.

- [ ] 2.2 Implement and verify Unix descendant ownership
  - Depends: 2.1
  - Paths: tools/vfa-tui/src/subprocess/executor.rs; tools/vfa-tui/src/subprocess/signal.rs; tools/vfa-tui/src/subprocess/unix.rs; tools/vfa-tui/tests/process_contracts.rs
  - Design: D7, D10, D11
  - Requirements: R9.3, R9.8, R11.6, R14.4
  - Verification: V6
  - Checks: C1, C3, C6
  - Positive: Observe TERM/grace/KILL cleanup including root-first exit on both native Linux and macOS; retain ownership until descendants terminate.
  - Negative: Repeated cancellation, timeout races, escaping descendants and PID reuse cannot yield false clean cancellation; stop admission on unsupported ownership.
  - Estimate: 2-4 hours; timebox, not a completion guarantee.

- [ ] 2.3 Implement the Windows owned Job and structured launcher
  - Depends: 2.1
  - Paths: tools/vfa-tui/Cargo.toml; tools/vfa-tui/Cargo.lock; tools/vfa-tui/src/subprocess/windows.rs; tools/vfa-tui/tests/process_contracts.rs
  - Design: D5, D7, D11
  - Requirements: R9.3, R12.3, R15.3-R15.5
  - Verification: V6, V11
  - Checks: C2, C3, C6
  - Positive: Native MSVC tests prove Job ownership, no breakaway, failure cleanup, native quoted/Unicode argv and inherited-job behavior.
  - Negative: Assignment/creation-window failure and root-first exit leave no unobserved live descendants; no direct-child-kill or shell-string substitute.
  - Estimate: 2-4 hours; timebox, not a completion guarantee.

- [ ] 2.4 Bound output and separate protocol from display loss
  - Depends: 2.1
  - Paths: tools/vfa-tui/src/subprocess/executor.rs; tools/vfa-tui/src/subprocess/stream.rs; tools/vfa-tui/tests/output_budget.rs
  - Design: D6, D7
  - Requirements: R9.4-R9.6, R14.5
  - Verification: V5, V6
  - Checks: C2, C3, C6
  - Positive: Chunk reads share one measured 8 MiB budget across streams/framing/tail; protocol hooks precede lossy display and terminal delivery is reliable.
  - Negative: 64 MiB unterminated output and slow/disconnected UI cannot grow buffers, deadlock drain or discard terminal/effects evidence silently.
  - Estimate: 2-4 hours; timebox, not a completion guarantee.

- [ ] 2.5 Implement bounded stateful streaming redaction
  - Depends: 2.4
  - Paths: tools/vfa-tui/src/security/redact.rs; tools/vfa-tui/src/security/sanitize.rs; tools/vfa-tui/tests/streaming_redaction.rs
  - Design: D5, D6, D7, D11
  - Requirements: R9.5-R9.6, R15.5-R15.6
  - Verification: V6, V11
  - Checks: C1, C3, C6
  - Positive: Documented secret patterns stay suppressed across chunks/lines without buffering entire secret spans; raw protocol remains internal.
  - Negative: Long or unterminated PEM, oversized ambiguous tokens, EOF and cancellation disclose no recognized pending secret; conservative loss is marked.
  - Estimate: 2-4 hours; timebox, not a completion guarantee.

- [ ] 2.6 Integrate lifecycle races and the native feasibility barrier
  - Depends: 1.4, 1.5, 2.2, 2.3, 2.4, 2.5
  - Paths: tools/vfa-tui/src/engine/operations.rs; tools/vfa-tui/tests/lifecycle.rs; tools/vfa-desktop/tests/bridge.rs; tools/vfa-desktop/tests/native_smoke.rs
  - Design: D3, D7, D10, D11, D12, D13
  - Requirements: R9.1-R9.9, R12.1-R12.2, R14.4, R16.9
  - Verification: V6, V8, V10
  - Checks: C3, C4, C5, C6
  - Positive: Serialize exit/cancel/timeout into one outcome; real native bootstrap/bridge Wait or Cancel reaps and drains before runtime/window exit on all targets.
  - Negative: Prelaunch cancellation spawns nothing; duplicate/late events and cleanup uncertainty never pass admission. Durable journal recovery is completed in 4.3.
  - Estimate: 2-4 hours; timebox, not a completion guarantee.

- [ ] 3. Reusable catalog and governance services

- [ ] 3.1 Implement source identity and immutable catalog snapshots
  - Depends: 1.2, 1.5, 2.1
  - Paths: tools/vfa-tui/src/engine/session.rs; tools/vfa-tui/src/engine/snapshot.rs; tools/vfa-tui/tests/session_contracts.rs
  - Design: D2, D10, D11
  - Requirements: R1.1-R1.6, R2.1-R2.6, R11.4, R15.1
  - Verification: V3, V11
  - Checks: C2, C3
  - Positive: Reuse loader/models with canonical native identity, epochs, honest collection status and stable entity IDs; reject source replacement for every nonterminal state.
  - Negative: Invalid markers, malformed collections and duplicate IDs remain named failures; browse triggers no helper, target write or network request.
  - Estimate: 2-4 hours; timebox, not a completion guarantee.

- [ ] 3.2 Unify search, reload and stale-result rejection
  - Depends: 3.1
  - Paths: tools/vfa-tui/src/search/; tools/vfa-tui/src/catalog/store.rs; tools/vfa-tui/src/catalog/watcher.rs; tools/vfa-tui/src/engine/catalog.rs; tools/vfa-tui/tests/catalog_session.rs
  - Design: D2, D3, D10, D11
  - Requirements: R1.5, R3.1-R3.7, R4.1-R4.6
  - Verification: V3, V5, V9
  - Checks: C1, C2, C3
  - Positive: Shared ranking/filter intersection and other-entity matching are deterministic; watcher/manual refresh share revisions and stale-data rules.
  - Negative: Reordered requests cannot replace current data; failed/deleted reload is not current-empty; unchanged bytes keep revision and removed selection clears.
  - Estimate: 2-4 hours; timebox, not a completion guarantee.

- [ ] 3.3 Expose real dependency and integrity findings
  - Depends: 3.1
  - Paths: tools/vfa-tui/src/federation/dep_graph.rs; tools/vfa-tui/src/federation/integrity.rs; tools/vfa-tui/src/engine/findings.rs; tools/vfa-tui/tests/relationship_integrity.rs
  - Design: D2, D9, D10, D11
  - Requirements: R5.1-R5.5, R11.1
  - Verification: V4
  - Checks: C1, C3
  - Positive: Existing graph/hash algorithms return exact edges, cycles, source paths and matched/missing/mismatched findings with provenance.
  - Negative: Missing/invalid manifest is unavailable, unresolved references remain findings and inspected content is never executed or opened automatically.
  - Estimate: 2-4 hours; timebox, not a completion guarantee.

- [ ] 3.4 Populate governance results with honest input quality
  - Depends: 1.3, 3.1, 3.2, 3.3
  - Paths: tools/vfa-tui/src/engine/governance.rs; tools/vfa-tui/src/federation/scanner.rs; tools/vfa-tui/src/policy/; tools/vfa-tui/tests/governance_contracts.rs
  - Design: D9, D10, D11
  - Requirements: R6.1-R6.6, R6.8-R6.10, R10.3
  - Verification: V2, V4
  - Checks: C1, C3
  - Positive: Two fixed workspace fixtures yield exact coverage/drift/policy results, suppression semantics and report parity through existing services.
  - Negative: Absent/invalid/unreadable/mixed inputs produce NOT_CONFIGURED, UNAVAILABLE or PARTIAL with denominators/exclusions and stale provenance, not healthy zeros.
  - Estimate: 2-4 hours; timebox, not a completion guarantee.

- [ ] 4. Profile, evidence and execution safety

- [ ] 4.1 Implement private profile paths and exclusive ownership
  - Depends: 1.2
  - Paths: tools/vfa-tui/src/engine/profile.rs; tools/vfa-tui/tests/profile_ownership.rs
  - Design: D8, D11
  - Requirements: R9.8, R10.4-R10.6, R10.10
  - Verification: V7, V11
  - Checks: C2, C3
  - Positive: OS-appropriate private roots or explicit overrides use native identity and an OS owner lock held for the session; test two native processes.
  - Negative: A second instance cannot recover live jobs or write the owned profile; no temporary fallback, PID-file inference or active-lock unlink.
  - Estimate: 2-4 hours; timebox, not a completion guarantee.

- [ ] 4.2 Implement the isolated journal and real commit receipts
  - Depends: 4.1
  - Paths: tools/vfa-tui/src/persistence/operations.rs; tools/vfa-tui/src/persistence/operation_schema.rs; tools/vfa-tui/tests/journal_commits.rs
  - Design: D8, D10
  - Requirements: R10.1-R10.2, R10.4-R10.5, R10.8, R10.11
  - Verification: V7
  - Checks: C2, C3
  - Positive: Distinct schema identity and transactional event/projection updates return a receipt only after successful COMMIT; query actual persisted records.
  - Negative: Injected SQL/commit failures are errors, conflicting duplicate payloads reject, foreign/newer stores stay intact and legacy v4 fixture hashes never change.
  - Estimate: 2-4 hours; timebox, not a completion guarantee.

- [ ] 4.3 Implement evidence admission, recovery and idempotence
  - Depends: 1.5, 2.6, 4.2
  - Paths: tools/vfa-tui/src/engine/evidence.rs; tools/vfa-tui/tests/evidence_recovery.rs
  - Design: D3, D8, D10, D11
  - Requirements: R9.2, R9.8, R10.1, R10.5, R10.9-R10.11
  - Verification: V6, V7
  - Checks: C3, C6
  - Positive: Durable start gates action; observed outcome remains separate from audit status; bounded evidence retries deduplicate without replay under profile ownership.
  - Negative: Failed start launches nothing; terminal commit failure preserves known success plus audit FAILED; permanent errors never loop or trigger action replay.
  - Estimate: 2-4 hours; timebox, not a completion guarantee.

- [ ] 4.4 Resolve native helpers and validated launch specifications
  - Depends: 1.2, 2.6
  - Paths: tools/vfa-tui/src/engine/helper_runtime.rs; tools/vfa-tui/tests/helper_runtime.rs
  - Design: D5, D7
  - Requirements: R7.2-R7.3, R12.3, R15.3-R15.5
  - Verification: V6, V11
  - Checks: C2, C3
  - Positive: Resolve installed Node/npm CLI/Python and construct native ProgramSpec with filtered environment; execute real native Windows prerequisite fixtures.
  - Negative: Missing helpers diagnose without installing; spaces/Unicode/drive separators keep identity; injection-like values cannot become shell strings.
  - Estimate: 2-4 hours; timebox, not a completion guarantee.

- [ ] 4.5 Bind session trust to execution manifests
  - Depends: 3.1, 4.3, 4.4
  - Paths: tools/vfa-tui/src/engine/execution_manifest.rs; tools/vfa-tui/src/engine/trust.rs; tools/vfa-tui/tests/execution_trust.rs
  - Design: D2, D5, D10
  - Requirements: R7.6, R15.1-R15.3, R15.8-R15.9
  - Verification: V7, V11
  - Checks: C3
  - Positive: Enumerate known code/config/runtime identities without executing checkout code; grants bind current revision and disclosed effects.
  - Negative: Changed code invalidates grants; incomplete inventories require execution-specific trust; no persisted trust or browse-time helper/version probes.
  - Estimate: 2-4 hours; timebox, not a completion guarantee.

- [ ] 4.6 Implement confirmed local registry and report-file writes
  - Depends: 1.3, 1.5, 4.1, 4.3
  - Paths: tools/vfa-tui/src/engine/local_writes.rs; tools/vfa-tui/tests/local_writes.rs
  - Design: D8, D9, D10, D11
  - Requirements: R6.7, R10.1, R10.3, R10.5-R10.7, R10.10
  - Verification: V2, V7, V11
  - Checks: C3
  - Positive: Existing services supply registry/report content; confirmation binds path/change/overwrite; profile preferences and writes retain operation evidence.
  - Negative: Dismissed/stale confirmations or missing audit receipt produce zero target writes; preserve existing files and clean only owned uncommitted temporaries.
  - Estimate: 2-4 hours; timebox, not a completion guarantee.

- [ ] 4.7 Integrate cancellation and shutdown of real blocking workers
  - Depends: 1.5, 2.6, 3.4, 4.2, 4.3
  - Paths: tools/vfa-tui/src/engine/runtime.rs; tools/vfa-tui/tests/worker_shutdown.rs; tools/vfa-desktop/tests/bridge.rs; tools/vfa-desktop/tests/native_smoke.rs
  - Design: D3, D8, D10, D11, D12
  - Requirements: R9.2, R9.4, R9.7-R9.9, R10.9-R10.11
  - Verification: V6, V7, V8
  - Checks: C3, C4, C5, C6
  - Positive: Actual scans cooperate with cancellation; controlled SQLite busy/interruption, writer drain and foreground shutdown acknowledgment finish with observed worker exit and preserved evidence.
  - Negative: Dropping a future, returning from shutdown_timeout or passing subprocess-only admission cannot prove blocking workers stopped; no premature terminal/shutdown success under pressure.
  - Estimate: 2-4 hours; timebox, not a completion guarantee.

- [ ] 5. Authoritative export and gate integration

- [ ] 5.1 Build one authoritative final-artifact export planner
  - Depends: none
  - Paths: scripts/export-marketplace-agents.mjs; tests/export-plan.test.mjs
  - Design: D6
  - Requirements: R8.1-R8.3, R11.5
  - Verification: V5
  - Checks: C7
  - Positive: One Node planner/renderer serves legacy and plan-json modes with identities, collisions, companions, transforms and final-content digests.
  - Negative: Preview creates no target file/directory/lock; unknown effects fail; Codex destination-dependent bytes and existing export modes match fixed expectations.
  - Estimate: 2-4 hours; timebox, not a completion guarantee.

- [ ] 5.2 Implement verified apply and reliable protocol outcomes
  - Depends: 2.4, 2.5, 2.6, 4.3, 5.1
  - Paths: scripts/export-marketplace-agents.mjs; tools/vfa-tui/src/engine/export_protocol.rs; tests/export-apply.test.mjs; tools/vfa-tui/tests/export_protocol.rs
  - Design: D6, D7, D8
  - Requirements: R8.5, R8.7, R8.9-R8.10, R9.5-R9.6, R15.5
  - Verification: V5, V6, V7
  - Checks: C3, C6, C7
  - Positive: Revalidate whole plan then writes; derive final bytes once and parse bounded ordered outcomes before lossy/redacted projections.
  - Negative: Changed identities, malformed/oversized frames, missing summary and evidence failure stop writes with retained effects plus uncertainty; zero-exit truncation cannot pass.
  - Estimate: 2-4 hours; timebox, not a completion guarantee.

- [ ] 5.3 Wire single-use preview authorization to execution
  - Depends: 1.5, 3.1, 3.2, 4.3, 4.4, 4.5, 5.2
  - Paths: tools/vfa-tui/src/engine/export.rs; tools/vfa-tui/tests/export_authorization.rs
  - Design: D2, D3, D5, D6, D8, D10
  - Requirements: R4.6, R8.1-R8.10, R9.1-R9.2, R10.1, R10.5
  - Verification: V3, V5, V7, V11
  - Checks: C3, C7
  - Positive: Trusted/audited preview yields a current single-use permit; confirmation rechecks freshness/identities and durably records one apply authorization.
  - Negative: Duplicate/dismissed/stale inputs or failed audit yield zero apply calls; cancellation/timeouts preserve partial/unknown effects without replay or rollback.
  - Estimate: 2-4 hours; timebox, not a completion guarantee.

- [ ] 5.4 Integrate trusted authoritative gate execution
  - Depends: 1.5, 2.6, 3.1, 4.3, 4.4, 4.5
  - Paths: tools/vfa-tui/src/engine/validation.rs; tools/vfa-tui/tests/trusted_validation.rs
  - Design: D3, D5, D7, D8
  - Requirements: R7.1-R7.6, R9.1-R9.3, R10.5, R15.8-R15.9
  - Verification: V6, V7, V11
  - Checks: C1, C3
  - Positive: Existing gate definitions/dependencies and full npm validate pipeline keep their exact semantics; Run follows trust and durable authorization.
  - Negative: Missing runtime/gate, stale manifest or failed audit starts nothing; failed dependencies cannot yield aggregate success; retain actual exit/cleanup evidence.
  - Estimate: 2-4 hours; timebox, not a completion guarantee.

- [ ] 6. Native desktop workflows

- [ ] 6.1 Build the native shell and explicit checkout picker
  - Depends: 1.4, 1.5, 2.6, 3.1, 4.1, 4.7
  - Paths: tools/vfa-desktop/src/app.rs; tools/vfa-desktop/src/checkout_picker.rs; tools/vfa-desktop/tests/ui/context.rs; parent-approved Desktop-local product/surface context artifacts if absent
  - Design: D2, D3, D4
  - Requirements: R1.1-R1.6, R12.4, R15.1, R15.7
  - Verification: V3, V8, V11
  - Checks: C4, C5
  - Positive: Resolve Impeccable context/visual direction and craft-floor prerequisites before UI edits without inventing facts; canonical context, saved-source validation and unavailable/picker state work with keyboard and pointer through the real bridge.
  - Negative: No implicit source/trust grant, telemetry, download, helper/network/target write, credential prompt or cloud action at launch/browse; obsolete results cannot repopulate a switched context.
  - Estimate: 2-4 hours; timebox, not a completion guarantee.

- [ ] 6.2 Build catalog lists and complete entity details
  - Depends: 3.1, 6.1
  - Paths: tools/vfa-desktop/src/catalog/mod.rs; tools/vfa-desktop/src/catalog/details.rs; tools/vfa-desktop/tests/ui/catalog.rs
  - Design: D2, D4, D10
  - Requirements: R2.1-R2.6, R15.6
  - Verification: V3, V8, V11
  - Checks: C4
  - Positive: All six collections and their relationships/metadata use reusable native controls with stable IDs and dynamic counts/options.
  - Negative: Missing optional fields and failed/empty collections stay distinct; external content/links are inert and no fabricated metadata appears.
  - Estimate: 2-4 hours; timebox, not a completion guarantee.

- [ ] 6.3 Wire query filters, stable selection and freshness
  - Depends: 3.2, 6.2
  - Paths: tools/vfa-desktop/src/catalog/search.rs; tools/vfa-desktop/src/catalog/filters.rs; tools/vfa-desktop/src/catalog/freshness.rs; tools/vfa-desktop/tests/ui/search.rs
  - Design: D2, D3, D4, D10
  - Requirements: R3.1-R3.7, R4.1-R4.6
  - Verification: V3, V5, V8, V9
  - Checks: C3, C4
  - Positive: Actual controls drive shared intersection/ranking, Unicode, no-match editing and honest reload state with stable entity selection.
  - Negative: Out-of-order results cannot overwrite current state; deleted selection clears and failed refresh never becomes current-empty or confirms a stale preview.
  - Estimate: 2-4 hours; timebox, not a completion guarantee.

- [ ] 6.4 Build relationships and integrity evidence views
  - Depends: 3.3, 6.2
  - Paths: tools/vfa-desktop/src/relationships.rs; tools/vfa-desktop/src/integrity.rs; tools/vfa-desktop/tests/ui/findings.rs
  - Design: D4, D9, D11
  - Requirements: R5.1-R5.5
  - Verification: V4, V8
  - Checks: C3, C4
  - Positive: Selectable native findings show real edges, paths, hashes and provenance from engine fixtures.
  - Negative: Cycle/missing-reference/manifest failures remain visible rather than green status; selection never automatically opens or executes content.
  - Estimate: 2-4 hours; timebox, not a completion guarantee.

- [ ] 6.5 Build populated governance and registry workflows
  - Depends: 3.4, 4.6, 6.1
  - Paths: tools/vfa-desktop/src/governance/; tools/vfa-desktop/tests/ui/governance.rs
  - Design: D4, D8, D9, D10, D11
  - Requirements: R6.1-R6.10
  - Verification: V4, V7, V8
  - Checks: C3, C4
  - Positive: Real workspace coverage/drift/policy views include scopes/severity/provenance and explicit confirmed registration/removal.
  - Negative: Absent/invalid/mixed/stale inputs cannot look like complete healthy zeros; unconfirmed registry actions perform no mutation.
  - Estimate: 2-4 hours; timebox, not a completion guarantee.

- [ ] 6.6 Build validation jobs and honest output controls
  - Depends: 5.4, 6.1
  - Paths: tools/vfa-desktop/src/jobs/; tools/vfa-desktop/tests/ui/jobs.rs
  - Design: D3, D4, D5, D7, D8, D10, D11
  - Requirements: R7.1-R7.6, R9.1-R9.9
  - Verification: V6, V7, V8, V11
  - Checks: C3, C4, C6
  - Positive: Run/cancel, redacted bounded tail and actual lifecycle/exit/evidence states work from pointer and keyboard.
  - Negative: Missing helpers, failed dependencies, output loss and incomplete cleanup cannot be painted as success or complete output.
  - Estimate: 2-4 hours; timebox, not a completion guarantee.

- [ ] 6.7 Build preview-bound export workflow
  - Depends: 5.3, 6.1
  - Paths: tools/vfa-desktop/src/export/; tools/vfa-desktop/tests/ui/export.rs
  - Design: D4, D5, D6, D7, D8, D10, D11
  - Requirements: R8.1-R8.10
  - Verification: V5, V6, V7, V8
  - Checks: C3, C4, C7
  - Positive: Wizard presents authoritative source/destination/selection/conflicts/effects and consumes one explicit confirmation for an unchanged preview.
  - Negative: Cancel makes no target writes; double-click/stale inputs cannot reauthorize; partial effects and uncertainty survive failures without retry/rollback.
  - Estimate: 2-4 hours; timebox, not a completion guarantee.

- [ ] 6.8 Build persisted audit and report views
  - Depends: 1.3, 4.2, 4.3, 4.6, 6.1
  - Paths: tools/vfa-desktop/src/reports/; tools/vfa-desktop/tests/ui/reports.rs
  - Design: D4, D8, D9, D10, D11
  - Requirements: R10.1-R10.11
  - Verification: V2, V4, V7, V8
  - Checks: C3, C4
  - Positive: Audit queries actual records, durability stays independent of execution, and report/file-export quality and overwrite confirmation remain explicit.
  - Negative: A locked/corrupt/failed store is not an empty log or durable success; no legacy schema reset/import/deletion occurs.
  - Estimate: 2-4 hours; timebox, not a completion guarantee.

- [ ] 6.9 Complete common theme, keyboard and focus semantics
  - Depends: 6.1, 6.2, 6.3, 6.4, 6.5, 6.6, 6.7, 6.8
  - Paths: tools/vfa-desktop/src/ui/; tools/vfa-desktop/tests/ui/common_accessibility.rs
  - Design: D4, D11, D12
  - Requirements: R13.1-R13.9
  - Verification: V8
  - Checks: C4
  - Positive: All required flows have real keyboard actions, accessible names/state, modal restoration, OS/override theme and reachable scaled controls.
  - Negative: Disconnected callbacks despite matching labels fail; focus traps, color-only errors and clipped 200-percent controls fail. Native AT proof remains separate.
  - Estimate: 2-4 hours; timebox, not a completion guarantee.

- [ ] 7. Complete native acceptance

- [ ] 7.1 Implement and run complete native workflow scenarios
  - Depends: 2.6, 6.9
  - Paths: tools/vfa-desktop/tests/native_smoke.rs; tools/vfa-desktop/tests/native_smoke/
  - Design: D4, D12, D13
  - Requirements: R12.1-R12.6, R13.1-R13.9, R16.8-R16.9
  - Verification: V8, V10
  - Checks: C5, C8
  - Positive: Scenario driver runs built application workflows on all four native backend configurations with actual window/interaction evidence.
  - Negative: Wrong backend, missing native host or headless-only/compile-only artifacts cannot pass; packaged rerun is a distinct dependent check in 8.1.
  - Estimate: 2-4 hours; timebox, not a completion guarantee.

- [ ] 7.2 Implement native AT, IME and clipboard evidence capture
  - Depends: 6.9, 7.1
  - Paths: tools/vfa-desktop/tests/native_accessibility.rs; tools/vfa-desktop/tests/native_accessibility/
  - Design: D4, D12
  - Requirements: R13.1-R13.9
  - Verification: V8
  - Checks: C8
  - Positive: Harness records platform/AT versions and supports reproducible real keyboard, announcements, composition, clipboard and scaling observations.
  - Negative: Missing human/AT observations, synthetic tree-only records or wrong-target evidence are UNVERIFIED and cannot complete acceptance.
  - Estimate: 2-4 hours; timebox, not a completion guarantee.

- [ ] 7.3 Implement the fixed performance corpus and measurement harness
  - Depends: 6.3, 6.6, 6.9
  - Paths: tools/vfa-desktop/tests/performance.rs; tools/vfa-desktop/tests/fixtures/performance/
  - Design: D2, D3, D7, D12
  - Requirements: R14.1-R14.6
  - Verification: V6, V8, V9
  - Checks: C9
  - Positive: Exact 10000-entity corpus, query mix/pacing, warmup, simultaneous pressure, raw samples and nearest-rank budgets are measured per reference target.
  - Negative: Empty/changed fixtures, missing overlap, excluded slow samples or another platforms results cannot pass; no unmeasured performance claim.
  - Estimate: 2-4 hours; timebox, not a completion guarantee.

- [ ] 7.4 Audit native visual quality with Impeccable
  - Depends: 7.1, 7.2, 7.3
  - Paths: Read-only resolved Desktop source/tokens and native workflow evidence; write only the task-scoped audit report in the explicitly selected evidence bundle
  - Method: impeccable critique + audit
  - Design: D4, D11, D12
  - Requirements: R13.1-R13.9, R16.7-R16.9
  - Verification: V8, V9, V10, V12
  - Checks: C8, C9, C13
  - Positive: One batched native inspection produces scene-linked prioritized findings across the C13 matrix; paths are discovered and relocation/Unicode fixtures work; desktop-specific evidence and mobile-only exclusions are explicit.
  - Negative: Mock/web-only or stale/wrong-backend captures, invented desktop support, fixed install/output paths or score-only findings fail; audit makes no UI edits and does not waive AT/performance proof.
  - Estimate: 2-4 hours; timebox, not a completion guarantee.

- [ ] 7.5 Refine the audited UI with Impeccable and confirm once
  - Depends: 7.4
  - Paths: Only resolved Desktop UI source/tests in the parent-approved audit remediation set; task-scoped confirmation report in the selected evidence bundle
  - Method: impeccable polish
  - Design: D4, D11, D12
  - Requirements: R13.1-R13.9, R14.6, R16.7-R16.9
  - Verification: V8, V9, V10, V12
  - Checks: C4, C8, C9, C13
  - Positive: One scoped fix batch preserves the established identity and safety behavior; at most one native confirmation round binds before/after findings and C4/C8/C9 results to the final build; lesser deferrals have rationale/owner.
  - Negative: Unresolved P0/P1, stale AT/performance results, out-of-scope redesign/copy/behavior changes, unportable paths or repeated polishing cannot pass; stop and replan rather than mark unfinished work complete.
  - Estimate: 2-4 hours; timebox, not a completion guarantee.

- [ ] 8. Delivery and final readiness

- [ ] 8.1 Implement verified per-target packaging and notices
  - Depends: 1.4, 7.1
  - Paths: tools/vfa-desktop/packaging/; scripts/check-vfa-desktop-package.py; tools/vfa-desktop/tests/packaging/
  - Design: D1, D12, D13
  - Requirements: R12.1-R12.6, R16.1-R16.5, R16.8-R16.9
  - Verification: V1, V8, V10
  - Checks: C1, C2, C10
  - Positive: Packages carry actual assets/notices/checksums; clean-target unpacked apps rerun workflows from unrelated CWD with selected checkout and explicit helper prerequisites.
  - Negative: Missing runtime/asset/license, undeclared unsigned status, developer-CWD dependency or security bypass instructions fail; no dirty/no-verify publication-gate bypass.
  - Estimate: 2-4 hours; timebox, not a completion guarantee.

- [ ] 8.2 Wire cross-consumer local CI and supply-chain gates
  - Depends: 1.2, 1.4
  - Paths: .github/workflows/vfa-tui-ci.yml; .github/workflows/gpui-desktop-ci.yml; tools/vfa-desktop/deny.toml; tests/gpui-ci-contracts.test.mjs
  - Design: D1, D12, D13
  - Requirements: R11.1-R11.6, R12.1-R12.6, R16.1-R16.2, R16.6, R16.8-R16.9
  - Verification: V1, V2, V8, V10, V12
  - Checks: C1, C2, C4, C11
  - Positive: Path-filter fixtures require shared-engine/exporter changes to exercise both graphs and native gates; validate target/source/license policy without weakening TUI policy.
  - Negative: Filtered-out required consumer, green skipped jobs, incompatible toolkit snapshots or altered authoritative validation semantics fail; no external run/push/upload without approval.
  - Estimate: 2-4 hours; timebox, not a completion guarantee.

- [ ] 8.3 Implement final evidence readiness and regression closure
  - Depends: 7.1, 7.2, 7.3, 7.5, 8.1, 8.2
  - Paths: scripts/verify-gpui-desktop-readiness.py; tests/gpui-readiness/; .kiro/specs/gpui-desktop/evidence/; actually affected generated integrity artifacts
  - Design: D11, D12, D13
  - Requirements: R11.6, R12.5, R14.6, R16.3-R16.9
  - Verification: V1, V2, V3, V4, V5, V6, V7, V8, V9, V10, V11, V12
  - Checks: C1, C2, C4, C6, C7, C8, C9, C10, C11, C12, C13
  - Positive: Verifier binds every accepted criterion to underlying passing tests/native artifacts and exact source/lock identity; parent audits assertions, updates completion evidence and runs settled-tree gates.
  - Negative: Missing/stale/mismatched evidence, unsupported self-attested PASS-only records and passed skips fail; missing human observations, unresolved visual P0/P1 or stale post-refinement gates block acceptance, while valid attributed native observations are accepted.
  - Estimate: 2-4 hours; timebox, not a completion guarantee.

## Execution Contract

Use a hybrid risk-first sequence. Numbering is a valid topological order, not a
requirement to serialize independent work. A dependency must be complete with its
actual evidence before a consumer starts. In particular, task 2.6 is the native
feasibility barrier before extensive UI work; independent domain/service tasks may
progress while native runs are pending. Missing target access is a prerequisite
gap, not a passed test or permission to remove Windows/macOS/Wayland.

Each task is a 2–4-hour focused-work estimate/timebox, excluding compiler/host/human
waiting. If it exceeds that bound, stop and split the remaining implementation
with preserved requirements/dependencies rather than mark an incomplete task done.
Native access, dependency resolution/system packages and any external CI execution
need appropriate authorization; no installs, remote runners, pushes or release
uploads are granted merely by this task-planning document.

The parent/integration owner controls shared manifests, module declarations and
cross-file contracts, shared fixture registration and the `tests/ui.rs` aggregator.
Leaf UI tests have separate modules so workers need not edit that shared file.
At most three implementation workers, no recursive fanout.
Never run two tasks concurrently if their actual write sets overlap (including
Cargo.lock, module registration, exporter or shared fixtures), even if their graph
dependencies permit it. Each worker owns only assigned paths, preserves others'
changes, and does not commit/push/deploy. Sensitive authorization/evidence changes
receive parent review. Select actual available specialist profiles at dispatch;
the plan does not pretend to pin future serving models or isolate workers.

TUI library/default compatibility and both consumer lock graphs stay protected.
No target writes against real operator repositories in automated tests; use temp
fixtures and controlled helper processes. No green skips, missing-fixture returns,
copied expected values from production algorithms, or metadata-only UI assertions.
Do not change approved requirements/design to simplify a failing implementation.

## Portable Visual Quality Workflow

Use the installed `impeccable` skill by name, not a copied installation path.
Discover its base directory/launcher from the active harness's skill metadata and
follow its current instructions. Do not assume a home directory, repository-local
skill folder, OS path separator or global executable. Missing skill/launcher is
an explicit prerequisite failure, not permission to install/update it or use an
unrelated fallback. Run its context setup once per implementation session against
the resolved Desktop source target; follow its documented failure fallback if
context loading fails. Do not repair skill/config drift as a side effect.

Resolve the implementation source root from the explicitly chosen source checkout
or trusted repository metadata. Resolve package manifests by package identity and
Cargo metadata, validating membership in that source tree. In the commands below,
`ENGINE_MANIFEST` and `DESKTOP_MANIFEST` are discovered native paths, not baked-in
locations. Run repository-relative checks from the resolved source root. `Paths`
fields identify proposed relative source ownership, not runtime path constants;
each worker packet must resolve them to a concrete bounded write set at dispatch.
Keep the app's operator-selected data checkout distinct from its source/build
checkout. Never select either implicitly from CWD.

The operator/test invocation supplies the evidence destination explicitly. Resolve
native paths and pass them as structured arguments; do not assume a fixed output
folder, user name, drive, temporary directory or file URL. Store report/image links
relative to the selected evidence bundle where possible. Test a relocated source
checkout and destinations containing spaces/Unicode; absent/ambiguous roots,
missing skill, unwritable output and malformed arguments fail explicitly without
silent fallback or writes elsewhere. Do not execute untrusted discovered helpers.

This is a native Linux/macOS/Windows GPUI **Operate** surface. Impeccable provides
design judgment, not GPUI platform support: its shipped native audit guidance is
mobile-oriented, so adapt only applicable hierarchy, typography, spacing, theme,
contrast, focus, scaling, state and window-resizing heuristics. Mark mobile-only
checks not applicable with reasons; assess desktop-specific behavior using the
approved desktop requirements and actual native evidence. Unsupported desktop
assessments remain UNVERIFIED, never an inferred pass. Do not run web-only browser
overlays, `live`, `generate` or the HTML/CSS detector against Rust/native windows.

Before the first UI edit, follow Impeccable's context/new-surface prerequisites,
resolve product/visual authority without inventing missing facts, and load its
craft-floor guidance. Obtain any missing visual-direction decisions without
altering the approved architecture. Later critique/audit is read-only; refinement
preserves identity, factual copy, behavior, safety contracts and native affordances.
Use one initial batched inspection across required backends/themes/window states,
one prioritized fix batch, and at most one native confirmation round. Reuse fresh
workflow captures instead of adding redundant screenshot passes. Unresolved
blockers stop acceptance and require a reviewed replan, not an open-ended polish
loop. Neither screenshots nor a visual score replace real AT or performance runs.

## Check Registry

Commands/targets below are **proposed acceptance interfaces** unless already
present. Task owners implement their named harnesses before invoking them. Their
names/checkboxes are not evidence of a passed test. Native commands run on the
stated native host with explicit absolute evidence paths and record actual backend.
All shell invocations use the repository's `rtk` wrapper.

Check groups are stage-scoped: execute every target owned by the current task and
its completed dependencies; do not require a future task's still-unimplemented
test target. The completion gates explicitly run the full assembled suites. A
missing target already owed by a completed task is a failure, not an allowed skip.

### C1: Legacy Rust and compatibility

Run default `cargo fmt --manifest-path "${ENGINE_MANIFEST}" --check`,
`cargo clippy --manifest-path "${ENGINE_MANIFEST}" --all-targets --locked -- -D warnings`
and `cargo test --manifest-path "${ENGINE_MANIFEST}" --locked`, plus the new
`--test core_contracts` golden/public-import checks. Preserve existing supported
Linux/macOS targets; this does not claim Windows TUI support.

### C2: Presentation-free engine boundary

Run `cargo check --manifest-path "${ENGINE_MANIFEST}" --no-default-features --lib --locked`
and `cargo test --manifest-path "${ENGINE_MANIFEST}" --no-default-features --lib --test core_contracts --locked`.
The planned `tools/vfa-tui/tests/check_feature_boundary.py` checks the shared engine's
normal dependency subtree, resolved under each consumer lock graph, for terminal/
GPUI leakage. GPUI is expected in the whole Desktop graph, not forbidden there.
Task 1.2 first checks the engine graph; task 1.6 adds behavioral checks under the
Desktop graph once both consumer foundations exist.
Port platform-dependent test helpers; do not gate domain tests away.

### C3: Named engine contracts

For each named task target run
`cargo test --manifest-path "${ENGINE_MANIFEST}" --no-default-features --locked --test <TARGET>`.
Repeat relevant contract tests with default features and run C1/C2 before merging
shared changes. Targets are named explicitly in each task's owned paths.
From task 1.6 onward each shared-engine change also runs the C4 `engine_contracts`
target under Desktop's lockfile; the parent wires its shared regression cases.

### C4: Pinned GPUI build and in-process UI tests

Run Desktop fmt/clippy/all-targets check and tests with
`--manifest-path "${DESKTOP_MANIFEST}" --locked`; UI target is
`cargo test --manifest-path "${DESKTOP_MANIFEST}" --test ui --locked`.
Kit `test-support` is a same-version dev dependency. Assertions drive actual
callbacks/state, not labels alone. Verify production dependency closure separately.
From task 1.6 onward also run `cargo test --manifest-path "${DESKTOP_MANIFEST}" --test engine_contracts --locked`.
That target reuses independent shared-engine fixture cases, including every new
portability regression, under Desktop's actual dependency resolution. C3 results
under the TUI lockfile alone do not satisfy this second-graph behavioral check.

### C5: Early native bootstrap and ownership admission

Implement a main-thread `native_smoke` test target (`harness = false`, explicitly
selected), then run `cargo test --manifest-path "${DESKTOP_MANIFEST}" --test native_smoke --locked -- --scenario admission --backend <BACKEND> --evidence-dir <DIR>`.
Admission covers actual bootstrap/bridge/process cleanup on Linux X11 and Wayland,
macOS ARM64 and Windows x64 MSVC. Root-first exit, descendants, inherited Windows
jobs, shutdown and missing prerequisites are negative scenarios. No compile-only
or headless-only result passes this barrier.
Task 1.4 runs the narrower `--scenario bootstrap`; task 2.6 later runs the full
`--scenario admission`. Bootstrap success alone cannot complete task 2.6.
Task 4.7 adds `--scenario worker_shutdown` using real governance/SQLite workers,
which did not yet exist during the earlier process/bootstrap admission.

### C6: Process/output/privacy stress contracts

Run C3 targets `process_contracts`, `output_budget`, `streaming_redaction` and
`lifecycle`; compile the std-only Rust helper into temporary test output with the
test toolchain, not a published binary pointing inside excluded tests. Include
64 MiB unterminated output, all-buffer high-water accounting ≤8 MiB, long/EOF PEM,
slow/disconnected UI, protocol/display separation and reaped descendant evidence.
Task 2.1 supplies `process_contracts`, task 2.4 supplies `output_budget`, task 2.5
supplies `streaming_redaction`, and task 2.6 supplies `lifecycle`. Earlier owners
are not blocked by later targets; C6 at final closure runs all four.

### C7: Authoritative exporter and legacy modes

Run `node --test tests/export-plan.test.mjs tests/export-apply.test.mjs` after
those tests exist, plus existing `tests/test-vfa-export-coverage.test.mjs`,
`tests/test-codex-plugin-marketplace-install.test.mjs` and
`tests/export-cursor-kiro-skill-notice.test.mjs`. Test the actual Node planner and
legacy output, Codex final bytes/transforms and per-path effects, not Rust copies.
Task 5.1 runs planner plus existing legacy tests; task 5.2 adds apply tests. Later
export integration and final closure run the complete set.

### C8: Complete native workflows and assistive technology

Extend `native_smoke` with `--scenario workflows`, running on the same four native
backend configurations as C5. Separately run the planned `native_accessibility`
target with explicit evidence directory and platform AT/version. Real keyboard,
focus, screen-reader, IME/clipboard, theme and 200% scale observations are required;
headless pixels/semantic trees are not substitutes. Missing human/native evidence
is UNVERIFIED and blocks final acceptance, not a completed task.
Accept provenance-bound human observation records with platform/AT versions,
procedure, actual observations, exact source/build identity and reviewer
attribution. Presence checks cannot attest that an observation actually happened;
the parent reviews the underlying records rather than manufacture a PASS label.
Task 7.1 implements/runs workflow scenarios; task 7.2 adds native AT capture. The
complete C8 gate and task 8.3 require both, not an AT-only checklist or early stub.

### C9: Fixed performance protocol

Implement `tests/performance.rs` plus committed fixture/corpus and run
`cargo test --manifest-path "${DESKTOP_MANIFEST}" --test performance --locked -- --evidence-dir <DIR>`
on approved reference configurations. Use the exact requirements protocol,
sample counts/pacing/distribution, pressure overlap, raw samples and nearest-rank
p95. Reject empty/changed fixtures, excluded slow samples and absent backend runs.

### C10: Packages, licenses and clean-machine launch

Implement `scripts/check-vfa-desktop-package.py` and packaging manifests to build,
inventory, checksum and verify each target's relocatable artifact. Run C8 workflows
from the unpacked artifact and unrelated CWD on a clean native target with a
selected local checkout. Missing helpers/assets/notices/signing disclosure fail.
Verify the existing TUI Cargo package/semver gate from a clean controlled source
snapshot, without `--no-verify`, dirty-gate bypass or unauthorized Git commits.

### C11: Cross-consumer CI and repository gates

Run `node --test tests/gpui-ci-contracts.test.mjs` and validate local workflow definitions, dependency/source/license target
policy and path-filter fixtures, then `npm run validate` and available local
regression gates. Defining the pipeline is not executing its native jobs. At final
closure also run spell/Markdown lint and all affected C1–C10 gates. Regenerate only actually affected catalog/integrity
artifacts, integrity last on its own. Workflow edits are local code; running
external CI, pushing or publishing requires separate authorization.

### C12: Task structure and final evidence readiness

Current plan check: `python3 .kiro/specs/gpui-desktop/verify.py --tasks --self-test`.
Future `scripts/verify-gpui-desktop-readiness.py --evidence-dir <DIR>` checks exact
commit/lock hashes and requirement-to-actual-test/native artifacts. Its own missing,
stale, mismatched-target and fake-passed-skip fixtures must fail. It validates
evidence provenance/coverage, not the truth of an arbitrary PASS label; parent
reviews underlying assertions and native observations before completion.

### C13: Impeccable native visual audit and refinement

Task 7.4 uses `impeccable critique` and `impeccable audit` guidance against actual
native workflow captures/source; task 7.5 uses `impeccable polish` for the scoped
remediation batch. Resolve the installed skill/targets/output as described above;
these are skill workflows, not a claim of GPUI support in its mobile/web tooling.
Record target/build/lock identity, OS/backend, theme, window size/scale, fixture
state, procedure and image references. Cover catalog/search/details, governance,
jobs, trust/confirmation, export and audit/report paths in light/dark themes,
narrow/wide windows and 200-percent scale, including loading, empty, long-content,
unavailable/stale, permission/disabled and partial/failure states.

The report prioritizes P0-P3 findings by user impact and names the exact scene and
owned source, applicable approved criterion, recommendation and supporting image
or interaction observation. P0/P1 findings block acceptance; disposition lesser
findings with rationale and owner. A score, clean detector or screenshot presence
alone cannot close a finding. Parent reviews actual images and native interaction
evidence, including desktop-specific keyboard/focus/modal behavior. Missing or
stale captures, wrong-backend labels and hidden controls are negative fixtures.

Task 7.5 records before/after evidence and reruns C4/C8/C9 on the final build; batch
native confirmation with required AT/performance observations to avoid redundant
inspection rounds. No stale pre-refinement acceptance transfers silently. C12's
readiness verifier rejects unresolved P0/P1, missing dispositions, broken image
links, missing relocation tests, source/build mismatch or a bypassed visual gate.

## Admission and Completion Gates

- G1: 1.4 and 2.6 require real Linux X11/Wayland, macOS ARM64 and Windows x64
  bootstrap/process/runtime evidence before epic 6. Independent epics 3/4/5 service
  work may proceed only where its own dependencies are complete.
- G2: evidence admission, current trust and single-use confirmation are mandatory
  before real helper/target-write wiring; a mock screen is not functional progress.
- G3: task 7.1 built-binary workflow acceptance precedes packaging. Task 8.1 repeats
  workflows on unpacked artifacts; 7.2 AT and 7.3 performance remain mandatory final
  gates without unnecessarily preventing package construction. Task 7.1 does not
  depend on 8.1, avoiding a packaging loop.
- G4: final completion requires native visual/AT/performance/package evidence for
  every target, all criterion-specific tests and unchanged compatibility/data
  contracts. A missing host/tool/human observation remains a blocker, not PASS.
- G5: tasks 7.4/7.5 require Impeccable-guided visual audit and one bounded refinement
  cycle, portable path checks and final-build native evidence before 8.3. Packaging
  may begin after 7.1, but any later UI change invalidates affected package evidence;
  rebuild/recheck the final artifact before readiness rather than reuse old proof.
- User implementation authorization is still required after review of this plan.
  This task list authorizes no commit/push/publication or external CI operation.

## Suggested First Implementation Wave

After explicit authorization, start 1.1, then 1.2. Once the boundary is verified,
1.3 and 1.4 may run independently where write sets permit; 5.1 is also independent
against the approved exporter contract. Parent serializes manifest/module wiring
and approves each bounded worker packet. Do not dispatch the entire backlog or
silently continue past a failed native feasibility gate.
Task 8.2 can define local CI early after its foundation dependencies; it does not
wait for the native evidence that its future jobs are intended to produce. Remote
execution still requires explicit authorization and cannot be inferred from YAML.
