# Help — stop route

Show `select`, `design`, `refactor`, `audit`, `performance-test`, `feature-plan`,
`backup`, `restore`. Explain NoSQL-only scope, available feature IDs and that this
is guidance plus an offline selector, not host registration/cloud execution.
Show one selected example if asked, then stop. Do not inspect repository files,
credentials, installed SDKs, subscription, resources or source ledger from help.

Feature IDs: core, indexes, capacity, iam, encryption, ttl, backup, replication,
change-feed, vectors, embeddings, monitoring, provisioning.
Audit defaults offline. Backup defaults continuous; restore defaults
continuous-new-account. Features default core (recovery: backup).
Other APIs and AWS-specific features/method names fail, not silently remap.
