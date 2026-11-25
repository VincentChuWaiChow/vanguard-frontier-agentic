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
CB11  Any agent anywhere in agents/ whose companion_skills name a Cilium skill also declares
      cilium-evidence-boundary, so the rules that skill depends on are bundled with it. (A live
      guard that reused a Cilium skill without its companion lost checks silently once.)
CB12  A command whose verb mutates or destroys state (apply, delete, patch, create, replace,
      edit, scale, uninstall, upgrade, rollback, restart, flush, cleanup, disconnect, ...) may not
      carry a class below [C4] or sit under a `Class:` line below [C4]. This checks that the class
      is plausible, not merely present; a wrong but plausible class still needs human review.

Before matching, text is NFKC-normalized and zero-width characters are removed, and a leading
`sudo`, `VAR=value` assignment or path prefix is ignored, so `sudo kubectl ...`,
`KUBECONFIG=x kubectl ...` and `/usr/local/bin/kubectl ...` are recognized. Every line of a
command block is checked, not only the first, and HTML <code>/<pre> spans count as code.

Usage:
    python3 tests/validate-cilium-board.py           # check
    python3 tests/validate-cilium-board.py --write   # render harness adapters, then check
"""
from __future__ import annotations

import importlib.util
import json
import re
import sys
import unicodedata
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
COMMAND_TOOLS = ("cilium", "cilium-dbg", "hubble", "kubectl", "k", "oc", "kubeadm", "helm", "tcpdump",
                 "bpftool", "conntrack", "ip", "iptables", "nft")
MUTATING_VERBS = re.compile(
    r"\b(?:apply|delete|patch|create|replace|edit|scale|label|annotate|uninstall|upgrade|rollback|"
    r"rollout\s+restart|restart|flush|cleanup|disconnect|connect|drain|cordon|taint|install|rotate|"
    r"set\s+image|config\s+\S+=)\b", re.IGNORECASE)
READ_ONLY_HELP = re.compile(r"(?:--help|\bhelp\b|--dry-run=client|\btemplate\b|\bget\b\s+values)", re.IGNORECASE)
LOW_CLASS = re.compile(r"\[C[0-3]\]")
ZERO_WIDTH = re.compile("[\u200b\u200c\u200d\u2060\ufeff]")
PREFIX = re.compile(r"^(?:sudo\s+(?:-\S+\s+)*)?(?:[A-Za-z_][A-Za-z0-9_]*=\S*\s+)*(?:\S*/)?")
HTML_CODE = re.compile(r"<(?:code|pre)[^>]*>(.*?)</(?:code|pre)>", re.IGNORECASE | re.DOTALL)
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
TOOL_RE = re.compile(r"^(?:" + "|".join(re.escape(t) for t in COMMAND_TOOLS) + r")[ \t]", re.IGNORECASE)


def normalize(text: str) -> str:
    return ZERO_WIDTH.sub("", unicodedata.normalize("NFKC", text))


def is_command(text: str) -> bool:
    return bool(TOOL_RE.match(PREFIX.sub("", text.strip(), count=1)))


def mutating(text: str) -> bool:
    body = PREFIX.sub("", text.strip(), count=1)
    return bool(MUTATING_VERBS.search(body)) and not READ_ONLY_HELP.search(body)
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
    lines = normalize(path.read_text(encoding="utf-8")).splitlines()
    in_fence = False
    fence_marker = ""
    block_class: str | None = None
    for i, line in enumerate(lines):
        m = FENCE.match(line)
        if m and not in_fence:
            in_fence, fence_marker = True, m.group(2)
            if m.group(3).lower() in FORBIDDEN_FENCES and path.is_relative_to(SKILLS):
                errors.append(f"CB03 {rel}:{i + 1}: '{m.group(3)}' fence in a static-review skill; use 'text'")
            j = i - 1
            while j >= 0 and not lines[j].strip():
                j -= 1
            block_class = lines[j].strip() if j >= i - 2 and j >= 0 and CLASS_LINE.match(lines[j].strip()) else None
            continue
        if in_fence:
            if line.strip().startswith(fence_marker):
                in_fence = False
                continue
            stripped = line.strip()
            if not stripped or stripped.startswith("#") or not is_command(stripped):
                continue
            tag = CLASS_TAG.search(stripped)
            if block_class is None and not tag:
                errors.append(f"CB02 {rel}:{i + 1}: command line in a block without a preceding 'Class: [Cn]' line")
            effective = tag.group(0) if tag else (CLASS_TAG.search(block_class).group(0) if block_class else "")
            if effective and mutating(stripped) and LOW_CLASS.search(effective):
                errors.append(f"CB12 {rel}:{i + 1}: mutating command `{stripped[:60]}` classed {effective}; needs [C4] or [C5]")
            continue
        spans = CODE_SPAN.findall(line) + HTML_CODE.findall(line)
        for span in spans:
            if not is_command(span):
                continue
            tags = CLASS_TAG.findall(line)
            if not tags:
                errors.append(f"CB01 {rel}:{i + 1}: command `{span}` has no class tag on its line")
                break
            if mutating(span) and all(LOW_CLASS.match(tg) for tg in tags):
                errors.append(f"CB12 {rel}:{i + 1}: mutating command `{span[:60]}` classed {tags[0]}; needs [C4] or [C5]")
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

    cilium_skills = {p.name for p in SKILLS.iterdir() if (p / "SKILL.md").is_file()}
    for meta_path in sorted((ROOT / "agents").glob("*/*/metadata.json")):
        meta = json.loads(meta_path.read_text(encoding="utf-8"))
        companions = set(meta.get("companion_skills") or [])
        if companions & cilium_skills and SHARED_SKILL not in companions:
            errors.append(f"CB11 {meta_path.relative_to(ROOT)}: uses Cilium skill(s) {sorted(companions & cilium_skills)} without {SHARED_SKILL}")

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
