#!/usr/bin/env python3
"""Gate: the Cilium board's safety and structure contracts hold.

Deterministic (no network, no wall clock). Checks, each failing with its rule id:

CB01  Every command-shaped code span in a Cilium skill or agent contract (evidence ledgers
      excepted: a row records upstream behavior, it proposes nothing) carries its own command
      class tag ([C0]..[C5]): the first tag after the span's end and before the next
      command-shaped span on the same line. A tag that belongs to another span on the line
      does not count. Command-shaped = a backtick span or single-line HTML <code>/<pre> span
      whose first word (after the prefixes below) is one of COMMAND_TOOLS, followed by at least
      one argument. The board never runs commands, but it names them; a name like `status`,
      `observe` or `test` says nothing about side effects, so the class must be stated where
      the command is (skills/cilium/cilium-evidence-boundary owns the class definitions).
CB02  Every command line in a code block carries a tag on the line or sits under a
      `Class: [Cn] ...` line placed directly before the block (one blank line between is
      allowed, as markdownlint MD031 requires). Code blocks are fenced blocks, indented code
      blocks (4+ spaces or a tab, after a blank line whose preceding non-blank line is neither a
      list item nor indented text) and multi-line HTML <pre>/<code> blocks.
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
CB11  Any agent anywhere in agents/ whose companion_skills name a Cilium skill, and any role in
      catalog/install-roles.json whose skills name a Cilium skill other than the shared one,
      also declares cilium-evidence-boundary, so the rules that skill depends on are bundled
      with it. (A live guard that reused a Cilium skill without its companion lost checks
      silently once; a role install is the same bundle by another path.)
CB12  A command that mutates or destroys state may not carry a class below [C4] or sit under a
      `Class:` line below [C4]. Inline, a span is judged by its own paired tag (CB01). This checks
      that the class is plausible, not merely present; a wrong but plausible class still needs
      human review.

How CB12 reads a command. The command is tokenized with shlex (str.split if quoting is
unbalanced) and the tool token is dropped. The "subcommand words" are the leading bare words
(^[a-z][a-z0-9-]*$, any case), skipping flags and the value of a flag in VALUE_FLAGS, stopping at
the first other token (a path, KEY=VALUE, number, ...) or after three words. A command mutates
when a subcommand word is in MUTATING_VERBS (or TOOL_VERBS for its tool), two consecutive words
are in TWO_WORD_VERBS, a `config` word is followed by a KEY=VALUE argument (`cilium-dbg endpoint
config <id> KEY=VALUE`), or a flag token is in FLAG_MUTATIONS for its tool (iptables/ip6tables
-A/-I/-D/-R/-F/-X/-N/-P/-Z, conntrack -D/-F/-U/-I, nft -f, and their long forms). Verbs are
matched only at subcommand position, so `kubectl get pods -l app=install` is read-only and
`kubectl delete cm helm-template` is mutating. Read-only exemptions are positional too: a
`--help`/`-h` flag token (`-h` is not help for `ip`), first subcommand word `help`, `helm
template`, `helm get ...`, `kubectl auth can-i ...` (the verb there is a question, not an action)
and a `--dry-run=client` flag.

Before matching, text is NFKC-normalized and zero-width characters are removed, and prefixes are
stripped repeatedly until none is left: a `$ ` or `> ` prompt, `sudo` and its options (with the
values of -u/-g/-U/-C/-h/-p and similar), `env` with its options and assignments, bare `VAR=value`
assignments, the wrappers `watch`, `time`, `timeout` (with its duration), `nice`, `nohup`,
`stdbuf`, `exec` and `command` with their options, and a path prefix on the tool. So `sudo -E env
A=1 /usr/bin/kubectl ...`, `watch -n 5 kubectl ...` and `./cilium ...` are recognized. Every line
of a code block is checked, not only the first.

Usage:
    python3 tests/validate-cilium-board.py           # check
    python3 tests/validate-cilium-board.py --write   # render harness adapters, then check
"""
from __future__ import annotations

import bisect
import html
import importlib.util
import json
import re
import shlex
import sys
import unicodedata
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
AGENTS = ROOT / "agents" / "cilium"
SKILLS = ROOT / "skills" / "cilium"
INSTALL_ROLES = ROOT / "catalog" / "install-roles.json"
SHARED_SKILL = "cilium-evidence-boundary"
MAESTRO = "cilium-maestro-agent"
TAXONOMY = ROOT / "tests" / "fixtures" / "cilium-maestro-routing" / "taxonomy.json"
LIVE_GUARDS = {
    "kubernetes-live-network-policy-guard-agent",
    "kubernetes-live-network-architecture-mutation-guard-agent",
}
COMMAND_TOOLS = ("cilium", "cilium-dbg", "hubble", "kubectl", "k", "oc", "kubeadm", "helm", "tcpdump",
                 "bpftool", "conntrack", "ip", "iptables", "ip6tables", "nft")
TOOL_SET = frozenset(t.lower() for t in COMMAND_TOOLS)
# Single-word verbs, matched only against subcommand words (see mutating()).
MUTATING_VERBS = frozenset({
    "apply", "delete", "del", "patch", "create", "replace", "edit", "scale", "label", "annotate",
    "uninstall", "upgrade", "rollback", "restart", "flush", "cleanup", "disconnect", "connect",
    "drain", "cordon", "taint", "install", "rotate", "enable", "disable", "add", "update",
    "regenerate", "undo", "reset", "import", "debug",
})
TWO_WORD_VERBS = frozenset({
    ("rollout", "restart"), ("rollout", "undo"),
    ("set", "image"), ("set", "env"), ("set", "resources"),
    ("config", "set"),
})
# Verbs that mutate only for one tool (a bare `set` is harmless elsewhere, not for `ip link set`).
TOOL_VERBS = {
    "nft": frozenset({"add", "delete", "flush", "insert", "replace", "create", "destroy"}),
    "ip": frozenset({"set", "change", "append", "prepend"}),
    "bpftool": frozenset({"load", "loadall", "attach", "detach", "pin", "freeze"}),
}
_IPTABLES_FLAGS = frozenset({
    "-A", "-I", "-D", "-R", "-F", "-X", "-N", "-P", "-Z",
    "--append", "--insert", "--delete", "--replace", "--flush", "--delete-chain", "--new-chain",
    "--policy", "--zero",
})
# Flag-style mutations, matched case-sensitively against flag tokens (`-x` is not `-X`).
FLAG_MUTATIONS = {
    "iptables": _IPTABLES_FLAGS,
    "ip6tables": _IPTABLES_FLAGS,
    "conntrack": frozenset({"-D", "-F", "-U", "-I", "--delete", "--flush", "--update", "--create"}),
    "nft": frozenset({"-f", "--file"}),
}
# Flags whose next token is their value, skipped when collecting subcommand words. (-A is a bare flag.)
VALUE_FLAGS = frozenset({"-n", "--namespace", "--context", "-c", "--kubeconfig", "-l", "--selector",
                         "-o", "--output", "--cluster"})
HELP_FLAGS = frozenset({"--help", "-h"})
NO_SHORT_HELP = frozenset({"ip"})  # `ip -h` means -human, not help
# Wrapper commands stripped before the tool, with the options of each that take a value.
WRAPPERS = {
    "sudo": frozenset({"-u", "-g", "-U", "-C", "-h", "-p", "-D", "-r", "-t", "-T", "--user", "--group",
                       "--other-user", "--close-from", "--host", "--prompt", "--chdir", "--role", "--type",
                       "--command-timeout"}),
    "env": frozenset({"-u", "--unset", "-C", "--chdir", "-S", "--split-string"}),
    "watch": frozenset({"-n", "--interval", "-q", "--equexit"}),
    "time": frozenset({"-f", "--format", "-o", "--output"}),
    "timeout": frozenset({"-s", "--signal", "-k", "--kill-after"}),
    "nice": frozenset({"-n", "--adjustment"}),
    "nohup": frozenset(),
    "stdbuf": frozenset({"-i", "-o", "-e", "--input", "--output", "--error"}),
    "exec": frozenset({"-a"}),
    "command": frozenset(),
}
PROMPTS = frozenset({"$", ">"})
DURATION = re.compile(r"^\d+(?:\.\d+)?[smhd]?$")
ASSIGNMENT = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*=")
KEY_VALUE = re.compile(r"^[A-Za-z_][A-Za-z0-9_.-]*=")
BARE_WORD = re.compile(r"^[a-z][a-z0-9-]*$", re.IGNORECASE)
MAX_SUBCOMMAND_WORDS = 3
LOW_CLASS = re.compile(r"\[C[0-3]\]")
ZERO_WIDTH = re.compile("[\u200b\u200c\u200d\u2060\ufeff]")
HTML_CODE = re.compile(r"<(code|pre)\b[^>]*>(.*?)</\1\s*>", re.IGNORECASE | re.DOTALL)
HTML_TAG = re.compile(r"<[^>]+>")
HTML_TAG_LITERAL = re.compile(r"</?(?:code|pre)\b", re.IGNORECASE)
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
INDENTED = re.compile(r"^(?: {4}|\t| {1,3}\t)")
LIST_ITEM = re.compile(r"^\s*(?:[-*+]|\d+[.)])(?:\s|$)")


def normalize(text: str) -> str:
    return ZERO_WIDTH.sub("", unicodedata.normalize("NFKC", text))


def tokenize(text: str) -> list[str]:
    try:
        return shlex.split(text)
    except ValueError:
        return text.split()


def _drop_options(tokens: list[str], value_opts: frozenset[str]) -> list[str]:
    while tokens and tokens[0].startswith("-"):
        opt = tokens.pop(0)
        if opt == "--":
            break
        if opt in value_opts and tokens:
            tokens.pop(0)
    return tokens


def strip_prefix(tokens: list[str]) -> list[str]:
    """Strip prompts, sudo/env/wrappers, assignments and a path prefix until none is left."""
    tokens = list(tokens)
    while tokens:
        head = tokens[0]
        if head in PROMPTS or ASSIGNMENT.match(head):
            tokens.pop(0)
            continue
        base = head.rsplit("/", 1)[-1]
        if base != head and base:
            tokens[0] = base
            continue
        wrapper = head.lower()
        if wrapper in WRAPPERS:
            tokens = _drop_options(tokens[1:], WRAPPERS[wrapper])
            if wrapper == "timeout" and tokens and DURATION.match(tokens[0]):
                tokens.pop(0)
            continue
        break
    return tokens


def command_tokens(text: str) -> list[str] | None:
    """Tokens of a command (tool first) if text is command-shaped, else None."""
    tokens = strip_prefix(tokenize(text.strip()))
    if len(tokens) >= 2 and tokens[0].lower() in TOOL_SET:
        return tokens
    return None


def is_command(text: str) -> bool:
    return command_tokens(text) is not None


def subcommand_words(args: list[str]) -> list[tuple[int, str]]:
    """(index, lowercased word) for the leading bare words of args, skipping flags and flag values."""
    words: list[tuple[int, str]] = []
    i = 0
    while i < len(args) and len(words) < MAX_SUBCOMMAND_WORDS:
        tok = args[i]
        if tok.startswith("-"):
            i += 2 if tok in VALUE_FLAGS else 1
            continue
        if not BARE_WORD.match(tok):
            break
        words.append((i, tok.lower()))
        i += 1
    return words


def _config_assignment(args: list[str], config_index: int) -> bool:
    i = config_index + 1
    while i < len(args):
        tok = args[i]
        if tok.startswith("-"):
            i += 2 if tok in VALUE_FLAGS else 1
            continue
        if KEY_VALUE.match(tok):
            return True
        i += 1
    return False


def mutating(text: str) -> bool:
    tokens = command_tokens(text)
    if tokens is None:
        return False
    tool, args = tokens[0].lower(), tokens[1:]
    indexed = subcommand_words(args)
    words = [w for _, w in indexed]
    flags = [t for t in args if t.startswith("-")]
    help_flags = HELP_FLAGS - {"-h"} if tool in NO_SHORT_HELP else HELP_FLAGS
    if any(f in help_flags for f in flags) or "--dry-run=client" in flags:
        return False
    if words[:1] == ["help"]:
        return False
    if tool == "helm" and words[:1] in (["template"], ["get"]):
        return False
    if tool in ("kubectl", "k", "oc") and words[:2] == ["auth", "can-i"]:
        return False
    verbs = MUTATING_VERBS | TOOL_VERBS.get(tool, frozenset())
    if any(w in verbs for w in words):
        return True
    if any(pair in TWO_WORD_VERBS for pair in zip(words, words[1:])):
        return True
    if any(w == "config" and _config_assignment(args, i) for i, w in indexed):
        return True
    flag_set = FLAG_MUTATIONS.get(tool)
    return bool(flag_set) and any(f.split("=", 1)[0] in flag_set for f in flags)


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


def preceding_class(lines: list[str], i: int) -> str | None:
    """The `Class: [Cn]` line directly before line i (one blank line between allowed), if any."""
    j = i - 1
    while j >= 0 and not lines[j].strip():
        j -= 1
    if j >= 0 and j >= i - 2 and CLASS_LINE.match(lines[j].strip()):
        return lines[j].strip()
    return None


def check_block_line(rel: Path, lineno: int, text: str, block_class: str | None, kind: str,
                     errors: list[str]) -> None:
    """CB02/CB12 for one line of a code block (fenced, indented or HTML <pre>)."""
    stripped = text.strip()
    if not stripped or stripped.startswith("#") or not is_command(stripped):
        return
    tag = CLASS_TAG.search(stripped)
    if block_class is None and not tag:
        errors.append(f"CB02 {rel}:{lineno}: command line in {kind} without a preceding 'Class: [Cn]' line")
    effective = tag.group(0) if tag else (CLASS_TAG.search(block_class).group(0) if block_class else "")
    if effective and mutating(stripped) and LOW_CLASS.search(effective):
        errors.append(f"CB12 {rel}:{lineno}: mutating command `{stripped[:60]}` classed {effective}; needs [C4] or [C5]")


def html_text(fragment: str) -> str:
    return html.unescape(HTML_TAG.sub("", fragment)).replace("`", "")


def check_commands(path: Path, errors: list[str]) -> None:
    rel = path.relative_to(ROOT)
    text = normalize(path.read_text(encoding="utf-8"))
    lines = [ln.rstrip("\r") for ln in text.split("\n")]
    starts: list[int] = []
    pos = 0
    for raw in text.split("\n"):
        starts.append(pos)
        pos += len(raw) + 1

    def line_of(offset: int) -> int:
        return bisect.bisect_right(starts, offset) - 1

    # Pass 1: fenced blocks.
    kind = ["text"] * len(lines)
    block_class: list[str | None] = [None] * len(lines)
    in_fence, fence_marker, cls = False, "", None
    for i, line in enumerate(lines):
        if in_fence:
            if line.strip().startswith(fence_marker):
                in_fence, kind[i] = False, "fence-marker"
            else:
                kind[i], block_class[i] = "fence", cls
            continue
        m = FENCE.match(line)
        if m:
            in_fence, fence_marker, kind[i] = True, m.group(2), "fence-marker"
            if m.group(3).lower() in FORBIDDEN_FENCES and path.is_relative_to(SKILLS):
                errors.append(f"CB03 {rel}:{i + 1}: '{m.group(3)}' fence in a static-review skill; use 'text'")
            cls = preceding_class(lines, i)

    # Pass 2: HTML <code>/<pre> over the whole text, with fences and tag-literal backtick spans
    # (`<pre>` named in prose) blanked so neither can open or close a bogus match.
    masked = list(text)
    for i, k in enumerate(kind):
        if k != "text":
            for c in range(starts[i], starts[i] + len(lines[i])):
                masked[c] = " "
    for i, line in enumerate(lines):
        if kind[i] != "text":
            continue
        for sm in CODE_SPAN.finditer(line):
            if HTML_TAG_LITERAL.search(sm.group(1)):
                for c in range(starts[i] + sm.start(), starts[i] + sm.end()):
                    masked[c] = " "
    html_inline: dict[int, list[tuple[int, int, str]]] = {}
    html_blocks: list[tuple[int, int]] = []  # absolute (start, end) of multi-line matches
    for hm in HTML_CODE.finditer("".join(masked)):
        first = line_of(hm.start())
        if "\n" not in hm.group(0):
            col = hm.start() - starts[first]
            html_inline.setdefault(first, []).append((col, hm.end() - starts[first], html_text(hm.group(2))))
            continue
        html_blocks.append((hm.start(), hm.end()))
        cls = preceding_class(lines, first)
        content_line = line_of(hm.start(2))
        for k, piece in enumerate(text[hm.start(2):hm.end(2)].split("\n")):
            check_block_line(rel, content_line + k + 1, html_text(piece), cls, "a <pre> block", errors)
        for ln in range(first, line_of(hm.end()) + 1):
            if kind[ln] == "text":
                kind[ln] = "html"

    # Pass 3: indented code blocks (outside fences, HTML blocks, front matter and list content).
    fm_end = -1
    if lines and lines[0].strip() == "---":
        fm_end = next((j for j in range(1, len(lines)) if lines[j].strip() == "---"), -1)
    in_indent, cls = False, None
    for i, line in enumerate(lines):
        if kind[i] != "text":
            in_indent = False
            continue
        if not line.strip():
            continue
        if i > fm_end and INDENTED.match(line):
            if in_indent:
                kind[i], block_class[i] = "indented", cls
                continue
            if i > 0 and not lines[i - 1].strip():
                j = i - 1
                while j >= 0 and not lines[j].strip():
                    j -= 1
                prev = lines[j] if j >= 0 else ""
                if j < 0 or (not LIST_ITEM.match(prev) and not prev[:1].isspace()):
                    in_indent, cls = True, preceding_class(lines, i)
                    kind[i], block_class[i] = "indented", cls
            continue
        in_indent = False

    # Pass 4: block lines, then inline spans with per-span tag pairing.
    for i, line in enumerate(lines):
        if kind[i] in ("fence", "indented"):
            what = "a fenced block" if kind[i] == "fence" else "an indented code block"
            check_block_line(rel, i + 1, line, block_class[i], what, errors)
            continue
        if kind[i] not in ("text", "html"):
            continue
        spans: list[tuple[int, int, str]] = []
        for sm in CODE_SPAN.finditer(line):
            absolute = starts[i] + sm.start()
            if any(b0 <= absolute < b1 for b0, b1 in html_blocks):
                continue
            spans.append((sm.start(), sm.end(), sm.group(1)))
        spans += html_inline.get(i, [])
        commands: list[tuple[int, int, str]] = []
        for span in sorted(s for s in spans if is_command(s[2])):
            if commands and span[0] < commands[-1][1]:
                continue  # overlaps a span already judged (e.g. `...` inside <code>)
            commands.append(span)
        for n, (_, end, body) in enumerate(commands):
            window_end = commands[n + 1][0] if n + 1 < len(commands) else len(line)
            tag = CLASS_TAG.search(line, end, window_end)
            if not tag:
                errors.append(f"CB01 {rel}:{i + 1}: command `{body}` has no class tag of its own "
                              "(expected [Cn] after it, before the next command on the line)")
                continue
            if mutating(body) and LOW_CLASS.match(tag.group(0)):
                errors.append(f"CB12 {rel}:{i + 1}: mutating command `{body[:60]}` classed {tag.group(0)}; needs [C4] or [C5]")


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
    if not INSTALL_ROLES.is_file():
        errors.append(f"CB11 {INSTALL_ROLES.relative_to(ROOT)}: missing")
    else:
        roles = json.loads(INSTALL_ROLES.read_text(encoding="utf-8")).get("roles", {})
        for role_id, role in sorted(roles.items()):
            role_skills = set(role.get("skills") or [])
            used = sorted(role_skills & (cilium_skills - {SHARED_SKILL}))
            if used and SHARED_SKILL not in role_skills:
                errors.append(f"CB11 {INSTALL_ROLES.relative_to(ROOT)}: role '{role_id}' installs Cilium skill(s) {used} without {SHARED_SKILL}")

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
