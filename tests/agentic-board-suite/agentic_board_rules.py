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

CLI (prints one rule id per line, exit 1 on any violation):
    python3 tests/agentic-board-suite/agentic_board_rules.py <record-set.json> [...]
"""
from __future__ import annotations

import hashlib
import json
import re
import sys
from pathlib import Path

import jsonschema

ROOT = Path(__file__).resolve().parents[2]
SCHEMA_PATH = ROOT / "schemas" / "agentic-handoff.schema.json"

ARCH = "agentic-system-architecture-review-agent"
SEC = "agentic-security-review-agent"
FIN = "finance-capital-allocation-advisor-agent"

BOARD_ACTIONS = frozenset({"read", "analyze", "recommend", "challenge", "escalate"})
TIER_ORDER = ("R0", "R1", "R2", "R3", "R4", "R5")
CLASS_ORDER = ("public", "internal", "confidential", "restricted")
MAX_CHALLENGE_ROUNDS = 2
REQUIRED_COUNTERFACTUALS = frozenset({"existing-process", "conventional-automation", "single-agent"})
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

# Fixture oracle for mutation intent in a delegate's objective. Objectives must be phrased as
# review questions ("Assess whether ..."); one that opens with a mutation verb is a request to
# act, which this static board never dispatches. This regex runs only in this offline gate —
# it is NOT runtime enforcement, and it is deliberately broad (fail closed).
_LEAD = r"^\s*(?:(?:urgent(?:ly)?|asap|now|please|immediately|today|tonight|just|go\s+ahead\s+and|quickly)\b[\s:,!.-]*)*"
_MUTATION_VERBS = (
    r"deploy|roll\s?out|release|promote|ship|delete|destroy|purge|drop|truncate|wipe|grant|revoke|"
    r"assign|pay|refund|transfer|wire|post|book|send|e-?mail|notify|publish|approve|execute|run|apply|"
    r"merge|push|rotate|disable|enable|terminate|shut\s?down|restart|scale|migrate|provision|issue|sign|"
    r"create|update|modify|change|remove|install|uninstall|invoke|trigger|submit|cancel|close"
)
LIVE_GUARD_INTENT = re.compile(_LEAD + r"(?:" + _MUTATION_VERBS + r")\b", re.IGNORECASE)
# Instruction-override bait anywhere in an objective (OWASP ASI01 Agent Goal Hijack object).
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
    "HO-ESCALATE-TO-PRINCIPAL": "An escalate envelope is addressed to a principal.",
    "HO-PAYLOAD-RETURN": "A return envelope carries a verdict.",
    "HO-PAYLOAD-CHALLENGE": "A challenge envelope carries a challenge.",
    "HO-PAYLOAD-ESCALATE": "An escalate envelope carries an escalation.",
    "HO-PAYLOAD-DECISION-REQUEST": "A decision-request envelope carries a decision_request.",
    "HO-PAYLOAD-DECISION-RECORD": "A decision-record envelope carries a decision_record.",
    "HO-PAYLOAD-EXCLUSIVE": "An envelope carries no payload belonging to another message_type.",
    "HO-STOPPED-HAS-REASON": "status stopped requires at least one stop reason.",
    "HO-REASON-HAS-STATUS": "Any stop reason requires status stopped or escalated.",
    "HO-STOP-NOTE": "Declaring conflicting-instructions requires a stop_note.",
    # --- record set ---------------------------------------------------------------------------
    "HO-UNIQUE-TASK-ID": "task_id is unique within the record set.",
    "HO-ONE-CORRELATION": "All envelopes in the record set share one correlation_id.",
    "HO-ONE-ROOT": "Exactly one envelope in the record set has hop_count 0.",
    # --- chain: parent linkage, attenuation, monotonicity --------------------------------------
    "HO-CHAIN-PARENT": "A non-root envelope's parent is present and hop_count equals the parent's plus one.",
    "HO-CHAIN-TIME": "An envelope is not sent before its parent.",
    "HO-CHAIN-CUSTODY": "An envelope is sent by its parent's destination, or by its parent's sender when the parent was stopped.",
    "HO-ATTEN-ACTIONS": "authorized_actions never widen from parent to child.",
    "HO-ATTEN-BUDGET": "Budget limits never increase from parent to child, and the currency never changes.",
    "HO-MONO-TIER": "subject_risk_tier never decreases from parent to child.",
    "HO-MONO-CLASS": "data_classification never decreases from parent to child.",
    "HO-MONO-MATERIALITY": "materiality_flags never lose a flag from parent to child.",
    "HO-MONO-PROHIBITIONS": "do_not_do_list never loses an item from parent to child.",
    "HO-MONO-DEADLINE": "deadline never extends from parent to child.",
    "HO-MONO-MAX-HOPS": "max_hops never increases from parent to child.",
    # --- computed stop conditions -------------------------------------------------------------
    "STOP-DEPTH": "A delegate at hop_count >= max_hops declares delegation-depth-exceeded and is stopped/escalated.",
    "STOP-DEADLINE": "An envelope sent after its deadline declares deadline-passed and is stopped/escalated.",
    "STOP-BUDGET": "Exhausted budget (delegate: spent >= limit; other: spent > limit) declares budget-exhausted and is stopped/escalated.",
    "STOP-MISSING-EVIDENCE": "A delegate or decision-request with blocking missing evidence declares missing-evidence and is stopped/escalated.",
    "STOP-LIVE-GUARD": "A delegate whose objective opens with a mutation verb declares live-guard-intent and is stopped/escalated.",
    "STOP-INSTRUCTION-BAIT": "An objective carrying instruction-override or claimed-approval bait declares conflicting-instructions and is stopped/escalated.",
    "STOP-ROUTING-LOOP": "A delegate to an agent already delegated to earlier in its own chain declares routing-loop and is stopped/escalated.",
    "STOP-CHALLENGE-ROUNDS": "A challenge beyond round 2 declares challenge-rounds-exceeded and is stopped/escalated.",
    # --- challenges and decision requests -----------------------------------------------------
    "CH-PROPOSAL-LINK": "A challenge's proposal_task_id resolves to an envelope in the record set.",
    "DR-TO-PRINCIPAL": "A decision-request is addressed to a principal; agents never decide.",
    "DR-PROPOSER-IS-SENDER": "A decision-request is sent by the agent it names as proposer_agent (no attributing a proposal to another agent).",
    "DR-CHALLENGE-TARGETS-PROPOSAL": "Every attached challenge's proposal_task_id points at an envelope sent by the proposer.",
    "DR-CHALLENGE-RESOLVES": "Every challenge_task_ids entry resolves to a challenge envelope in the record set.",
    "DR-CHALLENGE-REQUIRED": "At R2+, with any materiality flag, or for automation-investment, at least one challenge is attached.",
    "DR-DESIGNATED-CHALLENGER": "When a challenge is required, the designated challenger for the decision class is among the challengers.",
    "DR-PROPOSER-NOT-CHALLENGER": "No attached challenge was raised by the proposer.",
    "DR-ALTERNATIVES": "When a challenge is required, at least one materially different alternative is recorded.",
    "DR-UNRESOLVED-DISSENT": "Every sustained or unresolved attached challenge has a dissent entry.",
    "DR-NO-APPROVE-OVER-DISSENT": "With any sustained or unresolved challenge, recommended_outcome is not approve.",
    "DR-HARD-GATE": "With a sustained or unresolved HARD-gate challenge, recommended_outcome is reject, redesign, or escalate.",
    "DR-VALUE-CASE-REQUIRED": "An automation-investment decision-request carries a value_case.",
    # --- decision records (principal-sourced) -------------------------------------------------
    "DRC-PRINCIPAL-SOURCE": "A decision-record is sent by a principal.",
    "DRC-DECIDER-IS-SENDER": "decision_record.decided_by is the record's own sender.",
    "DRC-REQUEST-BINDING": "request_task_id resolves to a decision-request and request_digest is the sha256 of its canonical JSON.",
    "DRC-NOT-REQUESTER": "The decider is not the principal who opened the correlation.",
    "DRC-SECOND-APPROVER": "An R5 decision carries a second approver distinct from the decider.",
    "DRC-HARD-GATE-ACCEPTANCE": "Approving over a sustained or unresolved HARD-gate challenge requires risk_acceptance_ref.",
    "DRC-APPROVAL-STATE": "approval_state is approved for approve/conditional-approve and rejected for reject/redesign.",
    # --- value case ---------------------------------------------------------------------------
    "VC-ORDER": "Every money range satisfies low <= point <= high.",
    "VC-NAV-ARITHMETIC": "net_annual_value equals benefit minus run, oversight, implementation and risk loss under interval arithmetic.",
    "VC-BENEFIT-CEILING": "verified_annual_benefit.high does not exceed baseline labor/error cost plus baseline loss/leakage.",
    "VC-OVERSIGHT": "A non-zero intervention or exception rate requires annual_oversight_cost.low > 0.",
    "VC-RUNS-BOUND": "successful_runs does not exceed runs.",
    "VC-UNIT-COST": "cost_per_successful_task_minor equals floor(annual_run_cost.point / successful_runs), or 0 with no successful runs.",
    "VC-COUNTERFACTUALS": "Counterfactuals include existing-process, conventional-automation and single-agent.",
    "VC-PROMOTION": "limited-production/production requires measured confidence and net_annual_value.low above every counterfactual's low.",
    "VC-SYNTHETIC": "A synthetic record never claims measured confidence or promotion beyond pilot.",
    "VC-RECOMMEND-INCREMENTAL": "Recommending approve/conditional-approve on an investment requires net_annual_value.point above every counterfactual's point.",
}

_validator = None


def _schema_validator():
    global _validator
    if _validator is None:
        schema = json.loads(SCHEMA_PATH.read_text(encoding="utf-8"))
        _validator = jsonschema.Draft202012Validator(schema)
    return _validator


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
    costs = ("annual_run_cost", "annual_oversight_cost", "annualized_implementation_cost", "expected_risk_loss")
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
    if mtype == "escalate" and dst["kind"] != "principal":
        out.add("HO-ESCALATE-TO-PRINCIPAL")
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
        if env["subject_risk_tier"] == "R5" and (
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
    return out


def check_record_set(doc: dict) -> list[str]:
    """Return the sorted list of violated rule ids for one record set."""
    messages = doc.get("messages")
    if not isinstance(messages, list) or not messages:
        return ["HO-SCHEMA"]
    validator = _schema_validator()
    violations: set[str] = set()
    valid: list[dict] = []
    for env in messages:
        if list(validator.iter_errors(env)):
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

    for env in valid:
        violations |= _check_envelope(env)
        if env["hop_count"] > 0:
            parent = by_id.get(env["parent_task_id"]) if env["parent_task_id"] else None
            if parent is None or env["hop_count"] != parent["hop_count"] + 1:
                violations.add("HO-CHAIN-PARENT")
            if parent is not None:
                violations |= _check_chain(env, parent)
        if env["message_type"] == "delegate" and env["destination"]["kind"] == "agent":
            earlier = {
                a["destination"].get("id")
                for a in _ancestors(env, by_id)
                if a["message_type"] == "delegate" and a["destination"]["kind"] == "agent"
            }
            if env["destination"]["id"] in earlier and not _stopped(env, "routing-loop"):
                violations.add("STOP-ROUTING-LOOP")
        if env["message_type"] == "challenge" and "challenge" in env:
            if env["challenge"]["proposal_task_id"] not in by_id:
                violations.add("CH-PROPOSAL-LINK")
        if env["message_type"] == "decision-request" and "decision_request" in env:
            violations |= _check_decision_request_links(env, by_id)
        if env["message_type"] == "decision-record" and "decision_record" in env:
            violations |= _check_decision_record_links(env, by_id, root)
    return sorted(violations)


def _attached_challenges(req: dict, by_id: dict) -> list[dict]:
    return [
        by_id[t] for t in req["challenge_task_ids"]
        if t in by_id and by_id[t]["message_type"] == "challenge" and "challenge" in by_id[t]
    ]


def _open(ch: dict) -> bool:
    return ch["challenge"]["outcome"] in ("sustained", "unresolved")


def _check_decision_request_links(env: dict, by_id: dict) -> set[str]:
    out: set[str] = set()
    req = env["decision_request"]
    attached = _attached_challenges(req, by_id)
    if len(attached) != len(req["challenge_task_ids"]):
        out.add("DR-CHALLENGE-RESOLVES")
    if _requires_challenge(env, req) and not req["challenge_task_ids"]:
        out.add("DR-CHALLENGE-REQUIRED")
    elif _requires_challenge(env, req):
        primary, fallback = DESIGNATED_CHALLENGERS[req["decision_class"]]
        designated = fallback if primary == req["proposer_agent"] else primary
        challengers = {c["source"].get("id") for c in attached if c["source"]["kind"] == "agent"}
        if designated == req["proposer_agent"] or designated not in challengers:
            out.add("DR-DESIGNATED-CHALLENGER")
    if any(c["source"].get("id") == req["proposer_agent"] for c in attached):
        out.add("DR-PROPOSER-NOT-CHALLENGER")
    for c in attached:
        target = by_id.get(c["challenge"]["proposal_task_id"])
        if target is not None and not (target["source"]["kind"] == "agent" and target["source"]["id"] == req["proposer_agent"]):
            out.add("DR-CHALLENGE-TARGETS-PROPOSAL")
    dissent_ids = {d["challenge_task_id"] for d in req["dissent"]}
    if any(_open(c) and c["task_id"] not in dissent_ids for c in attached):
        out.add("DR-UNRESOLVED-DISSENT")
    if any(_open(c) for c in attached) and req["recommended_outcome"] == "approve":
        out.add("DR-NO-APPROVE-OVER-DISSENT")
    if any(_open(c) and c["challenge"]["hard_gate"] for c in attached) and req["recommended_outcome"] not in (
        "reject", "redesign", "escalate"
    ):
        out.add("DR-HARD-GATE")
    return out


def _check_decision_record_links(env: dict, by_id: dict, root: dict | None) -> set[str]:
    out: set[str] = set()
    rec = env["decision_record"]
    request = by_id.get(rec["request_task_id"])
    if (request is None or request["message_type"] != "decision-request"
            or rec["request_digest"] != canonical_digest(request)):
        out.add("DRC-REQUEST-BINDING")
    if root is not None and root["source"]["kind"] == "principal" and rec["decided_by"]["subject"] == root["source"]["subject"]:
        out.add("DRC-NOT-REQUESTER")
    if request is not None and "decision_request" in request:
        attached = _attached_challenges(request["decision_request"], by_id)
        hard_open = any(_open(c) and c["challenge"]["hard_gate"] for c in attached)
        if hard_open and rec["outcome"] in ("approve", "conditional-approve") and "risk_acceptance_ref" not in rec:
            out.add("DRC-HARD-GATE-ACCEPTANCE")
    return out


def main(argv: list[str]) -> int:
    if not argv:
        print(__doc__.strip().splitlines()[-1])
        return 2
    failed = False
    for path in argv:
        doc = json.loads(Path(path).read_text(encoding="utf-8"))
        for rule in check_record_set(doc):
            failed = True
            print(f"{path}: {rule}")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
