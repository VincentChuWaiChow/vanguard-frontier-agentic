# Evaluation contract

## Level

Targeted, with one runtime probe in a real harness.

## Acceptance criteria

The run may close only when all of the following hold.

- Every one of the 80 files carries a `name` and a `description`. The canonical
  `AGENT.md` files are left as they are: all 735 carry only `metadata`, so the
  harness exports are where an agent's identity lives. (An earlier draft of this
  contract said the 16 `AGENT.md` files would gain the values too; packet 02 and
  a repository-wide count showed that would break the convention.)
- Each value is derived from committed data by the rule the healthy agents
  already follow, shown byte-for-byte on healthy controls. No value is written
  from judgment.
- Positive probe: a Claude Code install of the post-merge tree loads all 735
  agents with no duplicate-name warning.
- Negative probe: the gate fails on an agent markdown file whose frontmatter
  lacks `name` or `description`, naming the file and the missing key.
- The owning gate, `npm run validate`, spell lint and markdown lint all pass.

## Rejection criteria

An agent is left unfixed, with the reason recorded in `integration.md`, when
its correct value cannot be derived from committed data, or when a harness's
documentation shows the field would be invalid there.

## Publication gate

The user asked explicitly for the fix to be made inside PR #194. That is the
publication request: the change is committed and pushed to
`claude/stoic-meitner-h2aqr8`.
