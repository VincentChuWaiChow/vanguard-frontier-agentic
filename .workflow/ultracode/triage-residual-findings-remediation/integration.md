# Integration

## Accepted

- All three packet findings, each reproduced by the parent before acting.
- N1's root cause is a producer/consumer contract the specification describes
  and the exporter never implemented, not merely a Codex path mismatch.
- P1 was escalated from CONDITIONAL after tracing the data path: every MCP in
  the shipped catalog presented an undeclared posture, so the trust gate was
  decorative rather than merely permissive at the edges.
- T2's real hazard is regeneration, and the generator's delete-then-write order
  was an independent defect found while testing the fix.

## Rejected

- Injecting the export marker into `.json`. Neither `#` nor `//` is legal JSON;
  corrupting a kiro-cli agent file to satisfy a scanner is worse than the bug.
- Adding a timestamp to the marker. `ExportMeta` treats `installed_at` as
  optional, and a clock-derived payload would make two exports of one catalog
  differ, defeating content hashing.
- Weakening the two-signal confirmation rule to make exports pass.
- Widening `.plugin-scanner.toml` exclusions. Each entry was added against a
  confirmed false positive, and several cover deliberate credential bait.
- Flipping the advisory repository-root scan to blocking. Its findings on the
  current tree cannot be measured from here, so the change could redden CI on
  pre-existing conditions.
- Deciding the correct routing for the drifted `dotnet` fixture.

## Conflicts

The prior Ultracode run recorded that repository policy says finish means commit
and push, while the Ultracode convention forbids publishing without an explicit
user request. Here the user issued a standing goal to fix the proven findings and
observed and accepted successive pushes on this branch, so that is treated as the
explicit request. Publication proceeded and the resolution is recorded rather
than left implicit.

## Decisions

- The branch ruleset item is closed as out of scope on the author's instruction
  that rulesets are a paid feature; no gate was added for the checked-in
  template, since policing a file nobody installs is CI weight on a dead
  artifact.
- Secret coverage is closed by adding an owned deterministic gate rather than by
  editing the third-party scanner's configuration.
- Export metadata is emitted per destination format, never blanket-prepended.

## Final changes

- Exporter emits a deterministic `# VFA-EXPORT:` marker for `.toml` and `.md`,
  inside YAML frontmatter where one exists; idempotent on re-export.
- Catalog MCP references carry their source `trust_matrix`, and
  `validate:mcp-trust-matrix` now enforces source/catalog parity.
- An undeclared MCP trust posture is a violation rather than a pass, at both
  call sites and in the policy engine.
- Routing expectations are frozen, and generation is all-or-nothing per provider.
- New `validate:tracked-secrets` gate over every tracked text file.

## Verification still needed

- Confirmation of kiro-cli `.json` exports, which needs a sidecar manifest or
  scanner support for a JSON metadata key.
- An author decision on the drifted `accounting` and `dotnet` fixtures.
- A baseline run of the advisory repository-root scanner before it could be made
  blocking.

## Remaining risks

The new secret gate uses vendor-prefixed patterns only, so it will not catch a
high-entropy credential with no recognisable issuer prefix. It reduces the
uncovered surface from effectively everything to that residue; it does not make
the repository secret-proof.
