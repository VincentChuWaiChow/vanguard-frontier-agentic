#!/usr/bin/env python3
"""Gate: every NoSQL service has a complete, evidence-cited operations reference.

Each service in ``SERVICES`` must ship an operations file that:

* exists and is linked from its skill's ``SKILL.md`` (otherwise no agent loads it);
* contains every heading in ``SECTIONS``, once each, in that order;
* states exactly one ``**Status:**`` per section, from ``STATUSES``;
* cites at least one evidence row in every section whose status is VERIFIED,
  PARTIAL or UNSUPPORTED, and every cited row exists in the skill's ledger;
* in a VERIFIED section, cites only rows whose ledger status is VERIFIED;
* writes no unit prices or currency amounts in the Cost section (billing
  dimensions only; prices go stale by Region and over time).

The structure lives in ``templates/nosql-operations-template.md``. Add a service by
adding its entry here; never weaken a rule to get green.
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]

# service -> (skill directory, operations file relative to the skill)
SERVICES: dict[str, tuple[str, str]] = {
    "Amazon DynamoDB": ("skills/aws/aws-dynamodb-patterns", "references/operations.md"),
    "Amazon DocumentDB": ("skills/aws/aws-documentdb-architect", "references/operations.md"),
    "Amazon Keyspaces": ("skills/aws/aws-keyspaces-architect", "references/operations.md"),
    "Azure Cosmos DB for NoSQL": ("skills/azure/azure-cosmosdb-platform-operator", "references/operations.md"),
    "Azure Cosmos DB for MongoDB": ("skills/azure/azure-cosmosdb-platform-operator", "references/operations-mongodb.md"),
    "Azure Cosmos DB for Apache Cassandra": ("skills/azure/azure-cosmosdb-platform-operator", "references/operations-cassandra.md"),
    "Oracle NoSQL Database Cloud Service": ("skills/oci/oci-nosql-database-architect", "references/operations.md"),
    "Firestore (Native mode)": ("skills/gcp/gcp-firebase-developer", "references/firestore-operations.md"),
    "Firestore with MongoDB compatibility": ("skills/gcp/gcp-firebase-developer", "references/firestore-mongodb-operations.md"),
    "Cloud Bigtable": ("skills/gcp/gcp-bigtable-architect", "references/operations.md"),
    "Alibaba Cloud Tablestore": ("skills/alibaba/alibaba-tablestore-architect", "references/operations.md"),
    "Huawei Cloud GeminiDB": ("skills/huawei/huawei-geminidb-architect", "references/operations.md"),
    "Huawei Cloud DDS": ("skills/huawei/huawei-gaussdb-rds-dba", "references/dds-operations.md"),
    "Scaleway Managed MongoDB": ("skills/scaleway/scaleway-managed-mongodb-architect", "references/operations.md"),
    "OVHcloud Public Cloud Databases for MongoDB": ("skills/ovhcloud/ovhcloud-mongodb-architect", "references/operations.md"),
    "IONOS Cloud DBaaS MongoDB": ("skills/ionos/ionos-live-database-lifecycle-guard", "references/mongodb-operations.md"),
}
SECTIONS = [
    "Backup and restore",
    "Disaster recovery and replication",
    "Cost",
    "Monitoring",
    "Performance",
    "IAM",
    "Tagging",
    "Encryption",
    "Network isolation",
    "Audit logging",
]
STATUSES = {"VERIFIED", "PARTIAL", "UNSUPPORTED", "UNVERIFIED", "NOT APPLICABLE"}
NEEDS_CITATION = {"VERIFIED", "PARTIAL", "UNSUPPORTED"}
EVIDENCE_ID = r"[A-Z][A-Z0-9]*(?:-[A-Z0-9]+)*-E\d{2,}"
MARKER_RE = re.compile(r"\[(" + EVIDENCE_ID + r")\]")
ROW_RE = re.compile(r"^\|\s*(" + EVIDENCE_ID + r")\s*\|(.*)\|\s*$")
STATUS_RE = re.compile(r"^\*\*Status:\*\*\s*(.+?)\s*$", re.MULTILINE)
PRICE_RE = re.compile(r"(?:[$€£¥]\s?\d)|(?:\b\d[\d,.]*\s?(?:USD|EUR|GBP|CNY|RMB|JPY)\b)|(?:\b(?:USD|EUR|GBP|CNY|RMB|JPY)\s?\d)")


def ledger_status(skill: Path) -> dict[str, str]:
    ledger = skill / "references/evidence-ledger.md"
    rows: dict[str, str] = {}
    if ledger.exists():
        for line in ledger.read_text(encoding="utf-8").splitlines():
            match = ROW_RE.match(line.strip())
            if match:
                rows[match.group(1)] = match.group(2).split("|")[-1].strip()
    return rows


def check(service: str, skill_rel: str, ops_rel: str, errors: list[str]) -> None:
    skill = REPO / skill_rel
    ops = skill / ops_rel
    where = f"{service} ({skill_rel}/{ops_rel})"
    if not ops.exists():
        errors.append(f"{where}: operations file missing")
        return
    skill_md = skill / "SKILL.md"
    if not skill_md.exists() or f"]({ops_rel})" not in skill_md.read_text(encoding="utf-8"):
        errors.append(f"{where}: SKILL.md does not link {ops_rel}; no agent would load it")
    text = ops.read_text(encoding="utf-8")
    headings = re.findall(r"^## (.+?)\s*$", text, re.MULTILINE)
    found = [h for h in headings if h in SECTIONS]
    if found != SECTIONS:
        missing = [s for s in SECTIONS if s not in headings]
        errors.append(f"{where}: sections must be exactly {SECTIONS} in order; missing={missing}, found={found}")
        return
    rows = ledger_status(skill)
    starts = [m.start() for m in re.finditer(r"^## ", text, re.MULTILINE)] + [len(text)]
    for begin, end in zip(starts, starts[1:]):
        block = text[begin:end]
        title = block.splitlines()[0][3:].strip()
        if title not in SECTIONS:
            continue
        statuses = STATUS_RE.findall(block)
        if len(statuses) != 1:
            errors.append(f"{where} [{title}]: expected one **Status:** line, found {len(statuses)}")
            continue
        status = statuses[0]
        if status not in STATUSES:
            errors.append(f"{where} [{title}]: status '{status}' not in {sorted(STATUSES)}")
            continue
        cited = MARKER_RE.findall(block)
        if status in NEEDS_CITATION and not cited:
            errors.append(f"{where} [{title}]: status {status} needs at least one evidence citation")
        for evidence in cited:
            if evidence not in rows:
                errors.append(f"{where} [{title}]: {evidence} has no row in the skill's evidence ledger")
            elif status == "VERIFIED" and rows[evidence] != "VERIFIED":
                errors.append(f"{where} [{title}]: VERIFIED section cites {evidence}, whose ledger status is {rows[evidence]}")
        if title == "Cost":
            for price in PRICE_RE.findall(block):
                errors.append(f"{where} [Cost]: unit price or currency amount '{price}'; record billing dimensions only")


def main() -> int:
    errors: list[str] = []
    for service, (skill_rel, ops_rel) in SERVICES.items():
        check(service, skill_rel, ops_rel, errors)
    if errors:
        for error in errors:
            print(f"FAIL: {error}")
        print(f"validate-nosql-operations: {len(errors)} problem(s)")
        return 1
    print(f"validate-nosql-operations: OK ({len(SERVICES)} services x {len(SECTIONS)} sections)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
