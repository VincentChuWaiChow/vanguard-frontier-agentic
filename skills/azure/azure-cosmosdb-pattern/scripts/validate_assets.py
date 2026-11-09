#!/usr/bin/env python3
"""Offline illustrative-model invariants, NOT an Azure ARM schema validator."""
import json
from pathlib import Path


def validate(template, indexing):
    resources = template["resources"]
    if len(resources) != 3:
        raise ValueError("expected account/database/container only")
    account, db, container = resources
    if [r["type"] for r in resources] != [
        "Microsoft.DocumentDB/databaseAccounts", "Microsoft.DocumentDB/databaseAccounts/sqlDatabases",
        "Microsoft.DocumentDB/databaseAccounts/sqlDatabases/containers"]:
        raise ValueError("native resource types changed")
    if any(r["apiVersion"] != "2025-04-15" for r in resources):
        raise ValueError("review pinned API version change")
    if account["kind"] != "GlobalDocumentDB" or account["name"] != "[parameters('accountName')]":
        raise ValueError("NoSQL account/name mismatch")
    p = account["properties"]
    expected = {"databaseAccountOfferType": "Standard", "disableLocalAuth": True,
                "minimalTlsVersion": "Tls12", "publicNetworkAccess": "Disabled",
                "enableAutomaticFailover": False, "enableMultipleWriteLocations": False,
                "consistencyPolicy": {"defaultConsistencyLevel": "Session"},
                "backupPolicy": {"type": "Continuous", "continuousModeProperties": {"tier": "Continuous7Days"}}}
    if any(p.get(key) != value for key, value in expected.items()):
        raise ValueError("account security/capacity/backup invariants changed")
    if "keyVaultKeyUri" in p or "identity" in account or p.get("capabilities"):
        raise ValueError("base model isn't an implicit CMK/serverless/vector configuration")
    if p["locations"] != [{"locationName": "[parameters('location')]", "failoverPriority": 0, "isZoneRedundant": False}]:
        raise ValueError("base region assumptions changed")
    if db["properties"] != {"resource": {"id": "ordersdb"}, "options": {}}:
        raise ValueError("DB shared throughput or ID drift")
    c = container["properties"]
    if c["options"] != {"throughput": 400}:
        raise ValueError("dedicated manual capacity required; no mixed autoscale/serverless")
    if c["resource"]["id"] != "orders" or c["resource"]["partitionKey"] != {"paths": ["/tenantId"], "kind": "Hash", "version": 2}:
        raise ValueError("partition/ID mismatch")
    if c["resource"].get("defaultTtl") != -1 or c["resource"]["indexingPolicy"] != indexing:
        raise ValueError("TTL/indexing CLI-vs-ARM drift")
    expected_index = {"indexingMode": "consistent", "automatic": True,
        "includedPaths": [{"path": "/*"}],
        "excludedPaths": [{"path": "/_etag/?"}, {"path": "/payload/*"}],
        "compositeIndexes": [[{"path": "/createdAt", "order": "descending"}, {"path": "/id", "order": "ascending"}]]}
    if indexing != expected_index:
        raise ValueError("example index doesn't match access pattern")
    return {"status": "PASS", "scope": "local example invariants only", "azure_acceptance": "UNVERIFIED"}


if __name__ == "__main__":
    root = Path(__file__).resolve().parents[1] / "assets"
    print(json.dumps(validate(json.loads((root / "account.arm.json").read_text()),
                              json.loads((root / "indexing-policy.json").read_text())), indent=2))
