#!/usr/bin/env node
/**
 * Generate Kiro Powers from catalog/agents.json.
 *
 * Each Power is a directory under powers/ containing a single POWER.md.
 * Kiro Powers spec (kirodotdev/powers) restricts frontmatter to exactly
 * five fields: name, displayName, description, keywords, author. No
 * version, no repository, no license, no tags. The validator enforces
 * the strict-5 rule.
 *
 * Body content is templated from per-provider config (steering content
 * authored once, here) plus catalog facts (maestro id, live-guard list,
 * agent count) read at generate time. This keeps the steering tight
 * and the agent inventory accurate.
 *
 * Mode:
 *   --check  exit 1 if any on-disk Power does not match the generated one
 *   (default) write/overwrite all Powers
 */

import { readFileSync, writeFileSync, existsSync, mkdirSync } from "node:fs";
import { dirname, join, relative } from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = join(dirname(fileURLToPath(import.meta.url)), "..");
const catalogPath = join(repoRoot, "catalog", "agents.json");
const powersRoot = join(repoRoot, "powers");

const check = process.argv.includes("--check");

// Per-provider steering content. Description: max 3 sentences (Kiro
// constraint). Keywords: specific terms only — Kiro docs warn that broad
// keywords trigger false activations. Invariants: 2-4 bullets that the
// AI must keep in mind when handling this provider's work.
const PROVIDERS = {
  aws: {
    displayName: "Vanguard Frontier — AWS",
    description:
      "Curated AWS agents for IAM, EKS, Lambda, RDS, S3, and Bedrock with live-mutation guards. Routes via aws-maestro to specialist or live-guard agents based on task scope. Mutations on real AWS environments require account-ID, region, and approval confirmation before execution.",
    keywords: ["aws", "iam", "eks", "lambda", "rds", "s3", "bedrock", "live-guard"],
    invariants: [
      "Confirm AWS account ID and region before any live mutation.",
      "Live-guard agents (aws-live-*) must never be auto-dispatched; require explicit approval and rollback plan.",
      "IAM least-privilege review applies to every policy attachment, role assumption, and trust relationship.",
      "Cross-account access via assume-role must be reviewed by aws-iam-review-agent before activation.",
    ],
  },
  azure: {
    displayName: "Vanguard Frontier — Azure",
    description:
      "Curated Azure agents for Entra ID, AKS, App Service, Key Vault, Cosmos DB, and ARM/Bicep with live-mutation guards. Routes via azure-maestro to specialist or live-guard agents. Mutations on real Azure environments require subscription ID, tenant ID, resource group, and approval confirmation.",
    keywords: ["azure", "entra-id", "aks", "app-service", "key-vault", "cosmos-db", "bicep", "live-guard"],
    invariants: [
      "Confirm Azure subscription ID, tenant ID, and resource group before any live mutation.",
      "Live-guard agents (azure-live-*) must never be auto-dispatched; require approval and rollback plan.",
      "PIM (Privileged Identity Management) elevation is a separate decision from RBAC role assignment.",
      "Management group SCP-equivalent policies cascade — review blast radius before org-level changes.",
    ],
  },
  gcp: {
    displayName: "Vanguard Frontier — GCP",
    description:
      "Curated Google Cloud agents for IAM, GKE, Cloud Run, BigQuery, Vertex AI, and AlloyDB with live-mutation guards. Routes via gcp-maestro to specialist or live-guard agents. Mutations require project ID, region, and approval confirmation; org-level changes need additional review.",
    keywords: ["gcp", "iam", "gke", "cloud-run", "bigquery", "vertex-ai", "alloydb", "live-guard"],
    invariants: [
      "Confirm GCP project ID and region/zone before any live mutation.",
      "Live-guard agents (gcp-live-*) must never be auto-dispatched; require approval and rollback plan.",
      "IAM Conditions and workload identity federation are reviewed by gcp-iam-review-agent before activation.",
      "Org policy constraints take precedence over project-level IAM grants.",
    ],
  },
  oci: {
    displayName: "Vanguard Frontier — OCI",
    description:
      "Curated Oracle Cloud agents for IAM, OKE, Autonomous Database, Vault, and Resource Manager with live-mutation guards. Routes via oci-maestro to specialist or live-guard agents. Distinguishes commercial vs gov-cloud realm; mutations require tenancy, compartment, and region confirmation.",
    keywords: ["oci", "oracle-cloud", "iam", "oke", "autonomous-database", "vault", "resource-manager", "live-guard"],
    invariants: [
      "Confirm OCI tenancy OCID, compartment, and region before any live mutation.",
      "Live-guard agents (oci-live-*) must never be auto-dispatched; require approval and rollback plan.",
      "Commercial and government cloud realms have separate identity domains — verify realm before action.",
      "Compartment hierarchy enforces policy scope; review parent-compartment grants before sub-compartment changes.",
    ],
  },
  alibaba: {
    displayName: "Vanguard Frontier — Alibaba Cloud",
    description:
      "Curated Alibaba Cloud agents for RAM, ACK, PolarDB, OSS, and MaxCompute with live-mutation guards and China-region compliance. Routes via alibaba-maestro to specialist or live-guard agents. China mainland (cn-*) and international regions have separate billing and regulatory scope — always confirm context.",
    keywords: ["alibaba-cloud", "ram", "ack", "polardb", "oss", "maxcompute", "mlps-2", "live-guard"],
    invariants: [
      "Confirm region: China mainland (cn-hangzhou, cn-beijing, etc.) and international regions have separate billing accounts and different regulatory scope.",
      "MLPS 2.0 Level 3 mandates specific service configurations — alibaba-china-compliance-agent flags gaps before live changes.",
      "Live-guard agents (alibaba-live-*) must never be auto-dispatched; require approval and rollback plan.",
      "OSS bucket policies must be reviewed for public exposure and PIPL/DSL data-residency compliance before live changes.",
    ],
  },
  huawei: {
    displayName: "Vanguard Frontier — Huawei Cloud",
    description:
      "Curated Huawei Cloud agents for IAM, CCE, GaussDB, OBS, DEW (KMS+CSMS), and ModelArts with live-mutation guards and MLPS 2.0 compliance. Routes via huawei-maestro to specialist or live-guard agents. Enterprise Projects are billing constructs, not security boundaries — verify IAM and SCP scope independently.",
    keywords: ["huawei-cloud", "iam", "cce", "gaussdb", "obs", "dew", "modelarts", "live-guard"],
    invariants: [
      "Confirm Huawei Cloud account ID, region, and Enterprise Project before any live mutation.",
      "Enterprise Projects are billing/attribution constructs, NOT security boundaries — verify IAM policy and SCP scope independently.",
      "MLPS 2.0 Level 3 (GB/T 22239-2019) requires specific service configurations — huawei-compliance-sovereignty-agent flags gaps.",
      "Live-guard agents (huawei-live-*) must never be auto-dispatched; require approval and rollback plan.",
    ],
  },
  ovhcloud: {
    displayName: "Vanguard Frontier — OVHcloud",
    description:
      "Curated OVHcloud agents for IAM, Managed Kubernetes, networking, and KMS with live-mutation guards. Routes via ovhcloud-maestro to specialist or live-guard agents. EU-headquartered sovereignty cloud; mutations require project ID and region confirmation.",
    keywords: ["ovhcloud", "ovh", "iam", "managed-kubernetes", "kms", "eu-sovereignty", "live-guard", "data-residency"],
    invariants: [
      "Confirm OVHcloud project ID and region before any live mutation.",
      "Live-guard agents (ovhcloud-live-*) must never be auto-dispatched; require approval and rollback plan.",
      "EU sovereignty cloud — review data-residency requirements before cross-region replication.",
    ],
  },
  scaleway: {
    displayName: "Vanguard Frontier — Scaleway",
    description:
      "Curated Scaleway agents for IAM, Kapsule (managed Kubernetes), networking, and cost optimization with live-mutation guards. Routes via scaleway-maestro to specialist or live-guard agents. EU-region only (PAR, AMS, WAW); mutations require organization ID and region confirmation.",
    keywords: ["scaleway", "iam", "kapsule", "managed-kubernetes", "cost-optimizer", "eu-region", "live-guard"],
    invariants: [
      "Confirm Scaleway organization ID and region (PAR, AMS, WAW) before any live mutation.",
      "Live-guard agents (scaleway-live-*) must never be auto-dispatched; require approval and rollback plan.",
      "Kapsule rollout changes require PDB audit and health-signal verification.",
    ],
  },
  hetzner: {
    displayName: "Vanguard Frontier — Hetzner",
    description:
      "Curated Hetzner agents for infrastructure review, cost optimization, capacity planning, and live server-lifecycle and firewall-rule guards. Routes via the Hetzner pattern to specialist agents. EU-headquartered provider; mutations on real Hetzner projects require project ID and region confirmation.",
    keywords: ["hetzner", "infrastructure-review", "cost-optimizer", "capacity-planner", "server-lifecycle", "firewall-rules", "live-guard"],
    invariants: [
      "Confirm Hetzner project ID and location before any live mutation.",
      "Live-guard agents (hetzner-live-*) must never be auto-dispatched; require approval and rollback plan.",
      "Firewall rule changes require capture of current ruleset and explicit egress-blocking review.",
    ],
  },
  contabo: {
    displayName: "Vanguard Frontier — Contabo",
    description:
      "Curated Contabo agents for security hardening, cost optimization, capacity planning, and live instance-lifecycle and storage-operations guards. Routes via the Contabo pattern to specialist agents. Mutations on real Contabo accounts require account context and region confirmation.",
    keywords: ["contabo", "security-hardening", "cost-optimizer", "capacity-planner", "instance-lifecycle", "storage-operations", "live-guard"],
    invariants: [
      "Confirm Contabo account context and region before any live mutation.",
      "Live-guard agents (contabo-live-*) must never be auto-dispatched; require approval and rollback plan.",
      "Storage operations on object storage and block storage require backup verification before destructive actions.",
    ],
  },
  ionos: {
    displayName: "Vanguard Frontier — IONOS",
    description:
      "Curated IONOS agents for security and compliance review, datacenter design, cost optimization, Managed Kubernetes operations, and live database-lifecycle guards. Routes via ionos-maestro to specialist or live-guard agents. Mutations require contract ID and datacenter confirmation.",
    keywords: ["ionos", "security-compliance", "datacenter-designer", "managed-kubernetes", "database-lifecycle", "live-guard", "eu-sovereignty"],
    invariants: [
      "Confirm IONOS contract ID and datacenter before any live mutation.",
      "Live-guard agents (ionos-live-*) must never be auto-dispatched; require approval and rollback plan.",
      "DBaaS lifecycle mutations require backup verification and replication-status review.",
    ],
  },
  typescript: {
    displayName: "Vanguard Frontier — TypeScript",
    description:
      "Curated TypeScript agents for the TypeScript program and the published package — type soundness in shared code, runtime boundary contracts, module resolution and emit, Node execution compatibility, declaration governance, build-graph cost, static enforcement policy, async contract reliability, publication integrity, estate modernization, and MCP tool contracts. Routes via typescript-maestro to specialist agents based on task scope. Static review only; no live mutations.",
    // Deliberately narrow. Kiro selects Powers from these keywords, so a generic
    // term like `configuration-audit` or `best-practices` would activate this
    // Power for a database or Terraform review that carries no TypeScript
    // signal at all. Every term here is either the language itself or a
    // construct that only appears in TypeScript/Node work.
    keywords: ["typescript", "tsconfig", "tsc", "declaration-emit", "module-resolution", "type-safety", "npm-publish", "static-review"],
    invariants: [
      "Static review only — agents read source and sanitized configuration, and never compile, build, test, publish, deploy, sign, or contact a live system.",
      "Never request or accept secrets, npm or registry tokens, signing keys, connection strings, tenant identifiers, or customer data.",
      "This repository contains no TypeScript program of its own, so no verdict may be grounded in an assumed compiler, tsconfig, or Node version — a version-gated conclusion requires evidence the user supplies, and its absence is a refuse-and-ask.",
      "Compile-time is not runtime: types are erased, so a passing build is evidence about the source and never about the payload crossing an I/O boundary.",
      "Frontend application diffs, framework specifics, bundler configuration, dependency intake, and the monorepo task graph belong to the frontend board — hand them off rather than absorbing them.",
    ],
  },
  kubernetes: {
    displayName: "Vanguard Frontier — Kubernetes",
    description:
      "Curated Kubernetes agents for RBAC review, workload identity, Pod Security Admission, admission policies, network policies, ArgoCD GitOps, and live mutation guards across RBAC, admission, mesh, network, and rollout planes. Routes via kubernetes-maestro to specialist or live-guard agents. Cluster context and namespace must be confirmed before any live mutation.",
    keywords: ["kubernetes", "rbac", "workload-identity", "pod-security-admission", "admission-policies", "argocd", "live-guard"],
    invariants: [
      "Confirm cluster context (kubeconfig + namespace) before any live mutation.",
      "Live-guard agents (kubernetes-live-*) must never be auto-dispatched; require approval and rollback plan.",
      "RBAC ClusterRole and ClusterRoleBinding changes affect every namespace — review blast radius first.",
      "Admission policies (Kyverno, ValidatingAdmissionPolicy) apply at cluster scope; review for unintended workload rejection.",
    ],
  },
  terraform: {
    displayName: "Vanguard Frontier — Terraform",
    description:
      "Curated Terraform agents for plan/apply review, state safety, deletion protection, and blast-radius assessment of IaC changes. Routes via terraform-maestro to the terraform-reviewer for plan analysis. Plan review is required before any apply targeting real infrastructure; state-modifying commands require explicit approval.",
    keywords: ["terraform", "iac", "plan-review", "state-safety", "deletion-protection", "blast-radius"],
    invariants: [
      "Plan review (terraform plan output) must precede any apply on real infrastructure.",
      "Resource destruction and replacement (terraform plan: '-/+') require explicit confirmation with backup verification.",
      "State-modifying commands (terraform state rm, mv, push) require explicit approval — they bypass plan review.",
      "Workspace context (workspace, var-file, backend) must be confirmed before running plan or apply.",
    ],
  },
  nvidia: {
    displayName: "Vanguard Frontier — NVIDIA",
    description:
      "Curated NVIDIA agents for GPU resource governance, NIM model deployment, NGC registry hygiene, supply-chain integrity, and runtime evidence gating. Routes via nvidia-maestro to specialist agents and through the runtime-evidence-gate before runtime-affecting mutations. GPU resource changes require capacity, cost, and supply-chain review.",
    keywords: ["nvidia", "ngc", "nim", "gpu-governance", "runtime-evidence", "supply-chain"],
    invariants: [
      "Runtime mutations require evidence via nvidia-runtime-evidence-gate before execution.",
      "GPU resource allocation must be reviewed for capacity, cost, and tenant isolation impact.",
      "NGC container provenance and SBOM must be validated before deployment to runtime hosts.",
      "Driver and CUDA version changes have node-wide blast radius — review compatibility matrix first.",
    ],
  },
  salesforce: {
    displayName: "Vanguard Frontier — Salesforce",
    description:
      "Curated Salesforce agents for admin review, development, security, integration, revenue ops, service ops, marketing ops, Agentforce/AI risk, and compliance — static review only, no org mutations. Routes via salesforce-maestro to specialist agents covering Sales Cloud, Service Cloud, Experience Cloud, Marketing Cloud, MuleSoft, Tableau, and industry verticals. All Salesforce terminology and API surfaces are drift-prone; agents always verify against current official documentation before rendering findings.",
    keywords: ["salesforce", "agentforce", "crm", "apex", "lwc", "mulesoft", "compliance", "static-review"],
    invariants: [
      "Static review only — agents never request org credentials, session tokens, or user PII, and never mutate a Salesforce org.",
      "Salesforce API versions and feature availability vary by org edition and release; verify org context (edition, API version, enabled features) before applying any recommendation.",
      "Agentforce and Einstein AI configurations are adversarially reviewed for prompt-injection risk, ungrounded automation, and missing human-handoff controls before any approve-or-merge decision.",
      "Live-guard agent (salesforce-live-guard-agent) must never be auto-dispatched; require explicit approval, target org confirmation, and rollback plan.",
    ],
  },
  sap: {
    displayName: "Vanguard Frontier — SAP",
    description:
      "Curated SAP agents for S/4HANA, BTP, Integration Suite, ABAP Cloud, and transport management with clean-core review and guarded mutation gates. Routes via sap-maestro to specialist agents for landscape discovery, clean-core debt review, and guarded transport imports. Transport mutations require named approver, change ticket, target-system confirmation, and completed SoD check before dispatch.",
    keywords: ["sap", "s4hana", "btp", "abap", "clean-core", "transport-management", "live-guard"],
    invariants: [
      "Never auto-dispatch the transport-import operator agent — require explicit approval, change ticket, named approver, and completed SoD check before any tp or CTS import command.",
      "Clean-core debt review is static only — never connects to a live SAP system and never accepts embedded credentials or production system IDs.",
      "Landscape discovery agents use read-only roles only (BTP subaccount viewer, CF SpaceAuditor/OrgAuditor, ABAP display user) — refuse any request that maps to a create/update/delete/deploy/assign/rotate action.",
      "All SAP API surfaces and release contracts drift between versions; verify current SAP API Business Hub documentation before applying any remediation recommendation.",
    ],
  },
  microsoft: {
    displayName: "Vanguard Frontier — Microsoft",
    description:
      "Curated Microsoft 365 and Dynamics 365 agents for tenant governance, Entra identity and Conditional Access, Intune endpoints, Purview data security and compliance, Defender XDR, Teams/SharePoint/Exchange collaboration, Microsoft 365 Copilot readiness, Power Platform governance, and Dynamics 365 ERP/CRM (Finance, Supply Chain, Business Central, Sales, Customer Service, Field Service) — static review only, no tenant or production mutations. Routes via microsoft-maestro to M365, D365, Power Platform, and Copilot specialist agents. Microsoft licensing, certification, and API surfaces are drift-prone; agents always verify against current Microsoft Learn documentation before rendering findings.",
    keywords: ["microsoft", "m365", "d365", "entra", "purview", "copilot", "power-platform", "static-review"],
    invariants: [
      "Static review only — agents never request tenant credentials, tokens, customer data, or PII, and never mutate a Microsoft 365 tenant or Dynamics 365 environment.",
      "Apply Zero Trust by default: verify explicitly, least privilege (JIT/JEA), assume breach; confirm tenant, environment, and data classification before any recommendation.",
      "Microsoft 365 Copilot and Copilot Studio configurations are adversarially reviewed for oversharing, ungrounded Graph exposure, and missing human-handoff controls before any approve decision.",
      "Production-impacting actions (Conditional Access changes, D365 cutover, Power Platform prod deploy, MFA changes) are live-guard gated — never auto-dispatched; require explicit approval, scope confirmation, and rollback plan.",
    ],
  },
  databricks: {
    displayName: "Vanguard Frontier — Databricks",
    description:
      "Curated Databricks agents spanning a cloud-neutral lakehouse and AI board plus Azure-specific assets — static review only, no workspace or production mutations. Covers account and workspace topology, Unity Catalog governance (three-level namespace, GRANT model, workspace-catalog binding, governed tags), identity/network security (SCIM, service principals, OAuth vs personal access tokens, IP access lists, serverless egress, secret scopes), data protection and privacy (row filters, column masks, ABAC, classification, erasure via REORG/VACUUM, Delta Sharing egress, residency), Lakeflow pipelines and Delta table layout, Structured Streaming recovery, data quality and Lakehouse Monitoring, SQL warehouse performance, AI/BI Genie and metric views, MLflow and Model Serving, GenAI agent engineering and evaluation, Declarative Automation Bundles and CI/CD, operational evidence from system tables, FinOps cost attribution, and value realization. Databricks surfaces are drift-prone and differ by cloud, tier, and compute type; agents verify against current Databricks documentation, and pin version-sensitive client APIs against library documentation, before rendering findings.",
    keywords: ["databricks", "unity-catalog", "lakehouse", "lakeflow", "mlflow", "genai", "finops", "least-privilege", "data-engineering", "static-review"],
    invariants: [
      "Static review only — agents never request workspace tokens, service-principal secrets, storage keys, or customer data, and never mutate a Databricks workspace, Unity Catalog, or any cloud resource.",
      "Enforce least privilege: schema-scoped grants (CREATE TABLE/VOLUME/FUNCTION at schema level), no broad ALL PRIVILEGES, assign access to account groups not individuals, separate account/workspace/metastore admin roles.",
      "Storage access uses the platform's managed workload identity for the cloud in question — an Azure Access Connector managed identity, an AWS IAM role assumed by a storage credential, or a GCP service account — surfaced through a Unity Catalog storage credential and external location rather than embedded keys. Name the cloud before recommending a mechanism; do not assume Azure. Production workloads run as service principals, never as interactive users, on every cloud.",
      "Production grant/role/policy/cluster changes are live-guard gated — never auto-dispatched; require explicit approval, scope confirmation, and rollback plan.",
    ],
  },
  frontend: {
    displayName: "Vanguard Frontier — Frontend",
    description:
      "Curated frontend and web development agents for component architecture, accessibility, performance, testing, and security posture — static review only, no live builds, deploys, or dependency mutations. Routes via frontend-maestro to specialist agents. Framework, bundler, and testing-tool surfaces are drift-prone; agents always verify against current official documentation before rendering findings.",
    keywords: ["frontend", "web", "react", "vue", "accessibility", "performance", "testing", "static-review"],
    invariants: [
      "Static review only — agents never request API keys, auth tokens, or customer data, and never run live build, deploy, or dependency-mutation commands.",
      "Route all tasks through frontend-maestro for proper classification and dispatch to specialist agents.",
      "Review covers framework/component architecture, accessibility (WCAG) compliance, performance budgets, test coverage, and supply-chain integrity.",
      "Production-impacting actions (deploys, dependency upgrades, build-config changes) are live-guard gated — never auto-dispatched; require explicit approval and rollback plan.",
    ],
  },
  snowflake: {
    displayName: "Vanguard Frontier — Snowflake",
    description:
      "A cloud-neutral Snowflake board routed by snowflake-maestro: architecture and account topology, platform administration, identity and effective access, network and private connectivity, governance and privacy controls, independent compliance evidence, FinOps unit economics, query performance, batch pipelines, streaming ingestion reliability, analytical semantics, ML lifecycle, Cortex AI agent security, Native Apps and Marketplace, BCDR, DevOps/IaC, migration, and business value. Review agents are read-only; six approval-gated live guards each execute exactly one mutation behind an explicit written human gate. Snowflake surfaces are drift-prone — agents re-verify GA/preview status, deprecations, edition/cloud/region availability, and provider resource stability against current Snowflake documentation before rendering findings.",
    keywords: ["snowflake", "rbac", "least-privilege", "data-governance", "finops", "cortex-ai", "bcdr", "live-guard"],
    invariants: [
      "Review agents are static review only — they never request account credentials, private keys, OAuth or programmatic access tokens, account locators, or customer data, and never mutate a Snowflake account.",
      "Documentation proves supported platform behaviour, never configured account behaviour: edition, region, enabled bundles, authentication enforcement, and policy attachment are UNKNOWN until account evidence establishes them.",
      "Enforce least privilege: narrow custom roles, ACCOUNTADMIN forbidden for automation without exception, no GRANT ALL PRIVILEGES, no grants to PUBLIC, no unbounded future grants, and no password authentication for non-human identities.",
      "Live guards are never auto-dispatched: each executes exactly one mutation after explicit written human approval naming account, environment, target, mutation, and accepted blast radius, with prior-state capture, preflight, verification, and a rollback path.",
    ],
  },
};

const catalog = JSON.parse(readFileSync(catalogPath, "utf8"));

// --- Dynamic provider discovery and derivation ---

/** Special-case display name mappings for providers not in PROVIDERS. */
const DISPLAY_NAME_OVERRIDES = {
  dotnet: ".NET",
  hr: "HR",
  fluxcd: "FluxCD",
  argocd: "ArgoCD",
  opentelemetry: "OpenTelemetry",
  "cert-manager": "Cert-Manager",
  "multi-cloud": "Multi-Cloud",
};

/** Pre-authored keyword sets for derived providers. */
const DERIVED_KEYWORDS = {
  argocd: ["argocd", "gitops", "progressive-delivery", "application-sync"],
  dotnet: ["dotnet", "csharp", "aspnet-core", "ef-core", "nuget"],
  python: ["python", "asyncio", "pyproject", "dependency-confusion", "static-review"],
  marketing: ["marketing-governance", "consent-compliance", "advertising-fairness", "email-authentication"],
  hr: ["hr-governance", "employment-risk", "compensation-equity", "recruiting"],
  legal: ["legal-risk", "contract-review", "privacy-compliance", "regulatory"],
  generic: ["test-quality", "ci-pipeline", "helm-chart", "manifest-review", "agentic-security", "multi-agent-architecture"],
  "multi-cloud": ["finops", "cloud-pricing", "cost-optimization", "reserved-instances"],
  backstage: ["backstage", "scaffolder", "software-templates", "developer-portal"],
  "cert-manager": ["cert-manager", "x509", "certificate-lifecycle", "pki"],
  cilium: ["cilium", "network-policy", "ebpf", "cluster-mesh"],
  falco: ["falco", "runtime-threat", "syscall-rules", "container-security"],
  fluxcd: ["fluxcd", "gitops", "kustomization", "helm-release"],
  istio: ["istio", "service-mesh", "ambient-mesh", "mtls"],
  kyverno: ["kyverno", "admission-policy", "cluster-policy", "policy-enforcement"],
  opentelemetry: ["opentelemetry", "otel-collector", "tracing", "observability-pipeline"],
  prometheus: ["prometheus", "alertmanager", "metrics-cardinality", "scrape-config"],
  sigstore: ["sigstore", "cosign", "supply-chain-integrity", "image-signing"],
};

/**
 * Discover all unique providers from the catalog where at least one agent
 * has 'kiro' in its harnesses array.
 */
function discoverKiroProviders() {
  const providers = new Set();
  for (const entry of catalog) {
    if (
      entry.type === "agent" &&
      Array.isArray(entry.harnesses) &&
      entry.harnesses.includes("kiro")
    ) {
      providers.add(entry.provider);
    }
  }
  return [...providers].sort();
}

/**
 * Title-case a provider name, handling special cases.
 */
function titleCaseProvider(provider) {
  if (DISPLAY_NAME_OVERRIDES[provider]) return DISPLAY_NAME_OVERRIDES[provider];
  return provider
    .split("-")
    .map((w) => w.charAt(0).toUpperCase() + w.slice(1))
    .join("-");
}

/**
 * Derive a topic summary from agent IDs for description generation.
 */
function deriveTopics(entries) {
  const topics = entries
    .map((e) => e.id)
    .filter((id) => !id.endsWith("-maestro-agent"))
    .map((id) => {
      // Strip provider prefix and -agent / -review-agent suffix
      let topic = id
        .replace(/-review-agent$/, "")
        .replace(/-agent$/, "")
        .replace(/-run-agent$/, "");
      // Remove known provider prefixes
      const prefixes = [
        "dotnet-", "hr-", "legal-", "marketing-", "finops-",
        "argocd-", "backstage-", "cert-manager-", "cilium-",
        "falco-", "fluxcd-", "istio-", "kyverno-",
        "opentelemetry-", "prometheus-", "sigstore-",
      ];
      for (const pfx of prefixes) {
        if (topic.startsWith(pfx)) {
          topic = topic.slice(pfx.length);
          break;
        }
      }
      return topic.replace(/-/g, " ");
    })
    .slice(0, 4);
  return topics.join(", ");
}

/**
 * Directories under agents/ that actually hold a provider's agents. The provider id is
 * not a path: generic agents live in agents/qa/ and agents/cross-functional/, so naming
 * agents/<provider>/ would point readers at a directory that does not exist.
 */
function agentDirs(entries) {
  const dirs = [...new Set(entries.map((e) => e.path.split("/")[1]))].sort();
  return dirs.map((d) => `agents/${d}/`).join(" and ");
}

/**
 * Select the routing maestro for a provider's Power.
 *
 * A board may carry more than one `*-maestro-agent` (e.g. the python board has both
 * `python-maestro-agent` for static review and `python-live-governance-maestro-agent`
 * for the live control plane). A plain `endsWith("-maestro-agent")` picks whichever
 * sorts first in the catalog, which mis-routes the Power's static entry point. Always
 * prefer the exact `{provider}-maestro-agent`; only fall back to the first suffix match
 * when the board has no canonically named maestro.
 */
function selectMaestro(entries, provider) {
  return (
    entries.find((e) => e.id === `${provider}-maestro-agent`) ||
    entries.find((e) => e.id.endsWith("-maestro-agent"))
  );
}

/**
 * Select a board's live control-plane maestro, if it has one distinct from the canonical
 * static maestro (e.g. `python-live-governance-maestro-agent`).
 *
 * A two-plane board routes through two different maestros, and they are NOT
 * interchangeable: the static maestro's contract refuses live operations and its routing
 * taxonomy carries no live guards, so naming it as the router that "places live-guards in
 * live-guard-gate" is incoherent — it would send a mutation task to an agent built to
 * refuse it, while the only agent that actually gates the operators goes unmentioned.
 * Keyed on `-live-` so multi-maestro boards whose extra maestros are sub-routers rather
 * than live routers (e.g. microsoft's m365/d365/power-platform/copilot-governance) are
 * not misread as having a live plane.
 */
function selectLiveMaestro(entries, provider) {
  return entries.find(
    (e) =>
      e.id !== `${provider}-maestro-agent` &&
      e.id.endsWith("-maestro-agent") &&
      /-live-/.test(e.id),
  );
}

/**
 * Select a provider's live-mutation guards — the agents that are never
 * auto-dispatched and must pass a live-guard gate before executing a mutation.
 *
 * The authoritative signal is `execution_tier === "mutating-runtime"`. The older
 * `-live-` naming heuristic mislabels read-only agents: on a mixed-tier board a
 * read-only-runtime observer or the routing maestro can carry `-live-` in its id
 * (e.g. `python-live-system-inventory-agent`, the SAP `-live-readonly-*` discovery
 * agents) yet execute no mutation, and listing them as guards collapses the very
 * read-only/mutating boundary this catalog exists to keep explicit. So when the
 * board declares any mutating-runtime tier, filter strictly by tier; only fall back
 * to the naming heuristic for boards that predate execution_tier (the cloud boards,
 * where every entry's tier is absent).
 */
function selectLiveGuards(entries) {
  const hasMutatingTier = entries.some(
    (e) => e.execution_tier === "mutating-runtime",
  );
  return hasMutatingTier
    ? entries.filter((e) => e.execution_tier === "mutating-runtime")
    : entries.filter((e) => /-live-/.test(e.id));
}

/**
 * Auto-generate steering content for a provider NOT in the hardcoded
 * PROVIDERS object.
 */
function deriveProviderConfig(provider, catalogEntries) {
  const displayLabel = titleCaseProvider(provider);
  const displayName = `Vanguard Frontier \u2014 ${displayLabel}`;

  const entries = catalogEntries.filter(
    (e) => e.type === "agent" && e.provider === provider,
  );
  const maestro = selectMaestro(entries, provider);
  const liveMaestro = selectLiveMaestro(entries, provider);
  const liveGuards = selectLiveGuards(entries);

  // Build description (max 3 sentences — Kiro constraint enforced by validate:kiro-powers).
  // A board that carries live-guard agents is NOT "static review only"; and on a two-plane
  // board the static maestro must not be credited with routing live-guards it refuses.
  let description;
  if (maestro && entries.length > 2) {
    const topics = deriveTopics(entries);
    if (liveMaestro) {
      description = `Curated ${displayLabel} agents for ${topics}. Routes static review via ${maestro.id} and live control-plane work via ${liveMaestro.id}, which alone gates the live-guard operators. Live mutations require approval, target confirmation, evidence capture, and a rollback plan; static specialists never mutate.`;
    } else if (liveGuards.length) {
      description = `Curated ${displayLabel} agents for ${topics}. Routes via ${maestro.id} to specialist or live-guard agents based on task scope. Live-mutation agents require approval, target confirmation, evidence capture, and a rollback plan; static specialists never mutate.`;
    } else {
      description = `Curated ${displayLabel} agents for ${topics}. Routes via ${maestro.id} to specialist agents based on task scope. Static review only; no live mutations.`;
    }
  } else if (entries.length === 1) {
    // Single agent, no maestro
    const summary = entries[0].summary || "";
    // Split into sentences - skip "Agent for <id>." prefix if present
    const sentences = summary.split(/(?<=[.!?])\s/);
    let useSentence = sentences[0] || "";
    if (/^Agent for\s/i.test(useSentence) && sentences.length > 1) {
      useSentence = sentences[1];
    }
    // Remove trailing period for reassembly
    useSentence = useSentence.replace(/\.$/, "");
    // Strip leading "Static review of" / "Review" prefix to avoid doubling
    let core = useSentence
      .replace(/^Static,?\s+evidence-gated\s+review\s+of\s+/i, "")
      .replace(/^Static\s+review\s+of\s+/i, "")
      .replace(/^Review\s+(a\s+)?/i, "");
    // Truncate if too long, respecting word boundaries
    if (core.length > 120) {
      const truncated = core.substring(0, 117);
      const lastSpace = truncated.lastIndexOf(" ");
      core = (lastSpace > 0 ? truncated.substring(0, lastSpace) : truncated) + "...";
    }
    const sep = core.endsWith("...") ? " " : ". ";
    description = `Reviews ${core.charAt(0).toLowerCase() + core.slice(1)}${sep}Static review only; no live mutations.`;
  } else {
    // Multiple agents, no maestro
    const topics = deriveTopics(entries);
    description = `Curated ${displayLabel} review agents covering ${topics}. Reference agents directly under ${agentDirs(entries)}. Static review only; no live mutations.`;
  }

  // Keywords
  const keywords = DERIVED_KEYWORDS[provider] || [
    provider,
    "static-review",
    "configuration-audit",
    "best-practices",
  ];

  // Invariants
  const invariants = [];
  if (liveGuards.length > 0) {
    invariants.push(
      `Live-guard agents (the mutating-runtime operators) must never be auto-dispatched; require explicit approval, evidence capture, and a rollback plan${
        liveMaestro ? `, and may only be gated by ${liveMaestro.id}` : ""
      }. Read-only-runtime and static-review agents on this board are not guards.`,
    );
  }
  if (maestro) {
    invariants.push(
      liveMaestro
        ? `Route static-review tasks through ${maestro.id} and live control-plane tasks through ${liveMaestro.id}; never send a live-mutation task to ${maestro.id}, whose contract refuses live operations.`
        : `Route all tasks through ${maestro.id} for proper classification and dispatch.`,
    );
  }
  invariants.push(
    liveGuards.length
      ? "Mixed-tier board: static specialists analyze configuration without mutating live systems; live-guard agents mutate only under approval, target confirmation, evidence capture, and a pre-approved rollback plan."
      : "Static review only -- agents analyze configuration and provide findings without mutating live systems.",
  );
  // Add domain-specific invariants
  if (provider === "dotnet") {
    invariants.push("Review covers language runtime, frameworks, data access, testing, and supply-chain integrity.");
  } else if (provider === "hr") {
    invariants.push("All findings must respect employee privacy and data-minimization principles.");
  } else if (provider === "legal") {
    invariants.push("Agents provide risk-flagging only; output is not legal advice and does not create attorney-client privilege.");
  } else if (provider === "marketing") {
    invariants.push("Review covers consent, privacy, fairness, and regulatory compliance for marketing systems.");
  } else if (provider === "multi-cloud") {
    invariants.push("Cost recommendations are estimates based on public pricing; verify against actual billing before acting.");
  } else if (provider === "generic") {
    invariants.push("Agents are provider-agnostic: CI, Helm, manifest, and test-quality review under agents/qa/; cross-tier revenue-journey integrity and vendor-neutral agentic system architecture and security review under agents/cross-functional/. The agentic reviewers hand off through skills/cross-functional/agentic-board-protocol and never approve or execute.");
  } else if (provider === "argocd") {
    invariants.push("Sync and rollout strategies must be validated against the target cluster GitOps workflow.");
  } else if (provider === "backstage") {
    invariants.push("Template parameters and scaffolder actions must be reviewed for injection and secret-exposure risks.");
  } else if (provider === "cert-manager") {
    invariants.push("Certificate renewal windows and issuer trust chains must be validated before any policy change.");
  } else if (provider === "cilium") {
    invariants.push("Network policies must be reviewed for unintended traffic blocking across namespaces and cluster-mesh endpoints.");
  } else if (provider === "falco") {
    invariants.push("Rule changes must be evaluated for false-positive rate impact on production alerting.");
  } else if (provider === "fluxcd") {
    invariants.push("Kustomization and HelmRelease reconciliation intervals must align with the GitOps change cadence.");
  } else if (provider === "istio") {
    invariants.push("Service mesh policies affect traffic routing cluster-wide; review blast radius before changes.");
  } else if (provider === "kyverno") {
    invariants.push("Cluster-scoped policies can reject legitimate workloads; validate against existing deployments before applying.");
  } else if (provider === "opentelemetry") {
    invariants.push("Collector pipeline changes affect observability for all instrumented services; review cardinality impact.");
  } else if (provider === "prometheus") {
    invariants.push("Alerting rule and scrape config changes affect monitoring coverage; review for metric-name collisions.");
  } else if (provider === "sigstore") {
    invariants.push("Supply-chain policy changes can block valid deployments; verify cosign keyless trust roots before enforcement.");
  }

  return { displayName, description, keywords, invariants };
}

/**
 * Build a merged map combining hand-authored PROVIDERS with auto-derived
 * entries for all kiro-enabled providers in the catalog.
 */
function buildMergedProviders() {
  const kiroProviders = discoverKiroProviders();
  const merged = {};

  for (const provider of kiroProviders) {
    if (PROVIDERS[provider]) {
      merged[provider] = PROVIDERS[provider];
    } else {
      merged[provider] = deriveProviderConfig(provider, catalog);
    }
  }

  // Sort alphabetically for deterministic output
  const sorted = {};
  for (const key of Object.keys(merged).sort()) {
    sorted[key] = merged[key];
  }
  return sorted;
}

const allProviders = buildMergedProviders();

function summarize(provider) {
  const entries = catalog.filter(
    (e) => e.type === "agent" && e.provider === provider,
  );
  const kiroEntries = entries.filter(
    (e) => Array.isArray(e.harnesses) && e.harnesses.includes("kiro"),
  );
  const maestro = selectMaestro(entries, provider);
  const liveMaestro = selectLiveMaestro(entries, provider);
  const liveGuards = selectLiveGuards(entries)
    .map((e) => e.id)
    .sort();
  return {
    total: entries.length,
    kiroAvailable: kiroEntries.length,
    maestro,
    liveMaestro,
    liveGuards,
    dirs: agentDirs(entries),
  };
}

function renderPower(provider, cfg) {
  const { total, kiroAvailable, maestro, liveMaestro, liveGuards, dirs } = summarize(provider);
  const frontmatter = [
    "---",
    `name: "vanguard-${provider}"`,
    `displayName: "${cfg.displayName}"`,
    `description: "${cfg.description}"`,
    `keywords: [${cfg.keywords.map((k) => `"${k}"`).join(", ")}]`,
    `author: "VincentChuWaiChow"`,
    "---",
  ].join("\n");

  const liveGuardSection = liveGuards.length
    ? liveGuards.map((id) => `- \`${id}\` — never auto-dispatched; gate_mode only`).join("\n")
    : "- *(none — this provider has no live-mutation guards in the catalog)*";

  const maestroLine = maestro
    ? [
        `- **\`${maestro.id}\`** — classifies and routes the task to the right specialist`,
        ...(liveMaestro
          ? [
              `- **\`${liveMaestro.id}\`** — the live control-plane router; the only entry point that may place a live-guard operator in \`live-guard-gate\``,
            ]
          : []),
      ].join("\n")
    : `- *(no maestro for this provider; reference agents directly under \`agents/${provider}/\`)*`;

  const adapterNote =
    kiroAvailable === total
      ? total === 1
        ? `The single agent in this provider ships a Kiro adapter (\`harnesses/kiro-ide.agent.md\`, \`kiro-cli.agent.json\`).`
        : `All ${total} agents in this provider ship a Kiro adapter (\`harnesses/kiro-ide.agent.md\`, \`kiro-cli.agent.json\`).`
      : kiroAvailable === 0
        ? `This provider's ${total} agents do not yet ship Kiro adapters — this Power supplies steering content only. Use \`npx vfa-export-agents --platform kiro --provider ${provider}\` from the npm package once Kiro adapters land.`
        : `${kiroAvailable} of ${total} agents in this provider ship a Kiro adapter; the rest provide steering context only.`;

  const body = [
    "",
    `# ${cfg.displayName}`,
    "",
    cfg.description,
    "",
    "## When to engage this Power",
    "",
    `Activate when the task references ${provider === "kubernetes" ? "Kubernetes, cluster, namespace, RBAC, or admission policy" : provider === "terraform" ? "Terraform, IaC, plan, apply, or state" : provider === "nvidia" ? "NVIDIA, NGC, NIM, GPU, or CUDA" : `${cfg.displayName.replace(/^Vanguard Frontier — /, "")} services, resources, or operations`}. Do not activate on unrelated requests — narrow keyword matching is required to avoid false activations (Kiro Powers convention).`,
    "",
    "## Routing pattern",
    "",
    maestroLine,
    "",
    maestro
      ? liveMaestro
        ? `This board has two planes with two routers. Send static-review work to \`${maestro.id}\` — its contract refuses live operations, so it must never be given a mutation task. Send live control-plane work to \`${liveMaestro.id}\`, which is the only router that gates the live-guard operators below. Classify first, then dispatch to one specialist or a small parallel team; never have either maestro execute a mutation itself.`
        : "Use the maestro as the entry point: classify the task, then dispatch to one specialist or a parallel team of specialists. Never have the maestro itself execute a live mutation."
      : "Reference agents directly from " + dirs + " without maestro-based routing.",
    "",
    "## Live-guard agents (gate_mode only)",
    "",
    liveGuardSection,
    "",
    `Live-guard agents enforce approval, target confirmation, evidence capture, and rollback plans before executing a mutation. They are never auto-dispatched — ${
      liveMaestro ? `\`${liveMaestro.id}\`` : "the maestro"
    } must place them in \`live-guard-gate\` or \`runtime-evidence-gate\` mode.`,
    "",
    "## Invariants",
    "",
    cfg.invariants.map((s) => `- ${s}`).join("\n"),
    "",
    "## Where the agents live",
    "",
    `Agent specs and adapters are part of the [Vanguard Frontier Agentic](https://github.com/VincentChuWaiChow/vanguard-frontier-agentic) marketplace. For this provider, see ${dirs.split(" and ").map((d) => "`" + d + "`").join(" and ")} in that repository. ${adapterNote}`,
    "",
    "## Companion install paths",
    "",
    "- **Claude Code:** `/plugin marketplace add VincentChuWaiChow/vanguard-frontier-agentic` then `/plugin install vanguard-frontier-agentic@vanguard-frontier-agentic`",
    `- **Codex / Copilot / Cursor / Gemini CLI / Kiro (file export):** \`npx vfa-export-agents --platform <harness> --provider ${provider} --repo .\``,
    "",
  ].join("\n");

  return frontmatter + body;
}

function renderReadme() {
  const providerKeys = Object.keys(allProviders);
  const count = providerKeys.length;
  const tree = providerKeys
    .map((p, i) => {
      const prefix = i < count - 1 ? "├──" : "└──";
      return `${prefix} vanguard-${p}/POWER.md`;
    })
    .join("\n");

  return `# \`powers/\` — Kiro Powers

This directory holds **${count} Kiro Powers** for \`vanguard-frontier-agentic\`, one
per cloud/platform/IaC provider. Each Power is a directory containing a
\`POWER.md\` file with strict-5 frontmatter and steering content.

## What's in here

\`\`\`
powers/
${tree}
\`\`\`

Each \`POWER.md\` declares:

- **Frontmatter (strict-5):** \`name\`, \`displayName\`, \`description\` (≤ 3
  sentences), \`keywords\` (specific, non-broad), \`author\`. **No other fields
  permitted** by Kiro spec.
- **Body steering:** when to engage, routing pattern (\`<provider>-maestro\`),
  live-mutation discipline, provider-specific invariants (e.g. MLPS 2.0 for
  Alibaba/Huawei, Enterprise Project vs IAM scope for Huawei, account-ID
  /region confirmation for AWS).

## How users install

Kiro Powers don't have a one-command marketplace install — the Powers panel
is per-Power directory add. Users clone the repo and add each Power they
need via the Kiro UI:

\`\`\`bash
# 1. Clone the repo
git clone https://github.com/VincentChuWaiChow/vanguard-frontier-agentic
cd vanguard-frontier-agentic
\`\`\`

\`\`\`text
2. In Kiro:
   Open the Powers panel → "Add Custom Power" → "Local Directory"
   Paste the absolute path to the Power(s) you need:
      /absolute/path/to/vanguard-frontier-agentic/powers/vanguard-aws
      /absolute/path/to/vanguard-frontier-agentic/powers/vanguard-kubernetes
   Repeat for each provider you work with.
\`\`\`

## How to update

\`\`\`bash
# Regenerate the ${count} Powers from catalog/agents.json + per-provider config:
npm run kiro-powers:write

# Then verify everything is in sync:
npm run validate:kiro-powers
\`\`\`

The \`validate\` chain runs \`validate:kiro-powers\` automatically. The
validator enforces:

- strict-5 frontmatter (any extra field fails)
- lowercase kebab-case names
- name matches directory name
- description ≤ 3 sentences (decimal-aware — "MLPS 2.0" doesn't count as a
  sentence break)
- non-empty keywords list, no broad terms (\`cloud\`, \`devops\`, \`code\`,
  \`agent\`, \`ml\`, etc.) per Kiro's anti-false-activation guidance
- generator in sync (\`--check\`)

## Schema references (official Kiro docs)

- **Kiro Powers repo:** <https://github.com/kirodotdev/powers>
- **POWER.md frontmatter spec:**
  <https://github.com/kirodotdev/powers/blob/main/power-builder/POWER.md>
- **Interactive power builder:**
  <https://github.com/kirodotdev/powers/blob/main/power-builder/steering/interactive.md>
- **Testing a power locally:**
  <https://github.com/kirodotdev/powers/blob/main/power-builder/steering/testing.md>
- **Kiro IDE:** <https://kiro.dev/>

## Design notes

- **One Power per provider, not one mega-Power** — Kiro docs warn that
  broad keywords trigger false activations across unrelated tasks. One
  narrowly-scoped Power per provider keeps activation precise:
  \`vanguard-alibaba\` activates on Alibaba Cloud work only; \`vanguard-aws\`
  never activates on Azure questions.
- **Hetzner and Contabo Powers exist** even though their agents don't yet
  ship Kiro adapter files (their \`harnesses: [codex, claude-code]\`). Powers
  are steering-first — the steering content stands alone. When their Kiro
  adapter files land, the Powers will gain agent-routing as well.
- **No \`version\`, \`repository\`, \`license\`, or \`tags\`** — Kiro spec
  explicitly forbids these fields in frontmatter. The validator fails on
  any extra field.
`;
}

const errors = [];
const written = [];

for (const [provider, cfg] of Object.entries(allProviders)) {
  const dir = join(powersRoot, `vanguard-${provider}`);
  const file = join(dir, "POWER.md");
  const next = renderPower(provider, cfg);

  if (check) {
    if (!existsSync(file)) {
      errors.push(`${relative(repoRoot, file)} is missing`);
      continue;
    }
    if (readFileSync(file, "utf8") !== next) {
      errors.push(`${relative(repoRoot, file)} is stale; run npm run kiro-powers:write`);
    }
  } else {
    if (!existsSync(dir)) mkdirSync(dir, { recursive: true });
    writeFileSync(file, next);
    written.push(relative(repoRoot, file));
  }
}

// Generate/check README.md
const readmePath = join(powersRoot, "README.md");
const nextReadme = renderReadme();
if (check) {
  if (!existsSync(readmePath)) {
    errors.push(`${relative(repoRoot, readmePath)} is missing`);
  } else if (readFileSync(readmePath, "utf8") !== nextReadme) {
    errors.push(`${relative(repoRoot, readmePath)} is stale; run npm run kiro-powers:write`);
  }
} else {
  writeFileSync(readmePath, nextReadme);
  written.push(relative(repoRoot, readmePath));
}

if (check) {
  if (errors.length) {
    errors.forEach((e) => console.error(`ERROR: ${e}`));
    process.exit(1);
  }
  console.log(
    `OK: ${Object.keys(allProviders).length} Kiro Powers are in sync`,
  );
} else {
  console.log(`OK: wrote ${written.length} Kiro Powers (+ README.md)`);
  written.forEach((f) => console.log(`  ${f}`));
}
