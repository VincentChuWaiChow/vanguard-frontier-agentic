# Hypothesis workflow

## DIAG-01: discriminate before changing

Build a timeline of user impact, last known good behavior, deployment/policy/version changes, first failure and affected/unaffected paths. A recent change is a hypothesis, not proof of causality.

Separate DNS resolution, endpoint selection, capture/enrollment, proxy configuration, TLS negotiation, authorization, upstream reachability and application response. For each hypothesis record supporting evidence, opposing evidence, unknowns and the next observation that can discriminate it.

Do not recommend a restart, plaintext fallback, broad ALLOW, disabled validation or longer timeout merely to remove a symptom. A diagnostic step that changes state needs separate authority and can destroy useful evidence.

## DIAG-02: preserve visibility limits

Forbidden inventory, truncated logs, a missing time window and absent traffic correlation are distinct gaps. Do not turn any into proof that a policy/object/traffic event did not exist. Prefer bounded evidence requests tied to one hypothesis over cluster-wide dumps.

## Decision output

Rank hypotheses by evidence strength, not invented probabilities. Say "unknown" when causes remain indistinguishable. Give the single next useful observation per branch and the exact claim it would support or refute.

Source: NETWORK for relevant troubleshooting categories; hypothesis method is a suite design requirement. Tests: DIAG-01, DIAG-02.
