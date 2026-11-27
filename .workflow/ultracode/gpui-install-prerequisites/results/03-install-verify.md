# Result 03: Parent installation and verification
## Summary
User-installed four Linux packages verified. Locked dependencies restored. Two compiler-proven test defects repaired, second with additional explicit approval. All scoped Linux checks pass.
## Evidence
state.json verification.checks contains final criteria, exact commands and log paths; attempts retains failures. Native evidence hashes match actual executable/source/lock/provenance. Full shared case parity passes in Desktop and both TUI modes.
## Handoff
Read final-report.md and spec eval. Full tasks/native admission not complete; other required hosts remain unavailable.
## Files changed
Two Desktop test files; spec task notes/new eval; workflow artifacts. No production source, manifests, locks, approved inputs, curated assets or Git publication.
## Decisions
Keep pinned SDK; use existing dev-only painted-quads observation rather than enable bench-support. No optional WebView/audio or unnecessary compiler/build tool installs.
## Risks
Software-renderer-only Linux evidence; absent macOS/Windows, AT/performance/package/later-runtime evidence.
## Verification run
All final local checks pass; failed initial cache/build/display/Node attempts preserved, superseded only by actual successful corresponding checks.
## Open questions
Remaining target access and next separately bounded implementation task.
