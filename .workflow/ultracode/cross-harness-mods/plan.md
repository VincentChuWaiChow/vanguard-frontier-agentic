# Cross-harness mods execution

## Goal

Execute `.kiro/specs/cross-harness-mods/tasks.md` without inventing harness APIs
or weakening security validation to satisfy contradictory requirements.

## Success criteria

- Native adapters are supported by current primary documentation.
- Manifest, generator, runtime behavior, and validation agree on one contract.
- Positive and negative probes exercise actual generated implementations.
- Repository validation, spelling, and Markdown checks pass after integration.

## Current context

Branch: `feat/cross-harness-mods`; baseline: `1ee35965f16251ce04fa9ffbb01c714e9d0b3e07`.
Pre-existing untracked `.kiro/kiro-agents/` is outside scope and must be preserved.
The code-review graph is stale and will not be used as current evidence.
CLI observed: Codex 0.162.1, Node 24.15.0, Python 3.12.3.
IDE authentication, profile, parent model, and effort are UNVERIFIED; CLI evidence
does not attest the IDE session. No credentials or personal configuration read.

## Constraints

Reads: spec, repository conventions, selected skill references, official docs.
Initial writes: this workflow directory only. Implementation candidates, after
contract verification: `mods/`, generator and validator, focused tests, package
scripts, catalog integration, integrity manifest, scanner config, and docs named
by the task. No personal installation, dependencies, secrets, external mutation,
commit, push, or deployment. No unrelated cleanup.

## Risk level

High: hook permission decisions, secret handling, and audit claims.

## Approval gates

The user authorized executing the existing tasks. Material changes to harness
support, reach semantics, or claimed security behavior need clarification first.
Read-only investigation and local workflow notes may proceed meanwhile.

## Mode

Delegated. Ultracode is the sole workflow owner; Codex Agent Teams supplies routing.

## Work packets

1. Parent: verify native APIs, resolve contract, own security code and integration.
2. Read-only agent: find contradictions and invalid security/test assumptions.
3. Read-only agent: identify exact integration surfaces and existing gate behavior.

## Eval contract

Full contract: `eval-contract.md`. Define checks before implementation; do not
create a competing eval run root.

## Integration policy

Parent verifies source and every proposed change. No delegated implementation
before the contract is settled. No unsupported behavior presented as functional.

## Verification plan

Verify API claims through Context7 and primary sources. After approval and code:
focused tests, generated drift/idempotency checks, disposable CLI probes,
asset-integrity generation last, full validation, spelling, Markdown lint.
Rust checks apply only if Rust paths change (not currently planned).

## Completion criteria

Only check tasks complete with implementation and executed verification evidence.
If blocked, report the exact decisions needed and retain unchecked tasks.

## Approved continuation and commit scope

On 2026-10-14, the user approved the additional anchored-extension repair and
requested committing the updates. One extra read-only reviewer turn is allowed.
The parent owns the focused algorithm/tests and integration. No push, installation,
personal configuration, credentials, or unrelated `.kiro/kiro-agents/` changes.
A cohesive feature commit contains the coupled code/tests/gates/docs; a separate
workflow closeout records its receipt and final verification.
