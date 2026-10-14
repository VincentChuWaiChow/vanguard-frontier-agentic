# Result 01 — export detection contract

## Verdict

Confirmed, and the gap was wider than finding N1 described.

## Evidence

- Confirmation needs two distinct signals: `federation/scanner.rs:103-107`,
  `unique.len() >= 2`.
- `ContentSignature` can never fire in headless mode. The catalog index is built
  with `None` for every entry (`headless/reporter.rs:264-275`), so
  `id_to_template` is always empty and the branch at `scanner.rs:365-374` is
  unreachable.
- `MetadataComment` needs a `# VFA-EXPORT:` line (`scanner.rs:201-219`). The
  exporter copied bytes verbatim (`fs.copyFileSync`), and the string appeared
  nowhere outside the Rust scanner and the Kiro specs.
- Therefore only `Filename` could fire, for every harness, not just Codex.
- `ExportMeta` (`scanner.rs:62-69`) requires `id`; `version` and `installed_at`
  are `Option`, so the marker can be fully deterministic.
- Destination formats: `.toml` (codex), `.agent.md` (copilot), `.md`
  (claude-code, cursor, gemini, kiro-ide), `.json` (kiro-cli). Markdown carries
  YAML frontmatter; TOML and JSON do not.

## Parent action

Marker now emitted for `.toml` and `.md`, inside the frontmatter where one
exists. `.json` skipped, because neither `#` nor `//` is legal JSON. Verified
export to require_asset, including a control that strips the marker and sees the
check fail again, proving the two-signal rule was not weakened.
