# Plan

## Objective

Make every Gemini agent export declare a `name` that Gemini CLI accepts, fix it
where the files are generated so a regeneration cannot revert it, and gate it so
it cannot regress.

## Evidence that opened this run

Gemini CLI's local-subagent documentation (`docs/core/subagents.md` in
`google-gemini/gemini-cli`) lists `name` as required: "Unique identifier (slug)
used as the tool name for the agent. Only lowercase letters, numbers, hyphens,
and underscores."

Measured on this branch: of 733 `harnesses/gemini.agent.md` exports, 714 carry a
display name such as `"Azure Entra ID Specialist"` and 19 carry a slug. All 19
slugs are the agent id (the Salesforce board, in every harness), which is the
in-repository precedent. Every agent id is itself slug-valid, the longest is 57
characters, and ids are unique.

Found while fixing the nameless Azure agents in the previous run, where it was
recorded as open because it affects every agent and needs a naming decision.

## Scope

In scope: the `name` field of every `gemini.agent.md`, every script that writes
or rewrites those files, and the gate.

Out of scope: every other harness, whose documentation permits display names;
`description` and body text; the two agents that ship no Gemini export at all
(`nvidia-model-promotion-gatekeeper-agent`,
`playwright-e2e-execution-run-agent`), recorded rather than widened into.

## Baseline

Commit `1785e633`, branch `claude/stoic-meitner-h2aqr8`, tree clean, all 21 CI
checks green.

## Method

One read-only wave: packet 01 establishes Gemini CLI's actual contract from its
source code, not only its docs; packet 02 maps every writer and every check in
this repository that touches these files. The parent then chooses the naming
rule on that evidence, changes the generators first and the committed exports
second, extends the gate, and verifies with paired probes plus a Gemini CLI
load if the CLI can be run here.
