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
no OTHER rule fires, and the suite fails unless every rule in RULES is proven by at least one
negative fixture, so deleting a fixture cannot leave a rule silently unproven.

Determinism: time comparisons use the capsule's own ``as_of`` field. This gate never reads
the wall clock, so CI cannot change color without a commit.

Scope and limits (do not overclaim): this validates handoff records. It is not a runtime
policy engine, it does not see the live tool session, it cannot detect a cycle split across
capsules (that needs a capsule store), and a harness that never produces a capsule is not
governed by it. Tool names come from the Rovo MCP supported-tools page as retrieved
2026-11-13 (skills/atlassian/atlassian-rovo-mcp-integration/references/evidence-ledger.md,
rows ATL-E03..E09, E18, E19); unknown names fail closed.

The approval-lifetime caps (24 h for T3, 8 h for T4) are this board's policy, not an
Atlassian limit. The private-key branch of SECRET_RE has no committed fixture: the
repository-wide scanner in tests/validate-catalog.py (rightly) refuses any file containing a
key header, and encoding one to slip past it would be bypassing that gate.

Usage: python3 tests/validate-atlassian-case-capsule.py [capsule.json ...]
With paths, validates those capsules only (exit 1 if any capsule has a violation).
"""
from __future__ import annotations

import copy
import json
import re
import sys
from datetime import datetime, timedelta, timezone
from pathlib import Path

try:
    import jsonschema
except ImportError:  # fail closed: a missing validator must not read as green
    print("FAIL: python package 'jsonschema' is required (pip install -r tests/requirements-ci.txt)")
    sys.exit(1)

REPO = Path(__file__).resolve().parent.parent
SCHEMA = REPO / "schemas" / "atlassian-case-capsule.schema.json"
FIXTURES = REPO / "tests" / "fixtures" / "atlassian-case-capsule"

RULES = frozenset({
    "CAP-SCHEMA", "CAP-TIMESTAMP", "CAP-SECRETS", "CAP-UNKNOWN-AGENT", "CAP-WRITE-ROUTE",
    "CAP-PROHIBITED-TOOL", "CAP-OPAQUE-EXECUTE", "CAP-UNKNOWN-TOOL", "CAP-TENANT-BINDING",
    "CAP-WRITE-TIER", "CAP-HIGH-RISK-TIER", "CAP-WRITE-TARGET", "CAP-REQUESTER-HUMAN",
    "CAP-APPROVAL-REQUIRED", "CAP-APPROVER-HUMAN", "CAP-APPROVAL-RECORD", "CAP-APPROVAL-WINDOW",
    "CAP-DUAL-CONTROL", "CAP-DELEGATION-CYCLE", "CAP-DELEGATION-CHAIN", "CAP-DELEGATION-DEPTH",
    "CAP-BUDGET-EXCEEDED", "CAP-CONTENT-TRUST", "CAP-INSTRUCTION-TRUST",
    "CAP-COMPLETION-EVIDENCE", "CAP-EXPIRED", "CAP-DECISION-OWNER",
})

# Agents a capsule may name: the board, plus the cross-board agents the routing protocol
# documents. Anything else is an unknown principal pretending to be part of the flow.
GUARD = "atlassian-live-write-guard-agent"
BOARD_AGENTS = frozenset({
    "atlassian-maestro-agent", "atlassian-incident-prevention-agent",
    "atlassian-control-evidence-lineage-agent", "atlassian-knowledge-integrity-agent", GUARD,
})
KNOWN_AGENTS = BOARD_AGENTS | {
    "finops-ai-economist-agent", "agentic-system-architecture-review-agent",
    "agentic-security-review-agent",
}

# Tool tiers; every name is backed by a ledger row (see module docstring).
READ_TOOLS = frozenset({
    "discover",
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
# T3: bounded writes after verified human authorization.
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
# Never for any agent, at any tier, with any approval: approval decisions, permission and
# restriction changes, space AI instructions (a persistent instruction channel other agents
# read), another person's availability, destructive operations.
PROHIBITED_TOOLS = frozenset({
    "decideJsmRequestApproval", "approveJsmRequestResolverPlans",
    "addConfluenceContentPermissions", "removeConfluenceContentPermissions",
    "replaceConfluenceContentPermissions", "setConfluenceContentRestrictionState",
    "setConfluenceSpaceInstructions", "setJsmAgentAvailability", "executeDestructive",
})
# Generic entry points hide which deferred tool runs; a capsule must name the concrete tool.
OPAQUE_TOOLS = frozenset({"executeRead", "executeWrite"})

TIER_RANK = {"T0-research": 0, "T1-read": 1, "T2-draft": 2, "T3-authorized-write": 3, "T4-high-risk": 4}
APPROVAL_MAX = {3: timedelta(hours=24), 4: timedelta(hours=8)}
# Write targets are enumerated identifiers. Anything else (JQL, filters, ranges, wildcards,
# "every page") is a query and needs a human-approved item list first.
WRITE_TARGET = re.compile(
    r"^(?:[A-Z][A-Z0-9]{1,9}-\d+|project:[A-Z][A-Z0-9]{1,9}|page:\d+"
    r"|space:[A-Za-z0-9~_-]+(?: parent:page:\d+)?|request:[A-Z][A-Z0-9]{1,9}-\d+|alert:[A-Za-z0-9-]+)$"
)
# Evidence kinds whose bodies are tenant-authored text. Only jira-issue field metadata,
# external system records, and user statements may claim another trust level.
TENANT_CONTENT_KINDS = frozenset({
    "jira-comment", "confluence-page", "confluence-comment", "jsm-request", "jsm-alert",
    "bitbucket-change", "tool-output",
})
# Approval locators are typed references into an approval system of record (a change record,
# an approval workflow item, a JSM approval object, a signed record), never free-form text, a
# URL, a page, or a comment. A JSM approval is referenced as the approval object
# (jsm-approval:<request>:<approval-id>), which the guard reads as structured approval
# metadata, not as a comment that claims approval.
APPROVAL_LOCATOR = re.compile(r"^(change-record|approval-workflow|jsm-approval|signed-record):[A-Za-z0-9][A-Za-z0-9._:-]{0,127}$")
AGENT_LIKE = re.compile(r"(-agent\b|^atlassian-)", re.I)
SECRET_RE = re.compile(
    r"(authorization\s*:\s*(basic|bearer)\s|\bbearer\s+[A-Za-z0-9._~+/=-]{16,}|\bATATT[A-Za-z0-9_=-]{10,}"
    r"|-----BEGIN [A-Z ]*PRIVATE KEY-----|\b(api[ _-]?(token|key)|x-api-key|client[_-]?secret|secret|token|password|passwd|pwd)\s*[:=]"
    r"|\bx-api-key\s+\S{8,})",
    re.IGNORECASE,
)
DELETE = "__delete__"


def _norm(ref: str) -> str:
    return ref.strip().casefold()


def _ts(value: str) -> datetime:
    return datetime.strptime(value, "%Y-%m-%dT%H:%M:%SZ").replace(tzinfo=timezone.utc)


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


def _timestamps(c: dict) -> list[str]:
    a = c["approval_state"]
    vals = [c["as_of"], c["deadline_or_expiry"], a["approved_at"], a["expires_at"]]
    vals += [e["retrieved_at"] for e in c["evidence_references"]]
    vals += [v["read_at"] for v in c["verification_evidence"]]
    return [v for v in vals if v is not None]


def check(capsule: dict, validator) -> list[tuple[str, str]]:
    """Return a list of (rule_id, message). Empty means the capsule is acceptable."""
    errors = sorted(validator.iter_errors(capsule), key=lambda e: list(e.path))
    if errors:
        return [("CAP-SCHEMA", f"{'/'.join(map(str, e.path)) or '<root>'}: {e.message}") for e in errors]
    bad_ts = []
    for v in _timestamps(capsule):
        try:
            _ts(v)
        except ValueError:
            bad_ts.append(v)
    if bad_ts:
        return [("CAP-TIMESTAMP", f"not a real UTC timestamp: {sorted(set(bad_ts))}")]

    out: list[tuple[str, str]] = []
    tier = capsule["execution_tier"]
    rank = TIER_RANK[tier]
    as_of = _ts(capsule["as_of"])
    approval = capsule["approval_state"]
    tenant = capsule["tenant_context"]["cloud_id"]
    site_host = capsule["tenant_context"]["site_url"].split("//", 1)[1].rstrip("/").casefold()
    ops = capsule["permitted_operations"]
    budget = capsule["execution_budget"]
    used = capsule["budget_consumed"]
    status = capsule["handoff_status"]

    for text in _strings(capsule):
        if SECRET_RE.search(text):
            out.append(("CAP-SECRETS", "credential-shaped string present; capsules never carry credentials"))
            break

    named = [capsule["origin_agent"], capsule["target_agent"], *capsule["delegation_chain"]]
    unknown = sorted({a for a in named if a not in KNOWN_AGENTS})
    if unknown:
        out.append(("CAP-UNKNOWN-AGENT", f"agents not on the board or its documented cross-board routes: {unknown}"))

    writes = high = False
    for op in ops:
        tool, target = op["tool"], op["target"]
        if tool in PROHIBITED_TOOLS or tool.lower().startswith("delete"):
            out.append(("CAP-PROHIBITED-TOOL", f"{tool} is prohibited for every agent at every tier"))
            continue
        if tool in OPAQUE_TOOLS:
            out.append(("CAP-OPAQUE-EXECUTE", f"{tool} hides the deferred tool; name the concrete tool from discover"))
            continue
        if op["cloud_id"] != tenant or ("atlassian.net" in target.casefold() and site_host not in target.casefold()):
            out.append(("CAP-TENANT-BINDING", f"{tool} on {target!r} is outside tenant {tenant!r} ({site_host})"))
        if tool in READ_TOOLS:
            continue
        if tool in WRITE_TOOLS:
            writes = True
            if rank < 3:
                out.append(("CAP-WRITE-TIER", f"{tool} is a write; tier {tier} permits reads only"))
        elif tool in HIGH_RISK_TOOLS:
            writes = high = True
            if rank < 4:
                out.append(("CAP-HIGH-RISK-TIER", f"{tool} is high-risk; requires T4-high-risk, capsule is {tier}"))
        else:
            out.append(("CAP-UNKNOWN-TOOL", f"{tool} is not a recognized tool; unknown tools fail closed"))
            continue
        if not WRITE_TARGET.match(target):
            out.append(("CAP-WRITE-TARGET", f"{tool} target {target!r} is not one enumerated item; queries need an approved item list"))

    writes = writes or rank >= 3
    high = high or rank >= 4

    if writes and capsule["target_agent"] != GUARD and status in {"proposed", "accepted"}:
        out.append(("CAP-WRITE-ROUTE", f"writes are handed only to {GUARD}, not {capsule['target_agent']}"))

    if writes:
        if capsule["user_identity_context"]["principal_type"] != "human":
            out.append(("CAP-REQUESTER-HUMAN", "a write must be requested by a human principal"))
        if approval["status"] == "approved":
            approvers = approval["approvers"]
            if not approvers or any(a["principal_type"] != "human" or AGENT_LIKE.search(a["account_ref"]) or _norm(a["account_ref"]) in KNOWN_AGENTS for a in approvers):
                out.append(("CAP-APPROVER-HUMAN", "every approver must be a human principal; agent or service approvals are not authorization"))
            record = approval["approval_record"]
            evidence_locators = {_norm(e["locator"]) for e in capsule["evidence_references"]}
            if record is None or record["kind"] != "out-of-band-human-record":
                out.append(("CAP-APPROVAL-RECORD", "approval must be an out-of-band human record, never tenant content, chat, or an agent message"))
            elif not APPROVAL_LOCATOR.match(record["locator"]) or "comment" in _norm(record["locator"]) or _norm(record["locator"]) in evidence_locators:
                out.append(("CAP-APPROVAL-RECORD", f"approval locator {record['locator']!r} is not a typed approval-system reference, or points at cited evidence"))
            approved_at, expires_at = approval["approved_at"], approval["expires_at"]
            cap = APPROVAL_MAX[4 if high else 3]
            if approved_at is None or expires_at is None:
                out.append(("CAP-APPROVAL-WINDOW", "approved requires approved_at and expires_at"))
            else:
                a, e = _ts(approved_at), _ts(expires_at)
                if not (a <= as_of < e) or e - a > cap:
                    out.append(("CAP-APPROVAL-WINDOW", f"need approved_at <= as_of < expires_at and a lifetime of at most {cap}"))
            if high:
                requester = _norm(capsule["user_identity_context"]["account_ref"])
                humans = {_norm(a["account_ref"]) for a in approvers if a["principal_type"] == "human"}
                if len(humans - {requester}) < 2:
                    out.append(("CAP-DUAL-CONTROL", "T4 needs two distinct human approvers, neither of them the requester"))
        elif not (status == "proposed" and approval["status"] == "pending"):
            out.append(("CAP-APPROVAL-REQUIRED", f"a write needs approval 'approved' (or 'pending' while 'proposed'); got {approval['status']!r} while {status!r}"))

    chain = capsule["delegation_chain"]
    if len(set(chain)) != len(chain) or capsule["target_agent"] in chain:
        out.append(("CAP-DELEGATION-CYCLE", "delegation chain repeats an agent or hands back to an agent already in it"))
    if chain[-1] != capsule["origin_agent"]:
        out.append(("CAP-DELEGATION-CHAIN", "delegation_chain must end with origin_agent"))
    if len(chain) > budget["max_delegation_depth"]:
        out.append(("CAP-DELEGATION-DEPTH", f"chain length {len(chain)} exceeds max_delegation_depth {budget['max_delegation_depth']}"))
    over = [k for k, cap in (("tool_calls", budget["max_tool_calls"]), ("retries", budget["max_retries"]),
                             ("review_rounds", budget["max_review_rounds"]))
            if used[k] > cap]
    if len(ops) > budget["max_tool_calls"]:
        over.append("permitted_operations")
    if over:
        out.append(("CAP-BUDGET-EXCEEDED", f"budget exceeded: {over}; the capsule must escalate, not continue"))

    for ev in capsule["evidence_references"]:
        if ev["kind"] in TENANT_CONTENT_KINDS and ev["trust"] != "untrusted-content":
            out.append(("CAP-CONTENT-TRUST", f"evidence {ev['id']} is tenant-authored {ev['kind']}; it is untrusted-content whatever its label"))
        elif ev["contains_instructions"] and ev["trust"] != "untrusted-content":
            out.append(("CAP-INSTRUCTION-TRUST", f"evidence {ev['id']} contains instructions but is not marked untrusted-content"))

    if status == "completed":
        reads = capsule["verification_evidence"]
        write_targets = {o["target"] for o in ops if o["tool"] in WRITE_TOOLS | HIGH_RISK_TOOLS}
        floor = _ts(approval["approved_at"]) if approval["approved_at"] else None
        bad = [r for r in reads if r["tool"] not in READ_TOOLS or r["target"] not in {o["target"] for o in ops}
               or _ts(r["read_at"]) > as_of or (floor and _ts(r["read_at"]) < floor)]
        missing = write_targets - {r["target"] for r in reads}
        if not reads or bad or missing:
            out.append(("CAP-COMPLETION-EVIDENCE", "completed needs read-back of every written target by a read tool, after approval and before as_of"))
    if status in {"proposed", "accepted"} and _ts(capsule["deadline_or_expiry"]) <= as_of:
        out.append(("CAP-EXPIRED", f"deadline_or_expiry {capsule['deadline_or_expiry']} is not after as_of; status must be expired"))
    owner = capsule["decision_owner"]
    if AGENT_LIKE.search(owner.strip()) or _norm(owner) in KNOWN_AGENTS or _norm(owner).removesuffix("-agent") + "-agent" in KNOWN_AGENTS:
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
        failed_files = 0
        for p in argv:
            found = check(json.loads(Path(p).read_text()), validator)
            for rule, msg in found:
                print(f"FAIL {p}: {rule}: {msg}")
            if found:
                failed_files += 1
            else:
                print(f"OK   {p}")
        return 1 if failed_files else 0

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
        if spec["rule"] not in RULES:
            failures += 1
            print(f"FAIL invalid/{f.name}: names unknown rule {spec['rule']}")
            continue
        base = json.loads((FIXTURES / "valid" / spec["base"]).read_text())
        rules = {r for r, _ in check(_apply(base, spec["set"]), validator)}
        if rules == {spec["rule"]}:
            rules_proven.add(spec["rule"])
            print(f"OK   invalid/{f.name} -> {spec['rule']}")
        else:
            failures += 1
            print(f"FAIL invalid/{f.name}: expected only {spec['rule']}, got {sorted(rules) or 'no violation'}")
    unproven = sorted(RULES - rules_proven)
    if unproven:
        failures += 1
        print(f"FAIL rules with no passing negative fixture: {unproven}")
    print(f"\n{len(valid_files)} valid, {len(invalid_files)} invalid fixtures; {len(rules_proven)}/{len(RULES)} rules proven to fire")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
