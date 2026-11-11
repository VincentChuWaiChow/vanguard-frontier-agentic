# Design

1. Confirm NoSQL API, existing ownership, access-pattern matrix, tenant trust,
   item sizes/rates/growth, latency/consistency, transaction and recovery goals.
2. Specify `id` and exact partition-key path/value, point-read vs routed-query vs
   cross-partition paths, index policy, per-pattern RU/latency estimates and hot-key
   mitigation. State assumptions rather than inventing production measurements.
3. Compare embed/reference/materialized-view and tenant/container/account choices;
   don't default to one container per tenant or copy a DynamoDB single-table model.
4. Load selected IAM/key/vector/recovery guides only when in scope. If code/IaC
   requested, use the existing language/owner; load selected adapter/assets only.
5. Deliver matrix/model, invariants, small examples, migration/verification plan
   and explicit unresolved constraints. Creating resources is a separate approval.
