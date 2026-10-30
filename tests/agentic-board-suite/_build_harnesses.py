#!/usr/bin/env python3
"""Render the seven harness adapters of each agentic-board agent from its canonical AGENT.md.

The adapters are derived, never hand-edited: a reviewer reads AGENT.md, and this script is the
only way its body reaches the harness files, so a safety section cannot be dropped from one
adapter while staying in the canonical contract (the suite runs --check).

    python3 tests/agentic-board-suite/_build_harnesses.py          # write harnesses/
    python3 tests/agentic-board-suite/_build_harnesses.py --check  # exit 1 if any adapter is stale

Body: "# <name>", a one-line scope sentence, then AGENT.md from its "## Mission" heading down.
Model lines belong to the model policy (scripts/model-policy.mjs HARNESS_CAPABILITIES projects
them): codex `model` / `model_reasoning_effort` / `model_provider` from the TOML header (never from
the developer_instructions body), claude-code `model:` / `effort:` and cursor `model:` from the
frontmatter. They are carried over from the file on disk, in the place model-policy puts them, so
`model-policy:apply` and this script's --check agree.

Tool grants per harness, each from the vendor's own documentation:
- claude-code `tools:` allowlist (code.claude.com/docs/en/sub-agents: omitted inherits every tool)
- gemini `tools:` allowlist (geminicli.com/docs/core/subagents: omitted inherits all tools; names
  from geminicli.com/docs/reference/tools, kinds Read/Search)
- cursor `readonly: true` (cursor.com/docs/subagents: no file edits, no state-changing shell)
- copilot `tools:` read/search aliases (repository documentation only)
- codex `features.shell_tool = false`: developers.openai.com/codex/subagents says a custom agent
  file cannot override sandbox_mode (it inherits the parent session's sandbox) and documents
  feature keys set to false as the way to reduce a custom agent's capabilities. No documented
  per-agent key removes file editing, so no read-only sandbox is claimed for Codex.
- kiro-ide / kiro-cli: no verified restricting field, so none is emitted (prompt-only there).
"""
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
AGENT_DIRS = [
    ROOT / "agents/cross-functional/agentic-system-architecture-review-agent",
    ROOT / "agents/cross-functional/agentic-security-review-agent",
]
COPILOT_TOOLS = ["read", "search", "search/codebase"]
GEMINI_TOOLS = ["read_file", "grep_search", "glob", "list_directory"]
POLICY_LINE = re.compile(r"^model(?:_reasoning_effort|_provider)? = .*$", re.MULTILINE)
FRONTMATTER_POLICY = re.compile(r"^(?:model|effort): .*$", re.MULTILINE)


def _policy_lines(path: Path, toml: bool) -> str:
    """Model-policy lines already on disk: the codex TOML header, or a Markdown adapter's frontmatter."""
    if not path.exists():
        return ""
    text = path.read_text(encoding="utf-8")
    if toml:
        head = text.split('developer_instructions = """', 1)[0]
        return "".join(f"{line}\n" for line in POLICY_LINE.findall(head))
    parts = text.split("---\n", 2)
    if len(parts) < 3 or parts[0]:
        return ""
    lines = FRONTMATTER_POLICY.findall(parts[1])
    lines.sort(key=lambda line: not line.startswith("model:"))  # model-policy puts effort right after model
    return "".join(f"{line}\n" for line in lines)


def q(s: str) -> str:
    return json.dumps(s, ensure_ascii=False)


def render(agent_dir: Path) -> dict[str, str]:
    meta = json.loads((agent_dir / "metadata.json").read_text(encoding="utf-8"))
    agent_md = (agent_dir / "AGENT.md").read_text(encoding="utf-8")
    idx = agent_md.find("\n## Mission")
    if idx < 0:
        raise SystemExit(f"{agent_dir}: AGENT.md has no '## Mission' section")
    name, desc, aid = meta["name"], meta["summary"], meta["id"]
    scope_id = aid[: -len("-agent")] if aid.endswith("-agent") else aid
    body = f"# {name}\n\nUse this agent only for `{scope_id}` work.\n\n{agent_md[idx + 1:].rstrip()}\n"
    if '"""' in body:
        raise SystemExit(f"{agent_dir}: body contains a TOML triple quote")

    harnesses = agent_dir / "harnesses"
    policy_block = _policy_lines(harnesses / "codex.toml", toml=True)
    claude_policy = _policy_lines(harnesses / "claude-code.agent.md", toml=False)
    cursor_policy = _policy_lines(harnesses / "cursor.agent.md", toml=False)
    copilot_tools = "".join(f"  - {q(t)}\n" for t in COPILOT_TOOLS)
    gemini_tools = "".join(f"  - {q(t)}\n" for t in GEMINI_TOOLS)
    toml_body = body.replace("\\", "\\\\")
    return {
        "claude-code.agent.md": (f"---\nname: {q(name)}\ndescription: {q(desc)}\ntools: \"Read, Grep, Glob\"\n"
                                 f"{claude_policy}---\n\n{body}"),
        "cursor.agent.md": f"---\nname: {q(name)}\ndescription: {q(desc)}\nreadonly: true\n{cursor_policy}---\n\n{body}",
        "kiro-ide.agent.md": f"---\nname: {q(name)}\ndescription: {q(desc)}\n---\n\n{body}",
        "gemini.agent.md": (f"---\nname: {q(aid)}\ndisplay_name: {q(name)}\ndescription: {q(desc)}\nkind: \"local\"\n"
                            f"tools:\n{gemini_tools}---\n\n{body}"),
        "copilot.agent.md": (f"---\ndescription: {q(desc)}\nname: {q(name)}\ntools:\n{copilot_tools}"
                             f"disable-model-invocation: false\nuser-invocable: true\n---\n\n{body}"),
        "kiro-cli.agent.json": json.dumps({"name": name, "description": desc, "prompt": body},
                                          ensure_ascii=False, indent=2) + "\n",
        "codex.toml": (f"name = {q(aid.replace('-', '_'))}\ndescription = {q(desc)}\n{policy_block}"
                       f"\ndeveloper_instructions = \"\"\"\n{toml_body}\"\"\"\n\n"
                       f"[features]\nshell_tool = false\n\n"
                       f"[metadata]\nauthor = \"github: VincentChuWaiChow\"\n"),
    }


def main(argv: list[str]) -> int:
    stale = []
    for agent_dir in AGENT_DIRS:
        for fname, text in render(agent_dir).items():
            path = agent_dir / "harnesses" / fname
            if "--check" in argv:
                if not path.exists() or path.read_text(encoding="utf-8") != text:
                    stale.append(str(path.relative_to(ROOT)))
            else:
                path.parent.mkdir(exist_ok=True)
                path.write_text(text, encoding="utf-8")
    for s in stale:
        print(f"stale harness adapter: {s}")
    return 1 if stale else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
