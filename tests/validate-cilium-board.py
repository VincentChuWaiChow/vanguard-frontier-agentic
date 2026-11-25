#!/usr/bin/env python3
"""Gate: the Cilium board's safety and structure contracts hold.

Deterministic (no network, no wall clock). Checks, each failing with its rule id:

CB01  Every command-shaped code span in a Cilium skill or agent contract (evidence ledgers
      excepted: a row records upstream behavior, it proposes nothing) carries a command
      class tag ([C0]..[C5]) on the same line. Command-shaped = a code span that starts with
      one of COMMAND_TOOLS followed by a space. The board never runs commands, but it names
      them; a name like `status`, `observe` or `test` says nothing about side effects, so the
      class must be stated where the command is (skills/cilium/cilium-evidence-boundary owns
      the class definitions).
CB02  A fenced block whose first command starts with one of COMMAND_TOOLS is preceded by a
      `Class: [Cn] ...` line (a blank line between is allowed, as markdownlint MD031 requires).
CB03  No `bash`/`sh`/`shell`/`console` fences in Cilium skills: the skills are Read/Grep/Glob
      only, and those fence types are what validate-skill-coherence treats as executable.
CB04  Every Cilium skill except the shared one names `cilium-evidence-boundary` in SKILL.md,
      and every Cilium agent declares it in companion_skills.
CB05  Every companion skill a Cilium agent declares exists under skills/cilium/.
CB06  Every Cilium agent declares execution_tier `static-review`: its rendered adapters grant only
      read tools, so any other tier would claim an ability the agent does not have.
CB07  Every Cilium AGENT.md carries the canonical contract headings in REQUIRED_HEADINGS.
CB08  Harness adapters equal what tests/agentic-board-suite/_build_harnesses.py renders from
      AGENT.md (same renderer as the agentic board, so tool grants stay vendor-verified).
CB09  The routing fixture tests/fixtures/cilium-maestro-routing/taxonomy.json routes to every
      non-maestro Cilium agent, and its live_guards are exactly LIVE_GUARDS.
CB10  No Cilium agent contract grants a write-capable tool in its Claude Code adapter.

Usage:
    python3 tests/validate-cilium-board.py           # check
    python3 tests/validate-cilium-board.py --write   # render harness adapters, then check
"""
from __future__ import annotations

import importlib.util
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
AGENTS = ROOT / "agents" / "cilium"
SKILLS = ROOT / "skills" / "cilium"
SHARED_SKILL = "cilium-evidence-boundary"
MAESTRO = "cilium-maestro-agent"
TAXONOMY = ROOT / "tests" / "fixtures" / "cilium-maestro-routing" / "taxonomy.json"
LIVE_GUARDS = {
    "kubernetes-live-network-policy-guard-agent",
    "kubernetes-live-network-architecture-mutation-guard-agent",
}
COMMAND_TOOLS = ("cilium", "cilium-dbg", "hubble", "kubectl", "helm", "tcpdump", "bpftool", "conntrack", "ip")
REQUIRED_HEADINGS = (
    "## Mission",
    "## Decision ownership",
    "## Scope",
    "## Activation",
    "## Required skills",
    "## Input contract",
    "## Investigation workflow",
    "## Decision rules",
    "## Tool permissions",
    "## Security boundaries",
    "## Failure handling",
    "## Output contract",
    "## Handoff rules",
    "## Adversarial checks",
)
CLASS_TAG = re.compile(r"\[C[0-5]\]")
CLASS_LINE = re.compile(r"^Class: \[C[0-5]\]")
CODE_SPAN = re.compile(r"`([^`\n]+)`")
TOOL_RE = re.compile(r"^(?:" + "|".join(re.escape(t) for t in COMMAND_TOOLS) + r") ")
FENCE = re.compile(r"^(\s*)(```|~~~)\s*([\w+-]*)")
FORBIDDEN_FENCES = {"bash", "sh", "shell", "console"}


def load_renderer():
    path = ROOT / "tests" / "agentic-board-suite" / "_build_harnesses.py"
    spec = importlib.util.spec_from_file_location("_build_harnesses", path)
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    spec.loader.exec_module(module)
    return module


def markdown_files() -> list[Path]:
    # Evidence ledgers are exempt from CB01: a ledger row records what upstream says a
    # command does (that is how a class is justified); it never proposes running it.
    files = sorted(p for p in SKILLS.rglob("*.md") if p.name != "evidence-ledger.md")
    files += sorted(AGENTS.glob("*/AGENT.md"))
    return files


def check_commands(path: Path, errors: list[str]) -> None:
    rel = path.relative_to(ROOT)
    lines = path.read_text(encoding="utf-8").splitlines()
    in_fence = False
    fence_marker = ""
    fence_first = False
    for i, line in enumerate(lines):
        m = FENCE.match(line)
        if m and not in_fence:
            in_fence, fence_marker, fence_first = True, m.group(2), True
            if m.group(3).lower() in FORBIDDEN_FENCES and path.is_relative_to(SKILLS):
                errors.append(f"CB03 {rel}:{i + 1}: '{m.group(3)}' fence in a static-review skill; use 'text'")
            fence_open = i
            continue
        if in_fence:
            if line.strip().startswith(fence_marker):
                in_fence = False
                continue
            if fence_first and line.strip():
                fence_first = False
                if TOOL_RE.match(line.strip()):
                    j = fence_open - 1
                    while j >= 0 and not lines[j].strip():
                        j -= 1
                    if j < 0 or j < fence_open - 2 or not CLASS_LINE.match(lines[j].strip()):
                        errors.append(f"CB02 {rel}:{fence_open + 1}: command block without a preceding 'Class: [Cn]' line")
            continue
        for span in CODE_SPAN.findall(line):
            if TOOL_RE.match(span) and not CLASS_TAG.search(line):
                errors.append(f"CB01 {rel}:{i + 1}: command `{span}` has no class tag on its line")
                break


def main(argv: list[str]) -> int:
    errors: list[str] = []
    renderer = load_renderer()
    agent_dirs = sorted(p for p in AGENTS.iterdir() if (p / "metadata.json").is_file())

    if "--write" in argv:
        for agent_dir in agent_dirs:
            for fname, text in renderer.render(agent_dir).items():
                out = agent_dir / "harnesses" / fname
                out.parent.mkdir(exist_ok=True)
                out.write_text(text, encoding="utf-8")

    for md in markdown_files():
        check_commands(md, errors)

    for skill_dir in sorted(p for p in SKILLS.iterdir() if (p / "SKILL.md").is_file()):
        if skill_dir.name != SHARED_SKILL and SHARED_SKILL not in (skill_dir / "SKILL.md").read_text(encoding="utf-8"):
            errors.append(f"CB04 {skill_dir.relative_to(ROOT)}/SKILL.md: does not load {SHARED_SKILL}")

    specialists: set[str] = set()
    for agent_dir in agent_dirs:
        rel = agent_dir.relative_to(ROOT)
        meta = json.loads((agent_dir / "metadata.json").read_text(encoding="utf-8"))
        aid = meta["id"]
        if aid != MAESTRO:
            specialists.add(aid)
        companions = meta.get("companion_skills") or []
        if SHARED_SKILL not in companions:
            errors.append(f"CB04 {rel}/metadata.json: companion_skills lacks {SHARED_SKILL}")
        for skill in companions:
            if not (SKILLS / skill / "SKILL.md").is_file():
                errors.append(f"CB05 {rel}/metadata.json: companion skill '{skill}' not found under skills/cilium/")
        tier = meta.get("execution_tier")
        if not tier:
            errors.append(f"CB06 {rel}/metadata.json: execution_tier missing")
        elif tier != "static-review":
            errors.append(f"CB06 {rel}/metadata.json: execution_tier '{tier}' — Cilium board agents are static-review")
        body = (agent_dir / "AGENT.md").read_text(encoding="utf-8")
        for heading in REQUIRED_HEADINGS:
            if not re.search(rf"^{re.escape(heading)}\s*$", body, re.MULTILINE):
                errors.append(f"CB07 {rel}/AGENT.md: missing heading '{heading}'")
        try:
            rendered = renderer.render(agent_dir)
        except SystemExit as exc:
            errors.append(f"CB08 {rel}: {exc}")
            continue
        for fname, text in rendered.items():
            path = agent_dir / "harnesses" / fname
            if not path.is_file() or path.read_text(encoding="utf-8") != text:
                errors.append(f"CB08 {path.relative_to(ROOT)}: stale or missing; run python3 tests/validate-cilium-board.py --write")
        claude = agent_dir / "harnesses" / "claude-code.agent.md"
        if claude.is_file() and 'tools: "Read, Grep, Glob"' not in claude.read_text(encoding="utf-8"):
            errors.append(f"CB10 {claude.relative_to(ROOT)}: Claude Code adapter must grant only Read, Grep, Glob")

    if not TAXONOMY.is_file():
        errors.append(f"CB09 {TAXONOMY.relative_to(ROOT)}: missing")
    else:
        tax = json.loads(TAXONOMY.read_text(encoding="utf-8"))
        routed = {d["agent"] for d in tax.get("domains", {}).values()}
        for missing in sorted(specialists - routed):
            errors.append(f"CB09 {TAXONOMY.relative_to(ROOT)}: no domain routes to {missing}")
        if set(tax.get("live_guards", [])) != LIVE_GUARDS:
            errors.append(f"CB09 {TAXONOMY.relative_to(ROOT)}: live_guards must be exactly {sorted(LIVE_GUARDS)}")

    for err in errors:
        print(err)
    if errors:
        print(f"validate-cilium-board: {len(errors)} error(s)")
        return 1
    print(f"validate-cilium-board: OK ({len(agent_dirs)} agents, {len(markdown_files())} Markdown files)")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
