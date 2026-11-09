#!/usr/bin/env python3
"""Validate supplied sanitized recovery evidence and print a plan. No Azure calls."""
import argparse
from datetime import datetime, timezone
import json
import re


def instant(value):
    parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
    if parsed.tzinfo is None or parsed.utcoffset().total_seconds() != 0:
        raise ValueError("explicit UTC timestamp required")
    return parsed.astimezone(timezone.utc)


def plan(e):
    method = e["method"]
    if method not in {"continuous-new-account", "continuous-deleted-resource", "periodic-support"}:
        raise ValueError("unsupported recovery method")
    s, t = e["source"], e["target"]
    for obj, names in [(s, ("subscription", "resource_id", "instance_id", "account", "region", "api")),
                       (t, ("subscription", "resource_group", "account", "lookup"))]:
        for name in names:
            if not isinstance(obj.get(name), str) or not obj[name].strip():
                raise ValueError("explicit source/target identity required")
    if s["api"] != "NoSQL" or s["subscription"] != t["subscription"]:
        raise ValueError("NoSQL and same-subscription recovery only")
    expected = rf"/subscriptions/{re.escape(s['subscription'])}/resourceGroups/[^/]+/providers/Microsoft\.DocumentDB/databaseAccounts/{re.escape(s['account'])}"
    if not re.fullmatch(expected, s["resource_id"], re.IGNORECASE) or any(
            not re.fullmatch(r"[a-z0-9][a-z0-9-]{1,42}[a-z0-9]", account)
            for account in (s["account"], t["account"])):
        raise ValueError("source ID/target mismatch")
    if e.get("observed_source_instance_id") != s["instance_id"] or e.get("source_name_unambiguous") is not True:
        raise ValueError("source instance/name ambiguous or drifted")
    point = instant(e["restore_timestamp"])
    if not instant(s["earliest_restorable"]) <= point <= instant(s["latest_restorable"]):
        raise ValueError("point outside observed recovery window")
    if point > instant(e["evidence_as_of"]):
        raise ValueError("future restore point")
    if e.get("disable_ttl") is not True or e.get("key_dependencies_verified") is not True:
        raise ValueError("review TTL preservation/key dependencies first")
    if t["lookup"] != "ResourceNotFound":
        raise ValueError("target exists or absence unproven; errors aren't NotFound")
    argv = None
    if method == "continuous-deleted-resource":
        if s.get("backup_mode") != "Continuous" or t["account"] != s["account"]:
            raise ValueError("in-account restore requires same continuous source account")
        if e.get("deletion_event") != "Delete" or not point < instant(e["deleted_at"]):
            raise ValueError("actual deletion event and pre-deletion point required")
        kind = e.get("resource_kind")
        if kind not in {"database", "container"}:
            raise ValueError("only deleted database/container supported")
        if not isinstance(e.get("database"), str) or not e["database"]:
            raise ValueError("database required")
        if kind == "container" and (not isinstance(e.get("container"), str) or not e["container"]):
            raise ValueError("container required")
        argv = ["az", "cosmosdb", "sql", kind, "restore", "--subscription", t["subscription"],
                "--resource-group", t["resource_group"], "--account-name", s["account"],
                "--name", e["database"] if kind == "database" else e["container"]]
        if kind == "container":
            argv += ["--database-name", e["database"]]
    else:
        if t["account"] == s["account"]:
            raise ValueError("distinct NEW destination required; no source overwrite")
        expected_mode = "Periodic" if method == "periodic-support" else "Continuous"
        if s.get("backup_mode") != expected_mode:
            raise ValueError("observed policy doesn't support selected recovery method")
        if method == "continuous-new-account":
            if e.get("restore_scope") != "full-account" or e.get("approved_restore_scope") != "full-account":
                raise ValueError("helper requires explicit approved full-account restore scope")
            if any(key in e for key in ("database", "container", "databases", "containers", "collections", "databases_to_restore")):
                raise ValueError("helper doesn't implement subset restore; use reviewed manual scope, never broaden")
            argv = ["az", "cosmosdb", "restore", "--subscription", t["subscription"],
                    "--resource-group", t["resource_group"], "--account-name", s["account"],
                    "--target-database-account-name", t["account"], "--location", s["region"],
                    "--disable-local-auth", "true", "--public-network-access", "DISABLED"]
    if argv is not None:
        argv += ["--restore-timestamp", e["restore_timestamp"], "--disable-ttl", "true"]
    return {"execution": "none", "method": method, "change_approval_required": True,
            "source_instance": s["instance_id"], "argv_example": argv,
            "periodic_support_required": method == "periodic-support",
            "acceptance": "settings and application/security/RPO/RTO checks required; waiter insufficient"}


if __name__ == "__main__":
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("evidence", help="explicit local sanitized JSON evidence file")
    args = p.parse_args()
    try:
        with open(args.evidence, encoding="utf-8") as handle:
            print(json.dumps(plan(json.load(handle)), indent=2))
    except (ValueError, KeyError, TypeError) as error:
        p.error(str(error))
