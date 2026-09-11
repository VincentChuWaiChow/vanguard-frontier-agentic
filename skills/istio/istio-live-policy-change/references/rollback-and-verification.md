# Verification and rollback

## GUARD-05: verify behavior, not only apply success

After the phase, confirm relevant acceptance, attachment and data-plane propagation, then independently authorized permitted and prohibited request cases. Record actual path and correlate policy evidence. A return code of zero is not proof that access behaves correctly.

Stop when acceptance criteria fail, evidence is missing, targets drift or results conflict. Report partial application precisely. Do not mark an unrun test as passing to close an incident.

## GUARD-06: rollback is another bounded change

Use the independently approved rollback artifact only when its preconditions still hold. Compare current resource identities and ownership to the expected post-change state. Do not overwrite unrelated concurrent changes or remove an object merely because it was absent from an older snapshot.

Determine whether in-flight requests, existing sessions or external side effects survive rollback. A rollback manifest restores only its governed configuration; it cannot undo arbitrary business writes or a CRD storage migration. Require separate verification after rollback and retain the incident evidence.

## Exit record

Report requested action, actual changes, unperformed steps, result, surviving risk, evidence references, and next owner. Distinguish blocked before change, partial apply, verified change, attempted rollback and verified rollback.

Tests: GUARD-05, GUARD-06. These are suite operating controls, not claims of automatic enforcement by an LLM prompt.
