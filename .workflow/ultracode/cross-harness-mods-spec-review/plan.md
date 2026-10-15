# Cross-harness mods spec review

## Goal

Analyze the latest R17/R18, P3/P8 and T9 additions, verify the supplied Kiro claims,
and update requirements, design and tasks together. No runtime implementation.

## Success criteria

- Findings distinguish verified facts, proposed decisions and unknowns.
- Current catalog and TUI contracts ground every new API/schema claim.
- Every amended criterion maps to design, tests and dependency-ordered tasks.
- No null-prototype or error-offset claim is accepted without evidence.
- Implementation checkboxes stay open; unrelated work stays untouched.

## Current context

Baseline: `7c07f0c7`. Branch: `feat/cross-harness-mods`. Upstream integration and
new spec commits supersede the earlier PR-publication snapshot. The only initial
untracked path is `.kiro/kiro-agents/`; do not inspect or modify it.

## Constraints

Own only `.kiro/specs/cross-harness-mods/{requirements,design,tasks}.md` and this
run directory. Do not edit linked skill sources, runtime, generated catalogs or
personal configuration. Use RTK and Context7. No claim of Kiro SMT execution.

## Risk level

Medium: specification changes can prescribe incorrect security/UI behavior.

## Approval gates

User confirmed spec-document updates. Ask about material unresolved product
choices; do not invent the seven clarifications missing from the excerpt.

## Mode

Delegated manual analysis. Ultracode owns integration; agent teams routes two
read-only sidecars. Available Codex model names are not Claude model equivalents.

## Work packets

1. Parent: Context7 evidence, requirements decisions and all edits.
2. Read-only reviewer: R18 loader/store/schema failure-mode audit.
3. Read-only reviewer: TUI display coverage and task/test dependency audit.

## Eval contract

- Outcome: consistent, implementable spec amendments, not implemented T9.
- Shared surfaces: requirements/design/tasks; parent is sole writer.
- Required checks: source/API probes, traceability and graph audit, scoped lint,
  repository validation and spell/Markdown gates.
- Blocking conditions: unresolved intent or unsupported API assumption. The user
  resolved browser scope in favor of the full visible browser and footprint.
- Handoff: file:line evidence, exact commands and honest limitations.

## Integration policy

Read source independently; accept evidence, not reviewer confidence. Preserve
existing IDs and completed implementation history outside affected additions.

## Verification plan

Check valid/invalid JSON edge cases; trace each amended R18 criterion to design
and T9; audit wave conflicts; review full diff; run relevant repository gates.

## Completion criteria

All explicit objective claims assessed, affected artifacts synchronized, required
checks evidenced, and unresolved choices disclosed or resolved by the user.

## Approved scope decision

Retain the visible browser. Add generated footprint metadata plus navigation,
list/detail and actual App reload/render tests to the implementation plan. The
review itself changes no runtime source or generated catalog.
