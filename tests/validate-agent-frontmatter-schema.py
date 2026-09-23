#!/usr/bin/env python3
"""Validate AGENT.md frontmatter against schemas/agent.frontmatter.schema.json.

Mirrors tests/validate-skill-frontmatter-schema.py. Walks every
agents/<provider>/<name>-agent/AGENT.md, parses the YAML frontmatter with a
stdlib-only restricted parser, and validates against the schema using
jsonschema if available, else a hand-rolled fallback.

Required fields (empirical from current 141-file corpus):
  - metadata.author (non-empty string)
  - metadata.version (semver)

Optional but typed when present: name, description, model, allowed-tools,
tools, color. additionalProperties is permitted so harness-specific fields
do not break validation.

Also checks the markdown harness exports (harnesses/<harness>.agent.md). AGENT.md
carries only `metadata`, so these exports are where an agent gets its identity,
and every one must declare a non-empty `name` and `description`. Without a name,
Claude Code names a plugin agent after its file, and every export here is called
claude-code.agent.md, so nameless agents collapse into one and the rest are
dropped as duplicates. For the same reason claude-code names must be unique.
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
AGENTS_DIR = ROOT / "agents"
SCHEMA_PATH = ROOT / "schemas" / "agent.frontmatter.schema.json"

# ──────────────────────────────────────────────────────────────────────────────
# YAML frontmatter parser (stdlib only, mirrors validate-skill-frontmatter-schema.py)
# ──────────────────────────────────────────────────────────────────────────────

def parse_frontmatter_raw(text: str) -> dict | None:
    if not text.startswith("---\n"):
        return None
    end = text.find("\n---", 4)
    if end == -1:
        return None
    block = text[4:end]
    return _minimal_yaml_parse(block)


def _minimal_yaml_parse(block: str) -> dict:
    """Parse a restricted YAML subset into a Python dict."""
    result: dict = {}
    lines = block.splitlines()
    i = 0

    def parse_scalar(s: str):
        s = s.strip()
        if (s.startswith('"') and s.endswith('"')) or \
           (s.startswith("'") and s.endswith("'")):
            return s[1:-1]
        if s.lower() == "true":
            return True
        if s.lower() == "false":
            return False
        return s

    while i < len(lines):
        line = lines[i]
        if line and not line.startswith(" ") and ":" in line:
            key, _, rest = line.partition(":")
            key = key.strip()
            rest = rest.strip()

            if rest == "":
                children_list: list = []
                children_dict: dict = {}
                i += 1
                while i < len(lines) and (lines[i].startswith("  ") or lines[i] == ""):
                    child = lines[i]
                    if child.strip() == "":
                        i += 1
                        continue
                    if child.startswith("  - "):
                        children_list.append(child[4:].strip())
                    elif child.startswith("  ") and ":" in child:
                        ckey, _, crest = child.strip().partition(":")
                        children_dict[ckey.strip()] = parse_scalar(crest)
                    i += 1
                if children_list:
                    result[key] = children_list
                elif children_dict:
                    result[key] = children_dict
                else:
                    result[key] = None
                continue
            elif rest.startswith("[") and rest.endswith("]"):
                inner = rest[1:-1]
                result[key] = [t.strip().strip("'\"") for t in inner.split(",") if t.strip()]
            else:
                result[key] = parse_scalar(rest)
        i += 1

    return result


# ──────────────────────────────────────────────────────────────────────────────
# Schema-based validation
# ──────────────────────────────────────────────────────────────────────────────

NAME_RE = re.compile(r"^[a-z0-9][a-z0-9-]*$")
VERSION_RE = re.compile(r"^\d+\.\d+\.\d+(-[\w.-]+)?$")


def _try_jsonschema_validate(instance: dict, schema: dict) -> list[str]:
    try:
        import jsonschema  # type: ignore
        errors = []
        validator_cls = jsonschema.validators.validator_for(schema)
        validator = validator_cls(schema)
        for err in sorted(validator.iter_errors(instance), key=lambda e: list(e.path)):
            path = ".".join(str(p) for p in err.absolute_path) or "<root>"
            errors.append(f"  field '{path}': {err.message}")
        return errors
    except ImportError:
        return []


def _hand_rolled_validate(fm: dict) -> list[str]:
    errors: list[str] = []

    meta = fm.get("metadata")
    if meta is None:
        errors.append("  field 'metadata': required field is missing")
    elif not isinstance(meta, dict):
        errors.append(
            f"  field 'metadata': must be a mapping/object, got {type(meta).__name__}"
        )
    else:
        author = meta.get("author")
        if author is None:
            errors.append("  field 'metadata.author': required field is missing")
        elif not isinstance(author, str) or len(author) < 1:
            errors.append("  field 'metadata.author': must be a non-empty string")

        version = meta.get("version")
        if version is None:
            errors.append("  field 'metadata.version': required field is missing")
        elif not isinstance(version, str):
            errors.append(
                f"  field 'metadata.version': must be a string, got {type(version).__name__}"
            )
        elif not VERSION_RE.match(version):
            errors.append(
                f"  field 'metadata.version': '{version}' does not match semver pattern"
            )

    # Optional name pattern
    name = fm.get("name")
    if name is not None:
        if not isinstance(name, str):
            errors.append(f"  field 'name': must be a string, got {type(name).__name__}")
        elif not NAME_RE.match(name):
            errors.append(
                f"  field 'name': '{name}' does not match pattern ^[a-z0-9][a-z0-9-]*$"
            )

    # Optional description bounds
    desc = fm.get("description")
    if desc is not None:
        if not isinstance(desc, str):
            errors.append(
                f"  field 'description': must be a string, got {type(desc).__name__}"
            )
        else:
            if len(desc) < 20:
                errors.append(
                    f"  field 'description': length {len(desc)} is below minimum 20"
                )
            if len(desc) > 4000:
                errors.append(
                    f"  field 'description': length {len(desc)} exceeds maximum 4000"
                )

    # allowed-tools / tools shapes
    for key in ("allowed-tools", "tools"):
        v = fm.get(key)
        if v is None:
            continue
        if isinstance(v, str):
            if len(v.strip()) == 0:
                errors.append(f"  field '{key}': string value must not be empty")
        elif isinstance(v, list):
            if len(v) == 0:
                errors.append(f"  field '{key}': list must contain at least one item")
            for idx, item in enumerate(v):
                if not isinstance(item, str) or len(item.strip()) == 0:
                    errors.append(
                        f"  field '{key}[{idx}]': each item must be a non-empty string"
                    )
        else:
            errors.append(
                f"  field '{key}': must be a string or array, got {type(v).__name__}"
            )

    return errors


def validate_agent(agent_md: Path, schema: dict) -> list[str]:
    text = agent_md.read_text(encoding="utf-8")
    fm = parse_frontmatter_raw(text)
    if fm is None or not isinstance(fm, dict):
        return ["  no valid YAML frontmatter block found"]

    errors = _try_jsonschema_validate(fm, schema)
    if not errors:
        # double-check with hand-rolled to catch anything jsonschema might miss
        errors = _hand_rolled_validate(fm)
    return errors


# The markdown exports each harness loads as an agent definition. codex.toml and
# kiro-cli.agent.json carry identity in their own formats and are not covered.
HARNESS_MARKDOWN = ("claude-code", "cursor", "copilot", "gemini", "kiro-ide")

# Gemini CLI validates local agent frontmatter with a strict schema
# (localAgentSchema in packages/core/src/agents/agentLoader.ts): `name` must
# match /^[a-z0-9-_]+$/, unknown keys are rejected, and a file that fails is
# skipped rather than loaded. The agent id satisfies the slug rule and is unique,
# so it is what every Gemini export uses; the readable name is `display_name`.
GEMINI_ALLOWED_KEYS = {
    "kind", "name", "description", "display_name", "tools", "mcp_servers",
    "model", "temperature", "max_turns", "timeout_mins",
}


def validate_harness_identity() -> tuple[list[str], int]:
    """Every markdown harness export names and describes its agent, and no two
    claude-code exports share a name. Returns (errors, files checked)."""
    errors: list[str] = []
    checked = 0
    claude_names: dict[str, Path] = {}
    for harness in HARNESS_MARKDOWN:
        for path in sorted(AGENTS_DIR.glob(f"*/*/harnesses/{harness}.agent.md")):
            checked += 1
            rel = path.relative_to(ROOT)
            fm = parse_frontmatter_raw(path.read_text(encoding="utf-8"))
            missing = [
                key for key in ("name", "description")
                if not (isinstance(fm, dict) and isinstance(fm.get(key), str) and fm[key].strip())
            ]
            if missing:
                errors.append(f"{rel}: missing {' and '.join(missing)} in frontmatter")
                continue
            if harness == "gemini":
                agent_id = path.parent.parent.name
                if fm["name"] != agent_id:
                    errors.append(
                        f"{rel}: name {fm['name']!r} must be the agent id {agent_id!r}; "
                        "Gemini CLI skips agents whose name is not a slug"
                    )
                unknown = sorted(set(fm) - GEMINI_ALLOWED_KEYS)
                if unknown:
                    errors.append(
                        f"{rel}: frontmatter keys {unknown} are not in Gemini CLI's "
                        "local agent schema, which rejects unknown keys"
                    )
                if fm.get("kind", "local") != "local":
                    errors.append(f"{rel}: kind {fm['kind']!r} is not 'local'")
            if harness == "claude-code":
                prior = claude_names.setdefault(fm["name"], path)
                if prior != path:
                    errors.append(
                        f"{rel}: name {fm['name']!r} is already used by "
                        f"{prior.relative_to(ROOT)}; Claude Code would drop one"
                    )
    return errors, checked


# ──────────────────────────────────────────────────────────────────────────────
# Entry point
# ──────────────────────────────────────────────────────────────────────────────

def main() -> int:
    if not SCHEMA_PATH.exists():
        print(f"ERROR: schema not found at {SCHEMA_PATH}", file=sys.stderr)
        return 2

    schema = json.loads(SCHEMA_PATH.read_text(encoding="utf-8"))

    agent_files = sorted(AGENTS_DIR.glob("*/*/AGENT.md"))
    if not agent_files:
        print("ERROR: no AGENT.md files found", file=sys.stderr)
        return 2

    failed: list[tuple[Path, list[str]]] = []

    for agent_md in agent_files:
        errors = validate_agent(agent_md, schema)
        if errors:
            failed.append((agent_md, errors))

    harness_errors, harness_checked = validate_harness_identity()

    if failed:
        print(
            f"FAIL: {len(failed)} agent(s) failed AGENT.md frontmatter schema validation "
            f"(out of {len(agent_files)} checked):",
            file=sys.stderr,
        )
        for path, errs in failed:
            print(f"\n  {path}", file=sys.stderr)
            for e in errs:
                print(f"    {e}", file=sys.stderr)
    if harness_errors:
        print(
            f"FAIL: {len(harness_errors)} harness export(s) lack a usable agent "
            f"identity (out of {harness_checked} checked):",
            file=sys.stderr,
        )
        for e in harness_errors:
            print(f"  {e}", file=sys.stderr)
    if failed or harness_errors:
        return 1

    print(
        f"OK: all {len(agent_files)} agents passed AGENT.md frontmatter schema validation"
    )
    print(
        f"OK: {harness_checked} markdown harness exports declare name and description "
        f"(claude-code names unique; Gemini names are agent ids)"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
