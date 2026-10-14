# Packet 02 — repository writers and checks

Owner: read-only Haiku agent (`Explore`). Write scope: none.

## Question

Which code in this repository writes, rewrites, reads or checks
`harnesses/gemini.agent.md`, and what does each do with the `name` field?

## Asks

1. For each writer (`scripts/gen_*.py`, `scripts/model-policy.mjs`): `file:line`
   of the write, the expression producing `name`, and whether `kind` or other
   keys are added.
2. What `tests/validate-catalog.py` and `tests/validate-azure-oci-live-guards.sh`
   assert about Gemini files, with `file:line`, and whether either requires
   `name` to equal a display name.
3. How the export CLI installs a Gemini agent: destination path, file name, and
   any rewriting of frontmatter or marker insertion.
4. Whether `tools/vfa-tui` reads a Gemini export's `name` for detection or
   matching.

## Constraints

No edits, no test runs, no git. `file:line` for every claim.
