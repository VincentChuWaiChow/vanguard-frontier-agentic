#!/usr/bin/env python3
"""Gate: every NoSQL skill has a lazy-loaded audit with offline and live variants.

Covers each skill directory in ``validate-nosql-operations.py``'s ``SERVICES`` plus
the ported pattern skills. Each skill must ship:

* ``references/commands/audit.md`` (the router), linked from ``SKILL.md``;
* ``references/commands/audit/offline.md`` and ``references/commands/audit/live.md``,
  linked from the router and **not** from ``SKILL.md``, so an agent loads the
  router first and then exactly one variant;
* a router that says to load only one variant.

For the operations skills (not the pattern skills, whose audit is feature-based and
canonical upstream), both variants carry the ten operations headings in order, and
every live section either has a command block plus a citation, or a
``**No live command:**`` line giving the reason.

Command safety, for every skill:

* live commands are read-only: each cloud CLI or ``curl`` invocation must use a
  read verb (describe, get, list, show, ...) and no mutating verb, ``--force`` or
  secret-reading word; ``curl`` may not send a body or a non-GET method;
* data-plane shells (``mongosh``, ``cqlsh``, ...) are banned in both variants:
  audits read control-plane metadata, never customer data;
* offline blocks invoke no cloud CLI, network tool or Terraform command except
  ``terraform show`` (reads a local plan);
* every evidence ID cited in a live file is a VERIFIED row of the skill's ledger,
  and every ID cited in an offline file exists.

The file shapes live in ``templates/nosql-audit-template.md``. Never weaken a rule
to get green; reword the command or record ``**No live command:**`` instead.
"""
from __future__ import annotations

import importlib.util
import re
import shlex
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
_spec = importlib.util.spec_from_file_location("nosql_ops", REPO / "tests/validate-nosql-operations.py")
_ops = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(_ops)

PATTERN_SKILLS = [
    "skills/aws/aws-dynamodb-patterns",
    "skills/oci/oci-nosql-pattern",
    "skills/azure/azure-cosmosdb-pattern",
]
SKILLS = sorted({skill for skill, _ in _ops.SERVICES.values()} | set(PATTERN_SKILLS))
SECTIONS = _ops.SECTIONS
ROUTER = "references/commands/audit.md"
VARIANTS = {"offline": "references/commands/audit/offline.md", "live": "references/commands/audit/live.md"}

CLOUD_CLIS = {"aws", "az", "gcloud", "oci", "aliyun", "hcloud", "scw", "ovhcloud", "ionosctl", "cbt", "bq", "firebase"}
DATA_SHELLS = {"mongosh", "mongo", "cqlsh", "mongodump", "mongoexport", "mongorestore", "mongoimport"}
NETWORK_TOOLS = {"curl", "wget"}
READ_VERBS = ("describe", "get", "list", "show", "ls", "read", "lookup", "search", "view", "info", "status", "inspect", "query", "whoami")
MUTATING_EXACT = {"set", "add", "untag", "run", "put", "stop", "exec", "move", "edit", "copy", "grant", "apply", "patch",
                  "deploy", "drop", "write", "cancel", "purge", "rotate", "revoke", "assign", "unassign", "promote", "rename",
                  "replace", "scale", "resize", "attach", "detach", "upload", "invoke", "insert", "reset", "clone", "change",
                  "start", "reboot", "migrate", "upgrade"}
MUTATING_PREFIX = ("create", "delete", "update", "modify", "restore", "remove", "enable", "disable", "import", "export",
                   "failover", "switchover", "register", "deregister", "execute")
SECRET_WORDS = {"keys", "secret", "secrets", "password", "passwords", "credential", "credentials", "token", "tokens"}
ARRAY_ALIAS = re.compile(r'^"?\$\{\w+\[@\]\}"?$')
BLOCK_RE = re.compile(r"^```(?:bash|sh|shell)\s*\n(.*?)^```", re.MULTILINE | re.DOTALL)
NO_LIVE = "**No live command:**"


def commands(block: str) -> list[list[str]]:
    """Split a shell block into simple commands (tokens), skipping comments and heredoc bodies."""
    text = re.sub(r"<<-?\s*'?(\w+)'?.*?^\1\s*$", "", block, flags=re.MULTILINE | re.DOTALL)
    text = text.replace("\\\n", " ")
    result = []
    for line in text.splitlines():
        lexer = shlex.shlex(line, posix=True, punctuation_chars=";&|")
        lexer.whitespace_split = True
        lexer.commenters = "#"
        try:
            line_tokens = list(lexer)
        except ValueError:
            line_tokens = line.split()
        parts, current = [], []
        for token in line_tokens:
            if token and set(token) <= set(";&|"):
                parts.append(current)
                current = []
            else:
                current.append(token)
        parts.append(current)
        for tokens in parts:
            if tokens and re.match(r"^[A-Za-z_]\w*=\((?!\()", tokens[0]):
                continue  # array definition such as aws_target=(aws --profile ...); its uses are checked
            if tokens:
                tokens[0] = tokens[0].lstrip("(")
                tokens[-1] = tokens[-1].rstrip(")")
            while tokens and re.match(r"^[A-Za-z_]\w*=", tokens[0]):
                substituted = re.match(r"^[A-Za-z_]\w*=\$\((.*)$", tokens[0])
                tokens = ([substituted.group(1)] if substituted and substituted.group(1) else []) + tokens[1:]
                if substituted:
                    break
            if tokens and tokens[0] in {"sudo", "command", "exec", "time"}:
                tokens = tokens[1:]
            if tokens:
                result.append(tokens)
    return result


def is_cli(tokens: list[str]) -> bool:
    return tokens[0] in CLOUD_CLIS or bool(ARRAY_ALIAS.match(tokens[0]))


def words(tokens: list[str]) -> list[str]:
    """Bare subcommand words, split on hyphens and CamelCase (aliyun/hcloud actions)."""
    parts = []
    for token in tokens[1:]:
        if re.fullmatch(r"[A-Za-z][A-Za-z0-9-]*", token):
            for piece in token.split("-"):
                parts += [w.lower() for w in re.findall(r"[A-Z]+(?![a-z])|[A-Z]?[a-z0-9]+", piece)]
    return parts


def live_command_problems(tokens: list[str]) -> list[str]:
    head = tokens[0]
    shown = " ".join(tokens)[:120]
    if head in DATA_SHELLS:
        return [f"data-plane shell '{head}' in '{shown}'; audit control-plane metadata only"]
    if head in NETWORK_TOOLS:
        problems = []
        for i, token in enumerate(tokens):
            if token in {"-d", "--data", "--data-raw", "--data-binary", "--data-urlencode", "-F", "--form", "-T", "--upload-file", "--json"}:
                problems.append(f"curl sends a body ('{token}') in '{shown}'")
            if token in {"-X", "--request"} and i + 1 < len(tokens) and tokens[i + 1].upper() != "GET":
                problems.append(f"curl uses method {tokens[i + 1]} in '{shown}'; GET only")
            if token.startswith("-X") and len(token) > 2 and token[2:].upper() != "GET":
                problems.append(f"curl uses method {token[2:]} in '{shown}'; GET only")
        return problems
    if not is_cli(tokens):
        return []
    problems = []
    parts = words(tokens)
    for part in parts:
        if part in MUTATING_EXACT or part.startswith(MUTATING_PREFIX):
            problems.append(f"mutating word '{part}' in '{shown}'")
        if part in SECRET_WORDS:
            problems.append(f"secret-reading word '{part}' in '{shown}'")
    if any(token in {"--force", "--yes", "-y"} for token in tokens):
        problems.append(f"confirmation-bypass flag in '{shown}'")
    if not any(part.startswith(READ_VERBS) for part in parts):
        problems.append(f"no read verb ({', '.join(READ_VERBS)}) in '{shown}'")
    return problems


def offline_command_problems(tokens: list[str]) -> list[str]:
    head = tokens[0]
    shown = " ".join(tokens)[:120]
    if head in DATA_SHELLS or head in NETWORK_TOOLS or is_cli(tokens):
        return [f"'{head}' in offline audit ('{shown}'); offline means no cloud, CLI or network call"]
    if head in {"terraform", "tofu", "pulumi", "cdk"} and tokens[1:2] != ["show"]:
        return [f"'{shown}' can reach a provider; offline allows only 'terraform show' on a saved plan"]
    return []


def sections(text: str) -> dict[str, str]:
    starts = [m.start() for m in re.finditer(r"^## ", text, re.MULTILINE)] + [len(text)]
    found = {}
    for begin, end in zip(starts, starts[1:]):
        block = text[begin:end]
        found[block.splitlines()[0][3:].strip()] = block
    return found


def check(skill_rel: str, errors: list[str]) -> None:
    skill = REPO / skill_rel
    pattern = skill_rel in PATTERN_SKILLS
    skill_md = (skill / "SKILL.md").read_text(encoding="utf-8")
    router = skill / ROUTER
    if not router.exists():
        errors.append(f"{skill_rel}: missing {ROUTER}")
        return
    if f"]({ROUTER})" not in skill_md:
        errors.append(f"{skill_rel}: SKILL.md does not link {ROUTER}; no agent would load the audit")
    router_text = router.read_text(encoding="utf-8")
    if not re.search(r"\b(only|Do not read both)\b", router_text):
        errors.append(f"{skill_rel}/{ROUTER}: must say to load only the selected variant")
    rows = _ops.ledger_status(skill)
    for mode, rel in VARIANTS.items():
        where = f"{skill_rel}/{rel}"
        path = skill / rel
        if not path.exists():
            errors.append(f"{where}: missing")
            continue
        if f"]({rel.removeprefix('references/commands/')})" not in router_text:
            errors.append(f"{skill_rel}/{ROUTER}: does not link {rel.removeprefix('references/commands/')}")
        if f"]({rel})" in skill_md:
            errors.append(f"{skill_rel}: SKILL.md links {rel} directly; link only the router so variants load lazily")
        text = path.read_text(encoding="utf-8")
        blocks = BLOCK_RE.findall(text)
        for block in blocks:
            for tokens in commands(block):
                problems = live_command_problems(tokens) if mode == "live" else offline_command_problems(tokens)
                errors += [f"{where}: {p}" for p in problems]
        for evidence in _ops.MARKER_RE.findall(text):
            if evidence not in rows:
                errors.append(f"{where}: {evidence} has no row in the skill's evidence ledger")
            elif mode == "live" and rows[evidence] != "VERIFIED":
                errors.append(f"{where}: cites {evidence}, whose ledger status is {rows[evidence]}; live commands need VERIFIED sources")
        if pattern:
            continue
        if not blocks:
            errors.append(f"{where}: needs at least one bash command block")
        headings = [h for h in re.findall(r"^## (.+?)\s*$", text, re.MULTILINE) if h in SECTIONS]
        if headings != SECTIONS:
            errors.append(f"{where}: sections must be exactly {SECTIONS} in order; found {headings}")
            continue
        if mode == "live":
            for title, block in sections(text).items():
                if title not in SECTIONS:
                    continue
                has_command = bool(BLOCK_RE.search(block)) and bool(_ops.MARKER_RE.search(block))
                if not has_command and NO_LIVE not in block:
                    errors.append(f"{where} [{title}]: needs a cited command block or a '{NO_LIVE}' line with the reason")


def main() -> int:
    errors: list[str] = []
    for skill_rel in SKILLS:
        check(skill_rel, errors)
    if errors:
        for error in errors:
            print(f"FAIL: {error}")
        print(f"validate-nosql-audit: {len(errors)} problem(s)")
        return 1
    print(f"validate-nosql-audit: OK ({len(SKILLS)} skills, offline and live audit variants)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
