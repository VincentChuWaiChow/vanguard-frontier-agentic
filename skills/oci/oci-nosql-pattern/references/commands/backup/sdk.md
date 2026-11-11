# Application-owned SDK export alternative

Design, do not run: bounded primary-shard/SQL iteration into versioned JSON with
schema, lossless types, original row expiration, hashes and progress manifest.
Follow every continuation even on empty pages; checkpoints bind exact query/tenant/
schema and resume only against unchanged quiesced source. SDK pagination is NOT a
snapshot, native backup, CDC or PITR. Budget every retry/read/storage write. Keep
all writes quiesced or mark the artifact non-atomic and unable to meet exact-point
RPO. Test complete key coverage, duplicate handling and restore with representative
type/TTL cases. Do not promote the query helper into a complete backup tool.
