# Gateway API counterexamples

| Case | Required behavior |
| --- | --- |
| GATEWAY-01 | Distinguish same-named Gateway kinds by API group |
| GATEWAY-02A | ReferenceGrant alone does not authorize Route-to-Gateway attachment |
| GATEWAY-02B | Cross-namespace backend permission is a separate edge |
| GATEWAY-03 | Accepted=True for an older generation is stale |
| GATEWAY-04 | Current Programmed=True is not end-to-end traffic proof |
| GATEWAY-05 | Public TLS termination does not prove protected backend transport |

Include incorrect parent/controller and missing-condition variants. Proposed tests are not executed by this skill.
