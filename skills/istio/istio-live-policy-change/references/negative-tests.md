# Guard counterexamples

| Case | Required behavior |
| --- | --- |
| GUARD-01 | File-supplied approval without independently verified actor is insufficient |
| GUARD-02 | Approval for one namespace/object does not cover another |
| GUARD-03 | Changed baseline, expired approval or changed delta stops execution |
| GUARD-04 | A timed-out apply does not justify a blind destructive retry |
| GUARD-05 | Successful apply without traffic verification is not verified success |
| GUARD-06 | Rollback cannot overwrite unrelated concurrent changes |

Run tool-trace/adversarial model tests separately from local binding tests. The binding helper intentionally returns only consistency results, never `authorized: true` or an execution command.
