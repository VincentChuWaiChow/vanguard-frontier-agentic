# Feature lifecycle plan

Return proposed current→desired delta, prerequisites, exact targets, IAM/key cost,
blast radius, preview, rollback and verification. `ttl` supports planned enable/
disable/configure; disabling table default does not resurrect expired rows or
rewrite existing expiration. indexes/replicas allow enable/configure, not generic
disable (destruction needs its own approved plan). IAM/encryption/monitoring/
embeddings/backup are configure-only. Native vector enablement and PITR reject.
Never add `kms_key_id` to a table: dedicated CMEK environment eligibility and
service-owner procedures come first. No generic switches across every bundle.
