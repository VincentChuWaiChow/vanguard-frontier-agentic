# Review output contract

Preserve the host repository's existing evidence envelope when integrated. Do not overwrite its schema or reinterpret `approved`, `blocked`, or `needs-review`. The portable `review-contract.json` is an adjunct for this bundle, not a replacement for VFA's evidence-output specification.

## Required sections

1. **Decision and scope:** lead with the bounded verdict, environment, requested decision, and execution tier. State what was not reviewed.
2. **Applicability:** list version, mode, target, attachment and path assumptions; unresolved ones remain explicit.
3. **Findings:** give severity, rule ID, evidence locations, actual mechanism, user requirement at risk, counterexample, and minimum corrective action.
4. **Verification:** pair a permitted case with a prohibited or failure case. State whether each is proposed, observed, failed, or inconclusive, who authorized it, and what it can establish.
5. **Change handoff:** describe the smallest proposed delta, blast radius, dependencies, rollback criteria, and unresolved ownership. Mark it as a proposal, not an applied change.

## Per-finding record

Use a stable ID, title, mechanism, evidence class, evidence references, applicability, business/technical consequence, and required next discriminating observation. Severity describes impact; evidence classification describes certainty. Do not merge them into one score.

A useful finding says: "The supplied selector policy contains an HTTP method match and is modeled at receiving ztunnel; documented fail-safe denial threatens required access. Runtime attachment and an allowed-client test are still missing." It does not say: "Ambient is insecure."

## Test record

Record source identity, destination identity/name, resolved destination/port, protocol, host/SNI/method where relevant, intended proxy path, expected outcome, observed result, capture time, and correlating policy/configuration evidence. Never mark a proposed test passed. Do not silently promote an unsuccessful connection to proof of authorization denial; rule out DNS, TLS and transport failure.
