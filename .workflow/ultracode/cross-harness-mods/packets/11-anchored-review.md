# Anchored pattern repair review

## Objective
Review the focused repair for the admitted anchored-extension defect and regression coverage.

## Ownership
Read-only: scripts/mods/common.mjs, tests/mods.test.mjs, tests/fuzz-properties.test.mjs,
mods/secret-redactor/hooks/register.test.ts and native/core callers as one nearby hop.
No edits, commits, pushes, full gates, network, credentials or additional agents.

## Proposed approach (rejected in review)
Match the raw leaf before adding sibling/key context; preserve existing split-secret
matching and check the final sliced output again. Parent owns design and implementation.

## Success checks
Anchored extensions guard/redact scalar, object, argv and nested leaves; unchanged
keys/types and clean input; existing Bearer/AWS split context and fixed point survive.
Report concrete counterexamples or nonblocking documented limitations, file:line evidence.

## Implemented approach

Preserve contextual matching against the original leaf, then recheck the final
standalone leaf on both sliced return paths. Raw-first replacement was rejected
because it can expose part of a context-sensitive AWS secret.
