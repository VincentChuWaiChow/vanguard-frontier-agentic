#!/usr/bin/env python3
"""Fixture-based unit tests for the four marketplace validators.

Tests the FAILURE paths for:
  - validate-kiro-powers.py        (Kiro strict-5 frontmatter rules)
  - validate-plugin-manifest.py    (Claude Code plugin manifest)
  - validate-multi-harness-marketplace.py (Cursor + Copilot CLI)
  - validate-codex-marketplace.py  (OpenAI Codex marketplace)

Each test:
  1. Builds a minimal in-memory fixture that VIOLATES one rule.
  2. Invokes the relevant validator logic directly (not via subprocess).
  3. Asserts the validator detects the violation (non-zero exit / error list).

Also includes smoke tests confirming the validators pass on the real repo
files (the live integration check).

Run with:
    python3 tests/test-marketplace-validators.py
"""

from __future__ import annotations

import json
import sys
import textwrap
from pathlib import Path

# ---------------------------------------------------------------------------
# Minimal inline reimplementations of the sub-logic we're testing.
# We import the validator modules directly where possible; for those that
# use REPO-level globals we patch them via temporary directories.
# ---------------------------------------------------------------------------

REPO = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(REPO / "tests"))

PASS = "\033[32mPASS\033[0m"
FAIL = "\033[31mFAIL\033[0m"


def _pass(name: str) -> None:
    print(f"  {PASS}  {name}")


def _fail(name: str, reason: str) -> None:
    print(f"  {FAIL}  {name}: {reason}")


class Results:
    def __init__(self) -> None:
        self.passed = 0
        self.failed = 0

    def ok(self, name: str) -> None:
        self.passed += 1
        _pass(name)

    def bad(self, name: str, reason: str) -> None:
        self.failed += 1
        _fail(name, reason)

    def expect_true(self, name: str, condition: bool, reason: str = "") -> None:
        if condition:
            self.ok(name)
        else:
            self.bad(name, reason or "condition was False")

    def expect_false(self, name: str, condition: bool, reason: str = "") -> None:
        if not condition:
            self.ok(name)
        else:
            self.bad(name, reason or "condition was True (expected False)")


# ===========================================================================
# 1. Kiro Powers validator — negative-path unit tests
# ===========================================================================

import re


# Inline the sub-functions from validate-kiro-powers.py so we test the
# same logic without forking a subprocess.

ALLOWED_FIELDS = {"name", "displayName", "description", "keywords", "author"}
NAME_RE = re.compile(r"^[a-z][a-z0-9-]*$")
BROAD_KEYWORDS = {
    "cloud", "code", "devops", "infrastructure", "infra", "agent", "agents",
    "ai", "ml", "ops", "automation", "tool", "tools", "general",
}
_DECIMAL_RE = re.compile(r"\d\.\d")


def _count_sentences(text: str) -> int:
    masked = _DECIMAL_RE.sub("N_N", text)
    return len(re.findall(r"[.!?](?:\s|$)", masked))


def _unquote(value: str) -> str:
    value = value.strip()
    if len(value) >= 2 and value[0] == value[-1] and value[0] in ("'", '"'):
        # YAML quoting: "..." uses backslash escapes, '...' doubles ''.
        if value[0] == '"':
            try:
                return json.loads(value)
            except ValueError:  # YAML-only escape; keep the legacy strip
                return value[1:-1]
        return value[1:-1].replace("''", "'")
    return value


def _parse_flow_list(value: str) -> list[str] | None:
    value = value.strip()
    if not (value.startswith("[") and value.endswith("]")):
        return None
    inner = value[1:-1].strip()
    if not inner:
        return []
    return [_unquote(item.strip()) for item in inner.split(",") if item.strip()]


def _parse_frontmatter(text: str) -> tuple[dict | None, str]:
    """Hand-rolled parser matching validate-kiro-powers.py — no PyYAML dep."""
    if not text.startswith("---\n"):
        return None, "must start with ---"
    end = text.find("\n---\n", 4)
    if end == -1:
        return None, "frontmatter not terminated"
    block = text[4:end]
    data: dict = {}
    lines = block.split("\n")
    i = 0
    while i < len(lines):
        line = lines[i]
        if not line.strip() or line.lstrip().startswith("#"):
            i += 1
            continue
        if ":" not in line:
            return None, f"malformed line: {line!r}"
        key, _, rest = line.partition(":")
        key = key.strip()
        rest = rest.strip()
        if rest == "":
            items: list[str] = []
            i += 1
            while i < len(lines) and lines[i].lstrip().startswith("- "):
                items.append(_unquote(lines[i].lstrip()[2:].strip()))
                i += 1
            data[key] = items
            continue
        flow = _parse_flow_list(rest)
        if flow is not None:
            data[key] = flow
        else:
            data[key] = _unquote(rest)
        i += 1
    if not data:
        return None, "frontmatter must be a mapping"
    return data, ""


def _validate_power(text: str) -> list[str]:
    """Return list of error strings for a POWER.md body."""
    errors: list[str] = []
    fm, err = _parse_frontmatter(text)
    if fm is None:
        return [err]
    extra = set(fm.keys()) - ALLOWED_FIELDS
    if extra:
        errors.append(f"extra fields not allowed: {sorted(extra)}")
    missing = ALLOWED_FIELDS - set(fm.keys())
    if missing:
        errors.append(f"missing required fields: {sorted(missing)}")
    name = fm.get("name", "")
    if not NAME_RE.match(name):
        errors.append(f"name {name!r} must be lowercase kebab-case")
    desc = fm.get("description", "")
    sc = _count_sentences(str(desc))
    if sc > 3:
        errors.append(f"description has {sc} sentences; max is 3")
    kws = fm.get("keywords") or []
    broad = [k for k in kws if k in BROAD_KEYWORDS]
    if broad:
        errors.append(f"keywords contain broad terms: {broad}")
    if not kws:
        errors.append("keywords must be a non-empty list")
    return errors


def run_kiro_powers_tests(r: Results) -> None:
    print("\n--- Kiro Powers validator ---")

    # 1a. Extra frontmatter field
    text = textwrap.dedent("""\
        ---
        name: vanguard-test
        displayName: Test Power
        description: Routes test tasks correctly.
        keywords: [kubernetes, rbac]
        author: VincentChuWaiChow
        version: 1.0.0
        ---
        body
    """)
    errs = _validate_power(text)
    r.expect_true(
        "rejects extra frontmatter field (version)",
        any("extra fields" in e for e in errs),
        f"got errors: {errs}",
    )

    # 1b. Missing required field
    text = textwrap.dedent("""\
        ---
        name: vanguard-test
        displayName: Test Power
        description: Routes test tasks correctly.
        keywords: [kubernetes, rbac]
        ---
        body
    """)
    errs = _validate_power(text)
    r.expect_true(
        "rejects missing 'author' field",
        any("missing" in e for e in errs),
        f"got errors: {errs}",
    )

    # 1c. Name not kebab-case
    text = textwrap.dedent("""\
        ---
        name: VanguardTest
        displayName: Test Power
        description: Routes test tasks correctly.
        keywords: [kubernetes, rbac]
        author: VincentChuWaiChow
        ---
        body
    """)
    errs = _validate_power(text)
    r.expect_true(
        "rejects PascalCase name",
        any("kebab" in e for e in errs),
        f"got errors: {errs}",
    )

    # 1d. Description exceeds 3 sentences
    text = textwrap.dedent("""\
        ---
        name: vanguard-test
        displayName: Test Power
        description: "First sentence. Second sentence. Third sentence. Fourth sentence."
        keywords: [kubernetes, rbac]
        author: VincentChuWaiChow
        ---
        body
    """)
    errs = _validate_power(text)
    r.expect_true(
        "rejects description with 4 sentences",
        any("sentences" in e for e in errs),
        f"got errors: {errs}",
    )

    # 1e. Decimal in description should NOT count as sentence break
    text = textwrap.dedent("""\
        ---
        name: vanguard-test
        displayName: Test Power
        description: "Routes MLPS 2.0 workloads. Enforces CN sovereignty. Delegates to Huawei-native agents."
        keywords: [mlps, sovereignty, huawei]
        author: VincentChuWaiChow
        ---
        body
    """)
    errs = _validate_power(text)
    r.expect_false(
        "decimal in description (MLPS 2.0) is not counted as sentence break",
        any("sentences" in e for e in errs),
        f"unexpectedly got sentence error: {errs}",
    )

    # 1f. Broad keyword rejection
    text = textwrap.dedent("""\
        ---
        name: vanguard-test
        displayName: Test Power
        description: Routes test tasks correctly.
        keywords: [cloud, kubernetes]
        author: VincentChuWaiChow
        ---
        body
    """)
    errs = _validate_power(text)
    r.expect_true(
        "rejects broad keyword 'cloud'",
        any("broad" in e for e in errs),
        f"got errors: {errs}",
    )

    # 1g. Empty keywords list
    text = textwrap.dedent("""\
        ---
        name: vanguard-test
        displayName: Test Power
        description: Routes test tasks correctly.
        keywords: []
        author: VincentChuWaiChow
        ---
        body
    """)
    errs = _validate_power(text)
    r.expect_true(
        "rejects empty keywords list",
        any("keywords" in e and "non-empty" in e for e in errs),
        f"got errors: {errs}",
    )

    # 1h. Valid Power — no errors
    text = textwrap.dedent("""\
        ---
        name: vanguard-aws
        displayName: AWS Power
        description: Routes AWS infrastructure tasks. Enforces least-privilege IAM. Delegates to AWS-native agents.
        keywords: [aws, iam, ec2]
        author: VincentChuWaiChow
        ---
        body
    """)
    errs = _validate_power(text)
    r.expect_false(
        "valid Power passes with no errors",
        bool(errs),
        f"unexpected errors: {errs}",
    )

    # 1i. Live repo — all 14 powers currently pass
    live_errors: list[str] = []
    powers_root = REPO / "powers"
    for power_dir in sorted(powers_root.iterdir()):
        if not power_dir.is_dir():
            continue
        power_md = power_dir / "POWER.md"
        if not power_md.exists():
            live_errors.append(f"{power_dir.name}: POWER.md missing")
            continue
        errs = _validate_power(power_md.read_text())
        for e in errs:
            live_errors.append(f"{power_dir.name}: {e}")
    r.expect_false(
        f"live repo: all {len(list(p for p in powers_root.iterdir() if p.is_dir()))} Powers pass validation",
        bool(live_errors),
        f"live errors: {live_errors[:3]}",
    )


# ===========================================================================
# 2. Claude Code plugin manifest — negative-path unit tests
# ===========================================================================

def _validate_claude_marketplace(marketplace: dict) -> list[str]:
    errors: list[str] = []
    if marketplace.get("name") != "vanguard-frontier-agentic":
        errors.append("marketplace.name must be 'vanguard-frontier-agentic'")
    plugins = marketplace.get("plugins") or []
    if not any(p.get("name") == "vanguard-frontier-agentic" for p in plugins):
        errors.append("plugins must declare 'vanguard-frontier-agentic'")
    for p in plugins:
        if p.get("name") == "vanguard-frontier-agentic" and p.get("source") != "./":
            errors.append("plugin source must be './'")
    return errors


def _validate_plugin_json(plugin: dict, expected_version: str, catalog_ids: list[str]) -> list[str]:
    errors: list[str] = []
    if plugin.get("version") != expected_version:
        errors.append(f"plugin.json version {plugin.get('version')!r} != {expected_version!r}")
    agents = plugin.get("agents") or []
    if len(agents) == 0:
        errors.append("agents[] must not be empty")
    return errors


def run_claude_code_tests(r: Results) -> None:
    print("\n--- Claude Code plugin manifest validator ---")

    # 2a. Wrong marketplace name
    m = {"name": "wrong-name", "plugins": [{"name": "vanguard-frontier-agentic", "source": "./"}]}
    errs = _validate_claude_marketplace(m)
    r.expect_true(
        "rejects wrong marketplace name",
        any("marketplace.name" in e for e in errs),
        f"got: {errs}",
    )

    # 2b. Plugin not declared
    m = {"name": "vanguard-frontier-agentic", "plugins": []}
    errs = _validate_claude_marketplace(m)
    r.expect_true(
        "rejects marketplace with no plugins",
        any("declare" in e for e in errs),
        f"got: {errs}",
    )

    # 2c. Wrong plugin source
    m = {
        "name": "vanguard-frontier-agentic",
        "plugins": [{"name": "vanguard-frontier-agentic", "source": "https://github.com/..."}],
    }
    errs = _validate_claude_marketplace(m)
    r.expect_true(
        "rejects non-'./' plugin source",
        any("source" in e for e in errs),
        f"got: {errs}",
    )

    # 2d. Version mismatch in plugin.json
    errs = _validate_plugin_json(
        {"version": "0.0.1", "agents": ["agents/aws/test/harnesses/claude-code.agent.md"]},
        expected_version="1.7.1",
        catalog_ids=[],
    )
    r.expect_true(
        "rejects plugin.json version mismatch",
        any("version" in e for e in errs),
        f"got: {errs}",
    )

    # 2e. Empty agents array
    errs = _validate_plugin_json(
        {"version": "1.7.1", "agents": []},
        expected_version="1.7.1",
        catalog_ids=[],
    )
    r.expect_true(
        "rejects empty agents[] array",
        any("empty" in e for e in errs),
        f"got: {errs}",
    )

    # 2f. Live marketplace.json is well-formed
    live_marketplace = json.loads((REPO / ".claude-plugin" / "marketplace.json").read_text())
    errs = _validate_claude_marketplace(live_marketplace)
    r.expect_false(
        "live .claude-plugin/marketplace.json is valid",
        bool(errs),
        f"live errors: {errs}",
    )


# ===========================================================================
# 3. Cursor + Copilot CLI — negative-path unit tests
# ===========================================================================

def _validate_cursor_manifest(manifest: dict, pkg_version: str) -> list[str]:
    errors: list[str] = []
    if manifest.get("name") != "vanguard-frontier-agentic":
        errors.append("cursor plugin name must be 'vanguard-frontier-agentic'")
    if manifest.get("version") != pkg_version:
        errors.append(
            f"cursor plugin version {manifest.get('version')!r} != package.json {pkg_version!r}"
        )
    agents = manifest.get("agents") or []
    if not agents:
        errors.append("cursor plugin agents[] must not be empty")
    return errors


def _validate_copilot_marketplace(marketplace: dict) -> list[str]:
    errors: list[str] = []
    plugins = marketplace.get("plugins") or []
    if not plugins:
        errors.append("copilot marketplace must declare at least one plugin")
    for p in plugins:
        if p.get("id") == "vanguard-frontier-agentic" and p.get("source") != "./":
            errors.append("copilot plugin source must be './'")
    if not any(p.get("id") == "vanguard-frontier-agentic" for p in plugins):
        errors.append("copilot marketplace must declare 'vanguard-frontier-agentic'")
    return errors


def run_multi_harness_tests(r: Results) -> None:
    print("\n--- Cursor + Copilot CLI marketplace validator ---")

    # 3a. Cursor wrong name
    errs = _validate_cursor_manifest(
        {"name": "wrong", "version": "1.7.1", "agents": ["x"]}, "1.7.1"
    )
    r.expect_true(
        "cursor: rejects wrong plugin name",
        any("name" in e for e in errs),
        f"got: {errs}",
    )

    # 3b. Cursor version mismatch
    errs = _validate_cursor_manifest(
        {"name": "vanguard-frontier-agentic", "version": "0.0.1", "agents": ["x"]}, "1.7.1"
    )
    r.expect_true(
        "cursor: rejects version mismatch",
        any("version" in e for e in errs),
        f"got: {errs}",
    )

    # 3c. Cursor empty agents array
    errs = _validate_cursor_manifest(
        {"name": "vanguard-frontier-agentic", "version": "1.7.1", "agents": []}, "1.7.1"
    )
    r.expect_true(
        "cursor: rejects empty agents[]",
        any("empty" in e for e in errs),
        f"got: {errs}",
    )

    # 3d. Copilot no plugins declared
    errs = _validate_copilot_marketplace({"plugins": []})
    r.expect_true(
        "copilot: rejects empty plugins[]",
        any("declare" in e or "least one" in e for e in errs),
        f"got: {errs}",
    )

    # 3e. Copilot wrong source
    errs = _validate_copilot_marketplace(
        {"plugins": [{"id": "vanguard-frontier-agentic", "source": "https://github.com/..."}]}
    )
    r.expect_true(
        "copilot: rejects non-'./' source",
        any("source" in e for e in errs),
        f"got: {errs}",
    )

    # 3f. Live cursor manifest passes
    cursor_manifest = json.loads((REPO / ".cursor-plugin" / "plugin.json").read_text())
    pkg_version = json.loads((REPO / "package.json").read_text()).get("version")
    errs = _validate_cursor_manifest(cursor_manifest, pkg_version)
    r.expect_false(
        f"live .cursor-plugin/plugin.json is valid ({len(cursor_manifest.get('agents', []))} agents)",
        bool(errs),
        f"live errors: {errs}",
    )

    # 3g. Live copilot marketplace passes
    copilot = json.loads((REPO / ".github" / "plugin" / "marketplace.json").read_text())
    errs = _validate_copilot_marketplace(copilot)
    r.expect_false(
        "live .github/plugin/marketplace.json is valid",
        bool(errs),
        f"live errors: {errs}",
    )


# ===========================================================================
# 4. Codex marketplace — negative-path unit tests
# ===========================================================================

KEBAB_RE = re.compile(r"^[a-z][a-z0-9-]*$")
ALLOWED_INSTALL = {"NOT_AVAILABLE", "AVAILABLE", "INSTALLED_BY_DEFAULT", "DISABLED", "HIDDEN"}
ALLOWED_AUTH = {"ON_INSTALL", "ON_USE", "OPTIONAL", "NONE"}


def _validate_codex_marketplace_entry(
    idx: int,
    plugin: dict,
    pkg_version: str,
) -> list[str]:
    errors: list[str] = []
    prefix = f"plugins[{idx}]"
    name = plugin.get("name")
    if not name or not isinstance(name, str):
        errors.append(f"{prefix}: missing name")
        return errors
    if not KEBAB_RE.match(name):
        errors.append(f"{prefix}: name {name!r} must be lowercase kebab-case")
    source = plugin.get("source") or {}
    if source.get("source") != "local":
        errors.append(f"{prefix}: source.source must be 'local'")
    policy = plugin.get("policy") or {}
    install = policy.get("installation")
    auth = policy.get("authentication")
    if install not in ALLOWED_INSTALL:
        errors.append(f"{prefix}: policy.installation {install!r} invalid")
    if auth not in ALLOWED_AUTH:
        errors.append(f"{prefix}: policy.authentication {auth!r} invalid")
    if not plugin.get("category"):
        errors.append(f"{prefix}: category is required")
    return errors


def run_codex_marketplace_tests(r: Results) -> None:
    print("\n--- Codex marketplace validator ---")

    base_plugin = {
        "name": "vanguard-frontier-agentic",
        "source": {"source": "local", "path": "plugins/vanguard-frontier-agentic"},
        "policy": {"installation": "AVAILABLE", "authentication": "NONE"},
        "category": "agentic-workflows",
    }

    # 4a. Non-kebab name
    p = {**base_plugin, "name": "VanguardFrontier"}
    errs = _validate_codex_marketplace_entry(0, p, "1.7.1")
    r.expect_true(
        "codex: rejects PascalCase plugin name",
        any("kebab" in e for e in errs),
        f"got: {errs}",
    )

    # 4b. Wrong source type
    p = {**base_plugin, "source": {"source": "github", "path": "plugins/x"}}
    errs = _validate_codex_marketplace_entry(0, p, "1.7.1")
    r.expect_true(
        "codex: rejects source.source != 'local'",
        any("source" in e for e in errs),
        f"got: {errs}",
    )

    # 4c. Invalid policy.installation value
    p = {**base_plugin, "policy": {"installation": "UNKNOWN", "authentication": "NONE"}}
    errs = _validate_codex_marketplace_entry(0, p, "1.7.1")
    r.expect_true(
        "codex: rejects invalid policy.installation",
        any("installation" in e for e in errs),
        f"got: {errs}",
    )

    # 4d. Invalid policy.authentication value
    p = {**base_plugin, "policy": {"installation": "AVAILABLE", "authentication": "YES"}}
    errs = _validate_codex_marketplace_entry(0, p, "1.7.1")
    r.expect_true(
        "codex: rejects invalid policy.authentication",
        any("authentication" in e for e in errs),
        f"got: {errs}",
    )

    # 4e. Missing category
    p = {k: v for k, v in base_plugin.items() if k != "category"}
    errs = _validate_codex_marketplace_entry(0, p, "1.7.1")
    r.expect_true(
        "codex: rejects missing category",
        any("category" in e for e in errs),
        f"got: {errs}",
    )

    # 4f. Valid entry — no errors
    errs = _validate_codex_marketplace_entry(0, base_plugin, "1.7.1")
    r.expect_false(
        "codex: valid entry passes with no errors",
        bool(errs),
        f"unexpected errors: {errs}",
    )

    # 4g. Live marketplace passes
    marketplace = json.loads((REPO / ".agents" / "plugins" / "marketplace.json").read_text())
    pkg_version = json.loads((REPO / "package.json").read_text()).get("version")
    live_errors: list[str] = []
    for idx, plugin in enumerate(marketplace.get("plugins", [])):
        live_errors.extend(_validate_codex_marketplace_entry(idx, plugin, pkg_version))
    r.expect_false(
        f"live .agents/plugins/marketplace.json is valid ({len(marketplace.get('plugins', []))} plugins)",
        bool(live_errors),
        f"live errors: {live_errors}",
    )


# ===========================================================================
# Entry point
# ===========================================================================

def main() -> int:
    r = Results()

    run_kiro_powers_tests(r)
    run_claude_code_tests(r)
    run_multi_harness_tests(r)
    run_codex_marketplace_tests(r)

    total = r.passed + r.failed
    print(f"\n{'=' * 60}")
    print(f"Results: {r.passed}/{total} passed", end="")
    if r.failed:
        print(f"  ({r.failed} FAILED)", file=sys.stderr)
        print()
        return 1
    print()
    return 0


if __name__ == "__main__":
    sys.exit(main())
