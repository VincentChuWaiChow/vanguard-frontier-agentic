# Traffic review counterexamples

| Case | Required behavior |
| --- | --- |
| TRAFFIC-01A | Recognize equivalent relative weights 9:1 and 90:10 |
| TRAFFIC-01B | Do not invent remaining dropped traffic when total is 10 |
| TRAFFIC-02 | Keep configured weights distinct from observed distribution |
| TRAFFIC-03 | Review actual Service backends for HTTPRoute version splitting |
| TRAFFIC-04 | Examine caller and mesh retries plus write idempotency |
| TRAFFIC-05 | Include remaining capacity after outlier ejection |
| TRAFFIC-06 | Treat mirrored writes as real side-effect risks |
| TRAFFIC-07 | Require scope and approval for active fault experiments |

Do not run experiments from this read-only skill. Store proposed tests separately from observed results.
