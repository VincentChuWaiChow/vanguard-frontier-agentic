#!/usr/bin/env python3
"""Deterministic rule engine for agentic-board handoff records.

A *record set* is one JSON document ``{"messages": [envelope, ...]}`` whose envelopes share
one ``correlation_id``. Each envelope must satisfy ``schemas/agentic-handoff.schema.json``
(shape only); every cross-field and cross-message rule lives here, one predicate per rule
id, so a defect fails with exactly the id that names it.

Decisions come from committed data only: no wall clock, no network, no randomness. Time
rules compare the envelopes' own ``sent_at`` / ``deadline`` fields (UTC ``Z`` strings, so
string order is time order).

What this does NOT do: authenticate senders, verify that a principal is who it claims to
be, or enforce anything at runtime. A record that passes is well-formed and internally
consistent; it is not proof that any approval happened. That is the deploying runtime's job.

Schema patterns are evaluated with ECMA-262 semantics, as JSON Schema 2020-12 requires: a
pattern's final ``$`` matches only at the end of the string. Python's ``re`` lets ``$`` match
before a trailing newline, which would let ``"subject:...\n"`` pass the schema and then compare
unequal to the canonical subject in every identity rule.

CLI (prints one rule id per line, exit 1 on any violation; each file is checked on its own):
    python3 tests/agentic-board-suite/agentic_board_rules.py <record-set.json> [...]
Digest helper for a principal binding a decision-record (prints request_digest, exit 0):
    python3 tests/agentic-board-suite/agentic_board_rules.py --digest <request-envelope.json>
"""
from __future__ import annotations

import hashlib
from collections import Counter
import json
import re
import sys
from pathlib import Path

import jsonschema

ROOT = Path(__file__).resolve().parents[2]
SCHEMA_PATH = ROOT / "schemas" / "agentic-handoff.schema.json"
CATALOG_PATH = ROOT / "catalog" / "agents.json"

ARCH = "agentic-system-architecture-review-agent"
SEC = "agentic-security-review-agent"
FIN = "finance-capital-allocation-advisor-agent"

BOARD_ACTIONS = frozenset({"read", "analyze", "recommend", "challenge", "escalate"})
TIER_ORDER = ("R0", "R1", "R2", "R3", "R4", "R5")
CLASS_ORDER = ("public", "internal", "confidential", "restricted")
MAX_CHALLENGE_ROUNDS = 2
MAX_DEPTH = 32  # a schema-valid envelope nests about six deep; deeper input is rejected before validation
REQUIRED_COUNTERFACTUALS = frozenset({"existing-process", "conventional-automation", "deterministic-workflow", "single-agent"})
COST_RANGES = ("annual_run_cost", "annual_oversight_cost", "annualized_implementation_cost", "expected_risk_loss")
# Catalog execution tiers an agent may hold to receive board work (committed data, so deterministic).
DISPATCHABLE_TIERS = frozenset({"static-review", "read-only-runtime"})
PROMOTED_STATES = frozenset({"limited-production", "production"})
BUDGET_DIMENSIONS = ("tool_calls", "tokens", "cost_minor")
PAYLOAD_BY_TYPE = {
    "delegate": None,
    "return": "verdict",
    "challenge": "challenge",
    "escalate": "escalation",
    "decision-request": "decision_request",
    "decision-record": "decision_record",
}
PAYLOAD_KEYS = frozenset(k for k in PAYLOAD_BY_TYPE.values() if k)

# Designated challenger per decision class: (primary, fallback when the primary proposed).
DESIGNATED_CHALLENGERS = {
    "architecture": (SEC, ARCH),
    "security": (ARCH, SEC),
    "automation-investment": (FIN, FIN),
}

# Canary for mutation intent in a delegate's objective. Objectives must be phrased as review
# questions ("Assess whether ..."); one that opens with a mutation verb is a request to act,
# which this static board never dispatches. This is a DENYLIST that runs only in this offline
# gate: it fails OPEN on phrasings it does not list ("Can you deploy ...", "We need to delete
# ..."), and it can false-stop a noun-led title. It is not runtime enforcement and not a
# semantic intent classifier; agents apply a semantic stop rule, never this regex.
_LEAD = r"^\s*(?:(?:urgent(?:ly)?|asap|now|please|immediately|today|tonight|just|go\s+ahead\s+and|quickly)\b[\s:,!.-]*)*"
_MUTATION_VERBS = (
    r"deploy|roll\s?out|release|promote|ship|delete|destroy|purge|drop|truncate|wipe|grant|revoke|"
    r"assign|pay|refund|transfer|wire|post|book|send|e-?mail|notify|publish|approve|execute|run|apply|"
    r"merge|push|rotate|disable|enable|terminate|shut\s?down|restart|scale|migrate|provision|issue|sign|"
    r"create|update|modify|change|remove|install|uninstall|invoke|trigger|submit|cancel|close"
)
LIVE_GUARD_INTENT = re.compile(_LEAD + r"(?:" + _MUTATION_VERBS + r")\b", re.IGNORECASE)
# Canary for instruction-override bait in an objective (OWASP ASI01 Agent Goal Hijack object).
# Same caveat: a denylist with low recall that fails open; tests pin its known misses.
INSTRUCTION_BAIT = re.compile(
    r"\b(?:ignore|disregard|override|bypass|forget)\s+(?:all\s+|any\s+|the\s+|your\s+)?"
    r"(?:previous\s+|prior\s+|above\s+|earlier\s+)?(?:rules|instructions|guardrails|policies|gates|do-not-do\s+list)\b"
    r"|\b(?:already\s+approved|pre-?approved|approval\s+(?:was\s+)?(?:already\s+)?(?:given|granted))\b",
    re.IGNORECASE,
)

RULES: dict[str, str] = {
    # --- single envelope -------------------------------------------------------------------
    "HO-SCHEMA": "Envelope matches schemas/agentic-handoff.schema.json (shape only).",
    "HO-ROOT-PARENT": "parent_task_id is null if and only if hop_count is 0.",
    "HO-NO-SELF-ADDRESS": "source and destination are different parties.",
    "HO-NO-EXECUTION": "authorized_actions is a subset of read/analyze/recommend/challenge/escalate.",
    "HO-AGENT-NO-APPROVAL": "An agent-sourced envelope carries approval_state not-required or pending-human-review.",
    "HO-APPROVAL-ONLY-ON-RECORD": "A principal-sourced envelope other than a decision-record carries approval_state not-required or pending-human-review.",
    "HO-ESCALATE-TO-PRINCIPAL": "An escalate envelope, and any envelope with status escalated, is addressed to a principal.",
    "HO-PAYLOAD-RETURN": "A return envelope carries a verdict.",
    "HO-PAYLOAD-CHALLENGE": "A challenge envelope carries a challenge.",
    "HO-PAYLOAD-ESCALATE": "An escalate envelope carries an escalation.",
    "HO-PAYLOAD-DECISION-REQUEST": "A decision-request envelope carries a decision_request.",
    "HO-PAYLOAD-DECISION-RECORD": "A decision-record envelope carries a decision_record.",
    "HO-PAYLOAD-EXCLUSIVE": "An envelope carries no payload belonging to another message_type.",
    "HO-STOPPED-HAS-REASON": "status stopped requires at least one stop reason.",
    "HO-REASON-HAS-STATUS": "Any stop reason requires status stopped or escalated.",
    "HO-STOP-NOTE": "Declaring conflicting-instructions requires a stop_note.",
    "HO-BLOCK-AS-CHALLENGE": "A return addressed to an agent never carries verdict block: an agent's adverse finding on another agent's work travels as a challenge.",
    "HO-RETURN-EVIDENCE": "A return whose verdict is not block cites at least one evidence_ref not labelled inference and lists no blocking missing evidence.",
    # --- record set ---------------------------------------------------------------------------
    "HO-UNIQUE-TASK-ID": "task_id is unique within the record set.",
    "HO-ONE-CORRELATION": "All envelopes in the record set share one correlation_id.",
    "HO-ONE-ROOT": "Exactly one envelope in the record set has hop_count 0.",
    # --- chain: parent linkage, attenuation, monotonicity --------------------------------------
    "HO-CHAIN-PARENT": "A non-root envelope's parent is present and hop_count equals the parent's plus one.",
    "HO-CHAIN-TIME": "An envelope is not sent before its parent.",
    "HO-CHAIN-CUSTODY": "An envelope is sent by its parent's destination, or by its parent's sender when the parent was stopped.",
    "HO-RETURN-TO-SENDER": "A return goes back up its chain: to its parent's sender, or to the sender of an earlier delegate in its chain that was addressed to the return's sender.",
    "HO-ATTEN-ACTIONS": "authorized_actions never widen from parent to child.",
    "HO-ATTEN-BUDGET": "Budget limits never increase from parent to child, and the currency never changes.",
    "HO-MONO-TIER": "subject_risk_tier never decreases from parent to child.",
    "HO-MONO-CLASS": "data_classification never decreases from parent to child.",
    "HO-MONO-MATERIALITY": "materiality_flags never lose a flag from parent to child.",
    "HO-MONO-PROHIBITIONS": "do_not_do_list never loses an item from parent to child.",
    "HO-MONO-DEADLINE": "deadline never extends from parent to child.",
    "HO-MONO-MAX-HOPS": "max_hops never increases from parent to child.",
    "HO-MONO-SPENT": "Budget spent counters are cumulative: they never decrease from parent to child.",
    "HO-DESTINATION-TIER": "Every agent destination is a catalog agent whose execution_tier is static-review or read-only-runtime.",
    # --- computed stop conditions -------------------------------------------------------------
    "STOP-DEPTH": "A delegate at hop_count >= max_hops declares delegation-depth-exceeded and is stopped/escalated.",
    "STOP-DEADLINE": "An envelope sent after its deadline declares deadline-passed and is stopped/escalated.",
    "STOP-BUDGET": "Exhausted budget (delegate: spent >= limit; other: spent > limit) declares budget-exhausted and is stopped/escalated.",
    "STOP-MISSING-EVIDENCE": "A delegate or decision-request with blocking missing evidence declares missing-evidence and is stopped/escalated.",
    "STOP-LIVE-GUARD": "A delegate whose objective opens with a mutation verb declares live-guard-intent and is stopped/escalated.",
    "STOP-INSTRUCTION-BAIT": "An objective carrying instruction-override or claimed-approval bait declares conflicting-instructions and is stopped/escalated.",
    "STOP-ROUTING-LOOP": "An agent-sourced delegate to an agent already delegated to earlier in its own chain (a stopped delegate did no work and does not count) declares routing-loop and is stopped/escalated, unless it is the one revisit: the destination was delegated to exactly once before in that chain and has challenged the sender's work there.",
    "STOP-CHALLENGE-ROUNDS": "A challenge beyond round 2 declares challenge-rounds-exceeded and is stopped/escalated.",
    "STOP-TERMINAL": "Every child of a stopped envelope is an escalate (which HO-ESCALATE-TO-PRINCIPAL sends to a principal), and no decision-record binds a decision-request that declares a stop reason.",
    # --- challenges and decision requests -----------------------------------------------------
    "CH-PROPOSAL-LINK": "A challenge's proposal_task_id resolves to an envelope in the record set that precedes the challenge: a challenge answers something already sent.",
    "CH-ROUND-COUNTED": "A challenge's round equals the number of delegates, returns and challenges (not stopped) that the proposer of its target had sent to the challenger before it, at least 1. Rounds bound the exchange; they never clear a finding.",
    "DR-TO-PRINCIPAL": "A decision-request is addressed to a principal; agents never decide.",
    "DR-PROPOSER-IS-SENDER": "A decision-request is sent by the agent it names as proposer_agent (no attributing a proposal to another agent).",
    "DR-CHALLENGE-TARGETS-PROPOSAL": "Every attached challenge that is not a HARD gate points at an envelope sent by the proposer; a HARD gate is attached wherever it was raised.",
    "DR-CHALLENGE-RESOLVES": "Every challenge_task_ids entry resolves to a challenge envelope in the record set.",
    "DR-CHALLENGE-PRECEDES": "No attached challenge was sent after the decision-request or descends from it.",
    "DR-CHALLENGE-COMPLETE": "Every challenge on the proposer's envelopes, and every HARD-gate challenge anywhere in the correlation, that precedes the request is attached; for the request in force, so is every such challenge that is not causally after every decision-record binding it, the co-signing record included.",
    "DR-RISK-CARRIED": "A decision-request's subject_risk_tier is at least, and its materiality_flags include, those of every envelope in the correlation not causally after it; for the request in force, not causally after every decision-record binding it (those records aside). The decision carries the highest risk declared before it, wherever it was declared.",
    "DR-ONE-REQUEST": "A correlation has one decision-request in force: any other supersedes a decision-request from the same proposer sent no later than itself, no request is superseded twice, and supersession has no cycle.",
    "DR-OWNER-FROM-ROOT": "A correlation carrying a decision-request is opened by a principal whose root names a decision_owner other than itself, and every decision-request is addressed to that owner.",
    "DR-CLASS-FROM-ROOT": "A decision-request's decision_class equals the decision_class declared on the root envelope.",
    "DR-CHALLENGE-REQUIRED": "At R2+, with any materiality flag, or for automation-investment, at least one challenge is attached.",
    "DR-DESIGNATED-CHALLENGER": "When a challenge is required, the designated challenger for the decision class sent a completed challenge of the proposer's own envelopes.",
    "DR-PROPOSER-NOT-CHALLENGER": "No attached challenge was raised by the proposer.",
    "DR-ALTERNATIVES": "When a challenge is required, at least one materially different alternative is recorded.",
    "DR-UNRESOLVED-DISSENT": "Every attached challenge whose outcome is sustained or unresolved has a dissent entry; no agent, the challenger included, clears it by a later withdrawal.",
    "DR-NO-APPROVE-OVER-DISSENT": "With any attached challenge sustained or unresolved, recommended_outcome is not approve.",
    "DR-HARD-GATE": "With any attached HARD-gate challenge sustained or unresolved, recommended_outcome is reject, redesign, or escalate.",
    "DR-VALUE-CASE-REQUIRED": "An automation-investment decision-request carries a value_case.",
    # --- decision records (principal-sourced) -------------------------------------------------
    "DRC-PRINCIPAL-SOURCE": "A decision-record is sent by a principal.",
    "DRC-DECIDER-IS-SENDER": "decision_record.decided_by is the record's own sender.",
    "DRC-REQUEST-BINDING": "request_task_id resolves to a decision-request and request_digest is the sha256 of its canonical JSON.",
    "DRC-NOT-REQUESTER": "Neither the decider nor a named second approver is the principal who opened the correlation.",
    "DRC-DECIDER-IS-ADDRESSEE": "A decider's record is decided by the principal the decision-request was addressed to.",
    "DRC-AFTER-REQUEST": "A decision-record is not sent before the decision-request it binds, and that request does not descend from it.",
    "DRC-ONE-RECORD": "A decision-request has at most one decider's record, and each decider's record at most one co-signing record.",
    "DRC-COMPLETED": "A decision-record has status completed.",
    "DRC-SECOND-APPROVER": "A decider's record sent at R5, or binding an R5 decision-request, names a second approver distinct from the decider.",
    "DRC-CO-SIGNED": "The decider's record on the decision-request in force, when it names a second approver, is co-signed by that principal's own decision-record.",
    "DRC-CO-SIGN-BINDING": "A co-signing record names a decider's record on the same request with the same digest and outcome, comes from that record's second approver, descends from it in the parent chain, and names no second approver itself.",
    "DRC-HARD-GATE-ACCEPTANCE": "Approving a request with any attached HARD-gate challenge sustained or unresolved requires risk_acceptance_ref; only a principal lifts a HARD gate.",
    "DRC-APPROVAL-STATE": "approval_state is approved for approve/conditional-approve and rejected for reject/redesign.",
    # --- value case ---------------------------------------------------------------------------
    "VC-ORDER": "Every money range satisfies low <= point <= high.",
    "VC-COST-SIGN": "No cost range (run, oversight, implementation, risk loss) is negative.",
    "VC-NAV-ARITHMETIC": "net_annual_value equals benefit minus run, oversight, implementation and risk loss under interval arithmetic.",
    "VC-BENEFIT-CEILING": "verified_annual_benefit.high does not exceed baseline labor/error cost plus baseline loss/leakage.",
    "VC-OVERSIGHT": "A non-zero intervention or exception rate requires annual_oversight_cost.low > 0.",
    "VC-RUNS-BOUND": "successful_runs does not exceed runs.",
    "VC-UNIT-COST": "cost_per_successful_task_minor equals floor(annual_run_cost.point / successful_runs), or 0 with no successful runs.",
    "VC-COUNTERFACTUALS": "Counterfactuals include existing-process, conventional-automation, deterministic-workflow and single-agent.",
    "VC-PROMOTION": "limited-production/production requires measured confidence and net_annual_value.low above every counterfactual's low.",
    "VC-SYNTHETIC": "A synthetic record never claims measured confidence or promotion beyond pilot.",
    "VC-RECOMMEND-INCREMENTAL": "Recommending approve/conditional-approve on an investment requires net_annual_value.point above every counterfactual's point.",
}

_validator = None
_tiers: dict | None = None
_ecma_cache: dict[str, re.Pattern] = {}


def _ecma_pattern(validator, pattern, instance, schema):
    """JSON Schema ``pattern`` with ECMA-262 semantics: a final ``$`` matches only at the end."""
    if not validator.is_type(instance, "string"):
        return
    compiled = _ecma_cache.get(pattern)
    if compiled is None:
        src = pattern[:-1] + r"\Z" if pattern.endswith("$") and not pattern.endswith(r"\$") else pattern
        compiled = _ecma_cache[pattern] = re.compile(src, re.ASCII)
    if not compiled.search(instance):
        yield jsonschema.ValidationError(f"{instance!r} does not match {pattern!r}")


def _schema_validator():
    global _validator
    if _validator is None:
        schema = json.loads(SCHEMA_PATH.read_text(encoding="utf-8"))
        strict = jsonschema.validators.extend(jsonschema.Draft202012Validator, {"pattern": _ecma_pattern})
        _validator = strict(schema)
    return _validator


def _catalog_tiers() -> dict:
    global _tiers
    if _tiers is None:
        _tiers = {a["id"]: a.get("execution_tier") for a in json.loads(CATALOG_PATH.read_text(encoding="utf-8"))}
    return _tiers


def _encodable(env) -> bool:
    """A lone surrogate survives json.loads but cannot be digested; treat it as a shape error."""
    try:
        json.dumps(env, ensure_ascii=False).encode("utf-8")
    except (UnicodeEncodeError, TypeError, ValueError):
        return False
    return True


def _depth(value) -> int:
    """Nesting depth, computed without recursion so hostile input cannot exhaust the stack."""
    deepest, stack = 0, [(value, 1)]
    while stack:
        cur, d = stack.pop()
        if isinstance(cur, (dict, list)):
            deepest = max(deepest, d)
            if d <= MAX_DEPTH:
                stack.extend((v, d + 1) for v in (cur.values() if isinstance(cur, dict) else cur))
    return deepest


def canonical_digest(envelope: dict) -> str:
    """sha256 over canonical JSON: sorted keys, compact separators, UTF-8."""
    blob = json.dumps(envelope, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode("utf-8")
    return "sha256:" + hashlib.sha256(blob).hexdigest()


def _party_key(party: dict) -> tuple:
    return ("agent", party.get("id")) if party.get("kind") == "agent" else ("principal", party.get("subject"))


def _tier(t: str) -> int:
    return TIER_ORDER.index(t)


def _cls(c: str) -> int:
    return CLASS_ORDER.index(c)


def _requires_challenge(env: dict, req: dict) -> bool:
    return (
        _tier(env["subject_risk_tier"]) >= _tier("R2")
        or bool(env["materiality_flags"])
        or req["decision_class"] == "automation-investment"
    )


def _budget_exhausted(env: dict) -> bool:
    b = env["budget"]
    if env["message_type"] == "delegate":
        return any(b[f"spent_{d}"] >= b[f"limit_{d}"] for d in BUDGET_DIMENSIONS)
    return any(b[f"spent_{d}"] > b[f"limit_{d}"] for d in BUDGET_DIMENSIONS)


def _stopped(env: dict, reason: str) -> bool:
    return reason in env["stop_reasons"] and env["status"] in ("stopped", "escalated")


def _money_ranges(vc: dict):
    for key in ("verified_annual_benefit", "annual_run_cost", "annual_oversight_cost",
                "annualized_implementation_cost", "expected_risk_loss", "net_annual_value"):
        yield vc[key]
    for cf in vc["counterfactuals"]:
        yield cf["net_annual_value"]


def _check_value_case(env: dict, vc: dict, recommended: str | None) -> set[str]:
    out: set[str] = set()
    if any(not (r["low"] <= r["point"] <= r["high"]) for r in _money_ranges(vc)):
        out.add("VC-ORDER")
    costs = COST_RANGES
    if any(min(vc[c]["low"], vc[c]["point"], vc[c]["high"]) < 0 for c in costs):
        out.add("VC-COST-SIGN")
    ben, nav = vc["verified_annual_benefit"], vc["net_annual_value"]
    exp_low = ben["low"] - sum(vc[c]["high"] for c in costs)
    exp_point = ben["point"] - sum(vc[c]["point"] for c in costs)
    exp_high = ben["high"] - sum(vc[c]["low"] for c in costs)
    if (nav["low"], nav["point"], nav["high"]) != (exp_low, exp_point, exp_high):
        out.add("VC-NAV-ARITHMETIC")
    if ben["high"] > vc["baseline_labor_error_cost"] + vc["baseline_loss_leakage"]:
        out.add("VC-BENEFIT-CEILING")
    if (vc["intervention_rate_bp"] > 0 or vc["exception_rate_bp"] > 0) and vc["annual_oversight_cost"]["low"] <= 0:
        out.add("VC-OVERSIGHT")
    if vc["successful_runs"] > vc["runs"]:
        out.add("VC-RUNS-BOUND")
    expected_unit = vc["annual_run_cost"]["point"] // vc["successful_runs"] if vc["successful_runs"] > 0 else 0
    if vc["cost_per_successful_task_minor"] != expected_unit:
        out.add("VC-UNIT-COST")
    kinds = {cf["kind"] for cf in vc["counterfactuals"]}
    if not REQUIRED_COUNTERFACTUALS <= kinds:
        out.add("VC-COUNTERFACTUALS")
    cf_lows = [cf["net_annual_value"]["low"] for cf in vc["counterfactuals"]]
    cf_points = [cf["net_annual_value"]["point"] for cf in vc["counterfactuals"]]
    if vc["promotion_state"] in PROMOTED_STATES and (
        vc["confidence"] != "measured" or (cf_lows and nav["low"] <= max(cf_lows))
    ):
        out.add("VC-PROMOTION")
    if env["synthetic"] and (vc["confidence"] == "measured" or vc["promotion_state"] in PROMOTED_STATES):
        out.add("VC-SYNTHETIC")
    if recommended in ("approve", "conditional-approve") and cf_points and nav["point"] <= max(cf_points):
        out.add("VC-RECOMMEND-INCREMENTAL")
    return out


def _check_envelope(env: dict) -> set[str]:
    """Rules that read one schema-valid envelope."""
    out: set[str] = set()
    mtype, src, dst = env["message_type"], env["source"], env["destination"]
    if (env["parent_task_id"] is None) != (env["hop_count"] == 0):
        out.add("HO-ROOT-PARENT")
    if _party_key(src) == _party_key(dst):
        out.add("HO-NO-SELF-ADDRESS")
    if not set(env["authorized_actions"]) <= BOARD_ACTIONS:
        out.add("HO-NO-EXECUTION")
    if src["kind"] == "agent" and env["approval_state"] not in ("not-required", "pending-human-review"):
        out.add("HO-AGENT-NO-APPROVAL")
    if (src["kind"] == "principal" and mtype != "decision-record"
            and env["approval_state"] not in ("not-required", "pending-human-review")):
        out.add("HO-APPROVAL-ONLY-ON-RECORD")
    if (mtype == "escalate" or env["status"] == "escalated") and dst["kind"] != "principal":
        out.add("HO-ESCALATE-TO-PRINCIPAL")
    if mtype == "return" and "verdict" in env:
        if dst["kind"] == "agent" and env["verdict"]["outcome"] == "block":
            out.add("HO-BLOCK-AS-CHALLENGE")
        if env["verdict"]["outcome"] != "block" and (
            not any(r["label"] != "inference" for r in env["evidence_refs"]) or any(g["blocking"] for g in env["missing_evidence"])
        ):
            out.add("HO-RETURN-EVIDENCE")
    if dst["kind"] == "agent" and _catalog_tiers().get(dst["id"]) not in DISPATCHABLE_TIERS:
        out.add("HO-DESTINATION-TIER")
    own = PAYLOAD_BY_TYPE[mtype]
    for t, key in PAYLOAD_BY_TYPE.items():
        if key and t == mtype and key not in env:
            out.add("HO-PAYLOAD-" + t.upper())
    if any(k in env for k in PAYLOAD_KEYS - ({own} if own else set())):
        out.add("HO-PAYLOAD-EXCLUSIVE")
    if env["status"] == "stopped" and not env["stop_reasons"]:
        out.add("HO-STOPPED-HAS-REASON")
    if env["stop_reasons"] and env["status"] not in ("stopped", "escalated"):
        out.add("HO-REASON-HAS-STATUS")
    if "conflicting-instructions" in env["stop_reasons"] and "stop_note" not in env:
        out.add("HO-STOP-NOTE")
    # computed stop conditions
    if mtype == "delegate" and env["hop_count"] >= env["max_hops"] and not _stopped(env, "delegation-depth-exceeded"):
        out.add("STOP-DEPTH")
    if env["sent_at"] > env["deadline"] and not _stopped(env, "deadline-passed"):
        out.add("STOP-DEADLINE")
    if _budget_exhausted(env) and not _stopped(env, "budget-exhausted"):
        out.add("STOP-BUDGET")
    if (mtype in ("delegate", "decision-request") and any(m["blocking"] for m in env["missing_evidence"])
            and not _stopped(env, "missing-evidence")):
        out.add("STOP-MISSING-EVIDENCE")
    if mtype == "delegate" and LIVE_GUARD_INTENT.search(env["objective"]) and not _stopped(env, "live-guard-intent"):
        out.add("STOP-LIVE-GUARD")
    if INSTRUCTION_BAIT.search(env["objective"]) and not _stopped(env, "conflicting-instructions"):
        out.add("STOP-INSTRUCTION-BAIT")
    if (mtype == "challenge" and "challenge" in env and env["challenge"]["round"] > MAX_CHALLENGE_ROUNDS
            and not _stopped(env, "challenge-rounds-exceeded")):
        out.add("STOP-CHALLENGE-ROUNDS")
    if mtype == "decision-request" and "decision_request" in env:
        req = env["decision_request"]
        if dst["kind"] != "principal":
            out.add("DR-TO-PRINCIPAL")
        if src["kind"] != "agent" or src["id"] != req["proposer_agent"]:
            out.add("DR-PROPOSER-IS-SENDER")
        if req["decision_class"] == "automation-investment" and "value_case" not in req:
            out.add("DR-VALUE-CASE-REQUIRED")
        if _requires_challenge(env, req) and not req["alternatives"]:
            out.add("DR-ALTERNATIVES")
        if "value_case" in req:
            out |= _check_value_case(env, req["value_case"], req["recommended_outcome"])
    if mtype == "decision-record" and "decision_record" in env:
        rec = env["decision_record"]
        if src["kind"] != "principal":
            out.add("DRC-PRINCIPAL-SOURCE")
        else:
            if rec["decided_by"]["subject"] != src["subject"]:
                out.add("DRC-DECIDER-IS-SENDER")
            approved = rec["outcome"] in ("approve", "conditional-approve")
            if env["approval_state"] != ("approved" if approved else "rejected"):
                out.add("DRC-APPROVAL-STATE")
        if env["status"] != "completed":
            out.add("DRC-COMPLETED")
        if env["subject_risk_tier"] == "R5" and "co_signs_task_id" not in rec and (
            "second_approver" not in rec or rec["second_approver"]["subject"] == rec["decided_by"]["subject"]
        ):
            out.add("DRC-SECOND-APPROVER")
    return out


def _ancestors(env: dict, by_id: dict) -> list[dict]:
    chain, seen = [], set()
    cur = by_id.get(env["parent_task_id"]) if env["parent_task_id"] else None
    while cur is not None and cur["task_id"] not in seen:
        seen.add(cur["task_id"])
        chain.append(cur)
        cur = by_id.get(cur["parent_task_id"]) if cur["parent_task_id"] else None
    return chain


class _Order:
    """Causal order over one record set: causal edges first, whole-second sent_at otherwise.

    The edges are parent links plus bindings: a decision-record comes after the request it binds,
    and a co-signing record after the record it co-signs. a precedes b when a reaches b through
    those edges, or a was sent in an earlier second. Two envelopes sent in the same second with
    no path between them are unordered: no rule may treat either one as later, so a task_id
    string never decides which round came last.
    """

    def __init__(self, by_id: dict):
        self.by_id = by_id
        self._anc: dict[str, frozenset] = {}

    def _preds(self, env: dict) -> list[str]:
        preds = [env["parent_task_id"]] if env["parent_task_id"] else []
        rec = env.get("decision_record") if env["message_type"] == "decision-record" else None
        if isinstance(rec, dict):
            preds.append(rec["request_task_id"])
            if "co_signs_task_id" in rec:
                preds.append(rec["co_signs_task_id"])
        return [t for t in preds if t in self.by_id]

    def ancestors(self, env: dict) -> frozenset:
        key = env["task_id"]
        if key not in self._anc:
            seen: set[str] = set()
            todo = self._preds(env)
            while todo:  # iterative and cycle-safe: a malformed record set cannot recurse or loop
                t = todo.pop()
                if t not in seen:
                    seen.add(t)
                    todo.extend(self._preds(self.by_id[t]))
            self._anc[key] = frozenset(seen)
        return self._anc[key]

    def precedes(self, a: dict, b: dict) -> bool:
        return a["task_id"] in self.ancestors(b) or a["sent_at"] < b["sent_at"]


def _check_chain(env: dict, parent: dict) -> set[str]:
    out: set[str] = set()
    if env["sent_at"] < parent["sent_at"]:
        out.add("HO-CHAIN-TIME")
    sender = _party_key(env["source"])
    if sender != _party_key(parent["destination"]) and not (
        parent["status"] == "stopped" and sender == _party_key(parent["source"])
    ):
        out.add("HO-CHAIN-CUSTODY")
    if not set(env["authorized_actions"]) <= set(parent["authorized_actions"]):
        out.add("HO-ATTEN-ACTIONS")
    cb, pb = env["budget"], parent["budget"]
    if cb["currency"] != pb["currency"] or any(cb[f"limit_{d}"] > pb[f"limit_{d}"] for d in BUDGET_DIMENSIONS):
        out.add("HO-ATTEN-BUDGET")
    if _tier(env["subject_risk_tier"]) < _tier(parent["subject_risk_tier"]):
        out.add("HO-MONO-TIER")
    if _cls(env["data_classification"]) < _cls(parent["data_classification"]):
        out.add("HO-MONO-CLASS")
    if not set(parent["materiality_flags"]) <= set(env["materiality_flags"]):
        out.add("HO-MONO-MATERIALITY")
    if not set(parent["do_not_do_list"]) <= set(env["do_not_do_list"]):
        out.add("HO-MONO-PROHIBITIONS")
    if env["deadline"] > parent["deadline"]:
        out.add("HO-MONO-DEADLINE")
    if env["max_hops"] > parent["max_hops"]:
        out.add("HO-MONO-MAX-HOPS")
    if any(cb[f"spent_{d}"] < pb[f"spent_{d}"] for d in BUDGET_DIMENSIONS):
        out.add("HO-MONO-SPENT")
    if parent["status"] == "stopped" and env["message_type"] != "escalate":
        out.add("STOP-TERMINAL")
    return out


def _counted_rounds(challenges: list[dict], by_id: dict, order: _Order) -> dict[str, int]:
    """A round is one proposal version received: the delegates and returns that the proposer of a
    challenge's target had delivered to the challenger before the challenge (at least 1).

    The challenger cannot raise its own round by naming other envelopes, its own, ones it never
    received or another proposer's; only the proposer delivering a revision moves it, and the
    round cap bounds how often that can happen.
    """
    delivered: dict[tuple, list[dict]] = {}
    for e in by_id.values():
        if e["message_type"] in ("delegate", "return", "challenge") and e["status"] != "stopped":  # a stopped one was never delivered
            delivered.setdefault((_party_key(e["source"]), _party_key(e["destination"])), []).append(e)
    rounds: dict[str, int] = {}
    for c in challenges:
        target = by_id.get(c["challenge"]["proposal_task_id"])
        if target is not None:
            received = delivered.get((_party_key(target["source"]), _party_key(c["source"])), [])
            rounds[c["task_id"]] = max(1, sum(1 for e in received if order.precedes(e, c)))
    return rounds


def _routing_loop(env: dict, by_id: dict) -> bool:
    """An agent re-delegating to an agent already in its chain, beyond the one bounded revisit."""
    if env["message_type"] != "delegate" or env["destination"]["kind"] != "agent" or env["source"]["kind"] != "agent":
        return False  # a principal re-scoping the work is a human decision, not an agent loop
    ancestors = _ancestors(env, by_id)
    dest = env["destination"]["id"]
    # A stopped delegate did no work, so after a principal cures the stop the agent may dispatch again.
    earlier = [a for a in ancestors if a["message_type"] == "delegate" and a["destination"].get("id") == dest
               and a["status"] != "stopped"]
    if not earlier:
        return False
    # Round 2: the proposer takes its revised proposal back to a challenger that challenged its
    # work. Counting delegates, not the targets a challenger names, allows that revisit exactly
    # once (MAX_CHALLENGE_ROUNDS exchanges in all), never repeatedly.
    challenged_sender = any(
        a["message_type"] == "challenge" and "challenge" in a and _party_key(a["source"]) == ("agent", dest)
        and a["challenge"]["proposal_task_id"] in by_id
        and _party_key(by_id[a["challenge"]["proposal_task_id"]]["source"]) == _party_key(env["source"])
        for a in ancestors)
    return not (challenged_sender and len(earlier) < MAX_CHALLENGE_ROUNDS)


def check_record_set(doc) -> list[str]:
    """Return the sorted list of violated rule ids for one record set."""
    messages = doc.get("messages") if isinstance(doc, dict) else None
    if not isinstance(messages, list) or not messages:
        return ["HO-SCHEMA"]
    validator = _schema_validator()
    violations: set[str] = set()
    valid: list[dict] = []
    for env in messages:
        if not isinstance(env, dict) or _depth(env) > MAX_DEPTH or not _encodable(env) or list(validator.iter_errors(env)):
            violations.add("HO-SCHEMA")
        else:
            valid.append(env)
    if not valid:
        return sorted(violations)

    ids = [e["task_id"] for e in valid]
    if len(ids) != len(set(ids)):
        violations.add("HO-UNIQUE-TASK-ID")
    if len({e["correlation_id"] for e in valid}) != 1:
        violations.add("HO-ONE-CORRELATION")
    roots = [e for e in valid if e["hop_count"] == 0]
    if len(roots) != 1:
        violations.add("HO-ONE-ROOT")
    by_id = {e["task_id"]: e for e in valid}
    root = roots[0] if len(roots) == 1 else None
    order = _Order(by_id)

    challenges = [e for e in valid if e["message_type"] == "challenge" and "challenge" in e]
    requests = [e for e in valid if e["message_type"] == "decision-request" and "decision_request" in e]
    records = [e for e in valid if e["message_type"] == "decision-record" and "decision_record" in e]
    rounds = _counted_rounds(challenges, by_id, order)
    superseded = {r["decision_request"]["supersedes_task_id"] for r in requests if "supersedes_task_id" in r["decision_request"]}
    ctx = {"by_id": by_id, "order": order, "superseded": superseded, "challenges": challenges,
           "records": records, "root": root, "live": {},
           "cosigners": {(r["decision_record"]["co_signs_task_id"], r["decision_record"]["decided_by"]["subject"])
                         for r in records if "co_signs_task_id" in r["decision_record"]}}

    for env in valid:
        violations |= _check_envelope(env)
        parent = by_id.get(env["parent_task_id"]) if env["parent_task_id"] else None
        if env["hop_count"] > 0:
            if parent is None or env["hop_count"] != parent["hop_count"] + 1:
                violations.add("HO-CHAIN-PARENT")
            if parent is not None:
                violations |= _check_chain(env, parent)
        if _routing_loop(env, by_id) and not _stopped(env, "routing-loop"):
            violations.add("STOP-ROUTING-LOOP")
        if env["message_type"] == "return" and parent is not None:
            # Back up the chain: to the parent's sender, or to whoever delegated to this sender earlier.
            askers = {_party_key(parent["source"])} | {
                _party_key(a["source"]) for a in _ancestors(env, by_id)
                if a["message_type"] == "delegate" and _party_key(a["destination"]) == _party_key(env["source"])}
            if _party_key(env["destination"]) not in askers:
                violations.add("HO-RETURN-TO-SENDER")
        if env["message_type"] == "challenge" and "challenge" in env:
            target = by_id.get(env["challenge"]["proposal_task_id"])
            if target is None or not order.precedes(target, env):
                violations.add("CH-PROPOSAL-LINK")
            elif env["challenge"]["round"] != rounds[env["task_id"]]:
                violations.add("CH-ROUND-COUNTED")
        if env["message_type"] == "decision-request" and "decision_request" in env:
            violations |= _check_decision_request_links(env, ctx)
            if root is not None and root.get("decision_class") != env["decision_request"]["decision_class"]:
                violations.add("DR-CLASS-FROM-ROOT")
            if root is not None and (
                root["source"]["kind"] != "principal" or "decision_owner" not in root
                or root["decision_owner"]["subject"] == root["source"]["subject"]
                or (env["destination"]["kind"] == "principal"  # an agent addressee is DR-TO-PRINCIPAL's
                    and env["destination"]["subject"] != root["decision_owner"]["subject"])
            ):
                violations.add("DR-OWNER-FROM-ROOT")
        if env["message_type"] == "decision-record" and "decision_record" in env:
            violations |= _check_decision_record_links(env, ctx)

    violations |= _check_supersession(requests, by_id, order)

    # One decider's record per request, one co-signature per decider's record.
    primaries = Counter(r["decision_record"]["request_task_id"] for r in records if "co_signs_task_id" not in r["decision_record"])
    cosigns = Counter(r["decision_record"]["co_signs_task_id"] for r in records if "co_signs_task_id" in r["decision_record"])
    if any(n > 1 for n in primaries.values()) or any(n > 1 for n in cosigns.values()):
        violations.add("DRC-ONE-RECORD")
    return sorted(violations)


def _check_supersession(requests: list[dict], by_id: dict, order: _Order) -> set[str]:
    """One decision-request in force: supersession forms one chain back to the original request."""
    if len([r for r in requests if "supersedes_task_id" not in r["decision_request"]]) > 1:
        return {"DR-ONE-REQUEST"}
    targets = [r["decision_request"]["supersedes_task_id"] for r in requests if "supersedes_task_id" in r["decision_request"]]
    if len(targets) != len(set(targets)):
        return {"DR-ONE-REQUEST"}
    for r in requests:
        if "supersedes_task_id" not in r["decision_request"]:
            continue
        old = by_id.get(r["decision_request"]["supersedes_task_id"])
        if (old is None or old is r or "decision_request" not in old or old["message_type"] != "decision-request"
                or order.precedes(r, old)  # a request supersedes only one sent no later than itself
                or old["decision_request"]["proposer_agent"] != r["decision_request"]["proposer_agent"]):
            return {"DR-ONE-REQUEST"}
        cur = r  # walk back to the original request; a walk longer than the request count is a cycle
        for _ in range(len(requests)):
            nxt = by_id.get(cur["decision_request"]["supersedes_task_id"])
            if nxt is None or "decision_request" not in nxt or "supersedes_task_id" not in nxt["decision_request"]:
                break
            cur = nxt
        else:
            return {"DR-ONE-REQUEST"}
    return set()


def _attached_challenges(req: dict, by_id: dict) -> list[dict]:
    return [
        by_id[t] for t in req["challenge_task_ids"]
        if t in by_id and by_id[t]["message_type"] == "challenge" and "challenge" in by_id[t]
    ]


def _open(ch: dict) -> bool:
    return ch["challenge"]["outcome"] in ("sustained", "unresolved")


def _live(request: dict, ctx: dict) -> list[dict]:
    """Attached challenges whose own outcome is sustained or unresolved.

    No agent clears a challenge, not even its own challenger: a later withdrawal is recorded beside
    the finding, never instead of it. Dissent is preserved, and a HARD gate stays live until a
    principal's decision-record carries risk_acceptance_ref. Every earlier definition of a "later
    round" that let a challenger clear its own finding had a carrier an adversary controlled, so the
    clearing path is gone and rounds only bound how many exchanges happen.
    """
    key = request["task_id"]
    if key not in ctx["live"]:
        ctx["live"][key] = [c for c in _attached_challenges(request["decision_request"], ctx["by_id"]) if _open(c)]
    return ctx["live"][key]


def _on_proposer(ch: dict, proposer: str, by_id: dict) -> bool:
    target = by_id.get(ch["challenge"]["proposal_task_id"])
    return target is not None and target["source"]["kind"] == "agent" and target["source"]["id"] == proposer


def _check_decision_request_links(env: dict, ctx: dict) -> set[str]:
    out: set[str] = set()
    by_id, order = ctx["by_id"], ctx["order"]
    req = env["decision_request"]
    attached = _attached_challenges(req, by_id)
    if any(order.precedes(env, c) for c in attached):
        out.add("DR-CHALLENGE-PRECEDES")
    # Completeness: the principal sees every challenge raised on the proposal before the request.
    # The request in force is also held to every challenge up to its decision, so a HARD gate cannot
    # be dropped by leaving it unattached or by deciding before it arrives; a superseded request is
    # judged as of its own sending, so re-asking with the late challenge attached cures it.
    # A HARD gate is owed wherever it was raised, so a change of proposer cannot leave one behind.
    if env["task_id"] in ctx["superseded"]:
        owed = [c for c in ctx["challenges"] if order.precedes(c, env)]
    else:
        points = [env] + [r for r in ctx["records"] if r["decision_record"]["request_task_id"] == env["task_id"]]
        owed = [c for c in ctx["challenges"] if not all(order.precedes(x, c) for x in points)]
    if any((c["challenge"]["hard_gate"] or _on_proposer(c, req["proposer_agent"], by_id))
           and c["task_id"] not in req["challenge_task_ids"] for c in owed):
        out.add("DR-CHALLENGE-COMPLETE")
    # The decision carries the highest risk declared before it anywhere in the correlation (a same-second
    # envelope counts), so no branch, re-ask or re-tiering can ask a lower-stakes question than the one raised.
    binding = [r for r in ctx["records"] if r["decision_record"]["request_task_id"] == env["task_id"]]
    points = [env] if env["task_id"] in ctx["superseded"] else [env] + binding
    carried = [e for e in by_id.values() if e is not env and not any(e is r for r in binding)
               and not all(order.precedes(x, e) for x in points)]
    if any(_tier(e["subject_risk_tier"]) > _tier(env["subject_risk_tier"])
           or not set(e["materiality_flags"]) <= set(env["materiality_flags"]) for e in carried):
        out.add("DR-RISK-CARRIED")
    if len(attached) != len(req["challenge_task_ids"]):
        out.add("DR-CHALLENGE-RESOLVES")
    if _requires_challenge(env, req) and not req["challenge_task_ids"]:
        out.add("DR-CHALLENGE-REQUIRED")
    elif _requires_challenge(env, req):
        primary, fallback = DESIGNATED_CHALLENGERS[req["decision_class"]]
        designated = fallback if primary == req["proposer_agent"] else primary
        challengers = {c["source"].get("id") for c in attached
                       if c["source"]["kind"] == "agent" and _on_proposer(c, req["proposer_agent"], by_id)
                       and c["status"] == "completed"}
        if designated == req["proposer_agent"] or designated not in challengers:
            out.add("DR-DESIGNATED-CHALLENGER")
    if any(c["source"].get("id") == req["proposer_agent"] for c in attached):
        out.add("DR-PROPOSER-NOT-CHALLENGER")
    for c in attached:
        target = by_id.get(c["challenge"]["proposal_task_id"])
        if (target is not None and not c["challenge"]["hard_gate"]
                and not (target["source"]["kind"] == "agent" and target["source"]["id"] == req["proposer_agent"])):
            out.add("DR-CHALLENGE-TARGETS-PROPOSAL")
    live = _live(env, ctx)
    dissent_ids = {d["challenge_task_id"] for d in req["dissent"]}
    if any(c["task_id"] not in dissent_ids for c in live):
        out.add("DR-UNRESOLVED-DISSENT")
    if live and req["recommended_outcome"] == "approve":
        out.add("DR-NO-APPROVE-OVER-DISSENT")
    if any(c["challenge"]["hard_gate"] for c in live) and req["recommended_outcome"] not in (
        "reject", "redesign", "escalate"
    ):
        out.add("DR-HARD-GATE")
    return out


def _check_decision_record_links(env: dict, ctx: dict) -> set[str]:
    out: set[str] = set()
    by_id, root = ctx["by_id"], ctx["root"]
    rec = env["decision_record"]
    cosign = "co_signs_task_id" in rec
    request = by_id.get(rec["request_task_id"])
    if (request is None or request["message_type"] != "decision-request"
            or rec["request_digest"] != canonical_digest(request)):
        out.add("DRC-REQUEST-BINDING")
    if root is not None and root["source"]["kind"] == "principal" and root["source"]["subject"] in (
        rec["decided_by"]["subject"], rec.get("second_approver", {}).get("subject")
    ):
        out.add("DRC-NOT-REQUESTER")
    if request is not None:
        if env["sent_at"] < request["sent_at"] or env["task_id"] in ctx["order"].ancestors(request):
            out.add("DRC-AFTER-REQUEST")
        if request["stop_reasons"]:  # stopped or escalated: a request that met a stop condition is not decidable
            out.add("STOP-TERMINAL")
    if request is not None and "decision_request" in request and not cosign:
        dst = request["destination"]
        if dst["kind"] == "principal" and rec["decided_by"]["subject"] != dst["subject"]:
            out.add("DRC-DECIDER-IS-ADDRESSEE")
        if request["subject_risk_tier"] == "R5" and (
            "second_approver" not in rec or rec["second_approver"]["subject"] == rec["decided_by"]["subject"]
        ):
            out.add("DRC-SECOND-APPROVER")
        hard_open = any(c["challenge"]["hard_gate"] for c in _live(request, ctx))
        if hard_open and rec["outcome"] in ("approve", "conditional-approve") and "risk_acceptance_ref" not in rec:
            out.add("DRC-HARD-GATE-ACCEPTANCE")
    if cosign:
        target = by_id.get(rec["co_signs_task_id"])
        trec = target.get("decision_record") if target is not None and target["message_type"] == "decision-record" else None
        if (trec is None or "co_signs_task_id" in trec or "second_approver" in rec
                or (trec["request_task_id"], trec["request_digest"], trec["outcome"])
                != (rec["request_task_id"], rec["request_digest"], rec["outcome"])
                or trec.get("second_approver", {}).get("subject") != rec["decided_by"]["subject"]
                or env["sent_at"] < target["sent_at"] or all(a is not target for a in _ancestors(env, by_id))):
            out.add("DRC-CO-SIGN-BINDING")
    elif ("second_approver" in rec and rec["second_approver"]["subject"] != rec["decided_by"]["subject"]
          and rec["request_task_id"] not in ctx["superseded"]  # a superseded request's record is void, not owed a co-sign
          and (env["task_id"], rec["second_approver"]["subject"]) not in ctx["cosigners"]):
        out.add("DRC-CO-SIGNED")
    return out


def _reject_duplicate_keys(pairs: list) -> dict:
    keys = [k for k, _ in pairs]
    if len(keys) != len(set(keys)):
        raise ValueError("duplicate object key")
    return dict(pairs)


def _reject_constant(name: str):
    raise ValueError(f"{name} is not JSON")


def load_json(path: str):
    """Strict load: duplicate keys, NaN and Infinity are errors (each would make the digest ambiguous).

    Raises ValueError (including JSONDecodeError), UnicodeDecodeError, RecursionError or OSError.
    """
    return json.loads(Path(path).read_text(encoding="utf-8"), object_pairs_hook=_reject_duplicate_keys,
                      parse_constant=_reject_constant)


MALFORMED = (ValueError, UnicodeDecodeError, RecursionError)


def main(argv: list[str]) -> int:
    if not argv or argv == ["--digest"]:
        print(__doc__.strip().splitlines()[-3])
        print(__doc__.strip().splitlines()[-1])
        return 2
    if argv[0] == "--digest":
        for path in argv[1:]:
            try:
                envelope = load_json(path)
            except OSError as exc:
                print(f"{path}: cannot read ({exc.strerror or exc})", file=sys.stderr)
                return 2
            except MALFORMED:
                print(f"{path}: not strict JSON", file=sys.stderr)
                return 2
            if (not isinstance(envelope, dict) or envelope.get("message_type") != "decision-request"
                    or _depth(envelope) > MAX_DEPTH or not _encodable(envelope)
                    or list(_schema_validator().iter_errors(envelope))):
                print(f"{path}: not a schema-valid decision-request envelope", file=sys.stderr)
                return 2
            print(canonical_digest(envelope))
        return 0
    failed, unreadable = False, False
    for path in argv:
        try:
            rules = check_record_set(load_json(path))
        except OSError as exc:
            print(f"{path}: cannot read ({exc.strerror or exc})", file=sys.stderr)
            unreadable = True
            continue
        except MALFORMED:
            rules = ["HO-SCHEMA"]
        for rule in rules:
            failed = True
            print(f"{path}: {rule}")
    return 2 if unreadable else 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
