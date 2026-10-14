# Agentic delegation workflow

`.claude/workflows/agentic-delegation.js` is the delegation doctrine in
`.claude/skills/agentic-delegation/SKILL.md` expressed as an executable workflow. The
skill states the doctrine; this workflow enforces it, so the cheap parallel work is
actually delegated and the judgment work actually stays with the orchestrator.

Reach for it when a task decomposes into cheap parallel reconnaissance plus a small amount
of genuine judgment — especially when the work encodes external technical facts (model
names, API behaviour, version-specific semantics) that must be grounded in primary sources
before they ship.

## Phases

Each phase names a model tier, and the tier is the point.

| Phase | Tier | Why that tier |
|---|---|---|
| Resolve sources | Haiku | Mechanical lookup, no judgment |
| Recon | Haiku | Read-only, one narrow question per agent |
| Spec | Session model, no override | Architecture never delegates downward |
| Implement | Opus (Sonnet for Markdown-only specs) | Code stays on the daily driver; prose-only specs go to a Sonnet writer |
| Verify | Sonnet | Must be adversarial, so it cannot be the cheapest tier |
| Gate | Haiku | Runs commands and reports raw output |

**Resolve sources** resolves Context7 library IDs once, centrally, and passes them into
every later prompt. It does not scan the repository.

**Recon** runs one agent per question, in parallel, each requiring `file:line` for repo
claims and a Context7 library ID or fetched vendor URL for external facts. An open-ended
"understand the system" question is rejected by the doctrine; when the caller supplies no
questions, one agent derives narrow ones rather than handing a delegate the whole task.

**Spec** turns reconnaissance into file-scoped specs a delegate can execute without
further judgment. Anything that is an architecture decision, a security-sensitive edit, a
surgical change to a gate or schema, or the commit itself is refused into
`orchestratorRetains[]` rather than becoming a spec. Each spec lists the external factual
claims the delegate will write down, separately, so the next phase can check them one by
one.

**Implement** writes only the files its spec names.

**Verify** is a different agent, told to disbelieve the implementer.

**Gate** runs the gate suite and reports raw failure output — never a paraphrase.

## Context7 grounding

Context7 is a documentation server. The workflow, not Context7, is what centralises
resolution: the first phase resolves each library ID once and injects it into every
downstream prompt.

That ordering is deliberate. Context7 caps resolution calls per question, so a fleet of
delegates each calling `resolve-library-id` independently spends its budget learning the
same identifier several times over and can exhaust it before asking a single real
question. Resolving once converts a per-agent cost into a fixed one.

Delegates are told to ground external facts in the supplied library IDs, and to fall back
to fetching official vendor documentation when no library resolved. A fact that cannot be
grounded either way is left out — the workflow fails closed rather than shipping an
unverifiable claim.

## Pipeline, not barrier

`implement` and `verify` run as a pipeline. Each spec moves through implementation and
into verification on its own, so one spec can be under verification while another is still
being written. A barrier would hold every verification until the slowest implementation
finished, for no benefit: verifying one spec needs that spec's files, not all of them.

The barrier that does exist is between recon and spec, and it is correct there — the spec
phase decides how to split file scope across delegates, and it cannot do that from a
partial map.

## Why verification re-reads the files

A delegate's self-report is not verification. An implementer that misread its spec will
report success in good faith, because it is reporting what it believed it did.

So the verify agent is separate, is told the implementer's report is a claim rather than
evidence, and must open the files itself. It returns `CONFIRMED`, `CONTRADICTED`, or
`UNVERIFIABLE` per claim, with corrected wording for a contradiction. A claim that is true
in spirit but wrong in detail is `CONTRADICTED`, not `CONFIRMED` — that distinction is
where most real defects live.

Verification covers two kinds of claim, and both are required:

- **External facts** — checked against Context7 or vendor documentation.
- **Internal fidelity** — statements about this repository's own files, checked against
  those files. A document describing a script is verified against the script, not against
  what the description sounds like.

Green gates are necessary, not sufficient. A file can lint cleanly, satisfy every schema,
and still say something false.

## Why it stops before committing

The workflow returns `readyToCommit` and `blockers` and never commits.

The commit is orchestrator work because committing is the moment the change is accepted,
and acceptance requires reading the diff. A delegate that commits removes the review step
that the whole structure exists to protect. The orchestrator reads the diff, resolves any
blocker, and commits — a workflow reporting `readyToCommit: true` is reporting that
nothing blocked, not that nobody needs to look.

## Related

- [Agentic delegation skill](../.claude/skills/agentic-delegation/SKILL.md) — the doctrine
  this workflow enforces, including the three workflow templates.
- [Execution tiers](./execution-tiers.md) — the least-privilege tier contract that governs
  what any delegated agent may do.
- [Compatibility](./compatibility.md) — the harness support contract.
