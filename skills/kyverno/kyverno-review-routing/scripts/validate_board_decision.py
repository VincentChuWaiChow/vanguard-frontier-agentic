#!/usr/bin/env python3
"""Validator for Kyverno board decision records (ships inside the kyverno-review-routing skill).

A board can be unanimous, persuasive and wrong. This validator rejects decision records whose
structure hides that: a verdict that does not follow from the outcome, a promotion approved
without an independent challenge, a coverage number that mixes populations, an exception
"expiry" that is only a date in an annotation, a rollback claim built on a Git revert alone, an
UPDATE parity claim resting on offline tests that cannot express old/new pairs, or an Atlassian
handoff that would create a duplicate ticket. It checks the record, not the cluster: a passing
record is internally consistent, not true.

Deterministic: no clock (the record carries a caller-supplied ``as_of``), no network.

Rules (each proven to fire by a negative fixture in the repository gate):
  KYB-SCHEMA        structure violates assets/board-decision.schema.json
  KYB-VERDICT       repository verdict does not match the board outcome
  KYB-EVIDENCE      a claim, test, referral or block cites evidence the record does not contain,
                    or an observed/derived claim cites nothing
  KYB-UNKNOWN       an unknown claim names no smallest settling item, or an assumption no owner
  KYB-SUPPORTED     "supported" without a passed positive AND a passed negative test with evidence
  KYB-CONDITIONAL   "conditional" without named conditions
  KYB-TESTSTATE     a passed or failed test without an observation and evidence
  KYB-CHALLENGE     a consequential decision without an independent challenge, a self-challenge,
                    or "supported" while the challenge overturned or did not run
  KYB-DISSENT       "supported" while dissent is recorded (no majority vote)
  KYB-REFERRAL      referral to the lead itself, wrong decision ID, or duplicate referral
  KYB-EXECUTION     an advisory request carries a live handoff, or an execution request lacks one
  KYB-AUDIT         Audit/Warn severity that ignores intent, stage or deadline
  KYB-COVERAGE      coverage arithmetic mixes populations or overclaims
  KYB-EXPIRY        automatic expiry claimed without expiresAt and reconciliation evidence
  KYB-ROLLBACK      restoration claimed from a Git revert alone
  KYB-PARITY        parity supported with a differing operation or without old/new pair evidence
  KYB-ARTIFACT      Atlassian artifact that would duplicate, lose its ID, or update blind
  KYB-SECRET        a credential-shaped value appears anywhere in the record

Usage:
  python3 scripts/validate_board_decision.py record.json [record.json ...]
  (the repository gate imports run_suite() with its fixture directory)
"""
from __future__ import annotations

import copy
import json
import re
import sys
from pathlib import Path

try:
    import jsonschema
except ImportError:  # pragma: no cover - reported at runtime
    jsonschema = None

HERE = Path(__file__).resolve().parent
SCHEMA = HERE.parent / "assets" / "board-decision.schema.json"
DELETE = "__DELETE__"

VERDICT_FOR = {"supported": "approved", "conditional": "needs-review", "blocked": "blocked", "indeterminate": "needs-review"}
REQUEST_PAIRS = {"admission-trace", "controlled-test"}  # evidence that can carry a real old/new UPDATE pair
OBSERVED_STATE = {"installed-config", "admission-trace", "population-report", "controlled-test"}  # observed, not desired, state
SECRET_PATTERNS = [
    re.compile(r"-----BEGIN [A-Z ]*PRIVATE KEY-----"),
    re.compile(r"\bBearer\s+[A-Za-z0-9._~+/=-]{16,}"),
    re.compile(r"\beyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\."),
    re.compile(r"\b(?:password|passwd|secret|token|api[_-]?key)\s*[:=]\s*\S{6,}", re.IGNORECASE),
    re.compile(r"\bgh[pousr]_[A-Za-z0-9]{20,}"),
    re.compile(r"\bAKIA[0-9A-Z]{16}\b"),
]

RULES = {
    "KYB-SCHEMA", "KYB-VERDICT", "KYB-EVIDENCE", "KYB-UNKNOWN", "KYB-SUPPORTED", "KYB-CONDITIONAL",
    "KYB-TESTSTATE", "KYB-CHALLENGE", "KYB-DISSENT", "KYB-REFERRAL", "KYB-EXECUTION", "KYB-AUDIT",
    "KYB-COVERAGE", "KYB-EXPIRY", "KYB-ROLLBACK", "KYB-PARITY", "KYB-ARTIFACT", "KYB-SECRET",
}


def _strings(node):
    if isinstance(node, str):
        yield node
    elif isinstance(node, dict):
        for v in node.values():
            yield from _strings(v)
    elif isinstance(node, list):
        for v in node:
            yield from _strings(v)


def load_validator():
    if jsonschema is None:
        raise SystemExit("jsonschema is required: pip install jsonschema")
    return jsonschema.Draft202012Validator(json.loads(SCHEMA.read_text(encoding="utf-8")))


def check(rec: dict, validator) -> list[tuple[str, str]]:
    """Return (rule, message) pairs. Empty means internally consistent."""
    found: list[tuple[str, str]] = []

    def add(rule: str, msg: str) -> None:
        found.append((rule, msg))

    # Scan for credentials first, and never echo instance values in schema diagnostics:
    # a malformed record must not leak a token into validation logs.
    for s in _strings(rec):
        if any(p.search(s) for p in SECRET_PATTERNS):
            add("KYB-SECRET", "credential-shaped value in record; redact it and keep only a reference")
            break
    errors = sorted(validator.iter_errors(rec), key=lambda e: list(e.path))
    if errors:
        for e in errors[:5]:
            where = "/".join(str(p) for p in e.absolute_path) or "<root>"
            add("KYB-SCHEMA", f"{where}: fails schema keyword '{e.validator}'")
        return found  # semantic rules assume a well-formed record

    outcome, verdict = rec["outcome"], rec["verdict"]
    if VERDICT_FOR[outcome] != verdict:
        add("KYB-VERDICT", f"outcome {outcome} maps to verdict {VERDICT_FOR[outcome]}, not {verdict}")

    ev: dict[str, dict] = {}
    for e in rec["evidence"]:
        if e["id"] in ev:
            add("KYB-EVIDENCE", f"duplicate evidence ID {e['id']}; every reference must resolve to exactly one item")
        ev.setdefault(e["id"], e)

    def refs_ok(refs, where):
        missing = [r for r in refs if r not in ev]
        if missing:
            add("KYB-EVIDENCE", f"{where} cites missing evidence {missing}")

    for c in rec["claims"]:
        refs_ok(c["evidence_refs"], f"claim {c['id']}")
        if c["class"] in ("observed", "derived") and not c["evidence_refs"]:
            add("KYB-EVIDENCE", f"{c['class']} claim {c['id']} cites no evidence")
        if c["class"] == "unknown" and not c.get("smallest_settling_item"):
            add("KYB-UNKNOWN", f"unknown claim {c['id']} names no smallest settling evidence item")
        if c["class"] == "assumption" and not c.get("confirm_owner"):
            add("KYB-UNKNOWN", f"assumption {c['id']} names no owner who must confirm it")
    for t in rec["tests"]:
        refs_ok(t["evidence_refs"], f"test {t['id']}")
        if t["state"] in ("passed", "failed") and (not t["observed"] or not t["evidence_refs"]):
            add("KYB-TESTSTATE", f"test {t['id']} is {t['state']} without an observation and evidence")
        if t["state"] in ("proposed", "not-run") and t["observed"]:
            add("KYB-TESTSTATE", f"test {t['id']} is {t['state']} but claims an observation")
    for r in rec["referrals"]:
        refs_ok(r["evidence_refs"], f"referral to {r['to']}")

    if outcome == "supported":
        passed = {t["polarity"] for t in rec["tests"] if t["state"] == "passed" and t["evidence_refs"]}
        if passed != {"positive", "negative"}:
            add("KYB-SUPPORTED", "supported needs a passed positive and a passed negative test with evidence")
        failed = [t["id"] for t in rec["tests"] if t["state"] == "failed"]
        if failed:
            add("KYB-SUPPORTED", f"supported while in-scope tests failed {failed}; narrow the scope in a separate decision or return blocked/conditional")
    if outcome == "conditional" and not rec["conditions"]:
        add("KYB-CONDITIONAL", "conditional needs named conditions, each with an owner")

    lead = rec["lead"]["agent"]
    ch = rec["challenge"]
    if rec["consequence_class"] != "none":
        if ch is None:
            add("KYB-CHALLENGE", f"{rec['consequence_class']} requires an independent challenge")
        elif ch["challenger"] == lead:
            add("KYB-CHALLENGE", "the lead cannot challenge (approve) its own conclusion")
        elif ch["challenger"] in {r["to"] for r in rec["referrals"]}:
            add("KYB-CHALLENGE", f"{ch['challenger']} already contributed through a referral and cannot be the independent challenger")
        elif outcome == "supported" and ch["result"] != "upheld":
            add("KYB-CHALLENGE", f"supported while the challenge result is {ch['result']}")
    elif ch is not None and ch["challenger"] == lead:
        add("KYB-CHALLENGE", "the lead cannot challenge its own conclusion")
    # Consequential decisions already require an upheld challenge above; an optional challenge on an
    # ordinary decision still binds once it is recorded.
    if rec["consequence_class"] == "none" and ch is not None and ch["result"] == "overturned" and outcome == "supported":
        add("KYB-CHALLENGE", "a recorded overturn contradicts supported; answer it with evidence or change the conclusion")

    if outcome == "supported" and rec["dissent"]:
        add("KYB-DISSENT", "dissent is recorded; resolve it with evidence or return conditional/indeterminate")

    seen_ref = set()
    for r in rec["referrals"]:
        if r["to"] == lead:
            add("KYB-REFERRAL", f"referral back to the lead {lead}")
        if r["decision_id"] != rec["decision_id"]:
            add("KYB-REFERRAL", f"referral carries {r['decision_id']}, not {rec['decision_id']}")
        key = (r["to"], r["return_question"].strip().lower())
        if key in seen_ref:
            add("KYB-REFERRAL", f"duplicate referral to {r['to']}")
        seen_ref.add(key)

    handoff = rec["execution"]["handoff"]
    if rec["request_kind"] == "advisory" and handoff is not None:
        add("KYB-EXECUTION", "an advisory request must not produce a live handoff")
    if rec["request_kind"] == "explicit-execution" and handoff is None:
        add("KYB-EXECUTION", "an explicit execution request needs a guard handoff with target, diff, blast radius, rollback and verification")

    aa = rec.get("audit_assessment")
    if aa and aa["configured_action"] in ("Audit", "Warn") and aa["control_intent"] == "mandatory-preventive":
        as_of_date = rec["as_of"][:10]
        if aa["rollout_stage"] == "approved-canary":
            missing = [k for k in ("deadline", "owner", "promotion_criteria") if not aa[k]]
            if missing:
                add("KYB-AUDIT", f"an approved canary needs {missing}")
            elif aa["deadline"] < as_of_date:
                if aa["severity"] != "high":
                    add("KYB-AUDIT", f"mandatory preventive control in {aa['configured_action']} past its deadline {aa['deadline']} must be high severity")
                if outcome == "supported":
                    add("KYB-AUDIT", "an overdue preventive control cannot be supported")
            elif aa["severity"] == "high":
                add("KYB-AUDIT", "an approved, in-window canary is not a high-severity defect; severity follows intent and deadline, not the field value")
        elif aa["severity"] in ("none", "low"):
            add("KYB-AUDIT", f"mandatory preventive control in {aa['configured_action']} outside an approved canary cannot be {aa['severity']} severity")

    cov = rec.get("coverage")
    if cov:
        blocked = cov["blocked_requests"]
        refs_ok([cov["population_source"]] + ([blocked["source"]] if blocked["source"] else []), "coverage")
        if blocked["count"] > 0 and ev.get(blocked["source"] or "", {}).get("type") not in ("runtime-metric", "admission-trace"):
            add("KYB-COVERAGE", "a nonzero blocked-request count needs a runtime-metric or admission-trace source")
        rep_as = cov["excepted_reported_as"]
        if rep_as["skip"] + rep_as["pass"] != cov["excepted"]:
            add("KYB-COVERAGE", "excepted_reported_as must account for every excepted object (skip by default, pass with reportResult: pass)")
        if cov["fresh_results"] + cov["without_fresh_result"] != cov["inventory_total"]:
            add("KYB-COVERAGE", "fresh + without-fresh-result must equal the inventory population")
        if cov["excepted"] > cov["fresh_results"]:
            add("KYB-COVERAGE", "excepted objects must be a subset of objects with fresh results")
        cl = cov["claimed"]
        if cl["denominator"] != cov["inventory_total"]:
            add("KYB-COVERAGE", "object coverage denominator must be the inventory population; blocked requests never became objects")
        if cl["numerator"] > cov["fresh_results"]:
            add("KYB-COVERAGE", "numerator exceeds objects with fresh results")
        b = cov["results_breakdown"]
        if b is None:
            if cov["compliance_percentage"] is not None:
                add("KYB-COVERAGE", "no compliance percentage without the result breakdown")
        else:
            if sum(b.values()) != cov["fresh_results"]:
                add("KYB-COVERAGE", "result breakdown must sum to objects with fresh results")
            if b["skip"] < rep_as["skip"] or b["pass"] < rep_as["pass"]:
                add("KYB-COVERAGE", "breakdown cannot hold the excepted objects where excepted_reported_as says they report")
            if cov["compliance_percentage"] is not None:
                population = cov["fresh_results"] - cov["excepted"]
                if population <= 0:
                    add("KYB-COVERAGE", "no compliance percentage without non-excepted objects with fresh results")
                else:
                    supported = 100.0 * (b["pass"] - rep_as["pass"]) / population
                    if abs(cov["compliance_percentage"] - supported) > 0.05:
                        add("KYB-COVERAGE", f"compliance percentage must equal non-excepted passes over non-excepted fresh results ({supported:.1f})")
        if cov["prevention_demonstrated"] > blocked["count"]:
            add("KYB-COVERAGE", "demonstrated prevention needs blocked admissions; report failures and exceptions are not prevention")

    ex = rec.get("exception_expiry")
    if ex:
        refs_ok(ex["reconciliation_evidence"], "exception_expiry")
        if ex["automatic_expiry_claimed"]:
            if not ex["expires_at"]:
                add("KYB-EXPIRY", "automatic expiry claimed with no expiresAt; a date in an annotation is not a mechanism")
            elif not any(
                ev.get(r, {}).get("type") in ("admission-trace", "population-report", "controlled-test")
                and ev[r]["captured_at"] is not None
                and ex["expires_at"] < ev[r]["captured_at"] <= rec["as_of"]
                for r in ex["reconciliation_evidence"]
            ):
                add("KYB-EXPIRY", "automatic expiry needs admission, report or controlled-test evidence captured after expiresAt and no later than as_of")
        if outcome == "supported" and ex["adjacent_workloads_covered"] == "unverified":
            add("KYB-EXPIRY", "adjacent workloads must be shown to remain covered before the exception is supported")

    rb = rec.get("rollback")
    if rb:
        refs_ok(rb["downstream_state_evidence"], "rollback")
        observed = [r for r in rb["downstream_state_evidence"] if ev.get(r, {}).get("type") in OBSERVED_STATE]
        if rb["restoration_claimed"] and (rb["basis"] != "downstream-state-evidence" or not observed):
            add("KYB-ROLLBACK", "restoration claimed without downstream state evidence; reverting Git does not restore mutated or deleted state")

    par = rec.get("parity")
    if par:
        refs_ok([par["corpus"]] + par["old_new_pair_evidence"], "parity")
        ops = par["operations"]
        if "differ" in ops.values() and outcome != "blocked":
            add("KYB-PARITY", "a required operation differs; parity is blocked for this cohort")
        if outcome == "supported":
            if "untested" in ops.values():
                add("KYB-PARITY", "parity supported with an untested operation")
            pairs = [ev[r]["type"] for r in par["old_new_pair_evidence"] if r in ev]
            if ops["UPDATE"] == "agree" and not any(t in REQUEST_PAIRS for t in pairs):
                add("KYB-PARITY", "UPDATE parity needs request-level old/new pairs (admission-trace or controlled-test); the CLI uses identical old and new objects")

    seen_art = set()
    for a in rec["artifacts"]:
        if a["local_id"] != rec["decision_id"]:
            add("KYB-ARTIFACT", f"{a['system']} {a['kind']} carries {a['local_id']}, not {rec['decision_id']}")
        expected_skill = "jira-ticket-authoring" if a["system"] == "jira" else "confluence-document-authoring"
        if a["skill"] != expected_skill:
            add("KYB-ARTIFACT", f"{a['system']} artifacts are authored with {expected_skill}")
        jira_kinds = {"operational-change", "incident", "spike", "bug"}
        if (a["system"] == "jira") != (a["kind"] in jira_kinds):
            add("KYB-ARTIFACT", f"{a['kind']} is not a {a['system']} artifact")
        if a["action"] == "update" and (not a["external_key"] or not a["revision_checked"]):
            add("KYB-ARTIFACT", "an update needs the existing artifact key and the revision checked before writing")
        key = (a["system"], a["kind"], a["local_id"])
        if key in seen_art:
            add("KYB-ARTIFACT", f"duplicate {a['system']} {a['kind']} for {a['local_id']}; update the existing draft instead")
        seen_art.add(key)

    return found


def _apply(record: dict, changes: dict) -> dict:
    result = copy.deepcopy(record)
    for dotted, value in changes.items():
        parts = [int(p) if p.isdigit() else p for p in dotted.split(".")]
        node = result
        for p in parts[:-1]:
            node = node[p]
        if value == DELETE:
            del node[parts[-1]]
        elif isinstance(node, list) and parts[-1] == len(node):
            node.append(value)
        else:
            node[parts[-1]] = value
    return result


def run_suite(fixtures: Path, extra_valid: list[Path] | None = None) -> int:
    """Run committed positive and negative fixtures (repository gate). Each invalid fixture names a
    valid base, a set of dotted-path changes, and the single rule that must fire."""
    validator = load_validator()
    failures = 0
    valid_files = sorted((fixtures / "valid").glob("*.json")) + list(extra_valid or [])
    invalid_files = sorted((fixtures / "invalid").glob("*.json"))
    if not valid_files or not invalid_files:
        print(f"FAIL: fixtures missing under {fixtures}/{{valid,invalid}}")
        return 1
    for f in valid_files:
        found = check(json.loads(f.read_text(encoding="utf-8")), validator)
        if found:
            failures += 1
            print(f"FAIL valid/{f.name}: " + "; ".join(f"{r}: {m}" for r, m in found))
        else:
            print(f"OK   valid/{f.name}")
    proven: set[str] = set()
    for f in invalid_files:
        spec = json.loads(f.read_text(encoding="utf-8"))
        if spec["rule"] not in RULES:
            failures += 1
            print(f"FAIL invalid/{f.name}: names unknown rule {spec['rule']}")
            continue
        base = json.loads((fixtures / "valid" / spec["base"]).read_text(encoding="utf-8"))
        rules = {r for r, _ in check(_apply(base, spec["set"]), validator)}
        if rules == {spec["rule"]}:
            proven.add(spec["rule"])
            print(f"OK   invalid/{f.name} -> {spec['rule']}")
        else:
            failures += 1
            print(f"FAIL invalid/{f.name}: expected only {spec['rule']}, got {sorted(rules) or 'no violation'}")
    unproven = sorted(RULES - proven)
    if unproven:
        failures += 1
        print(f"FAIL rules with no passing negative fixture: {unproven}")
    print(f"\n{len(valid_files)} valid, {len(invalid_files)} invalid fixtures; {len(proven)}/{len(RULES)} rules proven to fire")
    return 1 if failures else 0


def main(argv: list[str]) -> int:
    if not argv:
        print("usage: python3 validate_board_decision.py record.json [record.json ...]")
        return 2
    validator = load_validator()
    bad = 0
    for p in argv:
        found = check(json.loads(Path(p).read_text(encoding="utf-8")), validator)
        for rule, msg in found:
            print(f"FAIL {p}: {rule}: {msg}")
        if found:
            bad += 1
        else:
            print(f"OK   {p}")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
