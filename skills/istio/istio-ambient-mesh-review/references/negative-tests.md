# Ambient review counterexamples

Use the AMB cases in the suite evaluation set. The cases are specifications until run through a model harness or a versioned live environment; their presence is not a passing evaluation.

| Case | Input distinction | Required conclusion |
| --- | --- | --- |
| AMB-01 | Namespace label without workload/capture evidence | Participation unknown |
| AMB-02 | Service waypoint, direct-IP path unexamined | No blanket path approval |
| AMB-03 | L7 selector policy enforced at ztunnel | Fail-safe denial case, not generic bypass |
| AMB-04 | Caller policy at destination ztunnel after waypoint | Identity mismatch requires review |
| AMB-05 | PERMISSIVE setting | Plaintext admission differs from plaintext-only traffic |
| AMB-06 | Migration with continuous L7 requirement | Transition gap must be resolved |
| AMB-07 | Sidecar extension with no evidenced waypoint equivalent | Block or mark support unknown |

For every proposed request test include a permitted and prohibited source, the actual addressed destination, the expected proxy path, correlation evidence, rate and cleanup limits, and separate operator authorization.
