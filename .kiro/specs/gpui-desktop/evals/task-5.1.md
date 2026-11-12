# Task 5.1 evaluation contract

## Scope and route

User authorized task 5.1 alongside independent task 4.1, with separate local
commits. Native task 1.4 stays pending. Parent owns integration, manifests/integrity
generation, shared eval/workflow records and Git; one worker owns only
`scripts/export-marketplace-agents.mjs`, `tests/export-plan.test.mjs` and narrowly
required fixed export-plan fixtures. Do not revert other workers' edits.
One bounded independent JavaScript review and one parent repair pass. No task 5.2
apply mode, Rust export policy, root dependency/version changes, push, publication,
system install or remote CI dispatch. Parent serializes generated manifests.

## Observable checks defined before implementation

1. Add a bounded versioned `--plan-json` mode (`vfa-export/1`) to the actual Node
   CLI. It emits machine-readable plan data only on stdout; diagnostics/errors go
   to stderr with nonzero failure, never truncated success. Retain existing flags,
   selectors, alias mapping, human dry-run, skill notices and legacy text output.
2. One authoritative planner and final-artifact renderer serve preview and legacy
   execution. No parallel Rust renderer or independently recalculated destinations.
   Plan binds source/catalog/selection/options identity, canonical intended target,
   each source and expected destination state/hash, create/overwrite action,
   transformation details, exact final bytes/hash, conflicts and canonical digest.
   Unknown/invalid effects are explicit and unconfirmable; no fabricated no-op.
3. Preview creates no target file, directory, database or lock, including missing
   roots. Existing tree byte snapshots remain identical. Source/destination
   symlinks, escapes, malformed selection/metadata, destination collisions,
   non-regular files and conflicting no-force targets fail/record conflicts
   explicitly; no success plan promises writes the legacy executor would reject.
4. Codex destination-dependent skill rewrites use the same final renderer before
   writes, with no intermediate unrewritten file. Test final bytes in independent
   fixtures, including encoding/newline/file-mode behavior; source hash is not
   substituted for final-artifact hash. Include bundled companion trees and force/
   no-skills/provider/role rules, not only one happy-path agent file.
5. Fixed oracles reject wrong final bytes/digest/path/transformation/selection;
   repeated inputs produce stable canonical plan identity. Unicode/spaces and
   unrelated working directories preserve selected source/target identity.
   Missing fixtures fail rather than skip; expected data must not be generated
   by the production planner under test.
6. Run actual Node planner plus C7 existing suites: export coverage, Codex install
   and Cursor/Kiro notices. Exercise the CLI, not source-string assertions only.
   Capture legacy compatibility before modification; preserve outputs and actual
   per-path effects where unchanged, with explicit necessary safety deltas.
7. Parent regenerates asset integrity after the covered script/tests change,
   without unrelated catalog regeneration. Before its task commit, run `npm run
   validate`, spelling/Markdown, exact scope/approval/Git-identity checks and AST
   update. Commit this task only with GitHub no-reply identities; no push.

## Graders

`node --test tests/export-plan.test.mjs tests/test-vfa-export-coverage.test.mjs
tests/test-codex-plugin-marketplace-install.test.mjs
tests/export-cursor-kiro-skill-notice.test.mjs` runs planner and legacy contracts.
Parent validates actual emitted plans/target bytes and reviews all changed code.
Task 5.2 later adds apply-plan/outcome protocol tests; no premature apply claim.

## Results and bounded review

| Criterion | Result | Evidence |
| --- | --- | --- |
| Machine protocol and shared renderer | PASS | Actual CLI canonical `vfa-export/1` plans, final hashes/bytes/modes, selection/options/source identity and repeated stable digests |
| No-write preview and failures | PASS | Missing Unicode/space target remains absent; existing tree/mode/mtime snapshots unchanged; links, malformed inputs, collisions and limits fail explicitly |
| Legacy compatibility | PASS | Eight fixed scenarios captured before modification; parent reran the original exporter against the new independent fixtures and matched outputs/bytes/modes, including seven forced-overwrite cases |
| Planner tests | PASS | Fourteen named tests; real subprocesses, not source-string checks; fixed expectations are not recomputed by the production renderer |
| Full C7 CLI regression | PASS | Parent combined Node run: seventeen passed, zero failed/skipped; coverage/provider-role matrix and Codex/Cursor/Kiro contracts retained |
| Review repair | PASS | Actual RED for metadata/catalog/script/outside-target hard-link aliases and machine-help stdout; both repaired, then fourteen planner tests passed |
| Repository validation | PASS | Required integrity manifest regenerated last after final covered code; full gate exits 0 on stable exporter bytes |
| Later apply/native behavior | UNVERIFIED | No apply-plan mode, authorization race-proofing, native marketplace installation or desktop execution claimed |

The TypeScript reviewer (configured Sol/high, read-only) found that O_TRUNC of an
existing hard link could modify an unplanned path, including source metadata or
an unrelated outside-target sentinel. Parent added real negative tests first:
all four alias classes exposed the flaw. Existing multi-link destinations now
fail before any write and link count is part of the destination fingerprint.
The legacy writer rechecks that fingerprint. This is an intentional safety delta
for aliased files, not a claim of protection against malicious same-user races.

The same repair rejects `--plan-json --help` before ordinary help handling, so a
successful machine response cannot contain human usage text. Plain help remains
compatible. No broader apply protocol or speculative refactor was added.

### Reproducible graders

```bash
node --check scripts/export-marketplace-agents.mjs
node --check tests/export-plan.test.mjs
node --test --test-reporter=tap tests/export-plan.test.mjs \
  tests/test-vfa-export-coverage.test.mjs \
  tests/test-codex-plugin-marketplace-install.test.mjs \
  tests/export-cursor-kiro-skill-notice.test.mjs
python3 tests/validate-asset-integrity.py --write
npm run validate
```

Actual parent logs in the session-selected temporary location:
`vfa-task-5-1-review-red.log` (two expected failing review regressions),
`vfa-task-5-1-plan.log` (fourteen passing), `vfa-task-5-1-c7.log` (seventeen
passing), `vfa-task-5-1-legacy-parent.log` (original exporter compatibility),
`vfa-task-5-1-legacy-evidence.json`, `vfa-task-5-1-integrity.log` and
`vfa-task-5-1-npm-validate.log`. Node subprocess tests use approved local-test
execution because this sandbox otherwise returns EPERM/empty pipes; no network,
installation, Git or system mutation is part of those test commands.

Original exporter SHA-256 from the tracked pre-change source:
`c79e3e7decbb3b9ba77f33e05145a22892bcfbf6a15231a74191a1d60e47cbac`.
Parent independently checked that hash and executed that source against the fixed
legacy oracle. Destination-dependent TOML paths normalize only the declared
`<TARGET>` token; binary data, newlines and modes remain compared literally.

### Scope and route

One builtin worker owned the script/test/fixed-fixture subtree; inherited worker
model/effort are unobservable. One configured TypeScript reviewer and one parent
repair/integration stream were sufficient. The parallel profile worker had a
disjoint Rust write set. No additional review/research wave was needed.

Commit only the exporter, its new test and fixed fixture package, required asset
integrity manifest, this eval/shared index/task status and existing workflow
records. Do not stage journal files, claim native acceptance or push. Task 1.4
remains unchecked; full Rust baseline failures stay recorded, not repaired here.

The fixed binary/newline oracle intentionally includes CRLF Markdown fixtures.
Commit whitespace checking recognizes CR-at-EOL explicitly while retaining
blank-at-EOL/EOF and space-before-tab checks; no fixture normalization, persistent
Git setting or ordinary trailing-space waiver was used.
