# Routing table and domain taxonomy

Use this reference when classifying a task or selecting the right specialist(s).

## Domain taxonomy

| Domain | Keywords and signals |
|---|---|
| `architecture` | solution design, Well-Architected review, architecture diagram, reference architecture, landing zone, multi-account, migration, cutover, resilience, BCDR, API gateway design, event-driven design, networking topology, VPC |
| `compute` | EC2, ECS, Fargate, EKS, Lambda, serverless, container, pod, fleet, autoscaling, AMI, launch template, capacity reservation, spot, deployment rollout, hotfix |
| `data` | RDS, Aurora, DynamoDB, S3, database, query performance, data modeling, index, backup, data perimeter, bucket policy, data protection, restore |
| `security-iam` | IAM, policy, role, permission, SCP, KMS, key rotation, secrets, Secrets Manager, posture, GuardDuty, SecurityHub, compliance, evidence, Bedrock security |
| `pki` | ACM PCA, AWS Private CA, aws-privateca-issuer, AWSPCAIssuer, AWSPCAClusterIssuer, certificate template ARN, CRL distribution, CRL S3, IRSA cert-manager, cross-account PCA, RAM-shared CA, SubordinateCACertificate, private certificate authority |
| `cost` | cost, spend, billing, anomaly, savings plan, reserved instance, rightsizing, waste, budget |
| `devops-cicd` | pipeline, CI/CD, CodePipeline, CodeBuild, GitHub Actions, IaC, CloudFormation, Terraform, CDK, patch, release engineer, deploy, rollback |
| `operations` | observability, CloudWatch, X-Ray, incident, alert, runbook, triage, ticket, escalation, change impact, briefing, daily ops, non-destructive automation |
| `live-guard` | live deploy, live rollout, live release, production push, approve pipeline, ECS rollout to prod, serverless release to prod, IaC apply to prod, requires human gate |
| `ai-genai` | Bedrock, generative AI, foundation model, agent, AgentCore, prompt, RAG, LLM, Bedrock Agents, DevOps agent skill |
| `networking` | VPC, subnet, route table, Transit Gateway, Direct Connect, VPN, PrivateLink, security group, NACLs, network ACL, API edge delivery, CloudFront, WAF, network architect |

## Full routing table

### Architecture

| Agent | Domain(s) | Use when… |
|---|---|---|
| `aws-solution-architect-agent` | architecture | Designing or reviewing a multi-service AWS solution, Well-Architected assessment, or cross-domain architecture decision |
| `aws-network-architect-agent` | architecture, networking | Designing VPC topology, Transit Gateway, PrivateLink, Direct Connect, or hybrid network patterns |
| `aws-landing-zone-governor-agent` | architecture | Setting up or reviewing an AWS Organizations / Control Tower landing zone, multi-account governance |
| `aws-migration-cutover-architect-agent` | architecture | Planning or executing a migration cutover, wave planning, dependency mapping before go-live |
| `aws-resilience-bcdr-review-agent` | architecture | Reviewing or designing for resilience, disaster recovery targets (RTO/RPO), multi-region failover |
| `aws-api-edge-delivery-review-agent` | architecture, networking | Reviewing API Gateway, CloudFront, WAF, or edge delivery performance and security posture |
| `aws-event-driven-architecture-review-agent` | architecture | Reviewing or designing EventBridge, SNS, SQS, Kinesis, or event-driven integration patterns |

### Compute

| Agent | Domain(s) | Use when… |
|---|---|---|
| `aws-ec2-compute-operations-steward-agent` | compute | Managing EC2 fleet operations, AMIs, instance health, capacity, patching, or lifecycle events |
| `aws-ecs-fargate-platform-operator-agent` | compute | Running ECS/Fargate services, task definitions, service configuration, or platform-level operations |
| `aws-ecs-service-remediation-operator-agent` | compute | Remediating a stuck, failing, or misconfigured ECS service |
| `aws-eks-platform-operator-agent` | compute | Operating EKS clusters, node groups, add-ons, upgrades, or workload scheduling |
| `aws-serverless-production-readiness-agent` | compute | Reviewing Lambda or serverless workloads for production readiness (concurrency, cold starts, error handling) |
| `aws-serverless-rollout-corrector-agent` | compute | Correcting a failed or stalled serverless deployment or rollout |
| `aws-deployment-hotfix-operator-agent` | compute, devops-cicd | Applying an urgent hotfix to a running deployment with minimum blast radius |

### Data

| Agent | Domain(s) | Use when… |
|---|---|---|
| `aws-rds-aurora-performance-investigator-agent` | data | Investigating RDS or Aurora performance issues, slow queries, wait events, or parameter tuning |
| `aws-dynamodb-data-modeling-performance-review-agent` | data | Reviewing DynamoDB table design, access patterns, GSI/LSI choices, or throughput planning |
| `aws-dynamodb-patterns-agent` | data | Building or changing DynamoDB tables and access code: design, refactor, runtime contract, PITR/on-demand backup and new-table restore, vector indexes and embeddings, performance-test and feature plans, provisioning; review-only questions stay with the review agent |
| `aws-s3-data-perimeter-governor-agent` | data, security-iam | Auditing or enforcing S3 bucket policies, access points, and data perimeter controls |
| `aws-data-protection-backup-steward-agent` | data | Reviewing backup strategy, AWS Backup vaults, retention policies, and restore readiness |

### Security / IAM

| Agent | Domain(s) | Use when… |
|---|---|---|
| `aws-iam-least-privilege-review-agent` | security-iam | Reviewing IAM policies, roles, or permission boundaries for least-privilege compliance |
| `aws-bedrock-agent-security-governor-agent` | security-iam, ai-genai | Reviewing Bedrock agent or model access security, guardrails, and data handling posture |
| `aws-kms-secrets-lifecycle-steward-agent` | security-iam | Managing KMS key lifecycle, rotation policies, or Secrets Manager secret health |
| `aws-security-posture-hardening-agent` | security-iam | Hardening AWS account posture: GuardDuty, SecurityHub, Config rules, and remediation |
| `aws-compliance-evidence-mapper-agent` | security-iam | Mapping AWS controls to compliance frameworks (SOC 2, PCI, HIPAA, NIST) and gathering evidence |
| `aws-private-ca-issuer-review-agent` | pki | Reviewing AWS ACM Private CA issuer config for cert-manager: CA hierarchy, template ARN scope, IRSA permissions, CRL reachability, and cross-account RAM-shared CA |

### Cost

| Agent | Domain(s) | Use when… |
|---|---|---|
| `aws-cost-anomaly-watch-coordinator-agent` | cost | Investigating a cost anomaly, spike, or unexpected billing change |
| `aws-cost-optimization-governor-agent` | cost | Reviewing overall cost posture, rightsizing opportunities, Savings Plans, and waste elimination |

### DevOps / CI-CD

| Agent | Domain(s) | Use when… |
|---|---|---|
| `aws-ci-cd-release-engineer-agent` | devops-cicd | Designing or reviewing a CI/CD pipeline, release strategy, or deployment flow |
| `aws-pipeline-fix-operator-agent` | devops-cicd | Diagnosing and fixing a broken or stalled pipeline |
| `aws-iac-patch-executor-agent` | devops-cicd | Applying a targeted IaC patch (CloudFormation, CDK, Terraform) in a non-production context |
| `aws-iac-change-safety-review-agent` | devops-cicd | Reviewing an IaC change for safety, blast radius, and drift before apply |

### Operations

| Agent | Domain(s) | Use when… |
|---|---|---|
| `aws-observability-incident-responder-agent` | operations | Investigating an active or recent incident using CloudWatch, X-Ray, or structured runbooks |
| `aws-daily-operations-briefing-coordinator-agent` | operations | Generating a daily or weekly operational health briefing across accounts or services |
| `aws-ticket-triage-escalation-coordinator-agent` | operations | Triaging a support ticket or escalation, routing to the right team or remediation path |
| `aws-change-impact-advisor-agent` | operations | Assessing the blast radius and rollback options for a proposed change before execution |
| `aws-non-destructive-task-automation-advisor-agent` | operations | Advising on or reviewing non-destructive automation tasks (read-only ops, safe runbooks) |

### AI / GenAI

| Agent | Domain(s) | Use when… |
|---|---|---|
| `aws-generative-ai-developer-agent` | ai-genai | Building or reviewing a generative AI application on AWS Bedrock or SageMaker |
| `aws-agentcore-agent` | ai-genai | Working with AWS AgentCore: memory, sessions, gateway, or managed runtime |
| `aws-devops-agent-skill-designer-agent` | ai-genai, devops-cicd | Designing or reviewing DevOps agent skills, agentic pipelines, or agent-driven automation |

### Live-guard (ALWAYS requires human gate)

| Agent | Domain(s) | Use when… |
|---|---|---|
| `aws-live-deployment-guarded-operator-agent` | live-guard | Orchestrating a guarded live deployment that requires an explicit human approval gate |
| `aws-live-ecs-rollout-guard-agent` | live-guard | Executing or approving a guarded ECS rolling update to a production environment |
| `aws-live-iac-change-guard-agent` | live-guard | Applying an IaC change to production infrastructure with a mandatory human confirmation gate |
| `aws-live-pipeline-approval-operator-agent` | live-guard | Managing pipeline approval steps and human-in-the-loop gates for production releases |
| `aws-live-serverless-release-guard-agent` | live-guard | Releasing a Lambda or serverless update to production with a guarded approval workflow |

## Live-guard gate protocol

Before routing to any live-guard agent, surface all three and wait for explicit written confirmation:

1. **Blast-radius assessment** — what resources, environments, or users are affected if this goes wrong?
2. **Rollback path** — what is the tested rollback procedure and estimated recovery time?
3. **Explicit confirmation** — "I confirm I understand the blast radius and rollback path. Proceed."

If the user cannot supply a rollback path, recommend routing to `aws-change-impact-advisor-agent` first.

## Response shape

Every Maestro response begins with the routing header:
```
Route: <agent-name(s)>
Reason: <one sentence>
Mode: <single | parallel (N specialists) | live-guard-gate>
```
Followed by: dispatched specialist output (summarized), then recommended next actions.
