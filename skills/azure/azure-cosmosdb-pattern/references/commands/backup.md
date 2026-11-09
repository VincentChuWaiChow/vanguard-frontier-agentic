# Backup / protection dispatcher

This is a policy/window review and change plan, **not** a user-named snapshot API.
Default continuous. Read [recovery target](recovery-target.md) and exactly one
method: [continuous](backup/continuous.md) OR [periodic](backup/periodic.md).
No named/on-demand backup create/list/download commands are provided because
Cosmos platform-managed backups aren't exposed as DynamoDB backup objects.

Capture policy/tier/retention, observed recoverable bounds, keys/identities,
regions, permission owner, RPO/RTO and last validated restore drill. Feature
selection cannot claim a backup exists or satisfy business continuity goals.
