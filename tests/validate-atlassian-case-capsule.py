#!/usr/bin/env python3
"""Gate: Atlassian case capsules obey the board's authorization policy.

The Atlassian board (docs/adr/0005-atlassian-agent-board.md) hands work between agents only
as an ``atlassian-case-capsule`` (schemas/atlassian-case-capsule.schema.json). JSON Schema
checks the shape; this gate checks the policy rules a schema cannot express, and proves them
with committed positive and negative fixtures:

  tests/fixtures/atlassian-case-capsule/valid/*.json      must pass every rule
  tests/fixtures/atlassian-case-capsule/invalid/*.json    mutation of a valid capsule that
                                                          must fail with exactly the named rule

An invalid fixture is ``{"base": "<valid file>", "rule": "<RULE-ID>", "why": "...",
"set": {"dotted.path": value, ...}}``. Path segments that are integers index lists; a value
of ``"__delete__"`` removes the key. Each fixture asserts that the named rule fires and that
no OTHER rule fires, so a rule that is accidentally broad (or a fixture that trips the wrong
check) fails the gate instead of passing for the wrong reason.

Determinism: time comparisons use the capsule's own ``as_of`` field. This gate never reads
the wall clock, so CI cannot change color without a commit.

Scope and limits (do not overclaim): this validates handoff records. It is not a runtime
policy engine, it does not see the live tool session, and a harness that never produces a
capsule is not governed by it. Tool names come from the Rovo MCP supported-tools page as
retrieved 2026-11-13 (skills/atlassian/atlassian-rovo-mcp-integration/references/
evidence-ledger.md); unknown names fail closed above read tier.

Usage: python3 tests/validate-atlassian-case-capsule.py [capsule.json ...]
With paths, validates those capsules only (exit 1 on any violation).
"""
from __future__ import annotations

import copy
import json
import re
import sys
from pathlib import Path

try:
    import jsonschema
except ImportError:  # fail closed: a missing validator must not read as green
    print("FAIL: python package 'jsonschema' is required (pip install -r tests/requirements-ci.txt)")
    sys.exit(1)

REPO = Path(__file__).resolve().parent.parent
SCHEMA = REPO / "schemas" / "atlassian-case-capsule.schema.json"
FIXTURES = REPO / "tests" / "fixtures" / "atlassian-case-capsule"

# Tool tiers. Names are verified against the supported-tools page (ledger ATL-E03..E09).
READ_TOOLS = frozenset({
    "discover", "executeRead",
    "getJiraIssue", "searchJiraIssuesUsingJql", "getJiraScreen",
    "getConfluenceContent", "listConfluenceSpaces", "getConfluenceSpace",
    "listConfluenceComments", "diffConfluenceContentVersions", "listConfluenceAttachments",
    "getConfluenceContentPermissions", "getConfluenceContentRestrictionState",
    "listConfluenceTasks", "searchConfluence",
    "getJsmRequest", "searchJsmRequests", "getJsmRequestActionMetadata",
    "getJsmRequestTypeFields", "getJsmOpsAlerts", "getJsmOpsScheduleInfo",
    "getJsmOpsTeamInfo", "getJsmRequestResolverPlans",
    "getTeamworkGraphObject",
})
# T3: bounded, reversible-by-edit writes after verified human authorization.
WRITE_TOOLS = frozenset({
    "createJiraIssue", "editJiraIssue", "addOrEditJiraIssueComment", "transitionJiraIssue",
    "createConfluenceContent", "updateConfluenceContent", "createConfluenceComment",
    "updateConfluenceComment", "addLabelsToConfluenceContent",
})
# T4: structural or customer/responder-visible changes; two distinct human approvers.
HIGH_RISK_TOOLS = frozenset({
    "moveConfluenceContent", "archiveConfluenceContent", "unarchiveConfluenceContent",
    "restoreConfluenceContentVersion", "setConfluenceContentStatus",
    "convertJiraIssueHierarchy", "createJsmRequest", "transitionJsmRequest",
    "updateJsmOpsAlert",
})
# Never for any agent, at any tier, with any approval:
#   approval decisions (an AI deciding a human approval), permission/restriction changes
#   (who can see content), space AI instructions (a persistent instruction channel other
#   agents will read), another person's availability, destructive deletes.
PROHIBITED_TOOLS = frozenset({
    "decideJsmRequestApproval", "approveJsmRequestResolverPlans",
    "addConfluenceContentPermissions", "removeConfluenceContentPermissions",
    "replaceConfluenceContentPermissions", "setConfluenceContentRestrictionState",
    "setConfluenceSpaceInstructions", "setJsmAgentAvailability", "executeDestructive",
})
# The generic write entry point hides which deferred tool runs; a capsule must name it.
OPAQUE_WRITE_TOOLS = frozenset({"executeWrite"})

TIER_RANK = {"T0-research": 0, "T1-read": 1, "T2-draft": 2, "T3-authorized-write": 3, "T4-high-risk": 4}
BULK_TARGET = re.compile(r"(\*|^(jql|cql):|\ball\b)", re.IGNORECASE)
SECRET_RE = re.compile(
    r"(authorization\s*:\s*(basic|bearer)\s|api[_-]?(token|key)\s*[:=]|password\s*[:=]|client[_-]?secret\s*[:=])",
    re.IGNORECASE,
)
DELETE = "__delete__"


def _strings(node):
    if isinstance(node, str):
        yield node
    elif isinstance(node, dict):
        for k, v in node.items():
            yield str(k)
            yield from _strings(v)
    elif isinstance(node, list):
        for v in node:
            yield from _strings(v)


def check(capsule: dict, validator) -> list[tuple[str, str]]:
    """Return a list of (rule_id, message). Empty means the capsule is acceptable."""
    errors = sorted(validator.iter_errors(capsule), key=lambda e: list(e.path))
    if errors:
        return [("CAP-SCHEMA", f"{'/'.join(map(str, e.path)) or '<root>'}: {e.message}") for e in errors]

    out: list[tuple[str, str]] = []
    tier = capsule["execution_tier"]
    rank = TIER_RANK[tier]
    as_of = capsule["as_of"]  # RFC 3339 'Z' timestamps compare correctly as strings
    approval = capsule["approval_state"]
    tenant = capsule["tenant_context"]["cloud_id"]
    ops = capsule["permitted_operations"]

    for text in _strings(capsule):
        if SECRET_RE.search(text):
            out.append(("CAP-SECRETS", "credential-shaped string present; capsules never carry credentials"))
            break

    needs_write_approval = False
    needs_dual = False
    for op in ops:
        tool = op["tool"]
        if tool in PROHIBITED_TOOLS or tool.lower().startswith("delete"):
            out.append(("CAP-PROHIBITED-TOOL", f"{tool} is prohibited for every agent at every tier"))
            continue
        if tool in OPAQUE_WRITE_TOOLS:
            out.append(("CAP-OPAQUE-EXECUTE", f"{tool} hides the deferred tool; name the concrete tool from discover"))
            continue
        if op["cloud_id"] != tenant:
            out.append(("CAP-TENANT-BINDING", f"{tool} targets cloud_id {op['cloud_id']!r}, capsule tenant is {tenant!r}"))
        if tool in READ_TOOLS:
            continue
        if tool in WRITE_TOOLS:
            needs_write_approval = True
            if rank < 3:
                out.append(("CAP-WRITE-TIER", f"{tool} is a write; tier {tier} permits reads only"))
        elif tool in HIGH_RISK_TOOLS:
            needs_write_approval = needs_dual = True
            if rank < 4:
                out.append(("CAP-HIGH-RISK-TIER", f"{tool} is high-risk; requires T4-high-risk, capsule is {tier}"))
        else:
            out.append(("CAP-UNKNOWN-TOOL", f"{tool} is not a recognized tool; unknown tools fail closed"))
            continue
        if BULK_TARGET.search(op["target"]):
            out.append(("CAP-BULK-TARGET", f"{tool} target {op['target']!r} is a query or wildcard; enumerate approved items"))

    if rank >= 3:
        needs_write_approval = True
    if rank >= 4:
        needs_dual = True

    if needs_write_approval:
        if approval["status"] != "approved":
            out.append(("CAP-APPROVAL-REQUIRED", f"tier {tier} with writes needs approval_state.status 'approved', got {approval['status']!r}"))
        else:
            approvers = approval["approvers"]
            if not approvers or any(a["principal_type"] != "human" for a in approvers):
                out.append(("CAP-APPROVER-HUMAN", "every approver must be a human principal; agent or service approvals are not authorization"))
            record = approval["approval_record"]
            untrusted = {e["locator"] for e in capsule["evidence_references"] if e["trust"] == "untrusted-content"}
            if record is None or record["kind"] != "out-of-band-human-record":
                out.append(("CAP-APPROVAL-RECORD", "approval must be an out-of-band human record, never tenant content, chat, or an agent message"))
            elif record["locator"] in untrusted:
                out.append(("CAP-APPROVAL-RECORD", "approval record points at untrusted tenant content"))
            approved_at, expires_at = approval["approved_at"], approval["expires_at"]
            if approved_at is None or expires_at is None or not (approved_at <= as_of < expires_at):
                out.append(("CAP-APPROVAL-WINDOW", f"approval must satisfy approved_at <= as_of ({as_of}) < expires_at"))
            if needs_dual:
                humans = {a["account_ref"] for a in approvers if a["principal_type"] == "human"}
                requester = capsule["user_identity_context"]["account_ref"]
                if len(humans - {requester}) < 2:
                    out.append(("CAP-DUAL-CONTROL", "T4 needs two distinct human approvers, neither of them the requester"))

    chain = capsule["delegation_chain"]
    budget = capsule["execution_budget"]
    if len(set(chain)) != len(chain) or capsule["target_agent"] in chain:
        out.append(("CAP-DELEGATION-CYCLE", "delegation chain repeats an agent or hands back to an agent already in it"))
    if chain[-1] != capsule["origin_agent"]:
        out.append(("CAP-DELEGATION-CHAIN", "delegation_chain must end with origin_agent"))
    if len(chain) > budget["max_delegation_depth"]:
        out.append(("CAP-DELEGATION-DEPTH", f"chain length {len(chain)} exceeds max_delegation_depth {budget['max_delegation_depth']}"))

    for ev in capsule["evidence_references"]:
        if ev["contains_instructions"] and ev["trust"] != "untrusted-content":
            out.append(("CAP-INSTRUCTION-TRUST", f"evidence {ev['id']} contains instructions but is not marked untrusted-content"))

    status = capsule["handoff_status"]
    if status == "completed" and not capsule["verification_evidence"]:
        out.append(("CAP-COMPLETION-EVIDENCE", "completed requires verification_evidence (read-back results), not a claim"))
    if status in {"proposed", "accepted"} and capsule["deadline_or_expiry"] <= as_of:
        out.append(("CAP-EXPIRED", f"deadline_or_expiry {capsule['deadline_or_expiry']} is not after as_of {as_of}; status must be expired"))
    if capsule["decision_owner"].endswith("-agent"):
        out.append(("CAP-DECISION-OWNER", "decision_owner must be a named human role, not an agent"))
    return out


def _apply(capsule: dict, changes: dict) -> dict:
    result = copy.deepcopy(capsule)
    for dotted, value in changes.items():
        parts = [int(p) if p.isdigit() else p for p in dotted.split(".")]
        node = result
        for p in parts[:-1]:
            node = node[p]
        if value == DELETE:
            del node[parts[-1]]
        else:
            node[parts[-1]] = value
    return result


def main(argv: list[str]) -> int:
    validator = jsonschema.Draft202012Validator(json.loads(SCHEMA.read_text()))
    if argv:
        bad = 0
        for p in argv:
            for rule, msg in check(json.loads(Path(p).read_text()), validator):
                print(f"FAIL {p}: {rule}: {msg}")
                bad += 1
            if not bad:
                print(f"OK   {p}")
        return 1 if bad else 0

    failures = 0
    valid_files = sorted((FIXTURES / "valid").glob("*.json"))
    invalid_files = sorted((FIXTURES / "invalid").glob("*.json"))
    if not valid_files or not invalid_files:
        print("FAIL: fixtures missing under tests/fixtures/atlassian-case-capsule/{valid,invalid}")
        return 1
    for f in valid_files:
        found = check(json.loads(f.read_text()), validator)
        if found:
            failures += 1
            print(f"FAIL valid/{f.name}: " + "; ".join(f"{r}: {m}" for r, m in found))
        else:
            print(f"OK   valid/{f.name}")
    rules_proven: set[str] = set()
    for f in invalid_files:
        spec = json.loads(f.read_text())
        base = json.loads((FIXTURES / "valid" / spec["base"]).read_text())
        found = check(_apply(base, spec["set"]), validator)
        rules = {r for r, _ in found}
        if rules == {spec["rule"]}:
            rules_proven.add(spec["rule"])
            print(f"OK   invalid/{f.name} -> {spec['rule']}")
        else:
            failures += 1
            print(f"FAIL invalid/{f.name}: expected only {spec['rule']}, got {sorted(rules) or 'no violation'}")
    print(f"\n{len(valid_files)} valid, {len(invalid_files)} invalid fixtures; {len(rules_proven)} distinct rules proven to fire")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
