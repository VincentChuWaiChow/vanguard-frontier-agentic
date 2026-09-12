#!/usr/bin/env python3
"""Offline structural facts, not an Istio policy engine or execution authorizer.

Read one explicit JSON operation from a file or stdin. Never access Kubernetes,
credentials, a network, a shell, a clock, or environment-derived configuration.
"""
from __future__ import annotations

import argparse
from datetime import datetime
from fractions import Fraction
import json
import math
from pathlib import Path
import re
import sys
from typing import Any

MAX_INPUT_BYTES = 2 * 1024 * 1024
MAX_JSON_DEPTH = 64


class InputError(ValueError):
    """An input is malformed or outside this helper's explicit scope."""


def _object(value: Any, label: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise InputError(f"{label} must be an object")
    return value


def _integer(value: Any, label: str, minimum: int = 0) -> int:
    if type(value) is not int or value < minimum:
        raise InputError(f"{label} must be an integer >= {minimum}")
    return value


def _string(value: Any, label: str) -> str:
    if not isinstance(value, str) or not value.strip():
        raise InputError(f"{label} must be a nonempty string")
    return value


def _strings(value: Any, label: str, nonempty: bool = True) -> list[str]:
    if not isinstance(value, list) or (nonempty and not value):
        raise InputError(f"{label} must be {'a nonempty ' if nonempty else 'an '}array")
    if any(not isinstance(v, str) or not v.strip() for v in value):
        raise InputError(f"{label} must contain nonempty strings")
    if len(value) != len(set(value)):
        raise InputError(f"{label} must not contain duplicates")
    return value


def _unique_pairs(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise InputError("JSON contains a duplicate object key")
        result[key] = value
    return result


def parse_json(data: bytes) -> Any:
    if len(data) > MAX_INPUT_BYTES:
        raise InputError("JSON input exceeds the 2 MiB limit")
    def invalid_constant(_: str) -> None:
        raise InputError("Non-finite JSON numbers are not accepted")
    def bounded_integer(text: str) -> int:
        if len(text.lstrip("-")) > 1000:
            raise InputError("JSON integer exceeds the 1000-digit limit")
        return int(text)
    def finite_float(text: str) -> float:
        result = float(text)
        if not math.isfinite(result):
            raise InputError("Non-finite JSON numbers are not accepted")
        return result
    try:
        text = data.decode("utf-8")
        # Bound nesting before parsing; brackets inside strings do not count.
        depth, quoted, escaped = 0, False, False
        for char in text:
            if quoted:
                if escaped:
                    escaped = False
                elif char == "\\":
                    escaped = True
                elif char == '"':
                    quoted = False
            elif char == '"':
                quoted = True
            elif char in "[{":
                depth += 1
                if depth > MAX_JSON_DEPTH:
                    raise InputError("JSON nesting exceeds the 64-level limit")
            elif char in "]}":
                depth -= 1
        return json.loads(text, object_pairs_hook=_unique_pairs,
                          parse_constant=invalid_constant,
                          parse_int=bounded_integer, parse_float=finite_float)
    except InputError:
        raise
    except (UnicodeDecodeError, ValueError, RecursionError) as exc:
        raise InputError("Input is not bounded, valid UTF-8 JSON") from exc


def rule_list_shape(spec: dict[str, Any]) -> dict[str, Any]:
    """Classify the list only. Do not resolve target, matches or a final verdict."""
    spec = _object(spec, "spec")
    action = spec.get("action", "ALLOW")
    if action not in ("ALLOW", "DENY", "CUSTOM", "AUDIT"):
        raise InputError("Unsupported action")
    if "rules" not in spec:
        shape, count, empty = "absent", 0, 0
    else:
        rules = spec["rules"]
        if not isinstance(rules, list) or any(not isinstance(r, dict) for r in rules):
            raise InputError("rules must be an array of objects; null is not normalized here")
        count = len(rules)
        empty = sum(not r for r in rules)
        shape = "empty-list" if not rules else "contains-empty-rule" if empty else "nonempty-constrained-rules"
    return {"action": action, "shape": shape, "rule_count": count,
            "empty_rule_count": empty, "effective_access": "not-evaluated"}


def relative_weights(weights: list[int]) -> dict[str, Any]:
    """Do arithmetic on explicit weights; do not implement API defaults/validation."""
    if not isinstance(weights, list) or not weights:
        raise InputError("weights must be a nonempty array")
    checked = [_integer(v, "weight") for v in weights]
    total = sum(checked)
    if total == 0:
        raise InputError("A zero total has no defined proportional allocation")
    return {"total": total, "shares": [str(Fraction(v, total)) for v in checked],
            "observed_distribution": "not-measured", "api_validation": "not-performed"}


def condition_freshness(generation: int, condition: dict[str, Any]) -> dict[str, Any]:
    generation = _integer(generation, "metadata.generation", 1)
    condition = _object(condition, "condition")
    _string(condition.get("type"), "condition.type")
    if condition.get("status") not in ("True", "False", "Unknown"):
        raise InputError("condition.status must be True, False or Unknown as a string")
    observed = condition.get("observedGeneration")
    if observed is None:
        relation = "unknown"
    else:
        observed = _integer(observed, "observedGeneration")
        relation = "current" if observed == generation else "stale" if observed < generation else "inconsistent-future-generation"
    return {"condition_type": condition["type"], "reported_status": condition["status"],
            "generation_relation": relation, "parent_and_controller_ownership": "not-checked",
            "end_to_end_behavior": "not-proven"}


def _version(value: str) -> tuple[int, int, int]:
    if not isinstance(value, str) or len(value) > 64 or not re.fullmatch(r"(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)", value):
        raise InputError("Version helper accepts explicit numeric major.minor.patch only")
    return tuple(map(int, value.split(".")))  # type: ignore[return-value]


def directional_skew(control_plane: str, data_plane: str) -> dict[str, Any]:
    cp, dp = _version(control_plane), _version(data_plane)
    if cp[0] != dp[0]:
        relation = "different-major-requires-specific-review"
    elif dp[1] > cp[1]:
        relation = "data-plane-minor-ahead"
    elif cp[1] - dp[1] > 1:
        relation = "control-plane-more-than-one-minor-ahead"
    elif cp[1] - dp[1] == 1:
        relation = "control-plane-one-minor-ahead"
    elif dp[2] > cp[2]:
        relation = "same-minor-data-plane-newer-patch"
    else:
        relation = "same-minor"
    return {"numeric_relation": relation, "support_status": "not-determined",
            "scope": "Only the supplied control-plane/data-plane pair; no release-support or security certification"}


def crd_storage_summary(crd: dict[str, Any]) -> dict[str, Any]:
    crd = _object(crd, "crd")
    versions = _object(crd.get("spec"), "crd.spec").get("versions")
    if not isinstance(versions, list) or not versions:
        raise InputError("spec.versions must be a nonempty array")
    names, current, served = [], [], []
    for item in versions:
        item = _object(item, "version")
        name = _string(item.get("name"), "version.name")
        if type(item.get("storage")) is not bool or type(item.get("served")) is not bool:
            raise InputError("storage and served must be booleans")
        if name in names:
            raise InputError("Duplicate CRD version name")
        names.append(name)
        if item["storage"]: current.append(name)
        if item["served"]: served.append(name)
    if len(current) != 1:
        raise InputError("Exactly one storage-write version is required")
    status = _object(crd.get("status", {}), "crd.status")
    stored = status.get("storedVersions")
    if stored is not None:
        stored = _strings(stored, "status.storedVersions", nonempty=False)
    return {"storage_write_version": current[0], "historical_stored_versions": stored,
            "historical_versions_not_served": None if stored is None else sorted(set(stored)-set(served)),
            "stored_object_migration": "not-proven"}


def aggregate_reviews(reviews: list[dict[str, Any]]) -> dict[str, Any]:
    if not isinstance(reviews, list) or not reviews:
        raise InputError("Provide all required review decisions, including unresolved ones")
    ids, scopes, verdicts = set(), set(), []
    for item in reviews:
        item = _object(item, "review")
        ident = _string(item.get("id"), "review.id")
        if ident in ids: raise InputError("Duplicate review ID")
        ids.add(ident)
        scopes.add(_string(item.get("scope_id"), "review.scope_id"))
        verdict = item.get("verdict")
        if verdict not in ("approved", "blocked", "needs-review"):
            raise InputError("Unknown review verdict")
        verdicts.append(verdict)
    if len(scopes) != 1:
        result, reason = "needs-review", "Align review scopes before aggregation"
    elif "blocked" in verdicts:
        result, reason = "blocked", "A required decision reports a confirmed blocker"
    elif "needs-review" in verdicts:
        result, reason = "needs-review", "A required decision is unresolved"
    else:
        result, reason = "approved", "Bounded review aggregation only; inputs are not independently verified"
    return {"verdict": result, "reason": reason, "required_set_completeness": "not-verified", "execution_authorization": "not-granted"}


OWNERS = {
    "diagnostics": "istio-dataplane-diagnostics-agent",
    "upgrade": "istio-upgrade-readiness-agent",
    "ambient": "istio-ambient-mesh-review-agent",
    "gateway": "istio-gateway-api-review-agent",
    "authorization": "istio-authorization-policy-review-agent",
    "traffic": "istio-traffic-resilience-review-agent",
}


def route_review(intent: str, decisions: list[str]) -> dict[str, Any]:
    if intent not in ("review", "collect", "mutate"):
        raise InputError("intent must be review, collect or mutate")
    decisions = _strings(decisions, "decisions", nonempty=False)
    if set(decisions)-OWNERS.keys():
        raise InputError("Unknown decision owner; route to an explicit external specialist")
    if intent == "mutate":
        return {"mode": "live-guard-handoff", "review_owners": [], "auto_dispatch": [],
                "handoff_owner": "kubernetes-live-mesh-policy-guard-agent"}
    if intent == "collect":
        return {"mode": "runtime-collection-handoff", "review_owners": [], "auto_dispatch": []}
    return {"mode": "review-plan" if decisions else "needs-review", "review_owners": [v for k,v in OWNERS.items() if k in decisions], "auto_dispatch": []}


BINDING_KEYS = {"cluster_id", "context", "namespace", "resources", "verbs", "baseline_sha256", "change_sha256", "rollback_sha256"}


def _binding(value: Any) -> dict[str, Any]:
    value = _object(value, "binding")
    if set(value) != BINDING_KEYS:
        raise InputError("Binding must contain exactly the documented target, resource, verb and hash fields")
    clean = dict(value)
    for key in ("cluster_id", "context", "namespace"):
        _string(value[key], f"binding.{key}")
    for key in ("baseline_sha256", "change_sha256", "rollback_sha256"):
        if not isinstance(value[key], str) or not re.fullmatch(r"[0-9a-f]{64}", value[key]):
            raise InputError(f"binding.{key} must be a lowercase SHA256 hex digest")
    clean["resources"] = sorted(_strings(value["resources"], "binding.resources"))
    clean["verbs"] = sorted(_strings(value["verbs"], "binding.verbs"))
    resource_id = re.compile(r"^[a-z0-9.-]+/[A-Za-z][A-Za-z0-9]*/[a-z0-9.-]+/[a-z0-9]([-a-z0-9.]*[a-z0-9])?$")
    if any(not resource_id.fullmatch(resource) for resource in clean["resources"]):
        raise InputError(
            "binding.resources must contain exact apiGroup/Kind/namespace/name resource IDs"
        )
    if set(clean["verbs"])-{"apply", "patch", "delete"}:
        raise InputError("Unknown binding verb; expand authority only through a reviewed contract change")
    return clean


def _instant(value: Any, label: str) -> datetime:
    _string(value, label)
    try:
        result = datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError as exc:
        raise InputError(f"{label} must be an ISO 8601 time") from exc
    if result.tzinfo is None or result.utcoffset() is None:
        raise InputError(f"{label} must carry an explicit timezone")
    return result


def approval_binding_check(approval: dict[str, Any], current: dict[str, Any], as_of: str) -> dict[str, Any]:
    """Check consistency only. Approval authenticity and operator authority are external."""
    approval = _object(approval, "approval")
    required = {"approval_id", "approver_ref", "review_ref", "valid_from", "valid_until", "binding"}
    if set(approval) != required:
        raise InputError("Approval fields must match the documented binding contract exactly")
    for key in ("approval_id", "approver_ref", "review_ref"):
        _string(approval[key], key)
    expected, actual = _binding(approval["binding"]), _binding(current)
    start = _instant(approval["valid_from"], "valid_from")
    end = _instant(approval["valid_until"], "valid_until")
    now = _instant(as_of, "as_of")
    if end <= start:
        raise InputError("Approval validity window must have positive duration")
    mismatches = sorted(k for k in BINDING_KEYS if expected[k] != actual[k])
    if not start <= now < end: mismatches.append("validity_window")
    return {"binding_matches": not mismatches, "mismatches": mismatches,
            "approval_authenticity": "not-verified", "execution_authorization": "not-granted"}


def dispatch(request: dict[str, Any]) -> dict[str, Any]:
    request = _object(request, "request")
    if set(request) != {"operation", "input"}:
        raise InputError("Request must contain exactly operation and input")
    name = request["operation"]
    data = _object(request["input"], "input")
    operations = {
        "rule-list-shape": (rule_list_shape, {"spec"}),
        "relative-weights": (relative_weights, {"weights"}),
        "condition-freshness": (condition_freshness, {"generation", "condition"}),
        "directional-skew": (directional_skew, {"control_plane", "data_plane"}),
        "crd-storage-summary": (crd_storage_summary, {"crd"}),
        "aggregate-reviews": (aggregate_reviews, {"reviews"}),
        "route-review": (route_review, {"intent", "decisions"}),
        "approval-binding": (approval_binding_check, {"approval", "current", "as_of"}),
    }
    if not isinstance(name, str) or name not in operations:
        raise InputError("Unknown operation")
    function, keys = operations[name]
    if set(data) != keys:
        raise InputError("Operation input fields do not match its documented contract")
    return function(**data)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("input", nargs="?", help="Explicit JSON request file; stdin when omitted")
    args = parser.parse_args()
    try:
        if args.input:
            path = Path(args.input)
            if path.is_symlink() or not path.is_file():
                raise InputError("Input must be a regular, non-symlink file")
            with path.open("rb") as handle:
                data = handle.read(MAX_INPUT_BYTES + 1)
        else:
            data = sys.stdin.buffer.read(MAX_INPUT_BYTES + 1)
        result = dispatch(parse_json(data))
        print(json.dumps(result, indent=2, sort_keys=True, allow_nan=False))
        return 0
    except (InputError, OSError, TypeError, OverflowError, RecursionError) as exc:
        # Do not echo supplied content, paths, credentials or arbitrary exception text.
        message = str(exc) if isinstance(exc, InputError) else "Input could not be safely processed"
        print(json.dumps({"error": message}), file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
