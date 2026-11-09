# Feature lifecycle plan

Generate a proposal, not a mutation. Confirm current/desired state, API/region/
SDK and IaC owner, dependencies, irreversibility, cost, compatibility, preview/
rollout/readback/rollback and approval boundary. Generic actions are conservative:

| Feature | Accepted generic action | Native limitation |
|---|---|---|
| ttl | enable / disable / configure | Disable changes expiry; never restores deleted items |
| vectors | enable / configure | No generic disable once enabled; policy changes require verified migration path |
| indexes, capacity, iam, encryption, backup, replication, change-feed, embeddings, monitoring | configure | No invented global on/off toggle; exact native property/lifecycle required |
| core, provisioning | none | Use design/refactor/provisioning, not feature toggles |

Backup is always present (periodic default or continuous). Continuous → periodic
isn't supported. Shorter retention can destroy recovery options. Encryption at
rest isn't an on/off feature. CMK disable/revoke/delete and network opening never
appear as incidental setup/rollback. Changes to data roles/network are separate
from feature availability. Multi-region isn't a backup for logical corruption.
