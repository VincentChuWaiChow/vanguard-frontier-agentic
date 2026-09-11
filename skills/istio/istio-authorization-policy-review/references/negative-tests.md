# Authorization counterexamples

| Case | Distinction to preserve |
| --- | --- |
| AUTH-01 | One ALLOW matching nothing does not cancel another applicable matching ALLOW |
| AUTH-02A | DENY with no rules has no matching rule |
| AUTH-02B | DENY with one empty rule matches all requests in scope |
| AUTH-03 | A valid targetRef without known revision/attachment is not proved enforcement |
| AUTH-04 | Missing JWT differs from invalid JWT |
| AUTH-05 | Nonempty peer principal differs from anonymous identity |
| AUTH-06 | Encrypted transport differs from authorized access |
| AUTH-07 | Missing HTTP attributes can broaden DENY effects on TCP |
| AUTH-08 | Proxy normalization and application normalization must agree |

Use complete policy sets and mode/version context. Pair positive and negative tests for every intended access row. A timeout does not establish policy denial; correlate it with enforcement evidence and exclude other failures.
