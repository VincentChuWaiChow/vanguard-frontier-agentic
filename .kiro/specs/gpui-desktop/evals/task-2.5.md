# Task 2.5: no-install streaming-redaction preparation

## Admission and pre-implementation contract

The user approved the three proposed engine-preparation slices with "go ... for
these 3 tasks". This admits task 2.5 with task 3.2 and 3.3 preparation, not their
full native dependencies or completion. Ultracode owns integration and reporting.

Owned code: `tools/vfa-tui/src/security/redact.rs`, `sanitize.rs` beside it,
`tools/vfa-tui/src/subprocess/bounded.rs`, one additive safe-projection entry
point in `subprocess/executor.rs`, and
`tools/vfa-tui/tests/streaming_redaction.rs`. Preserve legacy whole-string APIs
and raw export parsing. No dependencies, installs, native UI, Git publication,
credential inspection or frozen requirement/design edits.

Before implementation all following criteria are **UNVERIFIED**:

1. Documented existing credential classes are suppressed across arbitrary byte
   splits, ANSI/control splits and multiline PEM spans, including huge and
   unterminated candidates. Ordinary Unicode text remains usable.
2. Bounded recognizer state suppresses pending recognized/ambiguous content at
   EOF/cancellation without buffering a whole secret. Conservative loss is named.
3. Integrate before display truncation/drop; export bytes remain untouched and
   protocol effects survive. Redactor/sanitizer scratch is included in the shared
   8 MiB accounting. No unbounded line collection or process-global mutation.
4. Existing legacy redaction, output, process and export tests do not regress.

Graders (parent owns integrated failure disposition): `cargo test --locked
--offline --test streaming_redaction`, `--test output_budget`, complete suites
with default and `--no-default-features`, strict all-target clippy and fmt under
`tools/vfa-tui/Cargo.toml`. Record actual RED separately from missing-API compile
failure. No coverage percentage or native behavior is inferred from test counts.

Results and exact evidence will be appended after verification.

### Integration decision before implementation

The existing `spawn_bounded` API is explicitly unredacted internal transport and
its exact raw-budget/counter contracts are preserved. A new opt-in bounded
redacted entry point shares that supervisor/parser and applies streaming
projection before lossy display. Future UI/audit adapters must use the safe
projection; retaining the raw API is not proof it is safe to render.

## Implementation and capability results

**PASS (scoped):** `spawn_bounded_redacted` shares the owned supervisor but
projects sanitized streaming text before prefix truncation/queue loss. Existing
`spawn_bounded` stays unredacted/internal; legacy whole-string redaction and
sanitizer source remains byte-identical. Export stdout feeds the original raw
parser and is never displayed; validated protocol fields remain internal.

The recognizer holds a 256-byte token carry and finite PEM/ANSI/UTF-8 state,
not an entire span. Ambiguous overflow and incomplete pending candidates emit
explicit suppression markers. Safe mode reserves an additional 32 KiB inside
the existing 8 MiB cap and reduces both payload quotas. Queue saturation reaches
the reduced limits and held records retain/release their permits. Budget scope
excludes semantic models, external copies and allocator overhead as before.

TDD evidence: `2.5/runtime-red.log` shows actual old transport disclosure of a
synthetic credential prefix at a 4,096-byte cut. `api-red.log` is separate
missing-API evidence, not a behavioral pass/fail substitute. Targeted GREEN:
`2.5/green-default.log` and `green-no-default.log`, 10 tests each, plus existing
output/process/export contracts in the parent complete suites. Tests include
every split for existing classes, bytewise ANSI/Unicode, long/unterminated PEM,
EOF/cancel, dropped/slow queues and unchanged raw export effects.

This is documented-pattern protection, not complete DLP. Unknown secrets,
arbitrary paths/errors, the raw API and protocol semantic fields are not
automatically audit-safe. No GUI adapter or descendant ownership is delivered.

## Integrated checks and handoff

See the [parent integrated result](../../../../.workflow/ultracode/gpui-desktop/final-report.md) for exact complete-suite, strict lint,
feature-boundary, package, repository and scope commands/status. Parent rerun:
1,168 default / 947 engine tests pass, zero failures, one existing ignored
doctest each; both clippy modes pass. Targeted capability commands:

```sh
cargo test --manifest-path tools/vfa-tui/Cargo.toml --locked --offline --test streaming_redaction
cargo test --manifest-path tools/vfa-tui/Cargo.toml --locked --offline --no-default-features --test streaming_redaction
```

No native/GUI/MSRV/coverage or allocator instrumentation acceptance is inferred.
Full task stays unchecked. Task-local RED/GREEN logs are under ignored
`tools/vfa-tui/target/no-install-three/2.5/`.
