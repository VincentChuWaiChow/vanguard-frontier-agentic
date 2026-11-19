# Task 5.2: installation-free Rust consumer preparation

## Scope and user journey

As the future export controller, validate the authoritative Node stream against
its expected plan so that missing, malformed or contradictory evidence cannot
become success, and previously observed writes remain available on failure.

This newly admitted parser-only slice leaves full task 5.2 unchecked. No native,
process output-budget/redaction, lifecycle, trust/authorization or durable journal
integration is claimed. Existing Node producer evidence is [separate](task-5.2.md).
The pre-implementation [eval contract](../../../../.workflow/ultracode/gpui-desktop/eval-contract.md)
defines the exact scope, graders, failure ownership and stop conditions.

## TDD evidence

Journeys were derived from task5.2, D6 and the already implemented Node contract;
no new plan commands were trusted or executed as instructions.

1. **RED before production:** the initial offline attempt failed for missing
   cached `anyhow`, which is NOT TDD evidence. The previous temporary Cargo home
   no longer existed. Restoring unchanged locked dependencies into ignored local
   `target/consumer-cargo-home` allowed compilation to reach the intended failure:
   `error[E0432]: unresolved import vfa_tui::engine::export_protocol`, exit101.
   No production edits preceded that valid compile-time RED.
2. **GREEN:** the same test target passed14 tests. An independent Rust review
   then found one real compatibility error: rejecting all colon/backslash/control
   path characters imposed a policy absent from the Node producer.
3. **Review regression RED:** a disposable real-Node companion export with
   `time:zone.md`, `a\b.md` and a newline-containing filename previewed and applied
   successfully, then the parser failed with `InvalidPlan`, exit101. Only the
   copied test fixture was written; the exporter itself was unchanged.
4. **One bounded repair:** preserve structural absolute/drive/UNC/traversal/NUL
   checks but accept legal interior filename characters unchanged. Remove the
   invalid test assumption that a POSIX backslash is always a separator. Exact
   expected-path matching and separate display redaction remain mandatory.
5. **Final GREEN:** all15 parser tests pass in each feature mode, zero skips.
   Both real Node cases verify actual final hashes/sizes and Linux modes, with
   protocol input split into seven-byte chunks. No refactor wave or checkpoint
   commits: current request does not authorize commits or pushes.

## Guaranteed behavior

All targets below are in `tools/vfa-tui/tests/export_protocol.rs` and pass on the
current native Linux host. Most are pure unit contracts; the two Node cases are
local CLI integration tests, not Desktop E2E tests.

| Guarantee | Test targets | Result |
| --- | --- | --- |
| Exact plan-bound success at every byte split, including UTF-8 | `success_survives_every_byte_split_including_utf8` | PASS |
| Invalid/empty expected projection and unsafe relative structure reject | `empty_expected_plan_is_rejected_like_the_producer`, `invalid_expected_plan_is_not_accepted_as_a_trust_anchor` | PASS |
| Version, digest, sequence, count, index, path, status and final metadata match | `start_must_match_protocol_identity_count_and_order`, `outcomes_must_match_every_authoritative_plan_field` | PASS |
| Missing/duplicate/unknown fields, bad JSON/encoding and trailing values reject | `strict_json_rejects_duplicate_missing_and_unknown_fields` | PASS |
| Successful/failed summaries match observed counts and explicit uncertainty | Three summary/arithmetic tests | PASS |
| Missing LF/summary, bad exit and post-terminal data cannot succeed | `eof_exit_and_terminal_order_are_independent_success_requirements` | PASS |
| A later error remains latched without erasing preceding validated effects | `errors_latch_without_erasing_preceding_effects_even_in_one_chunk` | PASS |
| 65536-byte record and 8388608-byte protocol boundaries are exact; 64MiB unterminated input is rejected without copying it | Both limit tests | PASS |
| Actual Node apply matches preview and real file results, including legal POSIX filenames | Both `consumes_actual_node_*` tests | PASS |

The expected projection is supplied by the caller: this module does not parse,
hash-canonicalize or authorize a complete export plan and never renders files.
Known outcomes remain inspectable even after `feed`/`finish` fails. The controller
must stop further writes/arrange safe drain and cleanup on failure; this isolated
parser cannot enforce those actions. Unobserved effects remain unknown.

## Exact verification commands

All shell commands were run through `rtk proxy`. At repository root, final Cargo
runs used this environment (dependency downloads affected only this ignored home;
no dependency/lock/compiler change or global installation):

```bash
export CARGO_HOME="$PWD/tools/vfa-tui/target/consumer-cargo-home"
export CARGO_TARGET_DIR="$PWD/tools/vfa-tui/target/consumer-build"
```

Run each Cargo test/clippy command below once normally and once with
`--no-default-features`:

```bash
cargo test --manifest-path tools/vfa-tui/Cargo.toml --locked --offline --test export_protocol
cargo test --manifest-path tools/vfa-tui/Cargo.toml --locked --offline --test export_protocol --test core_contracts --test report_service_contracts --test runtime_bridge --test process_contracts --test profile_ownership --test journal_commits
cargo test --manifest-path tools/vfa-tui/Cargo.toml --locked --offline --lib
cargo clippy --manifest-path tools/vfa-tui/Cargo.toml --locked --offline --all-targets -- -D warnings
cargo fmt --manifest-path tools/vfa-tui/Cargo.toml --check
node --test tests/export-plan.test.mjs tests/export-apply.test.mjs tests/export-cursor-kiro-skill-notice.test.mjs
python3 .kiro/specs/gpui-desktop/verify.py --tasks --self-test
```

| Check | Status | Actual result |
| --- | --- | --- |
| Focused contracts, default/no-default | PASS | 60/59 tests, exit0 in both modes; includes15 parser tests each |
| Strict clippy, both modes; fmt | PASS | Exit0, warnings denied |
| Full library suites | MISS baseline only | Exit101:735 pass/17 fail default,558 pass/11 fail engine-only; exact failure identities match freshly captured pre-production baselines |
| Unchanged Node producer/planner/notices | PASS | 30 tests, exit0, no skips |
| Spec structure/negative mutants | PASS | Exit0; structural evidence only, not semantic/native acceptance |
| Repository validation | PASS | `npm run validate`, exit0, including exporter and GPUI CI contract gates; opt-in Codex native installation is not claimed |
| Integrity, scope and whitespace | PASS | `python3 tests/validate-asset-integrity.py`, local-link/state/scope assertions and `git diff --check`, exit0; exactly10 changed source/evidence files, no forbidden changes |
| Graph maintenance | PASS AST only | `graphify update .`, exit0; graph/structural report updated; HTML visualization exceeds its node limit, not native visual evidence |

Raw logs and source hashes are under ignored
`tools/vfa-tui/target/export-consumer-wave/`, including original valid RED
`vfa-export-consumer-fetch.log`, review RED/GREEN logs, both contract/library/clippy
logs and `results.json`. The initial setup failure is retained separately in
`vfa-export-consumer-red.log`; it is not mislabeled as valid RED. These ignored
artifacts are local, not shipped evidence or a remote CI attestation.

Final housekeeping commands also run:

```bash
npm run validate
python3 tests/validate-asset-integrity.py
graphify update .
git diff --check
```

## Team, sources and token discipline

Parent owns implementation/integration and reran the graders. One reused
`rust_reviewer` performed bounded discovery and one review follow-up; profile and
live role declaration specify gpt-6.1-sol/high, checked schema, read-only default.
Effective inherited session sandbox is workspace-write; read-only delegation is
behavioral, not independently isolated. Serving model/effort, parent profile and
ChatGPT authentication are not observable from these tools and stay UNVERIFIED.
No implementation worker was added for this tightly coupled parser/test work.
Two worker turns, one integrated repair, no fan-out; total token usage unknown.

Context7 resolved `/serde-rs/json` and retrieved primary documentation for
`from_slice` end-of-input validation, UTF-8/schema errors and strict deserialization.
Source: [Serde JSON](https://github.com/serde-rs/json). That documentation informed
the implementation; actual negative tests, not documentation alone, are proof.

## Coverage and gaps

No installed `cargo-llvm-cov`, LLVM coverage tools or rustup llvm-tools component
were found during discovery. Coverage percentage remains UNVERIFIED unless
available tooling is subsequently located; no installation is authorized.
Native/Desktop and integrated memory/durability/cleanup checks remain UNVERIFIED.
The required 80-percent coverage threshold has not been measured or certified.
