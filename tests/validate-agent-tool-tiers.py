#!/usr/bin/env python3
"""Gate: a Copilot agent's tool grant must match its declared execution tier.

The Copilot adapter is the one per-agent tool grant every tiered agent in this repo
carries, so this gate reads ``copilot.agent.md`` only. (A few agents also carry a Claude
Code or Gemini ``tools:`` allowlist or Cursor ``readonly: true``, checked by their own
suites, e.g. tests/agentic-board-suite; Codex ignores ``sandbox_mode`` in a custom agent
file, per developers.openai.com/codex/subagents.) That makes ``copilot.agent.md`` the
place where an agent's declared trust posture is checked here — and it had drifted both
ways:

  * ``static-review`` agents were granted ``execute/runInTerminal``, contradicting the
    tier's own definition ("no Bash", per schemas/agent.schema.json) and the agent's
    own contract.
  * Agents with no ``tools:`` block at all inherit every tool the harness offers, which
    is the opposite of least privilege — an implicit grant strictly wider than anything
    a tier permits.

Rules enforced (only for agents that DECLARE an execution_tier; see the note on
untiered agents below):

  0. A ``tools:`` block must be fully parseable in the canonical ``  - "tool"`` form.
     Anything else fails. A gate that silently skips an entry it does not understand
     reports green while the agent holds the very tool the gate forbids.
  1. An agent with a declared tier must carry an explicit ``tools:`` block. Inheriting
     the harness default is never a deliberate grant.
  2. ``static-review`` must not carry any execution tool. It reads source; it never runs.
  3. ``read-only-runtime`` must not carry an execution tool either, UNLESS the agent is
     in EXEC_ALLOWLIST — a small, explicitly justified set whose documented job is to
     run a command (not to mutate). Routers and advisors do not qualify.
  4. ``mutating-runtime`` may carry execution tools; that is the point of the tier. It is
     never *granted* them automatically — see default_tools().

Network egress is REPORTED, not failed. docs/execution-tiers.md defines T0 as "No
network egress", but a large number of static-review agents across the catalog already
declare ``web/fetch`` deliberately, because their own contracts require checking current
vendor documentation. Reconciling that conflict is a contract decision for those boards;
this gate surfaces the count rather than silently revoking capability.

Untiered agents are reported but not failed: ``execution_tier`` is optional in
schemas/agent.schema.json and a large share of the cloud-board agents omit it. Assigning
those tiers is a judgment call per agent and a separate change; this gate deliberately
does not guess a tier in order to police one.

Usage:
    python3 tests/validate-agent-tool-tiers.py            # check (CI)
    python3 tests/validate-agent-tool-tiers.py --write    # apply the minimal fix
    python3 tests/validate-agent-tool-tiers.py --report   # inventory, always exit 0

``--write`` is deliberately surgical and never widens a grant: it removes offending
execution entries, and where no block exists it inserts the narrowest useful set
(read/search only — no network, no terminal, for every tier). It never rewrites a grant
that already satisfies the rules, so hand-tuned per-agent grants are preserved.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
AGENTS = REPO / "agents"

# Tools that let an agent run a command. Anything matching these is an execution grant.
EXEC_PREFIXES = ("execute/",)
EXEC_EXACT = {"run_terminal_command", "runCommands", "terminal"}

# read-only-runtime agents whose documented purpose is to RUN a read-only command.
# Each entry needs a one-line justification tied to the agent's own contract.
EXEC_ALLOWLIST = {
    # "Execute an existing Playwright E2E suite against an operator-confirmed
    # non-production target"; runtime execution is a documented per-session opt-in.
    "playwright-e2e-execution-run-agent",
}

# Tools that reach the network. T0 (static review) is defined as "No network egress"
# in docs/execution-tiers.md, so a synthesized default must not include these.
NETWORK_TOOLS = {"web/fetch", "web/githubRepo", "fetch"}

# The default grant SYNTHESIZED when an adapter has no block at all. It is deliberately
# the narrowest useful set and contains no network and no execution tool, for either
# tier: inventing a grant is not the moment to hand out capability. An agent that needs
# network (to check current vendor docs) or a terminal must have it added explicitly,
# which is a reviewable diff rather than a side effect of running --write.
DEFAULT_TOOLS = ["read", "search", "search/codebase"]

# A tools block we cannot parse with full confidence is a FAILURE, not an empty list —
# see parse_tools(). Sentinel distinguishes "no block" (None) from "unparsable".
UNPARSABLE = object()

_TOOLS_KEY_RE = re.compile(r"^tools:[ \t]*$", re.M)
# One canonical list entry: two-space indent, dash, double-quoted scalar, nothing else.
_ENTRY_RE = re.compile(r'^[ \t]+- "([^"]*)"[ \t]*$')


def is_exec_tool(tool: str) -> bool:
    return tool.startswith(EXEC_PREFIXES) or tool in EXEC_EXACT


def default_tools(tier: str, agent_id: str) -> list[str]:
    """The grant synthesized for an adapter that declares none.

    Intentionally identical for every tier. `mutating-runtime` MAY hold execution
    tools, but --write must never be the thing that grants them: an operator that
    mutates through an API has no need of a shell, and silently widening it to a
    shell-capable agent is exactly the unjustified grant this gate exists to prevent.
    """
    return list(DEFAULT_TOOLS)


def parse_tools(text: str):
    """Return the declared tools, ``None`` if no block, or ``UNPARSABLE``.

    Fails closed on anything that is not the canonical ``  - "tool"`` form. A previous
    version matched only a leading run of double-quoted entries, so a perfectly valid
    YAML block that mixed in an unquoted scalar (``- execute/runInTerminal``) had its
    tail silently ignored — the agent held terminal execution while this gate reported
    success. Any deviation is now surfaced rather than skipped, so a bypass is
    impossible by construction: we either understand every entry or we refuse.
    """
    span = _block_span(text)
    if span is None:
        return None
    _, _, entry_lines = span
    tools: list[str] = []
    for line in entry_lines:
        m = _ENTRY_RE.match(line)
        if not m:
            return UNPARSABLE  # comment, unquoted scalar, nested map, anything else
        tools.append(m.group(1))
    if not tools:
        return UNPARSABLE  # `tools:` with no readable entries
    return tools


def _block_span(text: str):
    """Locate the tools block: (start_offset, end_offset, entry_lines).

    ``start`` is the offset of the ``tools:`` line, ``end`` the offset just past the
    last line belonging to the block. Block membership is by indentation: it ends at
    the first blank line or the first line that starts at column 0 (the next
    frontmatter key or the closing ``---``).
    """
    key = _TOOLS_KEY_RE.search(text)
    if not key:
        return None
    start = key.start()
    pos = key.end()
    if pos < len(text) and text[pos] == "\n":
        pos += 1  # step over the newline that terminates the `tools:` line itself
    entry_lines: list[str] = []
    while pos < len(text):
        nl = text.find("\n", pos)
        line = text[pos:] if nl == -1 else text[pos:nl]
        if not line.strip() or not line[:1].isspace():
            break
        entry_lines.append(line)
        if nl == -1:
            pos = len(text)
            break
        pos = nl + 1
    return start, pos, entry_lines


def render_block(tools: list[str]) -> str:
    return "tools:\n" + "".join(f'  - "{t}"\n' for t in tools)


def set_tools(text: str, tools: list[str]) -> str:
    """Replace an existing tools block, or insert one before the frontmatter close."""
    span = _block_span(text)
    if span is not None:
        start, end, _ = span
        return text[:start] + render_block(tools) + text[end:]
    # Insert before the closing delimiter of the YAML frontmatter (the 2nd '---').
    if not text.startswith("---\n"):
        raise ValueError("copilot adapter does not start with YAML frontmatter")
    close = text.index("\n---", 4)
    return text[: close + 1] + render_block(tools) + text[close + 1 :]


def allowed_violation(tier: str, agent_id: str, tools: list[str]) -> list[str]:
    """Return the execution tools this agent must not have."""
    if tier == "mutating-runtime":
        return []
    if tier == "read-only-runtime" and agent_id in EXEC_ALLOWLIST:
        return []
    return [t for t in tools if is_exec_tool(t)]


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--write", action="store_true", help="apply the minimal fix")
    ap.add_argument("--report", action="store_true", help="inventory only; exit 0")
    args = ap.parse_args()

    failures: list[str] = []
    fixed: list[str] = []
    untiered = 0
    checked = 0
    allowlisted_seen = 0
    network_grants = 0

    for meta_path in sorted(AGENTS.glob("*/*/metadata.json")):
        meta = json.loads(meta_path.read_text(encoding="utf-8"))
        tier = meta.get("execution_tier")
        agent_id = meta.get("id", meta_path.parent.name)
        adapter = meta_path.parent / "harnesses" / "copilot.agent.md"
        if not adapter.exists():
            continue
        if not tier:
            untiered += 1
            continue

        checked += 1
        if agent_id in EXEC_ALLOWLIST:
            allowlisted_seen += 1
        rel = adapter.relative_to(REPO)
        text = adapter.read_text(encoding="utf-8")
        tools = parse_tools(text)

        # Rule 0 — a block we cannot fully parse is a failure, never a pass. Silently
        # ignoring an entry we do not understand is how a gate reports green while the
        # agent holds the very tool the gate exists to forbid.
        if tools is UNPARSABLE:
            failures.append(
                f"{rel}: 'tools:' block is not in the canonical `  - \"tool\"` form; "
                f"every entry must be a double-quoted scalar so the grant can be "
                f"verified (refusing to guess at a partially-understood grant)"
            )
            continue

        # Track T0 network grants (docs/execution-tiers.md defines static review as
        # "No network egress"). Reported, not failed: ~260 static-review agents across
        # the catalog already declare web/fetch deliberately, and revoking that is a
        # contract decision for those boards, not a side effect of this gate.
        if tier == "static-review" and tools and any(t in NETWORK_TOOLS for t in tools):
            network_grants += 1

        # Rule 1 — a declared tier requires an explicit grant.
        if tools is None:
            if args.write:
                adapter.write_text(set_tools(text, default_tools(tier, agent_id)), encoding="utf-8")
                fixed.append(f"{rel}: added explicit {tier} tool grant")
            else:
                failures.append(
                    f"{rel}: tier {tier!r} but no 'tools:' block — inherits every harness "
                    f"tool (implicit grant wider than the tier allows)"
                )
            continue

        # Rules 2-4 — no execution grant outside mutating-runtime / the allowlist.
        offending = allowed_violation(tier, agent_id, tools)
        if offending:
            if args.write:
                kept = [t for t in tools if t not in offending]
                if not kept:
                    kept = list(READ_TOOLS)
                adapter.write_text(set_tools(text, kept), encoding="utf-8")
                fixed.append(f"{rel}: removed {', '.join(offending)} ({tier})")
            else:
                failures.append(
                    f"{rel}: tier {tier!r} must not grant execution tools, but grants "
                    f"{', '.join(offending)}"
                )

    if args.report:
        print(f"agents with a declared execution_tier and a copilot adapter: {checked}")
        print(f"agents without execution_tier (not policed here): {untiered}")
        print(f"violations that would be reported: {len(failures)}")
        print(f"static-review agents declaring a network tool (T0 says no egress; reported, not failed): {network_grants}")
        for f in failures[:20]:
            print(f"  - {f}")
        return 0

    if args.write:
        for f in fixed:
            print(f"  ~ {f}")
        print(f"OK: applied {len(fixed)} tool-grant fix(es) across {checked} tiered agent(s)")
        return 0

    if failures:
        for f in failures:
            print(f"FAIL {f}")
        print(
            f"\n{len(failures)} agent tool-grant violation(s). "
            f"Run: python3 tests/validate-agent-tool-tiers.py --write",
            file=sys.stderr,
        )
        return 1

    print(
        f"OK: {checked} tiered agent(s) carry an explicit copilot tool grant consistent "
        f"with their execution tier ({untiered} untiered agents not policed; "
        f"EXEC_ALLOWLIST holds {len(EXEC_ALLOWLIST)} read-only executor(s), of which "
        f"{allowlisted_seen} have a copilot adapter; {network_grants} static-review "
        f"agent(s) declare a network tool — see the T0 note in docs/execution-tiers.md)"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
