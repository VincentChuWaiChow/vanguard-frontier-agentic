# Evaluation contract

## Level

Targeted. Each accepted change must ship with one decisive positive probe and
one decisive negative probe, following the repository quality bar in
`CLAUDE.md`.

## Acceptance criteria

A finding may be closed only when all of the following hold.

- The defect is demonstrated on the current tree, not inferred from the audit
  text.
- A positive probe shows the intended behaviour now works.
- A negative probe shows the previously passing bad input now fails, with a
  message that names the cause.
- The owning gate passes, and the full gate suite passes.

## Rejection criteria

A finding is left open, with the reason recorded in `integration.md`, when any
of the following hold.

- Closing it requires inventing domain truth the repository does not state.
- Closing it would break a shipped output format.
- Closing it requires credentials, paid features, or repositories outside this
  session's allowed scope.
- The fix would redden CI on pre-existing conditions whose baseline cannot be
  measured here.

## Publication gate

The prior Ultracode run recorded a conflict: repository policy says finish means
commit and push, while the Ultracode convention forbids publishing without an
explicit user request. In this run the user issued an explicit standing goal to
fix the proven findings, and has observed and accepted three pushes on this
branch. That is treated as the explicit request, so commit and push proceed and
the resolution is recorded rather than left implicit.
