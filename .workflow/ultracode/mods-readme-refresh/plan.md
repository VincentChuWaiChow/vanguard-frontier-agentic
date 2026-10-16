# Mods README refresh

## Goal
Match the root README's welcoming, scannable style without weakening mod guidance.

## Success criteria
Centered branding, useful emoji headings, quick mod selection and expandable install
instructions; fewer words than the baseline; capability differences, safety limits,
state paths and working commands remain accurate.

## Current context
Baseline: e0759ae02bd132957d954691a0a83d3f5d0b22d6. Main target: mods/README.md.

## Constraints
No runtime, individual mod README, root README, scanner or configuration changes.
Preserve unrelated work. Refresh integrity after the settled README.

## Risk level
Low-risk documentation; security/support qualifications must survive condensation.

## Approval gates
User explicitly approved committing and pushing this refresh to PR 212.
No merge, deployment, force push or review-state changes.

## Mode
Small delegated workflow: parent writes; one independent read-only fact checker.

## Work packets
- Parent: rewrite, integrate and verify.
- 01-fact-check: review retained commands, limitations and support differences.

## Eval contract
- Outcome: concise, engaging README in the root README's visual idiom.
- Shared surfaces: mods/README.md and generated integrity.
- Required checks: semantic review, links/anchors, spelling, Markdown, integrity and full validate.
- Blocking conditions: unsupported claims, broken instructions, lost safety qualifications.
- Handoff evidence: file/line findings, word counts, commands and exit codes.

## Integration policy
Parent owns edits and accepts findings only after checking source evidence.

## Verification plan
Inspect diff; check local links/anchors and shell snippets; lint; regenerate integrity
last; run full repository validation. No runtime/native/Rust behavior changes.

## Completion criteria
Required checks pass; exact scoped changes committed and pushed; remote SHA verified.
