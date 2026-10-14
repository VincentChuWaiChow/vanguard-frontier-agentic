# 05 — Repo conventions recon (haiku, read-only)

Status: complete. Date: 2026-10-14. Paths are relative to the repo root.

1. **`--check` convention.** Exit codes: 0 = in sync, 1 = drift, 2 = bad input.
   - Drift prints `FAIL: … out of sync … Run: npm run …` to stderr. A clean check prints `OK: … in sync`.
   - Examples: `scripts/generate-workflow-catalog.mjs:288-300`, `scripts/generate-board-counts.mjs:296-312`, `scripts/generate-kiro-powers.mjs:29,819-853`.
   - Exception: `model-policy.mjs` uses a `check` subcommand instead of a flag.
2. **Gate output.** Success lines start with `OK: …` on stdout. Failures use `ERROR:` or `FAIL` on stderr.
   - Pattern: `main()` returns 1 or 0, and the script calls `sys.exit(main())`.
   - Examples: `tests/validate-catalog.py:538-541`, and `tests/validate-maestro-routing.py:191/198/236` (`FAIL`, `WARN [x]`, `OK   [x]`).
   - A WARN line does not fail the gate.
3. **`tests/fuzz-properties.test.mjs`.**
   - It uses a custom harness: `fc.assert(fc.property(...), {numRuns:200})`, then prints `PASS  …`.
   - It inlines copies of the logic it tests (`:18-52`).
   - There is no `test` script, and it is not part of `validate`. Only the CI job `fuzz` runs it (`ci.yml:55-72`), on pull requests.
4. **TOML parsing and tooling.**
   - The repo has no Node TOML parser and no `typescript` devDependency. Python `tomllib` is already used (`tests/validate-catalog.py:9`).
   - `node_modules` is absent locally, so run `npm install` first. `tsc` is not available.
5. **`catalog/index.json:5-11`.**
   - It lists five catalogs: skills, agents, rules, mcp_references, workflows. No code reads `index.json`.
   - In `tests/validate-catalog.py`, the `CATALOGS` list (`:14-19`) leaves out workflows. A new `mods.json` would therefore go unchecked unless a mods gate checks it.
   - `tests/validate-links.py:24` globs `catalog/*.json` and sends a HEAD request to every `official_docs` URL. Keep that field out of `mods.json`.
   - The vfa-tui catalog list is fixed (`store.rs:140-151`), so `mods.json` does not affect it.
6. **asset-integrity.**
   - `TREES` (`:33-57`) does not include `mods`. `ROOT_FILES` is at `:62-73`.
   - Files are hashed in sorted order within each tree, and the exec bit is not hashed. `--write` writes the manifest at `:200`.
7. **Secret scanning.**
   - The `.plugin-scanner.toml` header (`:4-8`) keeps every rule active on executable code. `ignore_paths` is at `:54-65`.
   - In the `hol-plugin-scanner` workflow, the `gate` job blocks but scans only `plugins/vanguard-frontier-agentic`. The `marketplace` job scans `.` but does not block.
   - **Repo-wide trip-wire:** `tests/validate-catalog.py:164-190` walks the whole repo with `rglob`.
     - It reads `.md/.json/.py/.toml/.yaml/.yml` files.
     - It applies `_CREDENTIAL_VALUE_RE` (`:94-96`) and `SECRET_PATTERNS` (`AKIA…`/`ASIA…`/`BEGIN … PRIVATE KEY`, `:106-110`).
     - `.mjs`, `.ts` and `.sh` files are **not** scanned.
8. **Lint configs.** `.codespellrc` has its skip list at `:5` and `ignore-words-list` at `:36`. `.markdownlint.jsonc` sets `default: false` and enables MD001, MD003, MD042, MD045 and MD052.
9. **PR jobs.**
   - Jobs: ci (validate, fuzz), codeql, docs-quality (markdownlint, codespell), hol-plugin-scanner (gate, marketplace), install-paths-smoke, packed-artifact-smoke, provider-scope-regression, and the vfa-tui gate (path-filtered).
   - These pick up a new top-level `mods/`: markdownlint, codespell, the validate-catalog secret scan, and the hol marketplace scan.
   - These do not: the codex marketplace, the npm `files` whitelist (`package.json:18-37`), the link check, docs-data, and the plugin manifest.
10. **Counts.** Nothing reads `catalog/index.json`, so mod counts appear nowhere unless code is added for them.
