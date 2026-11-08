# Oracle NoSQL SDK compatibility matrix

Load this only when the task involves application code. These are the
**Oracle NoSQL SDKs**. Do not substitute Oracle Database drivers (JDBC,
`python-oracledb`, `node-oracledb`). They target a different product.

"Documentation-verified" means the package name and API shape were read from
the provider's README or docs on the retrieval date. No example below was
compiled or executed in this repository.

| Language | Package | Runtime | Client / auth entry point | Status | Evidence |
| --- | --- | --- | --- | --- | --- |
| Java | Maven `com.oracle.nosql.sdk:nosqldriver` | Java 11+ | `NoSQLHandleConfig` + `SignatureProvider`, `NoSQLHandleFactory.createNoSQLHandle` | documentation-verified | [OCI-NOSQL-E17] [OCI-NOSQL-E20] |
| Python | `borneo` (+ `oci` for the cloud service) | Python 3.12+ | `NoSQLHandle(NoSQLHandleConfig(endpoint, provider))`, `borneo.iam.SignatureProvider` | documentation-verified | [OCI-NOSQL-E18] |
| Node.js / TypeScript | `oracle-nosqldb` | Node.js 24.0.0+; TypeScript 5.0.x+ | `NoSQLClient` | documentation-verified | [OCI-NOSQL-E19] |
| Go, .NET, Spring Data, Rust | official repos exist | not verified | not verified | unverified | [OCI-NOSQL-E16] [OCI-NOSQL-E21] |

Version pins: the Java README shows a version number inside its Maven
snippet. That is an example, not a statement of the latest release. Resolve the
current version from the package registry at implementation time; never copy a
pinned version from this file.

## Rules for generated code

1. **Authentication.** Use `SignatureProvider` with the user's OCI config
   profile, or an instance/resource principal where the runtime supports it.
   Never embed keys, fingerprints, tenancy OCIDs or passphrases in code.
   Instance/resource-principal class names were not verified in this pass, so
   look them up in the SDK API reference before writing them. *(rule)*
2. **Consistency.** Default to EVENTUAL and opt specific reads into ABSOLUTE
   (2x read units) [OCI-NOSQL-E03].
3. **Table creation in code.** `TableRequest` with DDL and `TableLimits` is
   the documented Java shape [OCI-NOSQL-E20]. Table DDL in application
   start-up paths is a production-mutation risk: keep it in deployment
   tooling behind approval. *(recommendation)*
4. **Unsupported languages.** For Go, .NET, Spring Data or Rust, say the
   details are unverified [OCI-NOSQL-E21]. Fetch that SDK's README before
   writing any code. Do not translate Java method names into another language.
5. **Throttling and retries.** Retry behavior and the exception class names
   for limit-exceeded errors were not verified in this pass. Look them up in the
   SDK API reference; do not invent exception names.
