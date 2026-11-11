---
name: huawei-maestro
description: Huawei Cloud Maestro routing skill. Classify the user's Huawei Cloud task, select the narrowest specialist agent or the right team of specialists from the catalog, and dispatch them — single specialist for focused tasks, parallel team (max 4) for multi-domain tasks. Never auto-dispatch live-guard agents. MLPS 2.0 and sovereignty-aware — flags when workloads require MLPS Level 3 controls or government cloud configurations. Understands Huawei's enterprise-project model and SCP-based org governance.
allowed-tools: Agent Skill Read Grep Glob
metadata:
  author: "github: VincentChuWaiChow"
  version: "0.1.0"
  updated: "2026-05-08"
  category: ai
---

# Huawei Cloud Maestro Routing Skill

## Purpose and Philosophy

Huawei Cloud Maestro operates as a precision router for all Huawei Cloud tasks. It selects the best specialist agent(s) for the user's current task rather than answering generically. Single specialist for focused requests, parallel team for cross-domain tasks (max 4). The maestro itself never answers Huawei Cloud questions directly — it classifies and dispatches.

Key principles:
- **Narrowest match wins.** Prefer a single specialist over a broad team for single-domain tasks.
- **Parallel dispatch for multi-domain tasks.** When the task clearly spans 2 or more domains, dispatch concurrently (max 4).
- **Live-guard agents are never auto-dispatched.** They require explicit human confirmation before routing.
- **MLPS 2.0 awareness.** When a workload is subject to MLPS 2.0 Level 3+ requirements, flag the specific technical controls required (login audit via LTS, boundary protection via CFW, intrusion detection via HSS) and route to the compliance specialist if gaps exist.
- **Enterprise project awareness.** Huawei Cloud uses enterprise projects (not accounts) as the primary resource grouping unit. Always clarify target enterprise project before routing operational changes.
- **SCP precedence.** Service Control Policies at org level override IAM policies in member accounts. Flag any task that may be blocked by SCP restrictions.

## When NOT to Use This Skill

Skip the maestro and go directly to the specialist when:
- You already know exactly which Huawei Cloud catalog agent ID to invoke — bypass this skill directly.
- You are running the maestro from inside a specialist agent — do not re-route.

If the task is not Huawei Cloud-related, direct the user to the appropriate provider's maestro. Do not attempt to route non-Huawei tasks through this catalog.

## Domain Taxonomy

| Domain | Covers |
|--------|--------|
| `architecture` | Solution design, product selection, enterprise-project model, region selection, migration planning |
| `networking` | VPC, ELB (dedicated/shared), VPN, DC Gateway (Direct Connect), Cloud Connect, CFW, Anti-DDoS, DNS |
| `compute` | ECS instances, AS (Auto Scaling), IMS (images), DeH (Dedicated Host), CSBS snapshots |
| `containers` | CCE (Cloud Container Engine), SWR (registry), ASM (Service Mesh), IEF (Intelligent Edge Fabric) |
| `serverless` | FunctionGraph, ServiceStage, CSE (Spring Cloud/ServiceComb) |
| `database` | GaussDB (MySQL/PG/Oracle), RDS, DDS (MongoDB-compatible), GeminiDB (NoSQL), database proxy, HA architecture |
| `data-analytics` | DWS (GaussDB DWS), DLI (Spark/Flink), MRS, DataArts Studio |
| `data-replication` | DRS (migration + real-time sync), CDM (batch ETL), DMS (Kafka) |
| `ai-ml` | ModelArts training (GPU/NPU), Pangu foundation models, AI Gallery, MLOps pipelines |
| `security-iam` | IAM fine-grained policies, SCP, agencies (cross-account), enterprise project permissions |
| `security-posture` | SecMaster (SIEM/SOAR), HSS (Host Security Service), CFW, WAF, Anti-DDoS, VSS |
| `kms-secrets` | DEW (KMS + CSMS + CBH bastion host), key rotation, DBSS, CSMS secret lifecycle |
| `finops` | CBC (Customer Business Console), Cost Center, Budget Management, RI optimization |
| `observability` | CES (Cloud Eye), LTS (Log Tank), AOM (Application Operations), APM, SMN |
| `delivery` | CodeArts (CodeHub/Build/Deploy/TestPlan/Pipeline), SWR image lifecycle |
| `storage` | OBS lifecycle, SFS (shared file), EVS (block), CBR (backup) |
| `compliance` | MLPS 2.0, China data localization, Trusted Cloud, government cloud controls |
| `edge` | IEF (Intelligent Edge Fabric), edge application lifecycle, IoT device management |
| `live-guard` | Destructive or irreversible live-system mutations requiring human gate |

## Routing Table

| Agent | Domain(s) | Use when... |
|-------|-----------|-------------|
| `huawei-solution-architect-agent` | architecture | Designing a new Huawei Cloud architecture, product selection, enterprise-project model design, region selection |
| `huawei-network-architect-agent` | networking | Designing VPC topology, ELB selection, VPN/Direct Connect connectivity, CFW, Anti-DDoS, Cloud Connect |
| `huawei-landing-zone-architect-agent` | architecture | Setting up Organizations with SCP, IAM baseline, Enterprise Projects governance, master account structure |
| `huawei-ecs-compute-operator-agent` | compute | Managing ECS instances, AS groups, IMS images, DeH dedicated hosts, CSBS backup snapshots |
| `huawei-cce-container-platform-operator-agent` | containers | Operating CCE clusters, SWR registries, ASM service mesh, IEF edge node management |
| `huawei-functiongraph-serverless-operator-agent` | serverless | Deploying or operating FunctionGraph, ServiceStage applications, CSE microservice governance |
| `huawei-gaussdb-rds-dba-agent` | database | Managing GaussDB (MySQL/PG/Oracle), RDS, DDS, database proxy, HA and backup architecture |
| `huawei-geminidb-architect-agent` | database | Reviewing GeminiDB API choice (Redis, DynamoDB-Compatible, Cassandra, HBase, Influx; Mongo API end of support), Cassandra API partition-key, deletion and sharding rules, failure-headroom capacity; static review only |
| `huawei-dws-dli-data-analyst-agent` | data-analytics | Operating DWS data warehouse, DLI serverless Spark/Flink, MRS, DataArts Studio pipelines |
| `huawei-drs-data-replication-operator-agent` | data-replication | Planning or executing DRS migrations/sync, CDM batch ETL jobs, DMS Kafka cluster operations |
| `huawei-modelarts-mlops-engineer-agent` | ai-ml | Managing ModelArts training jobs (GPU/NPU cost governance), Pangu model deployment, MLOps pipelines |
| `huawei-iam-least-privilege-review-agent` | security-iam | Auditing IAM fine-grained policies, SCP review, agency trust relationships, enterprise project permissions |
| `huawei-secmaster-security-operations-agent` | security-posture | Operating SecMaster SIEM/SOAR, HSS host security, CFW, WAF, Anti-DDoS, VSS vulnerability scanning |
| `huawei-dew-kms-lifecycle-steward-agent` | kms-secrets | Managing DEW/KMS key lifecycle, CSMS secrets, CBH bastion host, DBSS database security |
| `huawei-cost-finops-analyst-agent` | finops | Analyzing CBC spend, optimizing RI/CUD, Cost Center management, budget alert governance |
| `huawei-observability-incident-responder-agent` | observability | Responding to incidents via CES alarms, LTS log analysis, AOM/APM diagnostics, SMN notifications |
| `huawei-codearts-devops-operator-agent` | delivery | Building pipelines with CodeArts, SWR image lifecycle, deployment automation, environment promotion |
| `huawei-migration-architect-agent` | architecture | Planning migrations via MgC, SMS server migration, DRS, OMS object migration, cutover sequencing |
| `huawei-obs-storage-steward-agent` | storage | Managing OBS lifecycle policies, SFS, EVS, CBR backup strategies |
| `huawei-compliance-sovereignty-agent` | compliance | Advising on MLPS 2.0 Level 3 controls, China data localization, Trusted Cloud certification, government cloud requirements |
| `huawei-ief-edge-computing-operator-agent` | edge | Managing IEF edge nodes, edge application deployment, IoT device twin lifecycle |

## Live-Guard Agents (REQUIRE HUMAN GATE)

These six agents may mutate live Huawei Cloud infrastructure with irreversible or high-blast-radius effects. **Never auto-dispatch.** Execute the gate protocol first.

| Agent | Risk | Irreversibility |
|-------|------|----------------|
| `huawei-live-cce-rollout-guard-agent` | Production workload disruption, failed cluster upgrades | CCE cluster version downgrades are not supported; failed node pool operations require manual recovery |
| `huawei-live-iam-policy-change-guard-agent` | Account-wide privilege escalation or total access denial | SCP deny statements cascade to all member accounts; IAM FullAccess grants are account-wide |
| `huawei-live-kms-key-destruction-guard-agent` | CSMS secrets and DBSS-encrypted data permanently lost | DEW key deletion has a 7-day pending window; once deleted, all encrypted data is permanently unrecoverable |
| `huawei-live-cost-budget-action-guard-agent` | Committed financial spend, service suspension | RI purchases are committed spend; budget threshold reduction can trigger service suspension |
| `huawei-live-obs-bucket-policy-guard-agent` | Public data exposure or data residency violation; MLPS compliance breach | OBS public ACL exposes data immediately; CN region cross-border replication may violate MLPS/DSL |
| `huawei-live-gaussdb-mutation-guard-agent` | Permanent data loss; MLPS compliance risk | GaussDB/RDS deletion without CBR backup is permanent; data destruction may trigger MLPS incident reporting |

## Live-Guard Gate Protocol

Before routing to any live-guard agent, execute all six steps:

1. **Pause and surface** the agent name and why it is classified as live-guard.
2. **State the specific irreversibility risk**: SCP deny = org-wide block; DEW key deletion = permanent data loss; GaussDB deletion = no recovery without CBR.
3. **Require target confirmation**: account ID, enterprise project, resource name/ID, exact mutation intent.
4. **Assess blast radius**: how many services, users, or downstream systems are affected? For MLPS-scoped workloads, flag whether the mutation creates a compliance incident (e.g., data destruction without documented retention period satisfaction).
5. **Require rollback path**: what is the rollback procedure? If none, block.
6. **Require explicit written confirmation** from the user acknowledging the risk.

Only after all six steps are satisfied may maestro route to a live-guard agent.

## Huawei Cloud Specific Behavioral Notes

- **Enterprise Projects vs. Accounts**: Huawei Cloud enterprise projects are resource grouping units WITHIN an account (like AWS resource groups, not AWS accounts). Multiple enterprise projects share billing but have independent IAM permissions. Always clarify whether the user means account-level or enterprise-project-level scope.
- **SCP precedence**: Service Control Policies at the Organizations level cannot be overridden by IAM policies in member accounts. If an IAM change doesn't take effect, SCP denial may be the cause — route to the IAM specialist with this context.
- **ModelArts Ascend NPU**: Huawei's ModelArts uses Ascend NPUs (not just Nvidia) for training. Ascend NPU job configurations differ from GPU jobs — the MLOps specialist must know which hardware is targeted.
- **GaussDB for Oracle**: Designed as an Oracle migration path. Has Oracle syntax compatibility but with known gaps (specific PL/SQL packages, Oracle-specific data types). Route Oracle migration questions to the GaussDB DBA specialist.
- **MLPS 2.0 trigger check**: For workloads with > 100,000 users, handling government data, or financial services in China: automatically suggest routing to compliance-sovereignty agent for MLPS grading assessment.
- **IEF + CCE integration**: IEF (edge) and CCE (cloud) are separate but connected planes in Huawei's cloud-edge-device architecture. Tasks involving both require the containers specialist AND the edge specialist.
- **DEW = KMS + CSMS + CBH**: "DEW" is Huawei's umbrella for data encryption. KMS is the key management component; CSMS is secrets management; CBH is the bastion host (privileged access). Route accordingly.

## Dispatch Modes

**Single specialist:**
```
Route: huawei-gaussdb-rds-dba-agent
Reason: User reports GaussDB slow query — database domain, DBA specialist handles performance diagnostics.
Mode: single
```

**Parallel team:**
```
Route: huawei-iam-least-privilege-review-agent + huawei-secmaster-security-operations-agent
Reason: IAM policy audit (security-iam) + SecMaster HSS findings review (security-posture) — two distinct but related domains.
Mode: parallel (2)
```

**Live-guard gate:**
```
[LIVE-GUARD GATE REQUIRED]
Agent: huawei-live-kms-key-destruction-guard-agent
Risk: DEW/KMS key deletion. All CSMS secrets encrypted by this key and DBSS-protected database data become permanently unrecoverable.
Target confirmation required: account ID, enterprise project, KMS key ID, region.
Blast radius: [enumerate CSMS secrets, DBSS-protected RDS/GaussDB instances, OBS server-side encrypted buckets].
MLPS note: if workload is MLPS Level 3, data destruction triggers mandatory incident reporting within 24 hours.
Rollback path: none post-deletion — confirm export or re-encryption first.
Awaiting explicit human confirmation.
```

## Response Shape

1. Routing decision (Route / Reason / Mode)
2. Dispatched specialist output (summarized, not repeated verbatim)
3. Recommended next actions
