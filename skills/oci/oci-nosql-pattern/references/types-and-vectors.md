# Types, numeric fidelity and native capability boundary

NDCS supports fixed schema plus JSON: INTEGER(signed 32-bit),LONG(signed 64-bit),
FLOAT/DOUBLE, arbitrary-precision decimal NUMBER, STRING, BOOLEAN, BINARY/
FIXED_BINARY, TIMESTAMP, ENUM, ARRAY, MAP, RECORD and JSON. A field's optional/null
behavior belongs to its schema, not DynamoDB AttributeValue tags. Distinguish SQL
NULL, absent JSON fields and JSON null in queries. JavaScript ordinary numbers
cannot losslessly carry all LONG/NUMBER values; use the selected SDK's lossless
mapping and test round trips (PHP integer width varies). Encoding bytes are not
business precision or a vector dimension limit.

| Need | Correct boundary |
|---|---|
| Store embedding coordinates | `ARRAY(DOUBLE)` or validated numeric array inside JSON; dimension/type/finite checks are application contracts |
| Native vector type/ANN/HNSW/IVF query | Not established by the reviewed NDCS feature/type/index/API docs; this skill does not support native activation or fabricate vector DDL/search commands |
| Similarity retrieval | Separately approved Oracle AI Database/other vector service, with its own schema/index/IAM/API; not NoSQL's secondary indexes |
| Generation | External embedding model invocation; a source URI/ID/string is not a vector |

No verified NDCS-specific ANN dimension cap exists to quote here. Ordinary row and
index key limits still apply; an array within a 512 KB row is not an ANN promise.
Choose dimension from the exact model/version and downstream store limits; reject
length mismatch, NaN/Infinity, mixed spaces and unauthorized tenants. Model-specific
precision/normalization/distance and evaluation must be explicit.

[Types](https://docs.oracle.com/en/cloud/paas/nosql-cloud/rnpxl/),
[NDCS features](https://docs.oracle.com/en/cloud/paas/nosql-cloud/mmmmq/),
[distinct Oracle AI Database vector scope](https://docs.oracle.com/en/cloud/paas/autonomous-database/vcapi/concepts/oracle-ai-database-vector-search.html), checked 2026-11-08.
