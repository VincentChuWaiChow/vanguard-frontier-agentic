# Narrow implementation workflow

1. Declare API, workload/tenant trust, existing owner/language/SDK and evidence
   boundaries. Run explicit-date review reminder; don't inspect live state by default.
2. Select only requested features. Build access patterns before choosing partition
   key, embedding/reference strategy, indexes and capacity; explain alternatives.
3. Define deterministic negative tests: tenant mismatch, missing/wrong partition,
   ETag conflict, continuation/session behavior, empty page, 429/deadline, unknown
   write result, TTL expiry and incompatible model/vector policy.
4. Produce minimal existing-stack code and requested IaC/CLI; no auto-provisioning,
   ORM adoption, credentials generation or install. Preserve current source of truth.
5. Run offline syntax/contracts/fakes, inspect actual diffs and report source gaps.
   Only separately authorized live metadata checks and workload/drill validation
   can prove deployed behavior. Keep deployment/readback/runtime evidence distinct.

Deliver selected scope, assumptions, access matrix, small change/example,
compatibility/migration and validation plan, evidence, remaining risks and
PASS/MISS/UNVERIFIED. Don't claim that instructions register or enforce host routing.
