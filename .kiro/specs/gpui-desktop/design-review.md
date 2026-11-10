# GPUI Desktop design review

Status: READY FOR USER DESIGN REVIEW. The independent reviewer found five concrete
contract gaps in the initial design. The parent addressed them in one integrated
repair pass. This is not user design approval or native runtime proof.

The approved requirements hash and design contract are in `design.md` and
`evals.md`. The `architect` profile reviewed the initial draft independently.
Configured profile: `gpt-6-astra/high`; serving identity is unobservable. Its
verdict was repair-before-user-review, with permission to proceed to user review
after those repairs, not a demand to replace the selected package architecture.

The structural checker verifies criterion mapping and anchor presence only;
neither that check nor a reviewed design proves compiled/runtime/platform support.

## Findings and parent dispositions

| Finding | Severity / requirements | Reachable failure | Disposition and targeted evidence |
| --- | --- | --- | --- |
| F1: no cross-process recovery owner | High; R9.2, R9.8, R10.9–R10.11 | Second Desktop marks a live first instance's pending jobs interrupted | D8 now requires an OS-backed per-profile owner lock before writer/recovery/preferences. Second instance is read-only/busy; recovery only follows exclusive ownership. V7 includes two-process ownership and crash-release fixtures |
| F2: finite carry can leak long secrets | Medium; R9.5, R15.5 | PEM/token exceeds carry size before its terminating delimiter | D7 specifies bounded recognizer state and suppression through delimiter/EOF/cancel, including conservative oversized-candidate handling; V6 tests multiline/unterminated PEM and spans larger than carry. Existing source `security/redact.rs:287–307` confirms why full-line regex alone is insufficient |
| F3: protocol confused with lossy display | Medium; R8.7, R9.5–R9.6 | Slow UI drops authoritative file outcomes or redaction corrupts protocol fields | D6 parses/validates bounded raw protocol before any display loss/redaction, retains effects independently, and stops on protocol/evidence overflow while draining/cleaning. V5 covers UI-tail loss versus retained effects and protocol failure |
| F4: source copy is not final artifact | Medium; R8.2, R8.5, R11.5 | Codex rewrites destination-dependent skill paths after copying | D6 plans transformation identity/options and final-content digests via one authoritative Node renderer; final bytes are prepared before publication. V5 compares Codex final content and interrupts publication. Parent checked `scripts/export-marketplace-agents.mjs:432–442` |
| F5: component/test platform matrix absent | Medium; R16.9 | General framework support is mistaken for every selected control/test facility | D4 now records the selected `0.9.1` / `0.3.12` facility matrix across X11, Wayland, macOS and Windows. Declared renderer gaps, not-selected facilities and unverified application/runtime cells are distinct; V8/V10 remain required |

No architecture-wide rewrite or separately published core was justified by this
review. Parent retained feature-gated `vfa-tui`, separate Desktop dependency
resolution, pure report requests, the persistent runtime host and isolated
operation journal. The parent checked repaired clauses against the source-backed
findings; no second independent review or runtime experiment is claimed.

## Criteria

- **PASS — approved input:** requirements bytes match the recorded user approval.
- **PASS — structural traceability:** all 121 approved criteria map to existing
  design/test anchors; five design mutants and five requirements mutants fail.
- **PASS — bounded review process:** independent architect findings are recorded
  above and parent dispositions are present. This is not proof of no remaining
  design defects.
- **UNVERIFIED — design approval:** user has approved requirements/proceed-to-design
  only; task-breakdown remains unstarted.
- **UNVERIFIED — implementation evidence:** feature gating, protocol, runtime,
  native process ownership, UI/AT, performance and packaged delivery are plans,
  not built capabilities. Checksum/lock/toolchain capture and native spikes remain
  explicit prerequisites, not fabricated successes.

## Next gate

After repository/scope checks, ask the user to approve `design.md` or specify
revisions. Only an approved design transitions to the task-breakdown skill.
