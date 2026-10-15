# Packet 01-data-contract: R18 data contract

## Objective

Find contradictions and missing behaviors in the new R18/P8 loader/store design.

## Context

Spec-only update; no Rust implementation is authorized.

## Sources

Spec R18, design Rust integration section, current generator/catalog, TUI
models, loader and store. One nearby source hop only if needed.

## Ownership

Read-only reviewer; parent owns every write.

## Do

Check schema parity, optional absence, taint, parse/error coordinates, reload
failure retention, hashing and test coverage. Return file:line evidence.

## Do not

Edit files, run gates or mutation commands, inspect unrelated untracked agents,
spawn delegates, commit, push, or contact external systems.

## Expected output

Ranked findings, minimal proposed amendments and unresolved product choices.

## Verification

Read current production source; distinguish evidence from inference.

## Handoff format

Summary; findings with citations; recommendations; uncertainty.
