# Routing Taxonomy

The thirteen domains, the signals that select each one, and the boundaries that keep them apart.

- `type-soundness` owns whether a type abstraction in shared or published code proves what its signature claims — variance, type predicates, conditional and mapped types, `satisfies` versus annotation, branded types. It does NOT own frontend application diffs: those belong to `typescript-contracts-agent` on the frontend board. If the artifact is a frontend application diff the frontend agent owns it; if it is the type model of a library, service, or shared package the TypeScript board owns it; if both, the TypeScript board owns the type model and hands the diff audit back.
- `runtime-boundary-contract` owns every point where external data enters the program — HTTP, queues, environment and configuration, database reads, third-party SDKs, webhooks, file input, agent and tool calls — and the ruling that a generated type is a claim about a producer rather than a check on a payload. It does NOT own exploitation, authorization, or secrets, which belong to the application security board.
- `module-resolution-and-emit` owns whether a package resolves, imports, and emits correctly for every consumer mode it claims to support; `node-execution-compatibility` owns whether the code runs on the target Node and is type-checked somewhere. A failure observed in a consumer's build routes to resolution; a failure observed at runtime routes to execution.
- `build-graph-performance` owns the TypeScript program graph measured with evidence; `monorepo-dx-agent` on the frontend board owns the task graph and remote caching. A request to speed something up without a measurement is a refuse-and-ask, not a dispatch.
- `static-enforcement-policy` owns what the toolchain must prove and at what cost across packages; `type-soundness` owns whether one construct is sound. A question about which flags or lint rules the fleet must run routes to enforcement; a question about whether a specific predicate lies routes to soundness.
- `package-publication-integrity` owns publish authority, provenance, and what ships in the tarball; dependency intake, lockfile policy, and install-time scripts belong to `package-governance-agent` on the frontend board.
- `mcp-tool-contract` owns whether a declared MCP tool contract matches its TypeScript handler; whether to trust a specific third-party MCP server belongs to `agentic-security-review-agent`; transport and hosting belong to the `mcp/` references; organization MCP trust policy belongs to its named human policy owner; vendor connectors belong to their own agents.
- `engineering-economics` is never dispatched first: it consumes measurements another specialist produced and refuses to originate one. A cost question with no supplied measurements routes to the specialist who can measure it, not to economics.

## Routing table

| Agent | Route when the task is about… |
|---|---|
| `typescript-type-soundness-agent` | whether a type abstraction in shared or published code proves what its signature claims: variance, type predicates, conditional and mapped types, `satisfies`, branded types |
| `typescript-runtime-boundary-contract-agent` | external data entering the program — HTTP, queues, environment and configuration, database reads, third-party SDKs, webhooks — or a generated type trusted as if it were a check on the payload |
| `typescript-module-resolution-and-emit-agent` | how a package resolves, imports, or emits for its consumers: `exports`, condition ordering, ESM/CJS, `.mts`/`.cts`, the dual-package hazard |
| `typescript-node-execution-compatibility-agent` | whether the code runs on the target Node and is type-checked anywhere: type stripping, unsupported syntax, a missing `tsc --noEmit` gate |
| `typescript-public-api-and-declaration-governance-agent` | a `.d.ts` or exported-type change, a semver decision, declaration emit, or consumer compilation and type-level tests |
| `typescript-build-graph-performance-agent` | compile or editor slowness backed by a measurement: project references, `composite`, `.tsbuildinfo`, `--generateTrace` |
| `typescript-static-enforcement-policy-agent` | what the toolchain must prove and at what cost: strict-family policy, per-package divergence, typed-lint rules and Project Service, editor/CI parity |
| `typescript-async-contract-reliability-agent` | promises, cancellation, backpressure, or concurrency bounds on the server: floating promises, `AbortSignal`, unhandled rejections |
| `typescript-package-publication-integrity-agent` | who may publish and what ships: trusted publishing, provenance, the tarball and types surface, registry and scope configuration |
| `typescript-estate-modernization-governor-agent` | sequencing a migration or compiler-major upgrade across packages: staged strictness, suppression debt, burn-down |
| `typescript-mcp-tool-contract-agent` | an MCP tool schema, protocol version, error contract, cancellation, or drift between a declared contract and its handler |
| `typescript-business-critical-automation-governance-agent` | a privileged script — backfill, migration, reconciliation, admin CLI — and its dry-run, idempotency, blast radius, and rollback |
| `typescript-engineering-economics-agent` | what something costs or what is worth funding, when the measurements are supplied rather than requested |
