# Packet 01 — Gemini CLI name contract

Owner: read-only Haiku agent (`Explore`). Write scope: none.

## Question

What does Gemini CLI actually enforce on a local subagent's frontmatter `name`,
read from its source code, and what happens to a file that violates it?

## Asks

1. The frontmatter schema for local agent markdown files in
   `google-gemini/gemini-cli` (for example a zod schema): the exact `name`
   constraint, whether `description` is required, the allowed `kind` values,
   and whether unknown keys are rejected.
2. What the loader does with an invalid file: skip, warn or fail, and whether
   that aborts other agents.
3. Whether the file name matters, and any length limit on `name`.
4. A command that loads or lists local agents without an API key, plus the
   npm package name and current version.

## Constraints

No edits, no test runs, no git. Cite a Context7 library id and file, or the URL
fetched. Mark anything unconfirmed `UNVERIFIED`.
