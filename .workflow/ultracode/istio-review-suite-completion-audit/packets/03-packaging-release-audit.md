# Packet 03: Packaging and release audit

## Objective
Audit catalogs, model assignments, generated harnesses, standalone exports, asset integrity, provenance, and release-policy evidence.

## Context
The committed branch touches 346 files and multiple generated/public surfaces.

## Sources
Repository validators/exporters/catalogs, bundle provenance, Git commit and remote refs, relevant generated outputs.

## Ownership
Read-only agent; no file edits.

## Do
Verify consistency and identify any generated drift, missing export dependency, legal/provenance uncertainty, or promotion check that was not actually run.

## Do not
Do not regenerate, commit, push, install tools, or call external systems with side effects.

## Expected output
PASS/MISS/UNVERIFIED release-readiness report with exact evidence.

## Verification
Use check/dry-run modes and direct inspection only.

## Handoff format
Summary, evidence, decisions, risks, checks run, open questions.
