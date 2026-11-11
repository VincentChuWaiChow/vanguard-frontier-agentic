# Default managed encryption versus dedicated CMEK

| Choice | Scope / precondition | Recovery dependency |
|---|---|---|
| Oracle-managed master key | NDCS default encryption at rest; application still owns IAM/network/backup security | verify target/table settings, TLS and backup storage controls |
| Customer-managed master key | NDCS dedicated hosted environment configured for CMEK, OCI Vault/KMS and service-owner onboarding | preserve required keys, grants and environment eligibility throughout retained backups/recovery |

Do not invent a per-table Terraform `kms_key_id`/SSE switch in `oci_nosql_table`.
The bundled IaC is the default-managed path only. Dedicated CMEK configuration is
a separate owner-approved environment workflow; no live change is supplied here.
Rotation rewraps storage keys rather than re-encrypting every application record.
Disable/delete, revoked grants, inaccessible Vault or lost old-key dependencies can
break availability/recovery; stop destructive key actions and obtain a full usage/
backup dependency review. Object Storage export encryption is a separately selected
bucket/key setting, not automatically identical to table encryption. Never print
plaintext customer records to prove encryption.

[Oracle CMEK scope/lifecycle](https://docs.oracle.com/en/cloud/paas/nosql-cloud/icxer/), checked 2026-11-08.
