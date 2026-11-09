#!/usr/bin/env python3
"""Offline manifest safety check. Caller evidence is not independently verified."""
import argparse
import datetime as dt
import json
import re
from pathlib import Path


def validate(manifest):
    source = manifest["source"]
    dest = manifest["destination"]
    backup = manifest["backup"]
    for group, fields in [(source, ["tenancy", "region", "compartment", "table", "table_ocid"]),
                          (dest, ["region", "compartment", "table"]),
                          (backup, ["namespace", "bucket", "prefix", "schema_sha256"])]:
        for name in fields:
            value = group[name]
            if not isinstance(value, str) or not value or "REPLACE" in value:
                raise ValueError("exact reviewed identities/hashes required")
    if dest["table"].casefold() == source["table"].casefold() or dest.get("exists") is not False:
        raise ValueError("new differently named destination with observed absence required")
    if backup.get("verified") is not True or type(backup.get("failed_records")) is not int or backup.get("failed_records") != 0:
        raise ValueError("verified zero-failure export required")
    if backup.get("writes_quiesced") is not True:
        raise ValueError("this conservative recovery path requires writes quiesced")
    if not isinstance(backup.get("object_versions"), list) or not backup["object_versions"] or any(not isinstance(v, str) or not v.strip() for v in backup["object_versions"]) or not isinstance(backup.get("data_sha256"), dict) or not backup["data_sha256"]:
        raise ValueError("immutable object identities and data hashes required")
    hashes = [backup["schema_sha256"], *backup["data_sha256"].values()]
    if any(not isinstance(x, str) or not re.fullmatch(r"[0-9a-f]{64}", x) for x in hashes):
        raise ValueError("SHA-256 must be 64 lowercase hex digits")
    times = [dt.datetime.fromisoformat(backup[x].replace("Z", "+00:00")) for x in ["export_start", "export_end"]]
    if any(x.tzinfo is None for x in times) or times[1] < times[0]:
        raise ValueError("ordered timezone-aware export bounds required")
    if backup.get("include_ttl") is not True or manifest.get("ttl_policy") != "preserve-absolute-expiration":
        raise ValueError("explicit reviewed absolute TTL preservation required")
    return {"status": "locally-consistent", "execution": "none", "live_evidence": "UNVERIFIED",
            "cutover_authorized": False, "notice": "Does not prove object integrity, target absence or recovered application correctness"}


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("manifest", type=Path)
    args = p.parse_args()
    try:
        print(json.dumps(validate(json.loads(args.manifest.read_text())), indent=2))
    except (KeyError, TypeError, ValueError, OSError) as exc:
        p.error(str(exc))


if __name__ == "__main__":
    main()
