#!/usr/bin/env python3
"""Generator: NetSuite agents + companion skills from per-agent JSON data files.

Reads every `scripts/netsuite_data/agents/*.json` (one file per agent, authored to
`tmp/netsuite-build/DATA-CONTRACT.md`) and emits, following the salesforce
static-review (T0) house style:

  agents/netsuite/<id>/AGENT.md
  agents/netsuite/<id>/metadata.json
  agents/netsuite/<id>/LEAST-PRIVILEGES.md
  agents/netsuite/<id>/harnesses/{codex.toml,copilot.agent.md,claude-code.agent.md,
                                  cursor.agent.md,gemini.agent.md,kiro-ide.agent.md,
                                  kiro-cli.agent.json}
  agents/netsuite/<id>/README.md            (maestro only)
  skills/netsuite/<skill-id>/SKILL.md
  skills/netsuite/<skill-id>/metadata.json
  skills/netsuite/<skill-id>/references/*.md

Run:  python3 scripts/gen_netsuite_agents.py
Then: python3 scripts/update-catalog-new-agents.py --provider netsuite && npm run manifest:write:all
      && python3 tests/validate-asset-integrity.py --write && npm run validate
"""
from __future__ import annotations

import glob
import json
import os

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DATA_DIR = os.path.join(ROOT, "scripts", "netsuite_data", "agents")
DATE = "2026-06-09"
AUTHOR = "github: VincentChuWaiChow"
VERSION = "0.1.0"

# ---------------------------------------------------------------- helpers


def y(s: str) -> str:
    """Safely quote a string as a single-line YAML/TOML/JSON double-quoted scalar."""
    return json.dumps(s, ensure_ascii=False)


def snake(agent_id: str) -> str:
    return agent_id.replace("-", "_")


def bullets(items) -> str:
    return "\n".join(f"- {x}" for x in (items or []))


def numbered(items) -> str:
    return "\n".join(f"{i}. {x}" for i, x in enumerate(items or [], 1))


def write(path: str, content: str) -> None:
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w") as f:
        f.write(content)
    print(f"  wrote {path.replace(ROOT + '/', '')}")


def load_agents() -> list[dict]:
    files = sorted(glob.glob(os.path.join(DATA_DIR, "*.json")))
    agents = []
    for fp in files:
        with open(fp) as f:
            agents.append(json.load(f))
    return agents


# ---------------------------------------------------------------- agent body (shared by harnesses)


def agent_body(a: dict) -> str:
    """Canonical agent contract body — used in AGENT.md and all markdown harnesses."""
    aid = a["id"]
    skill = a.get("companion_skill")
    parts = [
        f"# {a['name']}",
        "",
        f"Use this canonical agent only for `{aid}` work.",
        "",
    ]
    if a.get("is_maestro"):
        parts += [
            "## Required Skills",
            "",
            "Before answering, read and follow:",
            "",
            "- `skills/cross-functional/netsuite-routing-protocol/SKILL.md`",
            "",
        ]
    elif skill:
        parts += [
            "## Required Skill",
            "",
            "Before answering, read and follow:",
            "",
            f"- `skills/netsuite/{skill['id']}/SKILL.md`",
            "",
            f"Load files under `skills/netsuite/{skill['id']}/references/` only when the task "
            "needs that reference. Do not dump reference text into the response.",
            "",
        ]
    parts += ["## Mission", "", a.get("mission", a.get("focus", "")), ""]
    if a.get("scope_owned"):
        parts += ["## Scope Owned", "", bullets(a["scope_owned"]), ""]
    if a.get("out_of_scope"):
        parts += ["## Out of Scope", "", bullets(a["out_of_scope"]), ""]
    if a.get("cert_alignment"):
        parts += ["## NetSuite Certification / Role Alignment", "", a["cert_alignment"], ""]
    if a.get("required_inputs"):
        parts += ["## Required Inputs", "", bullets(a["required_inputs"]), ""]
    parts += ["## Operating Rules", "", bullets(a.get("operating_rules", [])), ""]
    if a.get("evidence_requirements"):
        parts += ["## Evidence Requirements", "", bullets(a["evidence_requirements"]), ""]
    parts += ["## Refusal Triggers", "", bullets(a.get("refusal_triggers", [])), ""]
    parts += ["## Escalation Triggers", "", bullets(a.get("escalation_triggers", [])), ""]
    parts += [
        "## Permission / Tooling Posture",
        "",
        (
            "Static review only. Never invokes NetSuite SuiteTalk/REST/SOAP APIs, SuiteScript, "
            "SDF, or account credentials. Works from sanitized configuration excerpts. Does not "
            "approve, deploy, or mutate any NetSuite account. Routes every live-account change to "
            "`netsuite-live-org-mutation-guard-agent` with a named human decision owner."
        ),
        "",
        "## Output Format",
        "",
        numbered([
            (
                "Verdict (Critical / High / Medium / Low / Unknown — Unknown when account type, "
                "subsidiary, or material facts are absent)"
            ),
            "Brutal assessment (what is wrong or unproven)",
            (
                "Facts (label each [LIVE_EVIDENCE] / [REPOSITORY_EVIDENCE] / [USER_PROVIDED] / "
                "[OFFICIAL_DOCUMENTATION] / [INFERENCE] / [UNVERIFIED])"
            ),
            "Assumptions",
            "Findings with risk ratings",
            "Adversarial stress test",
            "Least-privilege posture (custom role, never Administrator)",
            "Safe next actions",
            "Escalation trigger (named target agent + human owner)",
            "Open questions",
        ]),
    ]
    return "\n".join(parts)


# ---------------------------------------------------------------- agent files


def agent_md(a: dict) -> str:
    return (
        "---\n"
        "metadata:\n"
        f'  author: "{AUTHOR}"\n'
        f'  version: "{VERSION}"\n'
        "---\n\n"
        f"# {a['name']}\n\n"
        f"> Agent for `{a['id']}`. {a['summary']}\n\n"
        "## Harness Variants\n\n"
        "- `harnesses/codex.toml` — Codex native agent configuration.\n"
        "- `harnesses/copilot.agent.md` — GitHub Copilot / VS Code custom agent definition.\n"
        "- `harnesses/claude-code.agent.md` — Claude Code Markdown-family adapter.\n"
        "- `harnesses/cursor.agent.md` — Cursor Markdown-family adapter.\n"
        "- `harnesses/gemini.agent.md` — Gemini CLI Markdown-family adapter.\n"
        "- `harnesses/kiro-ide.agent.md` — Kiro IDE Markdown-family adapter.\n"
        "- `harnesses/kiro-cli.agent.json` — Kiro CLI JSON adapter.\n\n"
        "## Canonical Contract\n\n"
        f"{agent_body(a)}\n"
    )


def agent_metadata(a: dict) -> str:
    companion = []
    if a.get("companion_skill"):
        companion = [a["companion_skill"]["id"]]
    aid = a["id"]
    hv = {
        "codex": f"agents/netsuite/{aid}/harnesses/codex.toml",
        "copilot": f"agents/netsuite/{aid}/harnesses/copilot.agent.md",
        "claude-code": f"agents/netsuite/{aid}/harnesses/claude-code.agent.md",
        "cursor": f"agents/netsuite/{aid}/harnesses/cursor.agent.md",
        "gemini": f"agents/netsuite/{aid}/harnesses/gemini.agent.md",
        "kiro-ide": f"agents/netsuite/{aid}/harnesses/kiro-ide.agent.md",
        "kiro-cli": f"agents/netsuite/{aid}/harnesses/kiro-cli.agent.json",
    }
    return json.dumps({
        "id": aid,
        "name": a["name"],
        "type": "agent",
        "provider": "netsuite",
        "harnesses": ["codex", "copilot", "claude-code", "cursor", "gemini", "kiro"],
        "harness_variants": hv,
        "summary": a["summary"],
        "source_type": a.get("source_type", "original"),
        **({"source_attribution": a["source_attribution"]} if a.get("source_attribution") else {}),
        "official_docs": a["official_docs"],
        "security_notes": a["security_notes"],
        "last_verified": DATE,
        "path": f"agents/netsuite/{aid}/",
        "companion_skills": companion,
        "execution_tier": "static-review",
        "lifecycle": "experimental",
        "author": AUTHOR,
        "version": VERSION,
    }, indent=2) + "\n"


def least_privileges_md(a: dict) -> str:
    lp = a.get("least_privilege", {}) or {}
    perms = lp.get("permissions", [])
    perm_lines = "\n".join(
        f"- **{p.get('name','?')}** ({p.get('level','View')}) — {p.get('why','')}" for p in perms
    ) or "- No standing NetSuite permissions required (static review of sanitized excerpts only)."
    forbidden = bullets(lp.get("forbidden", ["Administrator role"])) or "- Administrator role"
    skill = a.get("companion_skill")
    companion_line = (
        f"`{skill['id']}` — {skill.get('name','companion skill')}" if skill else "None (router/structural role)."
    )
    return (
        f"# Least-privilege NetSuite posture for {a['name']}\n\n"
        "## Execution tier\n\n"
        "**T0 — Static Review**\n\n"
        'Rationale: `execution_tier: "static-review"` declared in `metadata.json`. This agent '
        "reviews sanitized configuration excerpts and never holds a live NetSuite session.\n\n"
        "## Identity model\n\n"
        "No live NetSuite identity is required for the agent itself. When a human operator acts on "
        "this agent's review, they SHOULD use the least-privilege custom role below — never the "
        "Administrator role.\n\n"
        "## Recommended custom role\n\n"
        f"- **Custom role name:** {lp.get('custom_role_name', 'NetSuite Reviewer (custom)')}\n"
        f"- **Copy from standard role:** {lp.get('based_on_standard_role', 'a relevant standard role')} "
        "(NetSuite guidance: start from a copy of a standard role, then remove unneeded permissions).\n"
        f"- **Modules in scope:** {', '.join(lp.get('modules', []) or ['scoped to remit'])}\n"
        f"- **Two-Factor Authentication required:** {'Yes' if lp.get('requires_2fa') else 'Per account policy'}\n\n"
        "### Minimal permissions\n\n"
        f"{perm_lines}\n\n"
        "## Forbidden\n\n"
        f"{forbidden}\n\n"
        "## Blast-radius bound\n\n"
        "Even if fully compromised, this agent cannot mutate a NetSuite account: it has no live "
        "session, no API tokens, and no SDF deploy rights. It can only produce review text.\n\n"
        "## Refusal triggers\n\n"
        f"{bullets(a.get('refusal_triggers', []))}\n\n"
        "## Escalation path\n\n"
        "Route all live-account changes to `netsuite-live-org-mutation-guard-agent` with a named "
        "human decision owner and a structured case capsule.\n\n"
        "## Role creation steps\n\n"
        + numbered([
            "In the target SANDBOX, copy the standard role named above to a new custom role.",
            "Remove every permission not listed under Minimal permissions.",
            "Add only the listed permissions at the stated access level.",
            "Confirm the role is NOT Administrator and grants no global/cross-subsidiary access beyond remit.",
            "Enable 2FA enforcement if the role touches privileged permissions.",
            "Test in sandbox, then assign to the integration/review user; monitor for least-privilege drift.",
        ])
        + "\n\n## Companion skill\n\n"
        f"{companion_line}\n"
    )


# --------------------- harness adapters


def codex_toml(a: dict) -> str:
    skill = a.get("companion_skill")
    rules = "\n".join(a.get("operating_rules", [])[:8]) or "- Static review only."
    instr = (
        (f"Load and follow the bound `{skill['id']}` skill first.\n\n" if skill else "")
        + "Token discipline:\n"
        "- Read only SKILL.md first; load references only when the task requires them.\n"
        "- Keep answers compact: verdict, assessment, facts, assumptions, findings, stress test, "
        "least-privilege posture, safe next actions, escalation, open questions.\n\n"
        f"Role focus: {a.get('focus', a['summary'])}\n\n"
        "Safety contract:\n"
        f"{rules}\n"
        "- Static review only; never invokes NetSuite APIs, SuiteScript, SDF, or credentials.\n"
        "- Never depends on the Administrator role; recommends least-privilege custom roles.\n"
        "- Routes all live-account changes to netsuite-live-org-mutation-guard-agent.\n"
        "- Rate every finding Critical / High / Medium / Low / Unknown."
    )
    lines = [
        f"name = {y(snake(a['id']))}",
        f"description = {y(a['summary'])}",
        'model = "gpt-5.5"',
        'model_reasoning_effort = "high"',
        'sandbox_mode = "read-only"',
        "",
        'developer_instructions = """',
        instr,
        '"""',
        "",
        "[metadata]",
        f'author = "{AUTHOR}"',
        f'version = "{VERSION}"',
    ]
    if skill:
        lines += ["", "[[skills.config]]", f'path = "skills/netsuite/{skill["id"]}/SKILL.md"', "enabled = true"]
    return "\n".join(lines) + "\n"


def md_harness(a: dict) -> str:
    return f"---\nname: {y(a['name'])}\ndescription: {y(a['summary'])}\n---\n\n{agent_body(a)}\n"


# Gemini CLI requires `name` to be a slug (/^[a-z0-9-_]+$/ in
# packages/core/src/agents/agentLoader.ts) and skips any agent file that fails
# it, so the agent id goes there and the readable name moves to `display_name`.
def gemini_harness(a: dict) -> str:
    return f"---\nname: {y(a['id'])}\ndisplay_name: {y(a['name'])}\ndescription: {y(a['summary'])}\n---\n\n{agent_body(a)}\n"


def copilot_md(a: dict) -> str:
    fm = (
        "---\n"
        f"description: {y(a['summary'])}\n"
        f"name: {y(a['name'])}\n"
        "tools:\n"
        '  - "read"\n  - "search"\n  - "search/codebase"\n  - "web/fetch"\n'
        "disable-model-invocation: false\n"
        "user-invocable: true\n"
        "---\n\n"
    )
    return fm + agent_body(a) + "\n"


def kiro_cli_json(a: dict) -> str:
    return json.dumps({
        "name": a["id"],
        "description": a["summary"],
        "prompt": agent_body(a),
    }, indent=2, ensure_ascii=False) + "\n"


def maestro_readme(a: dict, agents: list[dict]) -> str:
    rows = []
    for x in agents:
        if x.get("is_maestro") or x.get("is_live_guard"):
            continue
        dk = x.get("domain_key", x["id"])
        kws = ", ".join(x.get("routing_keywords", [])[:6])
        rows.append(f"| `{dk}` | `{x['id']}` | {kws} |")
    table = "\n".join(rows)
    return (
        "# NetSuite Maestro Agent\n\n"
        "Entry point for the NetSuite domain. Classifies a NetSuite matter and routes it to the "
        "right specialist agent, or gates it to the live-operation guard. Classification and "
        "routing only — never executes or recommends executing a live NetSuite mutation.\n\n"
        "---\n\n"
        "## How routing works\n\n"
        "### Required skills\n\n"
        "- `skills/cross-functional/netsuite-routing-protocol/SKILL.md`\n\n"
        "### Routing modes\n\n"
        "- `single` — one specialist owns the matter.\n"
        "- `parallel (N)` — multiple domains co-own; escalate conflicts.\n"
        "- `live-guard-gate` — any live-account mutation intent; routes to the live-operation guard.\n"
        "- `unclassified` — insufficient signal; ask for sanitized evidence.\n\n"
        "### Escalation gates\n\n"
        "- Financial close / posting / revenue recognition impact → audit-controls-sox agent.\n"
        "- Cross-subsidiary (OneWorld) boundary risk → oneworld-multisubsidiary agent.\n"
        "- AI Connector / MCP tool scope → ai-connector-mcp agent.\n"
        "- Any live mutation → netsuite-live-org-mutation-guard-agent (named human owner).\n\n"
        "---\n\n"
        "## The NetSuite domain taxonomy\n\n"
        "| Domain | Primary agent | Typical signals |\n|---|---|---|\n"
        f"{table}\n\n"
        "Structural roles (excluded from keyword routing):\n\n"
        "| Role | Agent | Function |\n|---|---|---|\n"
        "| Maestro | `netsuite-maestro-agent` | Classify + route only |\n"
        "| Live Guard | `netsuite-live-org-mutation-guard-agent` | Gate all live mutations |\n\n"
        "---\n\n"
        "## What the maestro will refuse\n\n"
        "- Requests for account credentials, tokens, or the Administrator role.\n"
        "- Direct execution of any live NetSuite mutation.\n"
        "- Claiming a Coming-Soon certification is available.\n\n"
        "---\n\n"
        "## Eval coverage\n\n"
        "Routing is covered by `tests/fixtures/netsuite-maestro-routing/`. Run "
        "`npm run validate:maestro-routing`.\n\n"
        "---\n\n"
        "Part of the Vanguard Frontier Agentic NetSuite portfolio.\n"
    )


# ---------------------------------------------------------------- skill files


def skill_md(a: dict) -> str:
    s = a["companion_skill"]
    refs = s.get("references", [])
    ref_lines = "\n".join(
        f"- [{r['file']}](references/{r['file']}) — {r.get('purpose','')}" for r in refs
    ) or "- (no additional references)"
    frontmatter = "---\n" + f"name: {s['id']}\n" + f"description: {y(s['description'])}\n"
    if a.get("source_type") == "adapted":
        frontmatter += "license: UPL-1.0\n"
    return frontmatter + (
        "allowed-tools: Read Grep Glob\n"
        "metadata:\n"
        f'  author: "{AUTHOR}"\n'
        f'  version: "{VERSION}"\n'
        f'  updated: "{DATE}"\n'
        f"  category: {s.get('category', 'compliance')}\n"
        "  lifecycle: experimental\n"
        "  execution_tier: static-review\n"
        "  mcp_servers: []\n"
        "  oauth_scopes: []\n"
        "  run_as_permissions:\n"
        "    required: []\n"
        "    denied: []\n"
        "---\n\n"
        f"# {s['name']}\n\n"
        "## Purpose\n\n"
        f"{a.get('focus', a['summary'])} T0 static review — no NetSuite account connection required; "
        "output is a draft for human review.\n\n"
        "## When This Skill Owns the Task\n\n"
        f"{bullets(s.get('when', []))}\n\n"
        "## Recommended Workflow\n\n"
        f"{numbered(s.get('workflow_steps', []))}\n\n"
        "## Evidence Hierarchy\n\n"
        f"{s.get('evidence_hierarchy_note', 'LIVE_EVIDENCE > REPOSITORY_EVIDENCE > USER_PROVIDED > OFFICIAL_DOCUMENTATION > INFERENCE > UNVERIFIED > BLOCKED')}\n\n"
        "## Safety Checklist\n\n"
        f"{bullets(s.get('safety_checklist', []))}\n\n"
        "## Rules — Hard-Stop Constraints\n\n"
        "- Static review only; never connect to a live NetSuite account or invoke APIs/SuiteScript/SDF.\n"
        "- Never request or accept credentials, tokens, or secrets.\n"
        "- Never depend on the Administrator role; recommend least-privilege custom roles (note 2FA).\n"
        "- Prefer OAuth 2.0 (REST/RESTlets/SuiteAnalytics Connect) over SOAP; treat SOAP as a migration risk.\n"
        "- Never claim a Coming-Soon certification is available.\n\n"
        "## Refusal Triggers\n\n"
        f"{bullets(a.get('refusal_triggers', []))}\n\n"
        "## T0 Contract\n\n"
        "No account connection, no OAuth, no secrets. Output is draft review text for a human owner.\n\n"
        "## Security Notes\n\n"
        f"{a['security_notes']}\n\n"
        "## Reference File Index\n\n"
        f"{ref_lines}\n"
    )


def skill_metadata(a: dict) -> str:
    s = a["companion_skill"]
    obj = {
        "id": s["id"],
        "name": s["name"],
        "type": "skill",
        "provider": "netsuite",
        "harnesses": ["claude-code", "codex", "cursor", "gemini", "kiro", "other"],
        "summary": s["description"][:300] if len(s["description"]) >= 20 else a["summary"],
        "source_type": a.get("source_type", "original"),
        "category": s.get("category", "compliance"),
        "execution_tier": "static-review",
        "oauth_scopes": [],
        "mcp_servers": [],
        "run_as_permissions": {},
        "sandbox_only": False,
        "production_allowed": True,
        "official_docs": a["official_docs"],
        "security_notes": a["security_notes"],
        "last_verified": DATE,
        "path": f"skills/netsuite/{s['id']}",
        "author": AUTHOR,
        "version": VERSION,
    }
    if a.get("source_attribution"):
        obj["source_attribution"] = a["source_attribution"]
    return json.dumps(obj, indent=2, ensure_ascii=False) + "\n"


def reference_file(a: dict, r: dict) -> str:
    """Generate a focused reference doc."""
    fname = r["file"]
    purpose = r.get("purpose", "")
    if fname == "official-sources.md":
        body = "\n".join(f"- {u}" for u in a["official_docs"])
        return f"# Official Sources\n\n{purpose}\n\nVerified {DATE} against official Oracle/NetSuite documentation:\n\n{body}\n"
    if fname == "least-privilege.md":
        return least_privileges_md(a)  # reuse the same content for the skill reference
    if fname == "safety-checklist.md":
        s = a["companion_skill"]
        return f"# Safety Checklist\n\n{purpose}\n\n{bullets(s.get('safety_checklist', []))}\n\n## Refusal triggers\n\n{bullets(a.get('refusal_triggers', []))}\n"
    if fname == "release-drift.md":
        return (
            f"# Release Drift\n\n{purpose}\n\nNetSuite releases biannually. Content verified {DATE}.\n\n"
            "Release-sensitive items to re-verify each release:\n\n"
            "- SOAP web services removal timeline (REST + OAuth 2.0 recommended for new integrations "
            "from 2026.1; new SOAP integrations blocked at 2027.1).\n"
            "- Certification availability (AI Specialist/Professional and BI & Reporting Professional "
            "are Coming Soon — re-check status).\n"
            "- AI Connector / MCP permission names and role restrictions.\n"
        )
    # topic-specific or unknown reference: emit a focused stub seeded from the agent content
    return (
        f"# {fname.replace('.md','').replace('-',' ').title()}\n\n{purpose}\n\n"
        f"Scope: {a.get('focus', a['summary'])}\n\n"
        f"{bullets(a.get('scope_owned', []))}\n"
    )


# ---------------------------------------------------------------- build


def build() -> None:
    agents = load_agents()
    if not agents:
        print("No agent data files found in scripts/netsuite_data/agents/. Nothing to do.")
        return
    print(f"Generating {len(agents)} NetSuite agents + companion skills...\n")
    for a in agents:
        aid = a["id"]
        adir = os.path.join(ROOT, "agents", "netsuite", aid)
        hdir = os.path.join(adir, "harnesses")
        print(f"[{aid}]")
        write(os.path.join(adir, "AGENT.md"), agent_md(a))
        write(os.path.join(adir, "metadata.json"), agent_metadata(a))
        write(os.path.join(adir, "LEAST-PRIVILEGES.md"), least_privileges_md(a))
        write(os.path.join(hdir, "codex.toml"), codex_toml(a))
        write(os.path.join(hdir, "copilot.agent.md"), copilot_md(a))
        write(os.path.join(hdir, "claude-code.agent.md"), md_harness(a))
        write(os.path.join(hdir, "cursor.agent.md"), md_harness(a))
        write(os.path.join(hdir, "gemini.agent.md"), gemini_harness(a))
        write(os.path.join(hdir, "kiro-ide.agent.md"), md_harness(a))
        write(os.path.join(hdir, "kiro-cli.agent.json"), kiro_cli_json(a))
        if a.get("is_maestro"):
            write(os.path.join(adir, "README.md"), maestro_readme(a, agents))
        # companion skill
        s = a.get("companion_skill")
        if s:
            sdir = os.path.join(ROOT, "skills", "netsuite", s["id"])
            write(os.path.join(sdir, "SKILL.md"), skill_md(a))
            write(os.path.join(sdir, "metadata.json"), skill_metadata(a))
            for r in s.get("references", []):
                write(os.path.join(sdir, "references", r["file"]), reference_file(a, r))
    print("\nNetSuite agents + skills generated.")


if __name__ == "__main__":
    build()
