#!/usr/bin/env python3
"""Validation gate for the cross-harness mods under `mods/`.

Every mod is `mods/<id>/` with a `mod.toml` source of truth. This gate checks the
contract in `.claude/plan/cross-harness-mods.md`: the declared footprint matches
the derived reach level, every event is covered or declared unsupported per
harness, the generated manifests are well formed and in sync, the hand-written
code stays inside its declared capabilities, and the pattern data compiles.

Failures print `FAIL [mods] <path> <description>` and warnings `WARN [mods] ...`.
Stdlib only, deterministic (no clock, no network); `node` is the only subprocess.
The capability and env scans are line-oriented heuristics, not a JS parser: they
catch accidents, not deliberate obfuscation.

Usage: python3 tests/validate-mods.py [--root <dir>]
"""

from __future__ import annotations

import argparse
import json
import posixpath
import re
import subprocess
import sys
import tomllib
from dataclasses import dataclass
from pathlib import Path

HARNESSES = ("claude-code", "kiro", "codex")
EVENTS = ("PreToolUse", "PostToolUse", "SessionStart")
SURFACES = ("native", "classic")
EV_KEBAB = {"PreToolUse": "pre-tool-use", "PostToolUse": "post-tool-use", "SessionStart": "session-start"}
NATIVE_EVENT = {"PreToolUse": "tool.check", "PostToolUse": "tool.call", "SessionStart": "session.start"}
KIRO_HOOK_KEYS = {"name", "description", "trigger", "matcher", "action", "timeout", "enabled", "confirm"}
CATALOGS = ("agents", "skills", "rules")
TIMEOUT = 60

# Capability tokens: (label, regex, exemption). exemption "net" is lifted by
# egress_hosts, "spawn" by spawns; None is never allowed. `Function\(` also
# covers `new Function(`. `\b` cannot precede `$`, hence the bare `\$`.
CAPS = (
    ("fetch(", r"\bfetch\s*\(", "net"),
    ("WebSocket", r"\bWebSocket\b", "net"),
    ("XMLHttpRequest", r"\bXMLHttpRequest\b", "net"),
    ("node:net", r"node:net\b", "net"),
    ("node:http", r"node:http\b", "net"),
    ("node:https", r"node:https\b", "net"),
    ("node:tls", r"node:tls\b", "net"),
    ("node:dgram", r"node:dgram\b", "net"),
    ("$.network", r"\$\.network\b", "net"),
    ("child_process", r"\bchild_process\b", "spawn"),
    ("$.process", r"\$\.process\b", "spawn"),
    ("node:worker_threads", r"node:worker_threads\b", None),
    ("eval(", r"\beval\s*\(", None),
    ("Function(", r"\bFunction\s*\(", None),
    ("dynamic import(", r"\bimport\s*\(", None),
    ("require(", r"\brequire\s*\(", None),
)
# The spec list plus the promise/callback and other mutating siblings, so
# `fs.promises.writeFile` is not a free bypass.
WRITE_API = re.compile(
    r"\b(?:writeFileSync|appendFileSync|writeSync|writevSync|mkdirSync|renameSync|unlinkSync|rmSync|rmdirSync"
    r"|copyFileSync|truncateSync|ftruncateSync|symlinkSync|linkSync|createWriteStream|writeFile|appendFile)\b"
    r"|\$\.fs\.write\w*"
)
OPEN_SYNC = re.compile(r"\bopenSync\s*\(")
IMPORT_RE = re.compile(r"""\b(?:import|export)\b(?:[^;'"`]*?\bfrom)?\s*(['"])([^'"\n]+)\1""")
ON_RE = re.compile(r"""\bon\(\s*(['"])([\w.]+)\1""")
ENV_LITERAL = (
    (re.compile(r"\bprocess\s*\.\s*env\s*\.\s*([A-Za-z_$][\w$]*)"), 1),
    (re.compile(r"""\bprocess\s*\.\s*env\s*\[\s*(['"])([^'"\n]*)\1\s*\]"""), 2),
    (re.compile(r"""\$\.env\.get\(\s*(['"])([^'"\n]*)\1\s*\)"""), 2),
)
ENV_ANY = re.compile(r"""\bprocess\s*(?:\.\s*env\b|\[\s*['"]env['"]\s*\])|\$\.env\b""")
INLINE_FLAG = re.compile(r"(?<!\\)(?:\\\\)*\(\?[-aiLmsux]+[:)]")
NET_MODULES = {"node:net", "node:http", "node:https", "node:tls", "node:dgram"}


class Report:
    def __init__(self) -> None:
        self.lines: list[str] = []
        self.failed = 0

    def _add(self, level: str, path: str, desc: str) -> None:
        head, *rest = (desc.strip().splitlines() or [""])
        self.lines.append(f"{level} [mods] {path} {head}".rstrip())
        self.lines.extend(f"    {line}" for line in rest)

    def fail(self, path: str, desc: str) -> None:
        self.failed += 1
        self._add("FAIL", path, desc)

    def warn(self, path: str, desc: str) -> None:
        self._add("WARN", path, desc)


@dataclass
class Mod:
    id: str
    dir: Path
    rel: str
    t: dict

    def h(self, name: str) -> dict:
        return self.t["harnesses"][name]

    def sup(self, name: str) -> bool:
        return self.h(name)["supported"]

    def ev(self, name: str) -> list[str]:
        return self.h(name).get("events", []) if self.sup(name) else []

    @property
    def native(self) -> bool:
        return self.sup("claude-code") and "native" in self.h("claude-code").get("surfaces", [])

    @property
    def classic(self) -> bool:
        return self.sup("claude-code") and "classic" in self.h("claude-code").get("surfaces", [])

    def fp(self, key: str) -> list[str]:
        return self.t["footprint"][key]


def is_strs(v: object) -> bool:
    return isinstance(v, list) and all(isinstance(x, str) for x in v)


def exact(v: object, keys: set[str]) -> bool:
    return isinstance(v, dict) and set(v) == keys


def jdump(v: object) -> str:
    # sort_keys + strict types: `1` and `true` stay distinct, unlike dict equality.
    return json.dumps(v, sort_keys=True, default=str)


def run_node(root: Path, *args: str) -> tuple[int | None, str, str]:
    """Run node with an argument list; return (returncode, stdout, stderr).

    returncode is None, with the reason in stderr, when node could not run at all.
    """
    try:
        p = subprocess.run(
            ["node", *args], cwd=root, capture_output=True, text=True, errors="replace", timeout=TIMEOUT
        )
    except FileNotFoundError:
        return None, "", "node executable not found"
    except subprocess.TimeoutExpired:
        return None, "", f"timed out after {TIMEOUT}s"
    return p.returncode, p.stdout, p.stderr


def tail(text: str, n: int = 20) -> str:
    return "\n".join(text.strip().splitlines()[-n:])


# ---------------------------------------------------------------- loading


def shape_errors(t: dict) -> list[str]:
    """Structural problems that would make the policy checks crash."""
    mod, fp, hs = t.get("mod"), t.get("footprint"), t.get("harnesses")
    if not (isinstance(mod, dict) and isinstance(fp, dict) and isinstance(hs, dict)):
        return ["[mod], [footprint] and [harnesses] tables are all required"]
    errs = [f"[mod].{k} must be a string" for k in ("id", "name", "description", "version") if not isinstance(mod.get(k), str)]
    rl = mod.get("reach_level")
    if not isinstance(rl, int) or isinstance(rl, bool):
        errs.append("[mod].reach_level must be an integer")
    if not is_strs(mod.get("events")) or not mod["events"]:
        errs.append("[mod].events must be a non-empty array of strings")
    errs += [f"[footprint].{k} must be an array of strings" for k in ("env_reads", "file_writes", "spawns", "egress_hosts") if not is_strs(fp.get(k))]
    for name in HARNESSES:
        ht = hs.get(name)
        if not isinstance(ht, dict) or not isinstance(ht.get("supported"), bool):
            errs.append(f"[harnesses.{name}] with boolean `supported` is required")
            continue
        if ht["supported"]:
            if not is_strs(ht.get("events")):
                errs.append(f"[harnesses.{name}].events must be an array of strings")
            if name == "claude-code" and not is_strs(ht.get("surfaces")):
                errs.append("[harnesses.claude-code].surfaces must be an array of strings")
        elif not isinstance(ht.get("reason"), str) or not ht["reason"].strip():
            errs.append(f"[harnesses.{name}] has supported = false but no `reason`")
        for tbl in ("unsupported", "degraded"):
            v = ht.get(tbl, {})
            if not (isinstance(v, dict) and all(isinstance(x, str) for x in v.values())):
                errs.append(f"[harnesses.{name}.{tbl}] must map strings to strings")
    return errs


def load_mod(r: Report, d: Path) -> Mod | None:
    rel = f"mods/{d.name}"
    p = d / "mod.toml"
    if not p.is_file():
        r.fail(f"{rel}/mod.toml", "is missing")
        return None
    try:
        with p.open("rb") as f:
            t = tomllib.load(f)
    except (tomllib.TOMLDecodeError, UnicodeDecodeError, OSError) as exc:
        r.fail(f"{rel}/mod.toml", f"does not parse: {exc}")
        return None
    errs = shape_errors(t)
    for e in errs:
        r.fail(f"{rel}/mod.toml", e)
    if errs:
        return None
    if t["mod"]["id"] != d.name:
        r.fail(f"{rel}/mod.toml", f"[mod].id {t['mod']['id']!r} does not equal directory name {d.name!r}")
    return Mod(d.name, d, rel, t)


# ---------------------------------------------------------------- mod.toml policy


def check_toml_policy(r: Report, m: Mod) -> None:
    path = f"{m.rel}/mod.toml"
    mod = m.t["mod"]
    # 2. derived reach
    derived = 3 if m.fp("egress_hosts") else 2 if m.fp("spawns") else 1 if m.fp("file_writes") else 0
    if mod["reach_level"] != derived:
        r.fail(path, f"reach_level {mod['reach_level']} does not equal derived level {derived}")
    if derived == 3:
        r.warn(path, "reach level 3: declares network egress")
    if not re.fullmatch(r"\d+\.\d+\.\d+", mod["version"]):
        r.fail(path, f"version {mod['version']!r} must be MAJOR.MINOR.PATCH")
    # 7. env prefix
    prefix = "VANGUARD_" + m.id.upper().replace("-", "_") + "_"
    for name in m.fp("env_reads"):
        if not name.startswith(prefix):
            r.fail(path, f"env read {name!r} must start with {prefix}")
    # 3-5. events, surfaces, degraded
    mod_events = mod["events"]
    for e in mod_events:
        if e not in EVENTS:
            r.fail(path, f"unknown event {e!r} in [mod].events")
    if len(set(mod_events)) != len(mod_events):
        r.fail(path, "[mod].events has duplicates")
    for h in HARNESSES:
        if not m.sup(h):
            continue
        ht, ev = m.h(h), m.ev(h)
        un = set(ht.get("unsupported", {}))
        un_events = un - set(SURFACES) if h == "claude-code" else un
        if len(set(ev)) != len(ev):
            r.fail(path, f"harnesses.{h}.events has duplicates")
        for e in ev:
            if e not in mod_events:
                r.fail(path, f"harnesses.{h}.events lists {e}, which is not in [mod].events")
        for e in sorted(un_events - set(mod_events)):
            r.fail(path, f"harnesses.{h}.unsupported key {e!r} is not a [mod].events entry" + (" or surface" if h == "claude-code" else ""))
        for e in mod_events:
            n = (e in ev) + (e in un_events)
            if n == 0:
                r.fail(path, f"event {e} is in neither harnesses.{h}.events nor harnesses.{h}.unsupported")
            elif n == 2:
                r.fail(path, f"event {e} is in both harnesses.{h}.events and harnesses.{h}.unsupported")
        surfaces = set(ht.get("surfaces", [])) if h == "claude-code" else set()
        if h == "claude-code":
            for s in sorted(surfaces - set(SURFACES)):
                r.fail(path, f"unknown surface {s!r} in harnesses.claude-code.surfaces")
            un_surfaces = un & set(SURFACES)
            for s in sorted(surfaces & un_surfaces):
                r.fail(path, f"surface {s} is in both surfaces and harnesses.claude-code.unsupported")
            if (surfaces | un_surfaces) != set(SURFACES):
                r.fail(path, "harnesses.claude-code surfaces plus unsupported surfaces must be exactly {native, classic}")
        for k in ht.get("degraded", {}):
            if k not in ev and k not in surfaces:
                r.fail(path, f"harnesses.{h}.degraded key {k!r} is not a supported event" + (" or surface" if h == "claude-code" else ""))
    # An event no harness runs is dead weight in [mod].events, not a portable feature.
    for e in mod_events:
        if not any(e in m.ev(h) for h in HARNESSES):
            r.fail(path, f"event {e} is supported by no harness")
    # 6. UNSUPPORTED.md iff
    for h in HARNESSES:
        ht = m.h(h)
        need = (not ht["supported"]) or bool(ht.get("unsupported")) or bool(ht.get("degraded"))
        exists = (m.dir / h / "UNSUPPORTED.md").is_file()
        if need and not exists:
            r.fail(f"{m.rel}/{h}/UNSUPPORTED.md", "is missing (harness is unsupported, or has unsupported/degraded entries)")
        elif exists and not need:
            r.fail(f"{m.rel}/{h}/UNSUPPORTED.md", "must not exist: nothing is unsupported or degraded")


# ---------------------------------------------------------------- source scans


def strip_comments(src: str) -> str:
    """Drop `//` and `/* */` comments, leaving string contents alone.

    ponytail: quote-aware but not regex-literal-aware; a regex literal holding a
    quote can hide the rest of its line. Upgrade to a real JS tokenizer if that bites.
    """
    out: list[str] = []
    i, n, q = 0, len(src), ""
    while i < n:
        c = src[i]
        if q:
            out.append(c)
            if c == "\\" and i + 1 < n:
                out.append(src[i + 1])
                i += 2
                continue
            if c == q or (c == "\n" and q != "`"):
                q = ""
            i += 1
        elif src.startswith("//", i):
            j = src.find("\n", i)
            i = n if j < 0 else j
        elif src.startswith("/*", i):
            j = src.find("*/", i + 2)
            i = n if j < 0 else j + 2
            out.append(" ")
        else:
            if c in "'\"`":
                q = c
            out.append(c)
            i += 1
    return "".join(out)


def call_args(text: str, start: int) -> list[str]:
    """Top-level comma-split arguments of the call whose `(` is just before `start`."""
    args, depth, cur, q, i = [], 0, [], "", start
    while i < len(text):
        c = text[i]
        if q:
            cur.append(c)
            if c == "\\":
                cur.append(text[i + 1 : i + 2])
                i += 1
            elif c == q:
                q = ""
        elif c in "'\"`":
            q = c
            cur.append(c)
        elif c in "([{":
            depth += 1
            cur.append(c)
        elif c in ")]}":
            if depth == 0:
                break
            depth -= 1
            cur.append(c)
        elif c == "," and depth == 0:
            args.append("".join(cur).strip())
            cur = []
        else:
            cur.append(c)
        i += 1
    args.append("".join(cur).strip())
    return args


def write_hits(text: str) -> list[str]:
    hits = [m.group(0) for m in WRITE_API.finditer(text)]
    for m in OPEN_SYNC.finditer(text):
        args = call_args(text, m.end())
        if len(args) < 2:
            continue  # openSync(path) defaults to read-only
        lit = re.fullmatch(r"""(['"`])(.*)\1""", args[1])
        # a flag that is not a string literal cannot be judged, so it counts as a write
        if lit is None or set(lit.group(2)) & set("wa+"):
            hits.append(f"openSync(.., {args[1]})")
    return hits


def scan_files(m: Mod) -> dict[str, str]:
    """relpath-in-mod -> comment-stripped source for lib/*.mjs, bin/*.mjs, hooks/*.ts (not tests)."""
    found = sorted([*m.dir.glob("lib/*.mjs"), *m.dir.glob("bin/*.mjs"), *m.dir.glob("hooks/*.ts")])
    return {
        p.relative_to(m.dir).as_posix(): strip_comments(p.read_text(encoding="utf-8", errors="replace"))
        for p in found
        if p.is_file() and not p.name.endswith(".test.ts")
    }


def check_env_scan(r: Report, m: Mod, files: dict[str, str]) -> None:
    declared = set(m.fp("env_reads"))
    used: dict[str, str] = {}
    for rel, text in files.items():
        for am in ENV_ANY.finditer(text):
            name = None
            for rx, grp in ENV_LITERAL:
                lm = rx.match(text, am.start())
                if lm:
                    name = lm.group(grp)
                    break
            if name is None:
                snippet = " ".join(text[am.start() : am.start() + 40].split())
                r.fail(f"{m.rel}/{rel}", f"non-literal env access `{snippet}`; use process.env.NAME or $.env.get('NAME')")
            else:
                used.setdefault(name, rel)
    for name, rel in sorted(used.items()):
        if name not in declared:
            r.fail(f"{m.rel}/{rel}", f"reads env {name} which is not declared in [footprint].env_reads")
    for name in sorted(declared - set(used)):
        r.fail(f"{m.rel}/mod.toml", f"declared env read {name} is not used in lib/, bin/ or hooks/")


def import_allowed(m: Mod, rel: str, spec: str) -> bool:
    """Import policy per file role (see spec check 14 and 13)."""
    # runtime.mjs imports node:fs, so nothing the Node-free native runtime loads may import it.
    if rel.startswith("hooks/"):
        return spec == "claude-code" or (spec != "../lib/runtime.mjs" and re.fullmatch(r"\.\./lib/[a-z0-9-]+\.mjs", spec) is not None)
    if rel.startswith("bin/") or rel == "lib/runtime.mjs":
        ok = {"node:fs", "node:path"}
        if m.fp("spawns"):
            ok.add("node:child_process")
        if m.fp("egress_hosts"):
            ok |= NET_MODULES
        if spec in ok:
            return True
        if spec.startswith("./") or spec.startswith("../"):
            return not posixpath.normpath(posixpath.join(posixpath.dirname(rel), spec)).startswith("..")
        return False
    return spec != "./runtime.mjs" and re.fullmatch(r"\./[A-Za-z0-9._-]+\.mjs", spec) is not None  # other lib/*.mjs: Node-free runtime


def check_capabilities(r: Report, m: Mod, files: dict[str, str]) -> None:
    exempt = {"net": bool(m.fp("egress_hosts")), "spawn": bool(m.fp("spawns")), None: False}
    can_write = bool(m.fp("file_writes"))
    for rel, text in files.items():
        path = f"{m.rel}/{rel}"
        for im in IMPORT_RE.finditer(text):
            spec = im.group(2)
            if not import_allowed(m, rel, spec):
                hint = f" (use node:{spec})" if spec in ("fs", "path") else ""
                r.fail(path, f"import of {spec!r} is not allowed here{hint}")
        for label, rx, ex in CAPS:
            if re.search(rx, text) and not exempt[ex]:
                need = {"net": " without egress_hosts", "spawn": " without spawns", None: ""}[ex]
                r.fail(path, f"uses {label}{need}")
        hits = write_hits(text)
        if hits and rel in ("lib/common.mjs", "lib/runtime.mjs"):
            r.fail(path, f"shared module must not use write APIs: {', '.join(sorted(set(hits)))}")
        elif hits and not can_write:
            r.fail(path, f"uses write API {', '.join(sorted(set(hits)))} without file_writes")


def check_register(r: Report, m: Mod, text: str) -> None:
    path = f"{m.rel}/hooks/register.ts"
    if not re.search(r"\bexport\s+const\s+register\b", text):
        r.fail(path, "must contain `export const register`")
    registered = [x.group(2) for x in ON_RE.finditer(text)]
    want = {NATIVE_EVENT[e] for e in m.ev("claude-code") if e in NATIVE_EVENT}
    if set(registered) != want:
        r.fail(path, f"registers native events {sorted(set(registered))}, expected {sorted(want)} from harnesses.claude-code.events")
    guarded = sum(e in ("tool.check", "tool.call") for e in registered)
    catches = len(re.findall(r"\.catch\(", text))
    if catches < guarded:
        r.fail(path, f"has {catches} .catch( for {guarded} tool.check/tool.call registrations")


def check_sources(r: Report, root: Path, m: Mod) -> None:
    files = scan_files(m)
    check_env_scan(r, m, files)
    check_capabilities(r, m, files)
    if "hooks/register.ts" in files:
        check_register(r, m, files["hooks/register.ts"])
    # 11. syntax
    for p in sorted(m.dir.rglob("*.mjs")):
        rc, out, err = run_node(root, "--check", str(p))
        rel = f"{m.rel}/{p.relative_to(m.dir).as_posix()}"
        if rc is None:
            r.fail(rel, f"node --check could not run: {err}")
        elif rc != 0:
            r.fail(rel, f"node --check failed:\n{tail(err or out, 8)}")


def check_required_files(r: Report, m: Mod) -> None:
    need = ["lib/common.mjs", "lib/runtime.mjs", "lib/core.mjs", "bin/hook.mjs", "README.md"]
    if m.native:
        need += ["hooks/hooks.json", "hooks/register.ts", "hooks/register.test.ts"]
    for rel in need:
        if not (m.dir / rel).is_file():
            r.fail(f"{m.rel}/{rel}", "is missing")
    if not m.native and (m.dir / "hooks").exists():
        r.fail(f"{m.rel}/hooks", "must not exist: the native surface is not supported")


# ---------------------------------------------------------------- generated shapes


def read_json(r: Report, path: Path, rel: str) -> object | None:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (json.JSONDecodeError, UnicodeDecodeError, OSError) as exc:
        r.fail(rel, f"is not valid JSON: {exc}")
        return None


def check_command(r: Report, rel: str, cmd: str, harness: str, event: str) -> None:
    if "bin/hook.mjs" not in cmd:
        r.fail(rel, f"{event}: command does not run bin/hook.mjs")
    if re.findall(r"--harness\s+(\S+)", cmd) != [harness]:
        r.fail(rel, f"{event}: command must carry exactly `--harness {harness}`")
    if re.findall(r"--event\s+(\S+)", cmd) != [event]:
        r.fail(rel, f"{event}: command must carry exactly `--event {event}`")


def check_hook_map(r: Report, rel: str, data: object, harness: str, events: list[str]) -> None:
    if not exact(data, {"hooks"}) or not isinstance(data["hooks"], dict):
        r.fail(rel, 'must be {"hooks": {<Event>: [{"hooks": [{"type": "command", "command": ...}]}]}}')
        return
    hooks = data["hooks"]
    if set(hooks) != set(events):
        r.fail(rel, f"event keys {sorted(hooks)} must equal harnesses.{harness}.events {sorted(events)}")
    for ev, groups in sorted(hooks.items()):
        ok = (
            isinstance(groups, list)
            and len(groups) == 1
            and exact(groups[0], {"hooks"})
            and isinstance(groups[0]["hooks"], list)
            and len(groups[0]["hooks"]) == 1
            and exact(groups[0]["hooks"][0], {"type", "command"})
            and groups[0]["hooks"][0]["type"] == "command"
            and isinstance(groups[0]["hooks"][0]["command"], str)
        )
        if ok:
            check_command(r, rel, groups[0]["hooks"][0]["command"], harness, ev)
        else:
            r.fail(rel, f'{ev}: must be exactly [{{"hooks": [{{"type": "command", "command": <string>}}]}}]')


def check_kiro(r: Report, m: Mod) -> None:
    d = m.dir / "kiro"
    files = sorted(p.name for p in d.glob("*.json")) if d.is_dir() else []
    events = m.ev("kiro")
    seen: set[str] = set()
    for name in files:
        rel = f"{m.rel}/kiro/{name}"
        if not m.sup("kiro"):
            r.fail(rel, "must not exist: kiro is not supported")
            continue
        data = read_json(r, d / name, rel)
        if data is None:
            continue
        if not (
            exact(data, {"version", "hooks"})
            and data["version"] == "v1"
            and isinstance(data["hooks"], list)
            and len(data["hooks"]) == 1
            and isinstance(data["hooks"][0], dict)
        ):
            r.fail(rel, 'must be {"version": "v1", "hooks": [<exactly one hook>]}')
            continue
        hook = data["hooks"][0]
        extra = set(hook) - KIRO_HOOK_KEYS
        if extra:
            r.fail(rel, f"hook has unknown keys {sorted(extra)}")
        trig = hook.get("trigger")
        if trig not in events:
            r.fail(rel, f"trigger {trig!r} is not in harnesses.kiro.events")
            continue
        seen.add(trig)
        if name != f"{m.id}-{EV_KEBAB[trig]}-hook.json":
            r.fail(rel, f"file name must be {m.id}-{EV_KEBAB[trig]}-hook.json")
        if hook.get("name") != f"{m.id}:{trig}":
            r.fail(rel, f"hook name must be {m.id}:{trig}, got {hook.get('name')!r}")
        act = hook.get("action")
        if not (exact(act, {"type", "command"}) and act["type"] == "command" and isinstance(act["command"], str)):
            r.fail(rel, 'action must be {"type": "command", "command": <string>}')
        else:
            check_command(r, rel, act["command"], "kiro", trig)
    for e in events:
        if e not in seen and m.sup("kiro") and e in EV_KEBAB:
            r.fail(f"{m.rel}/kiro/{m.id}-{EV_KEBAB[e]}-hook.json", "is missing")


def check_generated(r: Report, m: Mod) -> None:
    check_kiro(r, m)
    for h, rel, want in (
        ("codex", "codex/hooks.json", m.sup("codex")),
        ("claude-code", "claude-code/settings.hooks.json", m.classic),
    ):
        path = m.dir / rel
        if not want:
            if path.exists():
                r.fail(f"{m.rel}/{rel}", f"must not exist: {h} is not supported here")
        elif not path.is_file():
            r.fail(f"{m.rel}/{rel}", "is missing")
        elif (data := read_json(r, path, f"{m.rel}/{rel}")) is not None:
            check_hook_map(r, f"{m.rel}/{rel}", data, h, m.ev(h))
    plugin = m.dir / ".claude-plugin" / "plugin.json"
    rel = f"{m.rel}/.claude-plugin/plugin.json"
    if not m.native:
        if plugin.exists():
            r.fail(rel, "must not exist: the native surface is not supported")
    elif not plugin.is_file():
        r.fail(rel, "is missing")
    elif (data := read_json(r, plugin, rel)) is not None:
        mod = m.t["mod"]
        for key, want in (("name", m.id), ("version", mod["version"]), ("description", mod["description"])):
            if not isinstance(data, dict) or data.get(key) != want:
                r.fail(rel, f"{key} must equal {want!r}")
    if m.native and (m.dir / "hooks" / "hooks.json").is_file():
        data = read_json(r, m.dir / "hooks" / "hooks.json", f"{m.rel}/hooks/hooks.json")
        if data is not None and data != {"modules": ["./register.ts"]}:
            r.fail(f"{m.rel}/hooks/hooks.json", 'must be {"modules": ["./register.ts"]}')


# ---------------------------------------------------------------- patterns


def check_patterns(r: Report, m: Mod) -> None:
    p = m.dir / "patterns.json"
    if not p.is_file():
        return
    rel = f"{m.rel}/patterns.json"
    data = read_json(r, p, rel)
    if data is None:
        return
    if not isinstance(data, list):
        r.fail(rel, "must be an array of pattern objects")
        return
    required = {"id": str, "name": str, "description": str, "regex": str, "flags": str, "context": list}
    optional = {"severity": str, "agents": list}
    ids: set[str] = set()
    for i, e in enumerate(data):
        if not isinstance(e, dict):
            r.fail(rel, f"entry {i} is not an object")
            continue
        pid = e.get("id", f"#{i}")
        bad = [k for k, ty in required.items() if not isinstance(e.get(k), ty)]
        bad += [k for k, ty in optional.items() if k in e and not isinstance(e[k], ty)]
        for k in bad:
            r.fail(rel, f"pattern {pid}: field {k!r} is missing or has the wrong type")
        for k in sorted(set(e) - set(required) - set(optional)):
            r.fail(rel, f"pattern {pid}: unknown field {k!r}")
        if bad:
            continue
        for k in ("context", "agents"):
            if k in e and not is_strs(e[k]):
                r.fail(rel, f"pattern {pid}: {k} must be an array of strings")
        if pid in ids:
            r.fail(rel, f"duplicate pattern id {pid!r}")
        ids.add(pid)
        if e["flags"] not in ("", "i"):
            r.fail(rel, f"pattern {pid}: flags {e['flags']!r} must be '' or 'i'")
            continue
        if INLINE_FLAG.search(e["regex"]):
            r.fail(rel, f"pattern {pid}: inline flag group in regex; use the `flags` field")
        try:
            re.compile(e["regex"], re.ASCII | (re.I if e["flags"] == "i" else 0))
        except re.error as exc:
            r.fail(rel, f"pattern {pid}: regex does not compile under re.ASCII: {exc}")


# ---------------------------------------------------------------- repo-level checks


def check_generator(r: Report, root: Path, mods: list[Mod], mod_dirs: list[str]) -> None:
    gen = "scripts/generate-mods.mjs"
    if not (root / gen).is_file():
        r.fail(gen, "is missing")
        return
    rc, out, err = run_node(root, gen, "--check")
    if rc is None:
        r.fail(gen, f"--check could not run: {err}")
        return
    if rc != 0:
        r.fail(gen, f"--check exited {rc}:\n{tail(err or out)}")
    rc, out, err = run_node(root, gen, "--print-manifests")
    if rc != 0:
        r.fail(gen, f"--print-manifests exited {rc}:\n{tail(err or out)}")
        return
    try:
        printed = json.loads(out)
    except json.JSONDecodeError as exc:
        r.fail(gen, f"--print-manifests output is not JSON: {exc}")
        return
    if not isinstance(printed, dict):
        r.fail(gen, "--print-manifests output must be a JSON object keyed by mod id")
        return
    for m in mods:
        if m.id not in printed:
            r.fail(f"{m.rel}/mod.toml", "missing from generator --print-manifests output")
        elif jdump(printed[m.id]) != jdump(m.t):
            r.fail(f"{m.rel}/mod.toml", "generator parse differs from tomllib parse")
    for extra in sorted(set(printed) - set(mod_dirs)):
        r.fail(gen, f"--print-manifests lists {extra!r}, which has no mods/{extra}/")


def check_catalog_mods(r: Report, root: Path, mods: list[Mod], mod_dirs: list[str]) -> None:
    rel = "catalog/mods.json"
    p = root / rel
    if not p.is_file():
        r.fail(rel, "is missing")
        return
    cat = read_json(r, p, rel)
    if cat is None:
        return
    if not isinstance(cat, list) or not all(isinstance(e, dict) for e in cat):
        r.fail(rel, "must be an array of objects")
        return
    ids = [e.get("id") for e in cat]
    if ids != sorted(ids, key=str):
        r.fail(rel, "is not sorted by id")
    for i in sorted({x for x in ids if ids.count(x) > 1}, key=str):
        r.fail(rel, f"duplicate entry for {i!r}")
    for i in sorted(set(mod_dirs) - set(ids)):
        r.fail(rel, f"has no entry for mods/{i}")
    for i in sorted(set(ids) - set(mod_dirs), key=str):
        r.fail(rel, f"has an entry for {i!r}, which has no mods/ directory")
    by_id = {e.get("id"): e for e in cat}
    for m in mods:
        mod = m.t["mod"]
        want = {
            "id": m.id,
            "name": mod["name"],
            "version": mod["version"],
            "description": mod["description"],
            "reach_level": mod["reach_level"],
            "events": mod["events"],
            "harnesses": sorted(h for h in HARNESSES if m.sup(h)),
            "path": m.rel,
        }
        got = by_id.get(m.id)
        if got is not None and jdump(got) != jdump(want):
            keys = sorted(k for k in set(want) | set(got) if jdump(want.get(k, "<absent>")) != jdump(got.get(k, "<absent>")))
            r.fail(rel, f"entry {m.id!r} differs from mod.toml in: {', '.join(keys)}")


def check_catalog_ids(r: Report, root: Path, mods: list[Mod]) -> None:
    for name in CATALOGS:
        rel = f"catalog/{name}.json"
        p = root / rel
        data = read_json(r, p, rel) if p.is_file() else None
        if data is None:
            if not p.is_file():
                r.fail(rel, "is missing, so mod id uniqueness cannot be checked")
            continue
        if not isinstance(data, list):
            r.fail(rel, "must be a top-level array")
            continue
        taken = {e.get("id") for e in data if isinstance(e, dict)}
        for m in mods:
            if m.id in taken:
                r.fail(f"{m.rel}/mod.toml", f"id {m.id!r} collides with an id in {rel}")


def check_write_overlap(r: Report, mods: list[Mod]) -> None:
    def prefix(glob: str) -> str:
        return re.split(r"[*?\[{]", glob, maxsplit=1)[0]

    globs = [(m, g) for m in mods for g in m.fp("file_writes")]
    for i, (ma, ga) in enumerate(globs):
        for mb, gb in globs[i + 1 :]:
            if ma.id == mb.id:
                continue
            pa, pb = prefix(ga), prefix(gb)
            if pa.startswith(pb) or pb.startswith(pa):
                r.fail(f"{ma.rel}/mod.toml", f"file_writes {ga!r} overlaps {gb!r} in {mb.rel}")


def check_node_tests(r: Report, root: Path) -> None:
    rel = "tests/mods.test.mjs"
    if not (root / rel).is_file():
        r.fail(rel, "is missing")
        return
    rc, out, err = run_node(root, "--test", rel)
    if rc is None:
        r.fail(rel, f"could not run: {err}")
    elif rc != 0:
        r.fail(rel, f"node --test exited {rc}:\n{tail(out + err)}")


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1], help="repository root (default: this repo)")
    root = ap.parse_args(argv).root.resolve()
    r = Report()

    mods_dir = root / "mods"
    if not mods_dir.is_dir():
        r.fail("mods/", "directory is missing")
        print("\n".join(r.lines))
        return 1
    dirs = sorted(d for d in mods_dir.iterdir() if d.is_dir() and not d.name.startswith("."))
    if not dirs:
        r.fail("mods/", "contains no mod directories")
    if not (mods_dir / "README.md").is_file():
        r.fail("mods/README.md", "is missing")

    mods: list[Mod] = []
    for d in dirs:
        m = load_mod(r, d)
        if m is None:
            continue
        mods.append(m)
        check_toml_policy(r, m)
        check_required_files(r, m)
        check_generated(r, m)
        check_sources(r, root, m)
        check_patterns(r, m)

    mod_dirs = [d.name for d in dirs]
    check_generator(r, root, mods, mod_dirs)
    check_catalog_mods(r, root, mods, mod_dirs)
    check_catalog_ids(r, root, mods)
    check_write_overlap(r, mods)
    check_node_tests(r, root)

    if r.failed == 0:
        n = {h: sum(m.sup(h) for m in mods) for h in HARNESSES}
        r.lines.append(f"OK: {len(mods)} mods valid (claude-code {n['claude-code']}, kiro {n['kiro']}, codex {n['codex']})")
    print("\n".join(r.lines))
    return 1 if r.failed else 0


if __name__ == "__main__":
    sys.exit(main())
