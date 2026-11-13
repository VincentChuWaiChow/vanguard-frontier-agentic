#!/usr/bin/env python3
"""Validate marketplace catalogs and asset metadata without external dependencies."""

from __future__ import annotations

import json
import re
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

CATALOGS = {
    "skill": ROOT / "catalog" / "skills.json",
    "agent": ROOT / "catalog" / "agents.json",
    "rule": ROOT / "catalog" / "rules.json",
    "mcp-reference": ROOT / "catalog" / "mcp-references.json",
}

ALLOWED_PROVIDERS = {
    "aws",
    "azure",
    "oracle",
    "oci",
    "gcp",
    "kubernetes",
    "terraform",
    "multi-cloud",
    "generic",
    "kyverno",
    "istio",
    "argocd",
    "cilium",
    "opentelemetry",
    "prometheus",
    "falco",
    "sigstore",
    "cert-manager",
    "fluxcd",
    "backstage",
    "velero",
    "alibaba",
    "huawei",
    "ovhcloud",
    "ionos",
    "scaleway",
    "hetzner",
    "contabo",
    "nvidia",
    "claude",
    "frontend",
    "marketing",
    "dotnet",
    "java",
    "kotlin",
    "hr",
    "legal",
    "salesforce",
    "netsuite",
    "atlassian",
    "accounting",
    "finance",
    "sap",
    "microsoft",
    "databricks",
    "snowflake",
    "php",
    "python",
    "typescript",
}
ALLOWED_HARNESSES = {"codex", "copilot", "claude-code", "cursor", "gemini", "kiro", "other"}
ALLOWED_SOURCE_TYPES = {"original", "adapted", "reference-only"}
REQUIRED_COMMON = {
    "id",
    "name",
    "type",
    "provider",
    "harnesses",
    "summary",
    "source_type",
    "official_docs",
    "security_notes",
    "last_verified",
    "path",
}
REQUIRED_MCP = {"official_project_url", "vendor", "auth_model", "install_example", "unofficial_warning"}
ID_RE = re.compile(r"^[a-z0-9][a-z0-9-]*$")
DATE_RE = re.compile(r"^\d{4}-\d{2}-\d{2}$")
SEMVER_RE = re.compile(r"^\d+\.\d+\.\d+$")
URL_RE = re.compile(r"^https?://")
# Credential value that is entirely a bracket-delimited placeholder: <anything-here>.
# Only values that fully match <...> are excluded; <tag>realvalue is still caught.
_PLACEHOLDER_RE = re.compile(r"^<[^>]{1,60}>$")
_CREDENTIAL_VALUE_RE = re.compile(
    r"(?i)(api[_-]?key|secret|token|password)\s*[:=]\s*['\"]([^'\"]{12,})['\"]"
)


def _has_credential_match(text: str) -> bool:
    for m in _CREDENTIAL_VALUE_RE.finditer(text):
        if not _PLACEHOLDER_RE.match(m.group(2)):
            return True
    return False


SECRET_PATTERNS = [
    re.compile(r"AKIA[0-9A-Z]{16}"),
    re.compile(r"ASIA[0-9A-Z]{16}"),
    re.compile(r"-----BEGIN (RSA |EC |OPENSSH |DSA )?PRIVATE KEY-----"),
]


def load_json(path: Path):
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:
        raise AssertionError(f"{path}: invalid JSON: {exc}") from exc


def assert_true(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def validate_item(item: dict, expected_type: str) -> None:
    missing = REQUIRED_COMMON - item.keys()
    if expected_type in {"skill", "agent"}:
        missing |= {"version"} - item.keys()
    if expected_type == "mcp-reference":
        missing |= REQUIRED_MCP - item.keys()
    assert_true(not missing, f"{item.get('id', '<unknown>')}: missing fields {sorted(missing)}")
    assert_true(item["type"] == expected_type, f"{item['id']}: expected type {expected_type}, got {item['type']}")
    assert_true(ID_RE.match(item["id"]) is not None, f"{item['id']}: invalid id format")
    if expected_type in {"skill", "agent"}:
        assert_true(SEMVER_RE.match(item["version"]) is not None, f"{item['id']}: invalid version {item['version']}")
    assert_true(item["provider"] in ALLOWED_PROVIDERS, f"{item['id']}: invalid provider {item['provider']}")
    assert_true(item["source_type"] in ALLOWED_SOURCE_TYPES, f"{item['id']}: invalid source_type {item['source_type']}")
    assert_true(DATE_RE.match(item["last_verified"]) is not None, f"{item['id']}: invalid last_verified")
    assert_true(isinstance(item["harnesses"], list) and item["harnesses"], f"{item['id']}: harnesses must be non-empty")
    bad_harnesses = set(item["harnesses"]) - ALLOWED_HARNESSES
    assert_true(not bad_harnesses, f"{item['id']}: invalid harnesses {sorted(bad_harnesses)}")
    assert_true(isinstance(item["official_docs"], list) and item["official_docs"], f"{item['id']}: official_docs must be non-empty")
    for url in item["official_docs"]:
        assert_true(URL_RE.match(url) is not None, f"{item['id']}: official_doc is not URL: {url}")
    assert_true(len(item["summary"]) >= 20, f"{item['id']}: summary too short")
    assert_true(len(item["security_notes"]) >= 20, f"{item['id']}: security_notes too short")
    target = ROOT / item["path"]
    assert_true(target.exists(), f"{item['id']}: path does not exist: {item['path']}")


def validate_metadata_file(item: dict) -> None:
    target = ROOT / item["path"]
    if target.is_dir():
        metadata_files = list(target.glob("metadata.json"))
    else:
        metadata_files = list(target.parent.glob(f"{target.stem}.metadata.json"))
    assert_true(metadata_files, f"{item['id']}: no metadata file beside asset")
    metadata = load_json(metadata_files[0])
    assert_true(metadata["id"] == item["id"], f"{item['id']}: metadata id mismatch in {metadata_files[0]}")
    if item["type"] in {"skill", "agent"}:
        assert_true(metadata.get("version") == item["version"], f"{item['id']}: metadata version mismatch in {metadata_files[0]}")


def validate_no_obvious_secrets() -> None:
    checked_suffixes = {".md", ".json", ".py", ".toml", ".yaml", ".yml"}
    # CHANGELOG.md is auto-generated by semantic-release from commit
    # messages and PR bodies; it legitimately quotes example credential
    # shapes from security-fix commit narratives which would otherwise
    # trip the scanner against the project's own historical documentation.
    # Skip it at the path level. Real secret detection runs on source,
    # not on changelog prose.
    skip_paths = {"CHANGELOG.md"}
    for path in ROOT.rglob("*"):
        # Skip .git internals and .claude/worktrees/** (subagent git-worktree checkouts
        # that re-expose CHANGELOG.md history, triggering credential-shape false positives).
        # The consecutive-pair guard prevents accidentally skipping unrelated folders named
        # "worktrees" (e.g. docs/worktrees/**). .git/worktrees/** is already covered by the
        # ".git" check.
        _parts = path.parts
        _in_claude_worktrees = any(
            _parts[i] == ".claude" and i + 1 < len(_parts) and _parts[i + 1] == "worktrees"
            for i in range(len(_parts) - 1)
        )
        if ".git" in _parts or "node_modules" in _parts or _in_claude_worktrees or path.is_dir() or path.suffix not in checked_suffixes:
            continue
        if path.relative_to(ROOT).as_posix() in skip_paths:
            continue
        text = path.read_text(encoding="utf-8", errors="ignore")
        for pattern in SECRET_PATTERNS:
            assert_true(pattern.search(text) is None, f"possible secret pattern in {path.relative_to(ROOT)}")
        assert_true(not _has_credential_match(text), f"possible secret pattern in {path.relative_to(ROOT)}")


def validate_codex_harness_adapters() -> None:
    required_fields = {"name", "description", "developer_instructions"}
    top_level_patterns = (
        "name",
        "description",
        "model",
        "model_reasoning_effort",
        "sandbox_mode",
        "developer_instructions",
        "[[skills.config]]",
        "[metadata]",
    )
    for path in ROOT.glob("agents/**/harnesses/codex.toml"):
        raw = path.read_text(encoding="utf-8")
        try:
            parsed = tomllib.loads(raw)
        except Exception as exc:  # noqa: BLE001 - deterministic parse failure
            raise AssertionError(f"{path.relative_to(ROOT)}: invalid TOML: {exc}") from exc
        missing = sorted(required_fields - parsed.keys())
        assert_true(not missing, f"{path.relative_to(ROOT)}: missing required fields {missing}")
        for lineno, line in enumerate(raw.splitlines(), start=1):
            for token in top_level_patterns:
                if line.startswith(f"    {token}"):
                    raise AssertionError(
                        f"{path.relative_to(ROOT)}:{lineno}: top-level codex TOML entries must not be indented"
                    )


def validate_markdown_agent_templates() -> None:
    markdown_paths = sorted(ROOT.glob("agents/**/AGENT.md"))
    markdown_paths.extend(sorted(ROOT.glob("agents/**/harnesses/*.agent.md")))
    for path in markdown_paths:
        raw = path.read_text(encoding="utf-8")
        lines = raw.splitlines()
        if not lines:
            raise AssertionError(f"{path.relative_to(ROOT)}: empty markdown agent template")

        index = 0
        if lines[0].strip() == "---":
            index = 1
            while index < len(lines) and lines[index].strip() != "---":
                index += 1
            assert_true(index < len(lines), f"{path.relative_to(ROOT)}: unterminated frontmatter")
            index += 1

        in_fence = False
        for lineno, line in enumerate(lines[index:], start=index + 1):
            stripped = line.strip()
            if stripped.startswith("```"):
                in_fence = not in_fence
                continue
            if in_fence or not stripped:
                continue
            assert_true(
                not line.startswith("    "),
                f"{path.relative_to(ROOT)}:{lineno}: markdown agent template content must not start with four-space indentation",
            )


def validate_guarded_live_aws_agents() -> None:
    expected_ids = {
        "aws-live-deployment-guarded-operator-agent",
        "aws-live-ecs-rollout-guard-agent",
        "aws-live-iac-change-guard-agent",
        "aws-live-pipeline-approval-operator-agent",
        "aws-live-serverless-release-guard-agent",
    }
    required_terms = (
        "explicit human approval",
        "account, region",
        "rollback",
        "target confirmation",
    )
    for agent_id in expected_ids:
        codex_path = ROOT / "agents" / "aws" / agent_id / "harnesses" / "codex.toml"
        agent_path = ROOT / "agents" / "aws" / agent_id / "AGENT.md"
        if not codex_path.exists() or not agent_path.exists():
            continue
        codex_raw = codex_path.read_text(encoding="utf-8")
        parsed = tomllib.loads(codex_raw)
        assert_true(
            parsed.get("sandbox_mode") == "workspace-write",
            f"{codex_path.relative_to(ROOT)}: guarded live AWS codex adapter must use workspace-write",
        )
        for term in ("explicit human approval", "rollback", "account, region", "preview, dry-run"):
            assert_true(
                term in codex_raw,
                f"{codex_path.relative_to(ROOT)}: missing guarded live term {term!r}",
            )
        agent_text = agent_path.read_text(encoding="utf-8").lower()
        for term in required_terms:
            assert_true(
                term.lower() in agent_text,
                f"{agent_path.relative_to(ROOT)}: missing guarded live contract term {term!r}",
            )


def validate_guarded_live_kubernetes_agents() -> None:
    expected_ids = {
        "kubernetes-live-rbac-mutation-guard-agent",
        "kubernetes-live-admission-policy-guard-agent",
        "kubernetes-live-mesh-policy-guard-agent",
        "kubernetes-live-network-policy-guard-agent",
        "kubernetes-live-argocd-sync-guard-agent",
        "kubernetes-live-velero-restore-guard-agent",
    }
    required_codex_terms = (
        "workspace-write",
        "explicit platform-team sign-off",
        "rollback",
        "cluster context",
        "current state",
    )
    required_agent_terms = (
        "explicit platform-team sign-off",
        "rollback",
        "cluster context",
        "current state",
    )
    for agent_id in expected_ids:
        codex_path = ROOT / "agents" / "kubernetes" / agent_id / "harnesses" / "codex.toml"
        agent_path = ROOT / "agents" / "kubernetes" / agent_id / "AGENT.md"
        assert_true(
            codex_path.exists(),
            f"guarded live Kubernetes agent '{agent_id}' is missing harnesses/codex.toml",
        )
        assert_true(
            agent_path.exists(),
            f"guarded live Kubernetes agent '{agent_id}' is missing AGENT.md",
        )
        if not codex_path.exists() or not agent_path.exists():
            continue
        codex_raw = codex_path.read_text(encoding="utf-8")
        parsed = tomllib.loads(codex_raw)
        assert_true(
            parsed.get("sandbox_mode") == "workspace-write",
            f"{codex_path.relative_to(ROOT)}: guarded live Kubernetes codex adapter must use workspace-write",
        )
        for term in required_codex_terms:
            assert_true(
                term in codex_raw,
                f"{codex_path.relative_to(ROOT)}: missing guarded live term {term!r}",
            )
        agent_text = agent_path.read_text(encoding="utf-8").lower()
        for term in required_agent_terms:
            assert_true(
                term.lower() in agent_text,
                f"{agent_path.relative_to(ROOT)}: missing guarded live contract term {term!r}",
            )


# Adapter files that prove a harness actually ships, mapped to the `harnesses` value
# that must then be declared.
HARNESS_ADAPTER_FILES = {
    "codex.toml": "codex",
    "copilot.agent.md": "copilot",
    "claude-code.agent.md": "claude-code",
    "cursor.agent.md": "cursor",
    "gemini.agent.md": "gemini",
    "kiro-ide.agent.md": "kiro",
    "kiro-cli.agent.json": "kiro",
}

# Kiro is the one harness whose single `harnesses` value covers TWO adapter files, and the
# exporter emits both. Collapsing them into one set member would let a half-shipped Kiro
# adapter pass: with only kiro-ide.agent.md present, `kiro` is still in the on-disk set, the
# declaration matches, and the export later fails on the missing kiro-cli.agent.json. So the
# pair is checked explicitly rather than through the collapsed mapping.
KIRO_REQUIRED_FILES = ("kiro-ide.agent.md", "kiro-cli.agent.json")


def validate_harness_declarations() -> None:
    """Every agent's `harnesses` must match the adapters it ships AND the catalog entry.

    Three independent failures, each of which has actually happened or was one edit away:

      * UNDER-declared (adapter on disk, harness absent from `harnesses`) makes a shipped
        adapter invisible to everything that filters on the catalog. ionos, ovhcloud, and
        scaleway each shipped all seven adapter files while declaring only codex and
        claude-code, so generate-cursor-plugin.mjs omitted 18 real cursor adapters and
        generate-kiro-powers.mjs dropped three real Kiro Powers.

      * OVER-declared (harness claimed, no adapter file) promises an install path that does
        not exist, failing at the user's `vfa-export-agents` rather than here.

      * CATALOG DRIFT (metadata.json and catalog/agents.json disagree). catalog/agents.json
        has no full regenerator — only an upsert — so fixing metadata.json does NOT update
        it, and the catalog is what every generator actually reads. Checking metadata
        against disk alone would report success while the original user-facing breakage
        remained. This is not hypothetical: resyncing those 18 catalog entries was a
        separate manual step from fixing their metadata.
    """
    problems: list[str] = []

    catalog_harnesses: dict[str, list] = {}
    catalog_agents = load_json(ROOT / "catalog" / "agents.json")
    if isinstance(catalog_agents, dict):
        catalog_agents = catalog_agents.get("agents", [])
    for entry in catalog_agents:
        if isinstance(entry, dict) and entry.get("type") == "agent":
            catalog_harnesses[entry["id"]] = entry.get("harnesses") or []

    for metadata_path in sorted(ROOT.glob("agents/*/*/metadata.json")):
        data = load_json(metadata_path)
        agent_id = data.get("id", metadata_path.parent.name)
        declared = set(data.get("harnesses") or [])
        harness_dir = metadata_path.parent / "harnesses"
        present_files = (
            {entry.name for entry in harness_dir.iterdir() if entry.is_file()}
            if harness_dir.is_dir()
            else set()
        )
        on_disk = {
            HARNESS_ADAPTER_FILES[name]
            for name in present_files
            if name in HARNESS_ADAPTER_FILES
        }
        # "other" is a catalog-only escape hatch with no adapter file of its own.
        declared.discard("other")

        if on_disk - declared:
            problems.append(
                f"{agent_id}: ships adapters for {sorted(on_disk - declared)} "
                f"but does not declare them in harnesses"
            )
        if declared - on_disk:
            problems.append(
                f"{agent_id}: declares harnesses {sorted(declared - on_disk)} "
                f"with no matching adapter file"
            )

        # Kiro's two adapter files are checked as a pair, not through the collapsed value.
        kiro_missing = [f for f in KIRO_REQUIRED_FILES if f not in present_files]
        if "kiro" in declared and kiro_missing:
            problems.append(
                f"{agent_id}: declares the kiro harness but is missing {sorted(kiro_missing)} "
                f"— both Kiro adapter files are required"
            )
        if kiro_missing and len(kiro_missing) == 1:
            problems.append(
                f"{agent_id}: ships a partial Kiro adapter — has "
                f"{sorted(set(KIRO_REQUIRED_FILES) - set(kiro_missing))}, missing {kiro_missing}"
            )

        # The catalog is what the generators read; metadata alone proves nothing about them.
        if agent_id in catalog_harnesses:
            if sorted(catalog_harnesses[agent_id]) != sorted(data.get("harnesses") or []):
                problems.append(
                    f"{agent_id}: catalog/agents.json declares "
                    f"{sorted(catalog_harnesses[agent_id])} but metadata.json declares "
                    f"{sorted(data.get('harnesses') or [])} — resync the catalog entry"
                )

    assert_true(
        not problems,
        "harness declarations disagree with shipped adapters or the catalog:\n  - "
        + "\n  - ".join(problems),
    )


def validate_tui_provider_enum() -> None:
    """Every catalog provider must have a variant in the vfa-tui `Provider` enum.

    tools/vfa-tui deserializes catalog/agents.json into a serde enum with
    `deny_unknown_fields`, so a provider the enum does not know makes the whole catalog
    fail to load and the TUI unusable. A Rust test already asserts this — but CI's
    vfa-tui workflow is path-filtered to `tools/vfa-tui/**` (see
    .github/workflows/vfa-tui-ci.yml), so a catalog-only PR that adds a provider never
    triggers it and merges green while leaving the TUI broken. CLAUDE.md documents that
    trap and asks contributors to remember `cargo test`; this makes remembering
    unnecessary, because `npm run validate` is not path-filtered.

    Parsed, never executed: variant names are read with a regex and converted to the
    kebab-case slugs serde produces via `rename_all`, plus any explicit `rename = "..."`.
    """
    enum_path = ROOT / "tools" / "vfa-tui" / "src" / "models" / "provider.rs"
    if not enum_path.exists():
        # The TUI is optional to the catalog's correctness; absence is not a catalog error.
        return
    source = enum_path.read_text(encoding="utf8")
    variants = re.findall(r"^\s{4}([A-Z][A-Za-z0-9]*),\s*$", source, re.MULTILINE)
    slugs = {re.sub(r"(?<!^)(?=[A-Z])", "-", v).lower() for v in variants}
    slugs |= set(re.findall(r'rename\s*=\s*"([a-z0-9-]+)"', source))

    providers = set()
    for entry in load_json(ROOT / "catalog" / "agents.json"):
        if isinstance(entry, dict) and entry.get("provider"):
            providers.add(entry["provider"])

    missing = sorted(providers - slugs)
    assert_true(
        not missing,
        "catalog providers with no variant in tools/vfa-tui/src/models/provider.rs: "
        + f"{missing}. The TUI deserializes catalog/agents.json with a strict enum, so "
        + "the catalog will fail to load entirely. Add the variant (kebab-case serde) and "
        + "run `cargo test` in tools/vfa-tui.",
    )


def main() -> int:
    errors: list[str] = []
    seen_ids: set[str] = set()
    for expected_type, catalog_path in CATALOGS.items():
        try:
            items = load_json(catalog_path)
            assert_true(isinstance(items, list), f"{catalog_path}: catalog must be a list")
            for item in items:
                validate_item(item, expected_type)
                assert_true(item["id"] not in seen_ids, f"duplicate id: {item['id']}")
                seen_ids.add(item["id"])
                validate_metadata_file(item)
        except AssertionError as exc:
            errors.append(str(exc))
    try:
        validate_no_obvious_secrets()
    except AssertionError as exc:
        errors.append(str(exc))
    try:
        validate_codex_harness_adapters()
    except AssertionError as exc:
        errors.append(str(exc))
    try:
        validate_harness_declarations()
    except AssertionError as exc:
        errors.append(str(exc))
    try:
        validate_tui_provider_enum()
    except AssertionError as exc:
        errors.append(str(exc))
    try:
        validate_markdown_agent_templates()
    except AssertionError as exc:
        errors.append(str(exc))
    try:
        validate_guarded_live_aws_agents()
    except AssertionError as exc:
        errors.append(str(exc))
    try:
        validate_guarded_live_kubernetes_agents()
    except AssertionError as exc:
        errors.append(str(exc))

    if errors:
        for error in errors:
            print(f"ERROR: {error}", file=sys.stderr)
        return 1
    print(f"OK: validated {len(seen_ids)} catalog entries and scanned for obvious secrets")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
