# Approved scanner fixture follow-up

GitHub code-scanning check 116335918605 failed at head 1ac18901 on the synthetic
URL fixture (tests/mods.test.mjs:30). All other GitHub checks passed. Source/blame
confirmed deliberate repeated-character test data, unchanged since 16cc31009;
this was not a discovered live credential. User explicitly approved one additional
focused repair and verification, without scanner suppression.

Parent constructs the identical string through URL components and adds an exact
wire-value regression. That regression passed against the original construction
before the edit; it is semantic-preservation evidence, not a fabricated scanner
RED. The actual RED is the recorded GitHub annotation/check failure.

Fourth/final native reviewer (requested gpt-6.1-sol medium, read-only) found no
actionable defect: ASCII component values preserve encoding, the new regression
checks the original bytes, P1 coverage is unchanged, and no config/rule/runtime
scope expanded. Reviewer ran no tests, external calls or recursive agents.

Parent owns full mod tests, generator drift, last standalone integrity refresh,
repository validation, spelling/Markdown/secret gates, normal commit/push and
fresh GitHub scanner verification. Rust/runtime/fuzz sources are unchanged;
previous local evidence remains valid and new-head CI is independently checked.

Actual follow-up: 63 mod regressions, full npm validate, generator drift,
integrity, spelling/Markdown and tracked-secret checks passed. Repair commit
`e50af843251319f9a6cd4ce0bed6b7ed3ad72ae1` was pushed and independently verified
on remote and PR. The scanner no longer reported the URL line, but check
116431099084 failed on another pre-existing synthetic token fixture at line 304.
The CI criterion is NOT met. Parent requested explicit approval before expanding
from the URL repair to other fixture constructions; no suppression was used.
