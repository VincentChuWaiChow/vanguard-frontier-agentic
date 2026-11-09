# Encryption and key lifecycle

At-rest encryption is always enabled. Don't import AWS's three-key-choice model.

| Choice | Configuration | Ownership/dependency |
|---|---|---|
| Service-managed (default) | Omit `keyVaultKeyUri` / `--key-uri` | Cosmos manages service encryption; omission is not plaintext |
| Customer-managed (CMK) | Account `keyVaultKeyUri`, declared/default identity and matching Key Vault permissions | Customer owns key availability/rotation/access/retention; extra encryption layer and operational dependency |
| Client-side encryption | Separate SDK/application design | Not provided automatically by either server-side choice; query/index/key exposure constraints differ |

Base ARM/CLI examples use the service-managed default consistently. Never pretend
setting an arbitrary Key Vault URI alone is a complete CMK deployment.

For a **reviewed new CMK account variant**, preserve account/database/container
model and add an existing approved user-assigned identity resource ID to account
`identity.type: UserAssigned`, `identity.userAssignedIdentities: {<id>: {}}`,
`properties.defaultIdentity: UserAssignedIdentity=<id>` and
`properties.keyVaultKeyUri: https://<vault>.vault.azure.net/keys/<key>` (versionless
URI for supported rotation flow). CLI equivalents: `--assign-identity "$UAMI_ID"`
`--default-identity "UserAssignedIdentity=$UAMI_ID" --key-uri "$KEY_URI"`.
This fragment is illustrative; resolve exact API/identity lifecycle and perform
authorized readback before deployment. Never create/grant/purge keys as a side effect.

Verify Key Vault soft delete AND purge protection, supported RSA key/region/cloud,
key enabled/expiry, Get/Wrap Key/Unwrap Key (or exact equivalent RBAC role such as
Key Vault Crypto Service Encryption User), correct principal/default identity,
service network reachability and existing policy ownership. Current docs require
public accessibility or trusted-service firewall bypass; a vault accessible only
through private endpoints isn't sufficient for this integration. Never “fix”
reachability by opening the vault/network without separate security review.

Continuous backup with CMK requires a system- or user-assigned managed identity;
Cosmos first-party identity isn't supported for this continuous-backup case.
Periodic backup restore can require the original key version and temporary
first-party identity access if source used system-assigned identity. Do not blindly
apply that periodic recipe to continuous backup. Preserve old/new versions and
permissions during rotation and through dependent backup retention/drill windows.

A restored CMK account must have verified key access/default identity, not merely
copied source identity metadata. Current CMK docs warn identity metadata can remain
source-bound; assign/rescope a reviewed destination identity with appropriate key
access, and prove destination works independently before any source retirement.
Do not delete source/identity or retire old keys after a successful restore waiter.
Revocation can block data/recovery; disabled/deleted keys can render data inaccessible.
No key deletion/disable/purge command belongs in cleanup/rollback examples here.

Metadata such as resource names, indexed paths and partition-key values isn't
all hidden by CMK; don't put secrets into those fields. Server-side key choice
doesn't add tenant row filtering or change item SDK signatures. Azure OpenAI,
Blob Storage, AI Search and logs have separate encryption/identity lifecycles.

Sources: [encryption](https://learn.microsoft.com/azure/cosmos-db/database-encryption-at-rest),
[CMK identity, backups and restore](https://learn.microsoft.com/azure/cosmos-db/how-to-setup-cmk).
