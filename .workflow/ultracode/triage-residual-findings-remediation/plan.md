# Plan

## Objective

Close the residual findings from the 2026-09-20 triage and 2026-09-21 verified
audit that were left open after the first three remediation commits, or record
an evidence-backed reason why each remains open.

## Scope

In scope: N1 (installed-asset detection contract), T2 (routing-expectation
independence), SEC1 and SEC2 (secret-scan coverage).

Out of scope by explicit author decision: installing the branch ruleset. GitHub
rulesets with required checks and reviews are a paid-plan feature for this
repository, so the author deprioritised the whole item. No gate policing the
checked-in template was added, because a gate guarding a file nobody installs is
CI weight on a dead artifact.

Out of scope, deferred with reason: relocating
`tests/integration/rbac-pre-flight/ci/kind-rbac-preflight.yaml` into
`.github/workflows/`. It is a complete workflow parked where GitHub never reads
it, so it has never run. Enabling it costs a four-version kind matrix per
triggering change and needs `helm/kind-action` pinned to a digest from a
repository outside this session's allowed scope.

## Baseline

Commit `880c6fb9`, branch `claude/stoic-meitner-h2aqr8`, tree clean, all gates
green at baseline.

## Already closed before this run

P2, R1, R2, R3 template, T1, T3, I1, I2, I3, A1, A2, A3, P1, and the Codex half
of N1. Each was verified with a positive and a negative probe rather than a
green gate alone.

## Method

One read-only review wave establishes the exact contract boundaries. The parent
then implements only what the evidence supports, verifies each change with a
paired positive and negative probe, and records anything it declines to change.
