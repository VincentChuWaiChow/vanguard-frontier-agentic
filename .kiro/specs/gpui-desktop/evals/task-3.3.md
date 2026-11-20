# Task 3.3: no-install dependency and integrity findings preparation

## Admission and pre-implementation contract

The user's three-task approval admits an engine-only slice over the existing
immutable snapshot APIs. Full task 3.1/native prerequisites stay pending.

Owned code: `tools/vfa-tui/src/engine/findings.rs` and
`tools/vfa-tui/tests/relationship_integrity.rs`. Narrow corrections in
`tools/vfa-tui/src/federation/dep_graph.rs` or `integrity.rs` are permitted only
when necessary; preserve existing public behavior for legacy consumers. Parent
owns module exports. Existing snapshot read APIs are frozen for this worker;
task 3.2 changes are additive and not its prerequisite.

Before implementation all following criteria are **UNVERIFIED**:

1. Reuse graph/hash algorithms with exact deterministic nodes/edges/cycles and
   source provenance. Unresolved references and cross-collection ID ambiguity
   remain named findings, not silently overwritten identities.
2. Real fixture files yield matched, mismatched and missing results. Absent,
   malformed, partial or unreadable manifests/inputs cannot yield healthy zeros.
3. Reject traversal, absolute/drive-like paths, escaping symlinks and special
   files; distinguish inaccessible/unsupported from missing. Bound reads and
   avoid check-then-open path races using owned no-follow handles where supported.
   Unsupported platforms fail closed rather than claim native verification.
4. Browsing/findings never executes content, opens links, writes targets or
   accesses the network. Retain captured snapshot provenance and explicit
   filesystem-observation limits; do not imply a multi-file atomic snapshot.

Graders: `cargo test --locked --offline --test relationship_integrity`, full
default/engine suites, strict all-target clippy, fmt and source/scope checks.
Parent owns integrated failures. Distinguish missing-API compile RED from actual
runtime failure; Unix-only checks do not prove macOS/Windows behavior.

Results and exact evidence will be appended after verification.

## Primary-source grounding

Parent Context7 lookup used Rust standard-library documentation for unique
owned-descriptor transfer: [FromRawFd](https://doc.rust-lang.org/stable/std/os/fd/trait.FromRawFd.html).
The [Linux open/openat manual](https://man7.org/linux/man-pages/man2/open.2.html)
was checked for descriptor-relative prefix traversal, no-follow behavior and
nonblocking/close-on-exec flags. These sources explain the chosen API boundary;
they do not prove implementation race coverage or another platform's behavior.

## Implementation and capability results

**PASS (scoped):** dependency findings reuse `DependencyGraph` construction/
cycle algorithms with sorted projections, captured collection hashes/status,
source paths, named unresolved references and cross-collection ID collisions.
Collisions block ambiguous untyped graph construction. Partial/unavailable/stale
inputs cannot be represented as complete healthy findings.

Integrity findings validate the supported manifest shape/version/algorithm,
report per-file expected/actual hashes and sizes as Pass/Fail/Missing/Unavailable,
and stream SHA-256 through an 8 KiB buffer with a hard 16 MiB per-file ceiling.
Unix directory-relative no-follow handles reject traversal, drive-like paths,
symlinks and special files. Held root identity is checked against the captured
source; owned descriptors close once. No legacy verifier/graph APIs changed.

TDD: `3.3/runtime-red.log` demonstrates a synthetic traversal read by the old
verifier; the final adapter regression rejects that path. Missing-API compile
RED is separate. `engine-green.log` and `default-green.log` each pass 14
contracts; parent full suites also execute deterministic NotFound/PermissionDenied
classification. Sparse oversized files, malformed/missing manifests, symlinks/
FIFO, replaced roots, partial inputs, collisions, edges/cycles and hashes are
asserted. Permission classification is not an OS permission/emulation proof.

Limits: no non-Unix safe backend, aggregate-digest verification, filesystem
sandbox, hard-link/mount exclusion, atomic multi-file snapshot or guarantee
against concurrent in-place modification. The implementation explicitly reports
these observation limits; no native cross-platform/race stress proof is claimed.

## Integrated checks and handoff

See the [parent integrated result](../../../../.workflow/ultracode/gpui-desktop/final-report.md) for exact complete-suite, strict lint,
feature-boundary, package, repository and scope commands/status. Parent rerun:
1,168 default / 947 engine tests pass, zero failures, one existing ignored
doctest each; both clippy modes pass. Targeted capability commands:

```sh
cargo test --manifest-path tools/vfa-tui/Cargo.toml --locked --offline --test relationship_integrity
cargo test --manifest-path tools/vfa-tui/Cargo.toml --locked --offline --no-default-features --test relationship_integrity
```

No native/GUI/MSRV/coverage or allocator instrumentation acceptance is inferred.
Full task stays unchecked. Task-local RED/GREEN logs are under ignored
`tools/vfa-tui/target/no-install-three/3.3/`.
