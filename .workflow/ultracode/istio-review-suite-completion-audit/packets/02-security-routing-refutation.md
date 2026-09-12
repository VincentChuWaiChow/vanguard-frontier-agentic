# Packet 02: Security, routing, and live-guard refutation

## Objective
Try to break live-operation detection, no-auto-dispatch behavior, and the existing Kubernetes guard's preserved safeguards.

## Context
The suite adds a static live companion while retaining the existing Kubernetes live mesh guard.

## Sources
Routing evaluator/taxonomy/fixtures, guard agent and harness projections, Istio skill references, relevant tests.

## Ownership
Read-only agent; no file edits.

## Do
Construct adversarial false-negative and false-positive prompts, inspect permission/approval/target/rollback rules, and report reproducible defects only.

## Do not
Do not invoke Kubernetes, mutate files, or infer enforcement from prose alone.

## Expected output
PASS/MISS/UNVERIFIED findings ordered by severity with reproduction steps.

## Verification
Safe evaluator/test invocations are allowed.

## Handoff format
Summary, evidence, decisions, risks, checks run, open questions.
