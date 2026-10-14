# Result 02 — routing fixture independence

## Verdict

Confirmed. Adversarial and some happy-path expectations follow the evaluator.

## Evidence

- Expectations are produced by `mod.evaluate(...)` at
  `tests/_generate_maestro_routing_fixtures.py:463` (adversarial) and `:476`
  (happy-path widening), then written at `:481`.
- The validator compares against those files at
  `tests/validate-maestro-routing.py:232-233`.
- Independent invariants that survive an evaluator regression: agent ids exist
  in the catalog (`:141-142`), domains declare keywords (`:143-144`), keywords
  are not date-shaped (`:152-156`), live guards exist (`:158-159`), secrets-bait
  tasks mark credentials `<FAKE>` (`:211-217`), and live-guard agents never
  appear outside a gate mode (`:224-230`).
- Expected files carry no provenance, so a generated answer is
  indistinguishable from a reviewed one on inspection.

## Parent action

Reviewed expectations are frozen; a changed answer for an existing fixture
aborts regeneration unless `--accept-baseline-changes` is passed. The comparison
was also moved ahead of every write, because the original delete-then-generate
order left fixtures half-deleted when it raised — reproduced twice during
testing.

## Unresolved

The committed `accounting` and `dotnet` fixtures predate the current naming
scheme, and `dotnet/adv-instruction-injection` resolves to a different agent
under a freshly built taxonomy than its reviewed file records. Choosing the
correct routing is domain judgment and was left to the author.
