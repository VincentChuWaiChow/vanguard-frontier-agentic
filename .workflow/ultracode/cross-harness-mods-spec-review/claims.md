# Evidence ledger and findings

This is a manual adversarial review, not execution of Kiro's analyzer or an SMT
solver. The supplied excerpt does not contain Kiro's original seven questions;
the seven grouped findings below are independently reconstructed, not attributed
to those missing questions.

## Verified API claims

- Serde rejects unknown fields with `deny_unknown_fields`; combining it with
  flatten is unsupported. No flatten is needed here. Renaming a field is a valid
  API, but does not establish that the input contains that field.
  Sources: Context7 `/websites/serde_rs`,
  [container attributes](https://serde.rs/container-attrs.html) and
  [field attributes](https://serde.rs/field-attrs.html).
- `serde_json::Error::column()` is line-relative, not an absolute file offset.
  Context7 `/websites/rs_serde_json` and
  [Error documentation](https://docs.rs/serde_json/latest/serde_json/struct.Error.html).
  Parent offline Rust probe using locked versions serde 1.0.229 / serde_json
  1.0.151 reproduced line 3, column 3 at offending UTF-8 input byte 12.
- Context7 `/dubzzz/fast-check` returned an imprecise v4 release summary alongside
  more specific object/record/dictionary docs. Installed 4.10.2 source confirms
  `object.withNullPrototype` defaults false, while record/dictionary may use null
  prototypes. A null prototype alone does not break JSON serialization.
  `probe-json-domain.mjs` passed 200 seeded JSON-value runs and explicit negative
  cases (undefined/non-finite values lose information; BigInt throws).
  Source: [object APIs](https://fast-check.dev/docs/core-blocks/arbitraries/composites/object/).

## Seven grouped spec findings

1. **Schema contradiction:** R18 requires a discriminator absent from generated
   JSON. Distinguish emitted fields from optional/future metadata.
2. **Optional-file ambiguity:** only actual OS NotFound is absence; existing
   generic helper hides other failures. Avoid existence prechecks.
3. **Validation/reload gap:** cover every string, reach range, failed-reload
   retention and unchanged-hash retries. Do not bypass validation on reload.
4. **Coordinate mismatch:** column, absolute byte offset and entry index are
   different. Diagnostics must name the coordinate actually reported.
5. **Evidence gap:** N=5 example tests do not establish P8, and Unchanged does not
   exercise changed reload. Add isolated varied fixtures and failure sequences.
6. **Product-scope conflict:** visible browser/footprint versus explicit no-UI
   design. User explicitly retained the full browser. R8.7, R18.16–18.20 and
   ADR-7 define generated footprint metadata, legacy unknown state and real UI
   tasks/tests; no silent narrowing to a data foundation.
7. **Execution-plan gap:** missing Rust struct literals and concurrent shared
   generators/test edits; include all affected consumers and final verification.

R17.5 clarification is safe to apply directly: state L0 explicitly rather than
requiring readers to infer it from empty footprint arrays. P3 must use a
JSON-compatible domain and production append path, not copied append logic.
