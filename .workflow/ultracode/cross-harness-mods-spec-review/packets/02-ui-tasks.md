# Packet 02-ui-tasks: TUI scope and task audit

## Objective

Audit whether R18's user story is fulfilled by T9 and whether tests/graph prove P8.

## Context

Spec-only update; UI/runtime code stays unchanged.

## Sources

Spec R17/R18/P8, T9 and graph, design Rust section, current TUI rendering,
watcher and catalog consumers. One nearby hop when necessary.

## Ownership

Read-only reviewer; parent owns every write.

## Do

Trace mod display, footprint availability, watcher coverage, fixture design,
test claims and graph conflicts. Return file:line citations and choices.

## Do not

Edit files, run full gates, inspect unrelated untracked agents, spawn workers,
commit, push or contact external systems.

## Expected output

Ranked gaps and concrete minimal requirements/design/task amendments.

## Verification

Check current source; no completion claim from prose or narrow examples.

## Handoff format

Summary; findings with citations; recommendations; uncertainty.
