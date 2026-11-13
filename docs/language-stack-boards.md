# Language and Stack Boards

Language and stack boards are topical agent collections scoped to a language,
runtime, or functional domain rather than to a cloud provider. They live
alongside the provider boards (`aws`, `azure`, `gcp`, and others) and share the
same `provider` faceting axis: each shipped topical board has its own dedicated
`provider` enum value.

This document covers the current boards: `frontend`, `.NET`, `Java`, `Kotlin`, `PHP`, `Python`, `TypeScript`, `legal`, `hr`, `marketing`,
`salesforce`, `netsuite`, `atlassian` (Jira, Jira Service Management and Confluence work intelligence), `accounting`, `finance`, `sap` (SAP S/4HANA + BTP
enterprise board), `microsoft` (Microsoft 365 / Dynamics 365), `databricks`,
and `snowflake` (data and analytics platforms). It also describes how to use
them for discovery and how to add a new board.

See [taxonomy.md](taxonomy.md) for the full provider and asset-type taxonomy
that governs all boards in this marketplace.

---

## What are language and stack boards?

A **language/stack board** is a dedicated directory pair — `agents/<name>/` and
`skills/<name>/` — plus a shared ID prefix (`<name>-*`) for all assets in that
collection. The board groups agents and skills that share a common subject area:
a language ecosystem (`.NET`), a professional function (`legal`, `hr`), or a
compliance domain (`marketing`).

Every board also gets an entry in `catalog/install-roles.json` under a named
install role so users can pull the full set with a single `--role` flag.

### How they differ from provider boards

| Dimension | Provider board (e.g. `aws`) | Language/stack board (e.g. `dotnet`) |
|-----------|----------------------------|--------------------------------------|
| `provider` field | `aws`, `azure`, `gcp`, … | dedicated board name (`dotnet`, `php`, `legal`, `hr`, `marketing`, `salesforce`, `netsuite`, `accounting`, `finance`, `sap`, `microsoft`, `databricks`, `snowflake`) |
| Directory | `agents/aws/` | `agents/dotnet/`, `agents/legal/`, … |
| ID prefix | `aws-*` | `dotnet-*`, `php-*`, `legal-*`, `hr-*`, `marketing-*`, `salesforce-*`, `netsuite-*`, `sap-*`, `microsoft-*`, `databricks-*`, `snowflake-*`, etc. |
| Subject scope | Cloud service surface | Language/runtime or professional function |
| Execution tier | Varies by agent | `static-review` (all language/stack boards, except the `python` board's governed `read-only-runtime`/`mutating-runtime` live control plane) |
| Faceting axis | `provider` enum | `provider` enum (dedicated value) plus shared ID prefix |

Provider boards target infrastructure and cloud services. Language/stack boards
target code quality, legal posture, HR process risk, and compliance governance.
The two sets coexist in the same catalog and can be installed independently or
together.

---

## Provider values for topical boards

The `provider` field is a faceting axis. It started as a cloud/platform axis,
but it also carries non-cloud topical boards: each shipped board gets its own
dedicated `provider` enum value. `dotnet`, `java`, `hr`, `legal`, `marketing`,
`salesforce`, `netsuite`, `atlassian`, `accounting`, `finance`, `sap`, `microsoft`,
`databricks`, `snowflake`, and `php` are all first-class `provider` values, listed in
`docs/taxonomy.md` under **Providers** and accepted by the schema and catalog
validators.

A dedicated `provider` value lets users filter the board directly — for
example `npx vfa-export-agents --platform claude-code --provider dotnet`
installs the entire `.NET` board. The shared ID prefix (`dotnet-*`, `hr-*`,
`legal-*`, `marketing-*`, `sap-*`, `microsoft-*`, `databricks-*`,
`snowflake-*`) remains the secondary discovery key and stays stable
even if the board's `provider` value ever changes.

A new topical board uses `provider: generic` only until it ships a coherent
agent/skill set; at that point it is promoted to its own `provider` value. The
`qa` board is the current pre-promotion example — its assets still declare
`provider: generic`. Promotion changes the `provider` field, the schema and
validator enums, and the catalog entries, but never the ID prefix.

---

## The boards

### .NET

The `.NET` board covers static review of C# applications and the surrounding
.NET ecosystem: runtime correctness, ASP.NET Core API architecture,
authentication and authorization, EF Core data access, test quality, CI/NuGet
supply-chain integrity, performance and Native AOT readiness, in-application
OpenTelemetry wiring, and .NET Aspire cloud-native posture.

| Property | Value |
|----------|-------|
| `provider` | `dotnet` |
| ID prefix | `dotnet-*` |
| Agent directory | `agents/dotnet/` |
| Skill directory | `skills/dotnet/` |
| Agents | <!-- count:board:dotnet:agents -->10<!-- /count --> |
| Skills | <!-- count:board:dotnet:skills -->10<!-- /count --> (1:1 companion skill per agent) |
| Install role | `dotnet-application-review-engineer` |
| Execution tier | `static-review` (all agents) |

**Agent directory layout**

```
agents/dotnet/
  dotnet-maestro-agent/
  dotnet-csharp-runtime-review-agent/
  dotnet-aspnetcore-api-review-agent/
  dotnet-aspnetcore-identity-authz-review-agent/
  dotnet-efcore-data-access-review-agent/
  dotnet-testing-quality-review-agent/
  dotnet-supply-chain-review-agent/
  dotnet-performance-aot-review-agent/
  dotnet-observability-otel-review-agent/
  dotnet-aspire-cloud-native-review-agent/
```

**Example agents**

| Agent | Scope |
|-------|-------|
| `dotnet-maestro-agent` | Router; classifies a `.NET` task and dispatches the narrowest specialist or a parallel team of up to four. Never answers `.NET` questions itself. |
| `dotnet-efcore-data-access-review-agent` | EF Core: DbContext lifetime, N+1 patterns, raw SQL safety, concurrency tokens, migrations, multi-tenant query filters. |
| `dotnet-supply-chain-review-agent` | CI/CD and NuGet supply chain: SDK pinning, package locking, feed trust, fork-PR secret exposure, vulnerability scanning. |
| `dotnet-aspire-cloud-native-review-agent` | .NET Aspire AppHost and service-defaults review for cloud-native readiness. |

Each agent reads source and sanitized configuration only. No agent runs
`dotnet build`, `dotnet test`, `dotnet ef`, contacts a live system, or edits
project files.

---

### Java

The `java` board covers adversarial, evidence-first static review of Java and
JVM enterprise applications: JDK-estate lifecycle and upgrade risk, virtual-thread
adoption correctness, JVM performance and GC diagnosis, containerized-JVM sizing,
framework production-readiness (Spring Boot / Quarkus / Micronaut), Spring
Security authorization and endpoint exposure, untrusted deserialization/parser
RCE surface, JPA/Hibernate fetch performance, transaction and cross-resource
consistency, schema-migration deploy safety, Kafka delivery semantics,
resilience-pattern composition, JVM test architecture, and the
application-server license-exit portfolio decision.

| Property | Value |
|----------|-------|
| `provider` | `java` |
| ID prefix | `java-*` |
| Agent directory | `agents/java/` |
| Skill directory | `skills/java/` |
| Agents | <!-- count:board:java:agents -->15<!-- /count --> |
| Skills | <!-- count:board:java:skills -->15<!-- /count --> (1:1 companion skill per agent) |
| Install role | `java-application-review-engineer` |
| Execution tier | `static-review` (all agents) |

**Agent directory layout**

```
agents/java/
  java-maestro-agent/
  java-jdk-lifecycle-and-upgrade-agent/
  java-concurrency-and-virtual-thread-agent/
  java-jvm-performance-and-gc-agent/
  java-container-and-kubernetes-readiness-agent/
  java-framework-production-readiness-agent/
  java-spring-security-agent/
  java-deserialization-and-parser-security-agent/
  java-jpa-hibernate-performance-agent/
  java-transaction-and-consistency-agent/
  java-database-migration-safety-agent/
  java-kafka-reliability-agent/
  java-resilience-pattern-agent/
  java-test-architecture-agent/
  java-application-server-exit-agent/
```

**Example agents**

| Agent | Scope |
|-------|-------|
| `java-maestro-agent` | Router; classifies a Java/JVM task and dispatches the narrowest specialist or a parallel team of up to four. Never answers Java questions itself. |
| `java-jdk-lifecycle-and-upgrade-agent` | JDK-estate upgrade/hold decision: flags a fleet approaching an LTS support or license boundary and prescribes an upgrade path, gated on supplied vendor-verified dates. |
| `java-jpa-hibernate-performance-agent` | JPA/Hibernate fetch-strategy correctness: N+1 risk, `JOIN FETCH` vs `@EntityGraph` vs `@BatchSize` vs DTO projection, and `open-in-view` misuse. |
| `java-deserialization-and-parser-security-agent` | Untrusted-deserialization/parser RCE surface: `ObjectInputStream` gadget chains, SnakeYAML `Constructor`, and Jackson polymorphic default typing without a validator. |

Each agent reads source and sanitized configuration only. No agent runs a build,
executes tests, opens a database or broker connection, contacts a live system, or
edits project files.

---

### Kotlin

The `kotlin` board covers adversarial, evidence-first static review of Kotlin
across the JVM, Android, and Kotlin Multiplatform: language and null-safety
correctness, coroutine and Flow reliability, public library API/ABI governance,
Ktor and Kotlin-on-Spring backend production readiness, kotlinx.serialization
wire contracts, Java-to-Kotlin estate modernization, Android lifecycle-aware
architecture, Jetpack Compose UI quality and accessibility, MASVS-aligned Android
security and privacy, measured Android runtime performance, the Kotlin
Multiplatform adopt-or-not decision and its source-set/interop boundary, Gradle
build engineering, Kotlin dependency and release supply-chain integrity, and
coroutine/Compose/KMP test architecture.

| Property | Value |
|----------|-------|
| `provider` | `kotlin` |
| ID prefix | `kotlin-*` |
| Agent directory | `agents/kotlin/` |
| Skill directory | `skills/kotlin/` |
| Agents | <!-- count:board:kotlin:agents -->16<!-- /count --> |
| Skills | <!-- count:board:kotlin:skills -->16<!-- /count --> (1:1 companion skill per agent) |
| Install roles | `kotlin-backend-engineer`, `android-kotlin-engineer`, `kotlin-multiplatform-engineer`, `kotlin-platform-build-engineer`, `kotlin-library-maintainer`, `kotlin-security-engineer`, `kotlin-engineering-leader` |
| Execution tier | `static-review` (all agents) |

**Agent directory layout**

```
agents/kotlin/
  kotlin-maestro-agent/
  kotlin-estate-modernization-governor-agent/
  kotlin-language-api-correctness-agent/
  kotlin-coroutines-flow-reliability-agent/
  kotlin-library-api-abi-governance-agent/
  kotlin-backend-production-readiness-agent/
  kotlin-serialization-wire-contract-agent/
  kotlin-android-architecture-agent/
  kotlin-compose-ui-quality-accessibility-agent/
  kotlin-android-security-privacy-agent/
  kotlin-android-performance-reliability-agent/
  kotlin-kmp-portfolio-decision-agent/
  kotlin-kmp-boundary-interop-agent/
  kotlin-gradle-build-engineering-agent/
  kotlin-supply-chain-release-integrity-agent/
  kotlin-test-architecture-agent/
```

**Example agents**

| Agent | Scope |
|-------|-------|
| `kotlin-maestro-agent` | Router; classifies a Kotlin/JVM/Android/KMP task and dispatches the narrowest specialist or a parallel team of up to four. Routes only — never reviews Kotlin work itself, and ejects Java-, cloud-, observability-, and signing-owned work to the right board. |
| `kotlin-coroutines-flow-reliability-agent` | Structured concurrency, cancellation cooperation, dispatcher/blocking confinement, cold-vs-hot Flow semantics, and context propagation across suspension — including the coroutine-aware persistence and trace/MDC/security-context hazards. |
| `kotlin-android-security-privacy-agent` | MASVS-aligned static review of exported components, deep links, WebView exposure, cleartext traffic, secret storage, backup exposure, permission minimization, and PII in logs. |
| `kotlin-kmp-portfolio-decision-agent` | The whether-to-share-code-at-all decision: organizational topology, roadmap alignment, platform differentiation, and reversibility — able to recommend against Kotlin Multiplatform. |

Each agent reads source and sanitized configuration only. No agent runs a build,
executes tests, opens a database or broker connection, contacts a live system, or
edits project files.

---

### Legal

The `legal` board covers adversarial, evidence-grounded triage of legal and
compliance risk: contracts, privacy, regulatory obligations, litigation holds,
IP, vendor procurement, ethics investigations, policy governance, and public
disclosure. It is scoped to the enterprise legal and compliance function across
the US, EU, UK, Singapore, and Australia.

| Property | Value |
|----------|-------|
| `provider` | `legal` |
| ID prefix | `legal-*` |
| Agent directory | `agents/legal/` |
| Skill directory | `skills/legal/` |
| Agents | <!-- count:board:legal:agents -->13<!-- /count --> |
| Skills | <!-- count:board:legal:skills -->1<!-- /count --> board-specific + 3 cross-functional (shared with `hr`) |
| Install role | `legal-hr-risk-reviewer` (shared with the `hr` board) |
| Execution tier | `static-review` (all agents) |

**Agent directory layout**

```
agents/legal/
  legal-maestro-agent/
  legal-counsel-review-agent/
  legal-contract-review-agent/
  legal-privacy-data-protection-agent/
  legal-employment-law-risk-agent/
  legal-litigation-discovery-hold-agent/
  legal-regulatory-compliance-agent/
  legal-ip-open-source-agent/
  legal-vendor-procurement-risk-agent/
  legal-ethics-investigations-agent/
  legal-policy-governance-agent/
  legal-public-disclosure-agent/
  legal-knowledge-management-agent/

skills/legal/
  legal-counsel-review/

skills/cross-functional/
  legal-hr-case-capsule/      # shared with hr board
  legal-hr-routing-protocol/
  legal-hr-risk-taxonomy/
```

**Example agents**

| Agent | Scope |
|-------|-------|
| `legal-maestro-agent` | Router; classifies a legal matter and routes it to the right specialist or coordinates multi-agent review. |
| `legal-contract-review-agent` | Contract clauses: indemnity, liability, termination, renewal, warranties, audit rights, governing law. |
| `legal-privacy-data-protection-agent` | Data protection posture, retention, cross-border transfer, DPIA readiness, vendor DPAs, employee-data processing. |
| `legal-ethics-investigations-agent` | Whistleblower, conflict of interest, anti-bribery, sanctions, and executive-misconduct intake triage. |

The `legal` board is paired with the `hr` board under a single install role
because cross-domain matters — employment disputes, investigations,
terminations — regularly require both. Cross-domain hand-off uses the
`legal-hr-case-capsule` skill; see
`docs/architecture/legal-hr-agent-routing.md`.

These agents give no legal advice and form no attorney-client relationship. All
outputs are risk-structured analysis for review by qualified counsel.

---

### HR

The `hr` board covers HR, employment-risk, and People-function review: employee
relations, workplace investigations, performance management, termination
readiness, leave and accommodation, recruiting, compensation equity, benefits,
workforce planning, HR analytics, culture and DEI, and HRIS process controls.

| Property | Value |
|----------|-------|
| `provider` | `hr` |
| ID prefix | `hr-*` |
| Agent directory | `agents/hr/` |
| Skill directory | `skills/hr/` |
| Agents | <!-- count:board:hr:agents -->15<!-- /count --> |
| Skills | <!-- count:board:hr:skills -->1<!-- /count --> board-specific + 3 cross-functional (shared with `legal`) |
| Install role | `legal-hr-risk-reviewer` (shared with the `legal` board) |
| Execution tier | `static-review` (all agents) |

**Agent directory layout**

```
agents/hr/
  hr-maestro-agent/
  hr-risk-triage-review-agent/
  hr-employee-relations-agent/
  hr-workplace-investigations-agent/
  hr-performance-management-agent/
  hr-termination-readiness-agent/
  hr-leave-accommodation-agent/
  hr-recruiting-selection-agent/
  hr-compensation-equity-agent/
  hr-benefits-payroll-agent/
  hr-workforce-planning-rif-agent/
  hr-learning-policy-agent/
  hr-analytics-people-data-agent/
  hr-culture-dei-agent/
  hr-hris-process-controls-agent/

skills/hr/
  hr-risk-triage-review/
```

**Example agents**

| Agent | Scope |
|-------|-------|
| `hr-maestro-agent` | Router; classifies an HR matter and routes it to the right specialist or coordinates cross-functional review. |
| `hr-risk-triage-review-agent` | Triage terminations, discipline, accommodations, wage/hour, discrimination, harassment, retaliation, and layoff risk for employment-law exposure. |
| `hr-workplace-investigations-agent` | Investigation planning, evidence mapping, witness sequencing, neutrality and confidentiality controls. |
| `hr-compensation-equity-agent` | Compensation, promotion, leveling, pay equity, incentives, calibration, and adverse-impact risk. |

No agent terminates, disciplines, denies leave, or sends an employee
communication. Every adverse or irreversible action routes to a named human
owner. These agents give no legal or HR advice and form no attorney-client
relationship.

---

### Marketing

The `marketing` board covers the marketing-technology compliance and security
surface: consent and data-collection posture (GDPR/ePrivacy/CCPA), advertising
pixel personal-data leakage, GPC signal handling, email sender authentication,
programmatic supply-chain integrity, AI advertising fairness, EU AI Act
applicability, lookalike audience upload compliance, email list retention,
influencer disclosure, and dark-pattern conversion flow review.

| Property | Value |
|----------|-------|
| `provider` | `marketing` |
| ID prefix | `marketing-*` (most agents); some use domain-specific prefixes |
| Agent directory | `agents/marketing/` |
| Skill directory | `skills/marketing/` |
| Agents | <!-- count:board:marketing:agents -->14<!-- /count --> |
| Skills | <!-- count:board:marketing:skills -->14<!-- /count --> (1:1 companion skill per agent) |
| Install role | `marketing-governance-reviewer` |
| Execution tier | `read-only-runtime` (maestro); the remaining <!-- count:board:marketing:specialists -->13<!-- /count --> agents declare no `execution_tier` |

**Agent directory layout**

```
agents/marketing/
  marketing-maestro-agent/
  marketing-consent-data-collection-review-agent/
  marketing-pixel-data-leakage-review-agent/
  martech-access-governance-review-agent/
  marketing-gpc-signal-honoring-review-agent/
  email-sender-authentication-review-agent/
  programmatic-supply-chain-integrity-review-agent/
  ai-advertising-targeting-fairness-review-agent/
  eu-ai-act-marketing-system-review-agent/
  lookalike-audience-upload-compliance-review-agent/
  marketing-email-list-retention-review-agent/
  influencer-disclosure-compliance-review-agent/
  marketing-conversion-flow-dark-pattern-review-agent/
  analytics-data-minimization-review-agent/
```

**Example agents**

| Agent | Scope |
|-------|-------|
| `marketing-maestro-agent` | Router; classifies a marketing-governance task and dispatches the appropriate specialist. |
| `marketing-pixel-data-leakage-review-agent` | Advertising pixels and conversion event tracking: PII leakage to ad networks, form-field auto-capture, pixel placement on sensitive pages. |
| `eu-ai-act-marketing-system-review-agent` | AI-powered marketing systems: EU AI Act risk-classification and prohibited-practice exposure. |
| `martech-access-governance-review-agent` | OAuth connected apps, API keys, CRM and marketing-automation roles, and integration scopes: least-privilege violations and stale credentials. |

Note: several agents in this board use non-`marketing-` prefixes
(`martech-*`, `ai-*`, `analytics-*`, `email-*`, `programmatic-*`,
`influencer-*`, `lookalike-*`, `eu-ai-act-*`). This reflects the
subject-area specificity of those agents. All live under `agents/marketing/`
and declare `provider: marketing` in `metadata.json`.

---

### PHP

The `PHP` board covers static review of PHP applications and the surrounding
PHP ecosystem: application security (session fixation, insecure
deserialization, file-upload exploits), Composer dependency supply-chain
governance, runtime version/EOL readiness with OPcache and PHP-FPM hardening,
and WordPress plugin/theme/REST API/block-editor security.

| Property | Value |
|----------|-------|
| `provider` | `php` |
| ID prefix | `php-*` (plus `composer-*`, `wordpress-*` for ecosystem-specific agents) |
| Agent directory | `agents/php/` |
| Skill directory | `skills/php/` |
| Agents | <!-- count:board:php:agents -->5<!-- /count --> |
| Skills | <!-- count:board:php:skills -->5<!-- /count --> (1:1 companion skill per agent) |
| Install role | `php-platform-engineer` |
| Execution tier | `static-review` (all agents) |

**Agent directory layout**

```
agents/php/
  php-maestro-agent/
  php-application-security-agent/
  composer-supply-chain-agent/
  php-runtime-upgrade-readiness-agent/
  wordpress-security-agent/
```

**Example agents**

| Agent | Scope |
|-------|-------|
| `php-maestro-agent` | Router; classifies a PHP task and dispatches the narrowest specialist. Never answers PHP questions itself. |
| `php-application-security-agent` | Session fixation, insecure deserialization, and file-upload exploit review in PHP application code. |
| `composer-supply-chain-agent` | Composer dependency audit: lockfile integrity, known-vulnerable packages, source trust. |
| `php-runtime-upgrade-readiness-agent` | PHP runtime end-of-life exposure, OPcache and PHP-FPM configuration hardening review. |
| `wordpress-security-agent` | WordPress REST API and block-editor (Gutenberg) security review. |

Each agent reads source, sanitized configuration, and dependency manifests
only. No agent runs `composer install`, `php`, `wp-cli`, contacts a live
system, or edits project files.

---

### Python

The `python` board covers adversarial, evidence-first static review of Python
applications and the surrounding runtime, framework, data, and packaging ecosystem:
application-security defects (unsafe deserialization, dynamic execution, subprocess and
shell injection, SSRF, path traversal, secrets, cryptography misuse), asyncio event-loop
reliability (blocking calls, cancellation, timeouts, structured concurrency,
backpressure), packaging and software-supply-chain integrity (pyproject metadata,
dependency locking and hash-checking, index trust and dependency confusion, build
isolation), numerical and scientific correctness (money-as-float, rounding, dtype
coercion, timezone handling, reproducibility), language contracts and gradual typing
(Any propagation, Protocols, generics and variance, overloads, TypedDict/dataclass),
web-service production readiness (FastAPI/Django/Flask/Starlette request lifecycle,
sync-vs-async endpoints, authorization, graceful shutdown, health checks), database
access and transactions (SQLAlchemy/Django ORM session and transaction scope, N+1,
connection pooling, migration safety), distributed task reliability (Celery/RQ/Dramatiq
idempotency, retries, dead-letters, duplicate execution), and test-suite quality
(pytest fixtures and isolation, mock misuse, determinism, coverage theater),
runtime-estate modernization (EOL/unsupported interpreters, upgrade sequencing,
deprecation exposure), performance and memory (profiling-vs-benchmarking rigor,
allocation/GC pressure, algorithmic complexity), free-threaded (no-GIL, PEP 703)
adoption (invalidated GIL assumptions, shared-state races, C-extension support),
native-extension interop (CPython C API reference ownership, stable ABI, buffer
protocol, PyO3/Cython boundaries), container and serverless runtime (PID 1 and
signals, worker model, graceful shutdown, cold start), data-pipeline reliability
(Airflow/Dagster/Prefect/PySpark idempotency, backfills, schema evolution),
ML/AI production (training-serving skew, leakage, artifact safety, reproducibility),
in-application observability (structured logs, trace context propagation, metric
cardinality, PII), developer tooling and build (gate efficacy, type/lint strictness,
CI matrix, build backend), and business-critical automation governance (unowned
scripts/notebooks, segregation of duties, reconciliation, key-person risk).

| Property | Value |
|----------|-------|
| `provider` | `python` |
| ID prefix | `python-*` |
| Agent directory | `agents/python/` |
| Skill directory | `skills/python/` |
| Agents | <!-- count:board:python:agents -->35<!-- /count --> (<!-- count:board:python:static -->20<!-- /count --> static-review board + <!-- count:board:python:live -->15<!-- /count --> live control plane) |
| Skills | <!-- count:board:python:skills -->35<!-- /count --> (1:1 companion skill per agent) |
| Install roles | 16 bundles: 10 static-review (`python-application-review-engineer` umbrella, `python-application-engineer`, `python-platform-reliability-engineer`, `python-data-engineer`, `python-ml-engineer`, `python-security-engineer`, `python-library-maintainer`, `python-automation-governance-lead`, `python-engineering-leader`, `python-reliability-data-engineer`) + 6 live-plane (`python-live-platform-operator`, `python-live-security-operator`, `python-live-data-operator`, `python-live-ml-governance-operator`, `python-live-automation-control-owner`, `python-live-audit-and-compliance-reviewer`) |
| Execution tier | `static-review` (<!-- count:board:python:static -->20<!-- /count --> board agents) **plus** `read-only-runtime` and `mutating-runtime` (<!-- count:board:python:live -->15<!-- /count --> live control-plane agents) — see the live control plane note below |

> [!IMPORTANT]
> **`python` is a deliberate, governed exception to the "language/stack boards are
> static-review only" posture.** It hosts a **live control plane** (<!-- count:board:python:live -->15<!-- /count --> agents under
> `python-live-*`) that interacts with live systems under controlled execution with
> provable accountability, routed by `python-live-governance-maestro-agent` — separate
> from the 20 static-review board agents routed by `python-maestro-agent`. Mutating
> operators are `mutating-runtime` **live-guards**: never auto-dispatched, gated behind an
> external signed approval bound to the target, target-scoped JIT credentials, a
> pre-approved rollback, and an immutable audit event (fail-closed if audit logging is
> unavailable for an R3+ action). This repo ships the governed **definitions, contracts,
> and eval fixtures** — not a running control plane; the audit log store, JIT issuance,
> approval system, and actual execution are the deploying organization's runtime, and
> compliance/legal classification remains its qualified owners' determination. See
> [evidence-output-spec.md](evidence-output-spec.md) and
> [docs/compliance/](compliance/).
>
> **Where the tier is mechanically enforced.** Be precise about this rather than assuming
> the tier is a sandbox everywhere. The read-only/mutating split is enforced *mechanically*
> in three places: the Codex adapter (`sandbox_mode` — `read-only` vs `workspace-write`),
> the companion `SKILL.md` (`allowed-tools` — only mutating operators are granted `Bash`),
> and the Copilot adapter (only mutating operators are granted `execute/*` tools). The
> Markdown-family adapters (claude-code, cursor, gemini, kiro-ide) carry `name` and
> `description` only — no adapter of those four types in this catalog declares a `tools:`
> block, and the `kiro-cli.agent.json` adapter has no tools key at all — so in those
> harnesses the tier is carried by the agent's operating rules and its bound skill's
> `allowed-tools`, not by a per-agent tool grant. A deploying organization that needs the
> boundary enforced in-harness must apply its own tool policy there; treat the agent
> contract as necessary, not sufficient. The Copilot grant is enforced catalog-wide by
> [execution-tiers.md](execution-tiers.md#harness-tool-grants-must-match-the-declared-tier)
> and the `validate:agent-tool-tiers` gate.

**Agent directory layout**

```
agents/python/
  python-maestro-agent/
  python-application-security-agent/
  python-async-concurrency-reliability-agent/
  python-packaging-supply-chain-agent/
  python-numerical-scientific-correctness-agent/
  python-language-contracts-typing-agent/
  python-web-service-production-readiness-agent/
  python-data-access-transaction-agent/
  python-distributed-task-reliability-agent/
  python-testing-quality-engineering-agent/
  python-estate-modernization-governor-agent/
  python-performance-memory-agent/
  python-free-threading-parallelism-agent/
  python-native-extension-interop-agent/
  python-container-serverless-runtime-agent/
  python-data-pipeline-reliability-agent/
  python-ml-ai-production-agent/
  python-observability-sre-agent/
  python-developer-tooling-build-agent/
  python-business-critical-automation-governance-agent/
```

**Example agents**

| Agent | Scope |
|-------|-------|
| `python-maestro-agent` | Router; classifies a Python task and dispatches the narrowest specialist or a parallel team of up to four. Routes only — never reviews Python work itself, gates production-mutation intent to a human owner, and ejects cloud-, Kubernetes-, observability-, and signing-owned work to the right board. |
| `python-application-security-agent` | Unsafe deserialization (`pickle`, `yaml.load`), dynamic execution (`eval`/`exec`), subprocess/shell injection, SSRF, path traversal, secrets exposure, and cryptography misuse — each finding traced from untrusted input to sink with a CWE label. |
| `python-async-concurrency-reliability-agent` | asyncio event-loop reliability: blocking calls that stall the loop, cancellation correctness, missing timeouts on external awaits, `TaskGroup` supervision, and backpressure on unbounded fan-out. |
| `python-packaging-supply-chain-agent` | pyproject/lockfile integrity, all-or-nothing hash-checking, index trust and dependency confusion, build isolation, and CI release-token exposure. |
| `python-numerical-scientific-correctness-agent` | Money-as-`float` vs `Decimal`, rounding mode, silent dtype coercion, timezone-naive timestamps, and unseeded/irreproducible results. |
| `python-data-access-transaction-agent` | SQLAlchemy/Django ORM session and transaction scope, commit/rollback boundaries, N+1 and lazy loading, connection-pool sizing, and expand-then-contract migration safety. |
| `python-distributed-task-reliability-agent` | Celery/RQ/Dramatiq idempotency under at-least-once delivery, `acks_late` timing, bounded retry backoff, dead-lettering poison messages, and the transactional-outbox boundary. |
| `python-free-threading-parallelism-agent` | Free-threaded (no-GIL, PEP 703) adoption: invalidated GIL thread-safety assumptions, shared-state races, C-extension `Py_mod_gil` support (an undeclaring extension re-enables the GIL), with an evidence-based adopt/pilot/defer verdict. |
| `python-container-serverless-runtime-agent` | PID 1 and SIGTERM handling, exec-form entrypoint, worker model, graceful shutdown, read-only-filesystem and cold-start assumptions in containerized/serverless Python. |
| `python-ml-ai-production-agent` | Training-serving skew, feature/data leakage, unsafe pickle/joblib model-artifact loading (RCE), reproducibility, and batch-vs-online consistency. |
| `python-business-critical-automation-governance-agent` | Unowned scripts/notebooks/schedulers with financial or operational exposure: ownership, segregation of duties, reconciliation, evidence retention, and a continue/harden/replatform/retire verdict (no accounting/legal conclusions). |

Each agent reads source, sanitized configuration, and dependency manifests only. No
agent runs `pip install`, executes or imports the code, opens a database or network
connection, deploys, publishes, or edits project files.

---

### TypeScript

The `typescript` board reviews the TypeScript **program** and the **published package** —
not the frontend application. Its unit of analysis is compiler configuration, module
resolution and emit, the declaration surface, program-graph cost, Node execution, and the
runtime boundaries that type erasure leaves undefended. TypeScript is not synonymous with
React: framework and application-diff work stays with the `frontend` board, and the split is
written into both boards' refusal lists rather than left to convention.

| Property | Value |
|----------|-------|
| `provider` | `typescript` |
| ID prefix | `typescript-*` |
| Agent directory | `agents/typescript/` |
| Skill directory | `skills/typescript/` |
| Agents | <!-- count:board:typescript:agents -->14<!-- /count --> |
| Skills | <!-- count:board:typescript:skills -->14<!-- /count --> (1:1 companion skill per agent) |
| Install roles | `typescript-application-review-engineer`, `typescript-library-maintainer`, `typescript-platform-build-engineer`, `typescript-agentic-contract-engineer` |
| Execution tier | `static-review` (all agents) |

**Agent directory layout**

```
agents/typescript/
  typescript-maestro-agent/
  typescript-type-soundness-agent/
  typescript-runtime-boundary-contract-agent/
  typescript-module-resolution-and-emit-agent/
  typescript-node-execution-compatibility-agent/
  typescript-public-api-and-declaration-governance-agent/
  typescript-build-graph-performance-agent/
  typescript-static-enforcement-policy-agent/
  typescript-async-contract-reliability-agent/
  typescript-package-publication-integrity-agent/
  typescript-estate-modernization-governor-agent/
  typescript-mcp-tool-contract-agent/
  typescript-business-critical-automation-governance-agent/
  typescript-engineering-economics-agent/
```

**Example agents**

| Agent | Scope |
|-------|-------|
| `typescript-maestro-agent` | Router; classifies a TypeScript task and dispatches the narrowest specialist or a parallel team of up to four. Never answers TypeScript questions itself. |
| `typescript-runtime-boundary-contract-agent` | Every point where external data enters the program — HTTP, queues, environment and configuration, database reads, third-party SDKs, webhooks — checking that the value is parsed rather than asserted, and that a generated type is never trusted as a runtime check. |
| `typescript-node-execution-compatibility-agent` | Whether the code runs on the target Node **and is type-checked somewhere**: type-stripping limits, unsupported syntax, and proof of a separate `tsc --noEmit` gate. Exists to kill the "Node runs TypeScript, so the compiler is optional" misconception. |
| `typescript-public-api-and-declaration-governance-agent` | `.d.ts` correctness, what is public versus accidentally exported, breaking-change classification against semver, and the consumer-compilation and type-level test matrix that proves it. |
| `typescript-build-graph-performance-agent` | The TypeScript program graph — project references, `composite`, `.tsbuildinfo`, type-instantiation cost. Refuses to prescribe a restructuring without a supplied measurement. |
| `typescript-mcp-tool-contract-agent` | Whether a declared MCP tool contract matches its TypeScript handler: `inputSchema`/`outputSchema` fidelity, JSON Schema dialect, `structuredContent`, protocol-version negotiation, and error contracts. |

Every agent reads source and sanitized configuration only. No agent compiles, builds, runs
tests, publishes, deploys, or contacts a live system — including the automation-governance
agent, which reviews privileged scripts precisely so that it can withhold approval, and the
economics agent, which consumes measurements other specialists produce and refuses to
originate one. The design record, including the candidates that were rejected and why, is in
[`.claude/workflow/typescript-board/`](https://github.com/VincentChuWaiChow/vanguard-frontier-agentic/tree/master/.claude/workflow/typescript-board).

---

### Atlassian

The `atlassian` board reviews Jira, Jira Service Management and Confluence work through the
Atlassian Rovo MCP Server. It is deliberately small: three read-only specialists answer one
evidence question each (is a corrective action evidenced as preventive, is a control's evidence
chain complete, is a page stale or a preserved historical record), the owner's
`jira-ticket-authoring` and `confluence-document-authoring` skills do all authoring, and a
gate-only live write guard decides go or no-go for a prepared write without declaring write tools.
Approval decisions, permission and restriction changes, space AI instructions and deletes are
prohibited for every agent. Handoffs are `atlassian-case-capsule` records checked by
`npm run validate:atlassian-capsule`. Design record: `docs/adr/0005-atlassian-agent-board.md`.

| Property | Value |
|----------|-------|
| `provider` | `atlassian` |
| ID prefix | `atlassian-*` (plus the two imported authoring skills) |
| Agent directory | `agents/atlassian/` |
| Skill directory | `skills/atlassian/` (protocol and capsule in `skills/cross-functional/`) |
| Agents | <!-- count:board:atlassian:agents -->5<!-- /count --> (<!-- count:board:atlassian:router -->1<!-- /count --> router + <!-- count:board:atlassian:readonly -->4<!-- /count --> read-only-runtime, one of them the gate-only write guard) |
| Skills | <!-- count:board:atlassian:skills -->7<!-- /count --> provider skills, plus the routing protocol and case capsule under `skills/cross-functional/` |
| Install roles | `atlassian-work-intelligence` |
| Execution tier | `static-review` (router) / `read-only-runtime` (specialists and the write guard); no agent declares `mutating-runtime` |

### Snowflake

The `snowflake` board is the one topical board that is **not** static-review throughout. It
pairs a router and 18 review specialists with six approval-gated live guards, because the
Snowflake decisions that matter most — a grant, a network policy, a masking policy, a
warehouse limit, a pipeline replay, a failover promotion — are mutations whose blast radius
is only knowable before they run. The board's organising idea is that documentation proves
supported *platform* behaviour and never proves configured *account* behaviour, so an
account-specific claim is `UNKNOWN` until account evidence establishes it.

| Property | Value |
|----------|-------|
| `provider` | `snowflake` |
| ID prefix | `snowflake-*` |
| Agent directory | `agents/snowflake/` |
| Skill directory | `skills/snowflake/` |
| Agents | <!-- count:board:snowflake:agents -->28<!-- /count --> (<!-- count:board:snowflake:router -->1<!-- /count --> router + <!-- count:board:snowflake:review -->18<!-- /count --> review specialists + <!-- count:board:snowflake:guards -->6<!-- /count --> live guards + <!-- count:board:snowflake:deprecated -->3<!-- /count --> deprecated Azure-scoped predecessors) |
| Skills | <!-- count:board:snowflake:skills -->28<!-- /count --> (1:1 companion skill per agent) |
| Install roles | `snowflake-platform-architect`, `snowflake-security-governance-engineer`, `snowflake-finops-performance-engineer`, `snowflake-data-engineer`, `snowflake-ai-analytics-engineer`, `snowflake-delivery-resilience-engineer`, `snowflake-data-product-manager` |
| Execution tier | `static-review` (router + specialists) / `mutating-runtime` (the <!-- count:board:snowflake:guards -->6<!-- /count --> `snowflake-live-*` guards) |

**Example agents**

| Agent | Scope |
|-------|-------|
| `snowflake-maestro-agent` | Router; classifies the task, names the failure domains, decides whether account evidence is required at all, and dispatches the narrowest specialist or at most four in parallel. Never answers a Snowflake question and never dispatches a live guard |
| `snowflake-identity-access-security-agent` | Effective access computed transitively across role grants, ownership edges, database roles, and future grants — expressed as paths, not counts. Owns authentication policy, MFA, federation, and the `SERVICE` / `SERVICE_AGENT` identity types |
| `snowflake-governance-privacy-agent` | Designs and reviews the data controls, and refuses the four equations that make governance theatre: tagged is not protected, classified is not compliant, lineage existing is not lineage complete, a policy existing is not a policy behaving |
| `snowflake-compliance-evidence-auditor-agent` | Independent assurance, deliberately holding no implementation mandate: whether a control is *provable* over an audit period, bounded by each evidence view's latency and retention |
| `snowflake-cortex-ai-agent-security-governor-agent` | The AI trust boundary — agent identity and effective data reach, tools and MCP connectors as privilege grants, retrieval corpora as an indirect-injection surface. Never reviews an AI system from its prompt alone |
| `snowflake-bcdr-resilience-agent` | Tracks RPO and RTO in three separate columns — requested, feasible, and **proven** — and owns the dependency matrix outside Snowflake that decides whether a promotion is a recovery or a relocation |
| `snowflake-business-value-adoption-strategist-agent` | The economic counterweight. Holds veto authority and may return `NO-GO: technically valid, economically unjustified` |

**Live guards**

Each guard owns exactly one mutation, and no two guards own the same one — ambiguous write
ownership is the failure a live-guard board exists to prevent.

| Live guard | Allowed mutation | Maximum scope |
|-------|-------|-------|
| `snowflake-live-rbac-grant-guard-agent` | one GRANT or REVOKE | one privilege · one securable · one custom role |
| `snowflake-live-auth-network-policy-guard-agent` | one network- or authentication-policy change | one policy object · one modification · one activation scope |
| `snowflake-live-warehouse-cost-change-guard-agent` | one warehouse or cost-governance setting | one warehouse, monitor, or budget · one setting |
| `snowflake-live-data-protection-policy-guard-agent` | one policy attach, detach, or replace | one object · one column · one policy |
| `snowflake-live-pipeline-streaming-change-guard-agent` | one task, stream, dynamic-table, or pipe operation, or one bounded backfill | one object · one operation · one bounded data window |
| `snowflake-live-failover-promotion-guard-agent` | one failover group promotion | one group · one target account |

`ACCOUNTADMIN` is forbidden for every guard without exception; each runs as a custom role
scoped to the single target named in the approval. Two guards carry a refusal that no
approval can override: the auth/network guard will not tighten reachability until a
*surviving administrative path* is demonstrated from login history, and the failover guard
will not promote without a declared incident or drill, a data-loss window computed from
replication refresh history, and dependency readiness confirmed by each owning team.

Routing, the live-guard gate, negative routing, and cross-agent conflict are covered by 27
scenarios in `tests/fixtures/snowflake-maestro-routing/`. Three Azure-scoped predecessors
(`*-at-azure`) are marked `lifecycle: deprecated` and excluded from routing so that no
mutation surface has two owners; they remain installable via the legacy
`azure-snowflake-platform-engineer` role.

---

### Databricks

The `databricks` board is static-review throughout. It pairs a router with review
specialists covering the lakehouse surface — platform architecture, Unity Catalog
governance, identity and network security, data protection, Lakeflow pipelines, streaming,
data quality, SQL performance, AI/BI Genie, MLOps, GenAI engineering and evaluation,
developer platform, reliability, cost, and value realization. No agent on it carries an
execution tool: mutation is reachable only through the pre-existing Unity Catalog grant
guard, which is never auto-dispatched.

Unlike `snowflake`, this board's Azure-scoped agents are **not** deprecated. The three
`*-at-azure` assets are actively maintained and coexist with the cloud-neutral board,
owning the deployment detail it deliberately does not duplicate — Entra ID federation,
ADLS Gen2 wiring via Access Connector managed identity, and VNet-level isolation. Because
they are live rather than retired, they are counted in every figure below; the board is
described by what the catalog can derive, not by a hand-typed cloud-neutral subtotal.

| Property | Value |
|----------|-------|
| `provider` | `databricks` |
| ID prefix | `databricks-*` |
| Agent directory | `agents/databricks/` |
| Skill directory | `skills/databricks/` |
| Agents | <!-- count:board:databricks:agents -->20<!-- /count --> (<!-- count:board:databricks:router -->1<!-- /count --> router + <!-- count:board:databricks:review -->18<!-- /count --> review specialists + <!-- count:board:databricks:guards -->1<!-- /count --> live guard) |
| Skills | <!-- count:board:databricks:skills -->20<!-- /count --> (1:1 companion skill per agent) |
| Install roles | `databricks-platform-engineer`, `databricks-solution-architect`, `databricks-data-engineer`, `databricks-analytics-engineer`, `databricks-governance-security-engineer`, `databricks-ml-engineer`, `databricks-genai-engineer`, `databricks-sre`, `databricks-finops-engineer` |
| Execution tier | `static-review` (<!-- count:board:databricks:static -->19<!-- /count --> agents — router + specialists) / `mutating-runtime` (the <!-- count:board:databricks:guards -->1<!-- /count --> Unity Catalog grant guard) |

**Example agents**

| Agent | Scope |
|-------|-------|
| `databricks-maestro-agent` | Router; classifies on seven axes (intent, business context, artifact type, blast radius, required evidence, implied runtime authority, specialist ownership) and dispatches a single specialist or at most four in parallel. Never answers a Databricks question and never dispatches the live guard |
| `databricks-unity-catalog-governance-agent` | Three-level namespace, GRANT inheritance and privilege cascades, single-owner-per-securable design, workspace-catalog binding in ISOLATED mode, governed tags, storage credentials, audit completeness |
| `databricks-data-protection-privacy-agent` | Row filters and column masks with their UDF cost implications, ABAC policies, classification, DELETE/MERGE/VACUUM/REORG coordination for GDPR erasure, Delta Sharing egress, residency, customer-managed keys |
| `databricks-lakeflow-pipeline-engineering-agent` | Lakeflow Spark Declarative Pipelines, medallion layering, liquid clustering, Auto Loader ingestion |
| `databricks-genai-evaluation-observability-agent` | MLflow Tracing, built-in judges, and regression detection — separated from agent authoring so evaluation is not graded by the same agent that built the thing |
| `databricks-value-realization-agent` | The economic counterweight. Refuses to state a number without a measured baseline, and names attribution limits and kill conditions rather than asserting ROI |

**Live guard**

| Live guard | Allowed mutation | Maximum scope |
|-------|-------|-------|
| `databricks-live-unity-catalog-grant-guard-at-azure-agent` | one GRANT or REVOKE | one securable · one principal · one privilege |

The guard requires a written approval token and a dry-run preflight, captures prior state,
and names its `REVOKE`/`GRANT` rollback before executing. It is reachable only in
`live-guard-gate` mode; the routing grader asserts it is never auto-dispatched.

Routing, the live-guard gate, and negative routing are covered by the scenarios in
`tests/fixtures/databricks-maestro-routing/`. The board is documented in full at
[The Databricks Board](databricks-board/).

---

## How to use language/stack boards

### Discovery via install roles

The primary entry point for users is the named install role in
`catalog/install-roles.json`. Each role bundles the minimal agent and skill
set for a given function.

| Role | Boards covered | Agents | Skills |
|------|---------------|--------|--------|
| `dotnet-application-review-engineer` | `.NET` | 10 | 10 |
| `legal-hr-risk-reviewer` | `legal` + `hr` | <!-- count:board:legal+hr:agents -->28<!-- /count --> | 5 (2 board-specific + 3 cross-functional) |
| `marketing-governance-reviewer` | `marketing` | 14 | 14 |
| `php-platform-engineer` | `php` | 5 | 5 |
| `python-application-review-engineer` | `python` | 20 | 20 |
| `python-application-engineer` | `python` | 6 | 6 |
| `python-platform-reliability-engineer` | `python` | 7 | 7 |
| `python-data-engineer` | `python` | 5 | 5 |
| `python-ml-engineer` | `python` | 5 | 5 |
| `python-security-engineer` | `python` | 3 | 3 |
| `python-library-maintainer` | `python` | 6 | 6 |
| `python-automation-governance-lead` | `python` | 5 | 5 |
| `python-engineering-leader` | `python` | 5 | 5 |
| `python-reliability-data-engineer` | `python` | 5 | 5 |
| `python-live-platform-operator` | `python` (live) | 6 | 6 |
| `python-live-security-operator` | `python` (live) | 6 | 6 |
| `python-live-data-operator` | `python` (live) | 4 | 4 |
| `python-live-ml-governance-operator` | `python` (live) | 5 | 5 |
| `python-live-automation-control-owner` | `python` (live) | 5 | 5 |
| `python-live-audit-and-compliance-reviewer` | `python` (live) | 5 | 5 |
| `sap-transformation-operations` | `sap` | 40 | 46 |
| `microsoft-365-d365-platform-advisor` | `microsoft` | 40 | 40 |
| `azure-databricks-platform-engineer` | `databricks` | 3 | 3 |
| `databricks-platform-engineer` | `databricks` | 6 | 6 |
| `databricks-solution-architect` | `databricks` | 6 | 6 |
| `databricks-data-engineer` | `databricks` | 5 | 5 |
| `databricks-analytics-engineer` | `databricks` | 5 | 5 |
| `databricks-governance-security-engineer` | `databricks` | 5 | 5 |
| `databricks-ml-engineer` | `databricks` | 5 | 5 |
| `databricks-genai-engineer` | `databricks` | 5 | 5 |
| `databricks-sre` | `databricks` | 5 | 5 |
| `databricks-finops-engineer` | `databricks` | 5 | 5 |
| `snowflake-platform-architect` | `snowflake` | 5 | 5 |
| `snowflake-security-governance-engineer` | `snowflake` | 8 | 8 |
| `snowflake-finops-performance-engineer` | `snowflake` | 5 | 5 |
| `snowflake-data-engineer` | `snowflake` | 5 | 5 |
| `snowflake-ai-analytics-engineer` | `snowflake` | 5 | 5 |
| `snowflake-delivery-resilience-engineer` | `snowflake` | 5 | 5 |
| `snowflake-data-product-manager` | `snowflake` | 5 | 5 |
| `azure-snowflake-platform-engineer` (deprecated) | `snowflake` | 3 | 3 |

Install a role with the export CLI:

```bash
npx vfa-export-agents --platform claude-code --role dotnet-application-review-engineer --repo .
npx vfa-export-agents --platform claude-code --role legal-hr-risk-reviewer --repo .
npx vfa-export-agents --platform claude-code --role marketing-governance-reviewer --repo .
```

### Routing

Each board includes a maestro router agent (`dotnet-maestro-agent`,
`legal-maestro-agent`, `hr-maestro-agent`, `marketing-maestro-agent`,
`php-maestro-agent`, `python-maestro-agent`). Address
the maestro with the task; it classifies the work and dispatches the narrowest
specialist or a small parallel team. Do not reach past the maestro and invoke a
specialist directly unless you already know which specialist applies.

The `python` board carries a **second** maestro: `python-live-governance-maestro-agent`
routes the live control-plane agents (`python-live-*`), while `python-maestro-agent`
routes the 20 static-review specialists. The two are kept separate on purpose — a
static-review request must never be routed into the live plane, and the live maestro
never auto-dispatches a mutating operator (those are live-guard gated). Kiro Powers and
other harness entry points therefore route the `python` board through
`python-maestro-agent` as the default classifier.

### Invocation

Agents install as harness-native adapter files. On Claude Code:

```
@dotnet-maestro-agent Review the EF Core migrations in /src/Data/
@legal-counsel-review-agent Here is the indemnity clause from our SaaS agreement: …
@hr-risk-triage-review-agent The manager wants to terminate an employee who returned from FMLA last week.
@marketing-pixel-data-leakage-review-agent Here is our Meta Pixel configuration and the pages it fires on.
```

On other harnesses (Copilot, Cursor, Codex, Gemini) the same agent IDs are
available via the harness's agent-selection UI or `@mention` syntax. See
[compatibility.md](compatibility.md) for per-harness adapter details.

### Individual agent install

If you need only one agent from a board rather than the full role:

```bash
npx vfa-export-agents --platform claude-code \
  --agents dotnet-efcore-data-access-review-agent --repo .
```

---

## Adding a new language/stack board

Follow this sequence to introduce a new board. Use `agents/dotnet/` as the
reference implementation — it is the most complete example with the taxonomy
note, tier table, and 1:1 agent-to-skill pairing.

### 1. Create the directory pair

```
agents/<board-name>/
  README.md
skills/<board-name>/
```

The `README.md` should state the board's subject scope, tier table, and
taxonomy note. Copy the note from `agents/dotnet/README.md` and adapt it to
clarify the board's `provider` value: a shipped board uses its own dedicated
`provider` value (added to the schema and validator enums); a board still
under construction uses `provider: generic` until it is promoted.

### 2. Define agents

For each agent, create:

```
agents/<board-name>/<board>-<role>-agent/
  AGENT.md
  metadata.json
  harnesses/
    claude-code.agent.md
    codex.toml
    copilot.agent.md
    cursor.agent.md
    gemini.agent.md
    kiro-ide.agent.md
    kiro-cli.agent.json
```

Key `metadata.json` fields for a language/stack board agent:

| Field | Value |
|-------|-------|
| `id` | `<board>-<role>-agent` |
| `type` | `"agent"` |
| `provider` | `"<board-name>"` (dedicated value) or `"generic"` until promoted |
| `execution_tier` | `"static-review"` |
| `companion_skills` | Array of companion skill IDs, or `[]` |
| `lifecycle` | `"experimental"` until the board stabilizes |

### 3. Define companion skills

For each agent, create a matching skill:

```
skills/<board-name>/<board>-<role>/
  SKILL.md          # YAML frontmatter + skill body
  metadata.json
```

The `SKILL.md` frontmatter must declare `allowed-tools` at the least-privilege
baseline. Language/stack board skills are read-only; a minimal baseline is
`Read Grep Glob`. See `schemas/skill.frontmatter.schema.json` for the full
contract and `skills/dotnet/dotnet-maestro/SKILL.md` for a worked example.

### 4. Add a maestro router

Include a `<board>-maestro-agent` and a paired `<board>-maestro` skill. The
maestro classifies tasks and dispatches specialists; it never answers domain
questions itself.

### 5. Add a board-level README

Document the board's scope, the tier table (router vs. review agents, default
access, live-execution policy), and a brief operating note for each agent. See
`agents/dotnet/README.md` and `agents/legal/README.md` as templates.

### 6. Register the provider value

A shipped board gets its own dedicated `provider` value. Add the board name to
every provider enum so the schema and catalog validators accept it:

- `schemas/agent.schema.json` — `provider.enum`
- `schemas/skill.schema.json` — `provider.enum`
- `tests/validate-catalog.py` — `ALLOWED_PROVIDERS`
- `docs/taxonomy.md` — the **Providers** list

Then set `provider: "<board-name>"` in every agent and skill `metadata.json`
for the board. A board still under construction may skip this step and use
`provider: generic` until it is promoted.

### 7. Update the catalog

Add each new agent to `catalog/agents.json` and each new skill to
`catalog/skills.json`, using the board's `provider` value. After adding
skills, regenerate the manifest:

```bash
npm run manifest:write
```

### 8. Wire an install role

Add an entry to `catalog/install-roles.json`:

```json
"<board>-<function>-reviewer": {
  "label": "<Human-readable label>",
  "description": "<One- or two-sentence description of scope and static-review posture>",
  "agents": ["<board>-maestro-agent", "<board>-specialist-a-agent", "…"],
  "skills": ["<board>-maestro", "<board>-specialist-a", "…"]
}
```

If the new board shares natural install-role scope with an existing board
(as `legal` and `hr` do), add the new agents and skills to the existing role
entry rather than creating a second one.

### 9. Run validation

```bash
npm run validate
```

All seven gates must pass before opening a pull request. The key gates for a
new board are `validate:catalog` (all entries reference real paths and satisfy
schemas), `validate:skill-schema`, `validate:agent-schema`,
`validate:allowed-tools`, and `validate:links`. If skills changed, confirm
`manifest:check` also passes.

---

## Trust posture

All language/stack board agents are scoped to `static-review` or an equivalent
read-only tier. This is a design constraint, not a default.

| Board | Tier | Constraint |
|-------|------|------------|
| `.NET` | `static-review` | Reads source and sanitized configuration; never runs `dotnet build`, `dotnet test`, `dotnet ef`, or contacts a live system |
| `legal` | `static-review` | Reads sanitized excerpts; never contacts regulators, triggers legal systems, or makes binding legal determinations |
| `hr` | `static-review` | Reads sanitized excerpts; never terminates, disciplines, denies leave or accommodation, or sends employee communications |
| `marketing` | `static-review` (specialists) / `read-only-runtime` (maestro) | Reads sanitized configuration and evidence; never mutates CMP, tag-manager, or ad-platform state |
| `php` | `static-review` | Reads sanitized PHP source, configuration, and dependency files; never executes payloads, installs packages, or mutates runtime or production. |
| `python` (static-review board) | `static-review` | Reads Python source, sanitized configuration, and dependency manifests; never runs `pip install`, executes or imports code, opens a database or network connection, or deploys/publishes |
| `python` (live control plane) | `read-only-runtime` / `mutating-runtime` | The `python-live-*` agents. Read-only agents perform allowlisted diagnostics and observation; mutating operators are live-guard gated — never auto-dispatched, requiring an independent approval bound to the target, target-scoped JIT credentials, a pre-approved rollback, and an immutable audit event (fail-closed for R3+). The repo ships definitions, contracts, and evals — not a running control plane — and no agent declares compliance. |
| `sap` | `static-review` | Reads sanitized SAP configuration and ABAP/BTP artifacts; never contacts SAP systems, triggers transports, or mutates landscape data |
| `microsoft` | `static-review` | Reads sanitized Microsoft 365 and Dynamics 365 configuration; never mutates tenant state, sends messages, or contacts Graph API |
| `databricks` (cloud-neutral board) | `static-review` | Reads sanitized notebooks, SQL, `databricks.yml`, job/pipeline and cluster-policy JSON, query profiles, and system-table output; never executes DDL/DML, `GRANT`/`REVOKE`, job or pipeline runs, warehouse or cluster changes, or model deployments, and never contacts Databricks REST APIs |
| `databricks` (Azure live guard) | `mutating-runtime` | The single `databricks-live-unity-catalog-grant-guard-at-azure-agent`. Executes exactly one `GRANT`/`REVOKE` on one securable for one principal, gated by a written approval token, dry-run preflight, prior-state capture, and a named `REVOKE`/`GRANT` rollback; never auto-dispatched |
| `snowflake` (review board) | `static-review` | The maestro and 18 review specialists. Read sanitized DDL, query plans, grants, policy definitions, and account-evidence extracts; never execute a statement, resize compute, attach a policy, or promote a replication target |
| `snowflake` (live guards) | `mutating-runtime` | The six `snowflake-live-*` guards. Each executes exactly one approved mutation behind an explicit written human gate with prior-state capture, preflight, verification, and a rollback path — never auto-dispatched. No harness adapter grants them an execution tool: the deliverable is the approved statement, which a named human operator runs |

Static review is the required **default** for language/stack boards. Every
exception is governed, never ad hoc: a board may ship live agents only behind the
live-guard contract, and the boards that do — `sap`, `microsoft`, and the Azure
`databricks` grant guard — each expose a small, named set of single-operation
guards rather than general mutation authority. The largest and most structured of
these is the `python` **live control plane** (`python-live-*`), which
carries `read-only-runtime`/`mutating-runtime` tiers under the controlled-execution
and audit-evidence contracts in [docs/compliance/](compliance/) and
[evidence-output-spec.md](evidence-output-spec.md); its mutating operators are
live-guard gated and never auto-dispatched. A new board that needs to build, run,
mutate, or contact a live system must either follow that governed live-plane model
explicitly or belong on a provider board with an appropriate `execution_tier` and
per-session opt-in controls — it is never an ungoverned addition to a static-review
board.

No **static-review** language/stack board agent:

- Runs, compiles, or deploys code
- Contacts an external API, database, or live service
- Makes a binding legal or HR determination
- Stores or echoes secrets, credentials, tokens, or personal data

The `python-live-*` control-plane agents operate under the separate governed
live-execution contracts above (approval, JIT credentials, rollback, audit event,
fail-closed for R3+), not this static-review list.

---

## Cross-references

- [taxonomy.md](taxonomy.md) — provider enum, asset types, skill categories,
  trust levels, and lifecycle values; includes the canonical taxonomy note for
  language/stack boards
- [marketplace-model.md](marketplace-model.md) — how the catalog, schemas, and
  validation gates fit together
- [compatibility.md](compatibility.md) — harness support contract and adapter
  format requirements
- `CONTRIBUTING.md` — step-by-step guide for adding agents and skills, including
  required metadata fields and catalog refresh commands
- `agents/dotnet/README.md` — reference implementation for the board README
  format and tier table
- `docs/architecture/legal-hr-agent-routing.md` — cross-domain routing between
  the `legal` and `hr` boards
