# Full mods TUI implementation

## Goal

Implement pending tasks 6.7, 8.6 and 9.1–9.10 from the approved cross-harness
spec, update task status from evidence, and close checkpoint 8.5 honestly.

## Success criteria

R18.1–20 work through real catalog loading, atomic reload and the primary
Catalog-tab browser. Generated footprint matches manifests; legacy metadata is
unknown, not empty. P3 exercises nested JSON and N=1/50 boundaries. Existing
navigation, catalog consumers and repository gates remain intact.

## Current context

Baseline `4f5278f6c67308f3d69e1d45dae8dcbfffce0fea`, branch
`feat/cross-harness-mods`. Only unrelated `.kiro/kiro-agents/` is untracked.
Offline Cargo initially lacked cached locked dependencies. Approved
`cargo fetch --locked` completed; Cargo.lock stayed unchanged and the full
pre-change baseline passed (826 lib, 10 binary, 94 integration, 173 property).

## Constraints

Preserve unrelated work. Parent owns models/loader/store, generator/validator,
generated artifacts, spec status, integration and publication. Worker ownership
is in packets. No credentials, deployments, global installs, force pushes,
provider metadata changes, runtime hook redesign or adjacent cleanup.

## Risk level

High: untrusted catalog input, data/hash retention and cross-surface UI flow.

## Approval gates

User explicitly authorized implementing this approved spec and updating tasks;
earlier commit/PR instructions still apply. Network dependency fetch and Git
writes use exact tool approvals. No broader external mutation is authorized.

## Mode

Delegated; Ultracode owns integration, agent teams routes native workers, and
eval-harness defines checks before implementation. Model labels are not a
claim of cross-vendor equivalence or effective runtime isolation.

## Work packets

- Parent: critical model/loader/store and generator/validator implementation.
- Worker UI: app/nav/detail and actual App render/reload regressions.
- Worker P3/docs: fuzz workload and explicit README reach disclosures.
- Independent read-only final review after integration.

## Eval contract

Full: `eval-contract.md`; shared type/data interfaces in `contracts/mod-catalog.md`.

## Integration policy

Parent is sole generator, Cargo-build and Git owner. No parallel app/store or
fuzz-file writers. Workers may read interfaces before producer code exists;
dependent tests wait for integrated foundations. Never accept self-report alone.

## Verification plan

Capture baseline and capability RED evidence; implement; run focused positive
and negative cases, Rust fmt/clippy/tests, production fuzz, generator check,
repository validate, spelling and Markdown lint. Refresh integrity last after
generators settle. Verify exact pushed SHA and existing PR head.

## Completion criteria

Audit every pending task and R18 criterion against source/tests, not checkboxes.
Update tasks only on proof. All required checks pass; owned changes committed
and pushed. Report preserved unrelated untracked files and live-harness limits.
