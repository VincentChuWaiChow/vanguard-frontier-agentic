---
name: sap-joule-governance-adoption-review
description: "Review SAP Joule generative AI copilot governance and adoption risk: scope grounding and boundary configuration, data access boundaries per business role, prompt and response auditability, role-aware answer configuration, hallucination and over-trust risk, and change management and adoption readiness. Does not access live Joule sessions or production data."
allowed-tools: Read Grep Glob WebSearch WebFetch
metadata:
  author: "github: VincentChuWaiChow"
  version: "0.1.0"
  updated: "2026-06-19"
  category: ai
  lifecycle: experimental
---

# SAP Joule Governance and Adoption Review

## Purpose

Assess the governance posture and adoption risk of SAP Joule, the generative AI copilot embedded across SAP S/4HANA Cloud, SAP SuccessFactors, SAP Ariba, SAP BTP, and other SAP solutions. Review Joule scope grounding and boundary configuration: whether Joule's skill and action scope is restricted to approved business processes, whether unapproved or overly broad skill activations exist, and whether the grounding sources for Joule responses are controlled and classified. Assess data access boundaries per business role: whether Joule respects the underlying SAP role and authorization model when surfacing data, whether it can expose data beyond what the user is authorized to see in the underlying application, and whether cross-application data access via Joule is governed. Review prompt and response auditability: whether Joule interactions are logged, whether log access is role-restricted, and whether logs provide sufficient audit trail for compliance. Assess role-aware answer configuration: whether Joule delivers contextually appropriate answers based on the user's business role and whether misconfigured role context can cause over-disclosure of business data. Evaluate hallucination and over-trust risk: whether users are trained to validate Joule outputs, whether Joule is used in decision flows that require human verification, and whether outputs for regulated processes carry appropriate disclaimers. Review change management and adoption readiness: governance policies for Joule rollout, user training coverage, acceptable-use policy existence, and feedback and monitoring mechanisms. Does not access live Joule sessions, execute Joule skills, or process production business data.

## When to use

Use this skill when the user asks to:

- review Joule skill activation scope: which Joule skills and actions have been enabled for each connected SAP solution, whether skill scope aligns with approved business processes, and whether any enabled skills allow write-back or state-mutation actions without an additional confirmation step,
- assess Joule data access boundary governance: whether Joule enforces the underlying SAP authorization model (S/4HANA authorization objects, SuccessFactors RBP, Ariba permission groups) when responding to data queries, and whether cross-application Joule capabilities expose data across systems beyond what the user is authorized to see in each system individually,
- evaluate Joule prompt and response audit log configuration: whether Joule interaction logs are enabled, where logs are stored, what data is captured (prompt text, response text, user context, timestamp, session ID), who can access logs, and whether log retention meets the organization's audit and compliance requirements,
- review role-aware Joule answer configuration: whether Joule's context-setting for business role is correctly configured for each connected SAP solution, whether role misconfiguration can cause over-disclosure of sensitive business data (financial positions, headcount data, procurement prices), and whether role-aware configuration has been tested,
- assess hallucination and over-trust risk: whether Joule is used in decision flows (financial approvals, procurement decisions, hiring recommendations) without a human verification step, whether Joule outputs are presented with source attribution or confidence indicators, whether users are aware that Joule outputs require validation before actioning, and whether regulated workflows (financial close, legal, HR) have explicit policies on Joule output use,
- review change management and adoption readiness: whether an acceptable-use policy exists for Joule, whether user training covers Joule output validation, whether a rollout governance process defines which Joule capabilities are approved for which user populations, and whether a feedback and monitoring mechanism is in place to detect misuse or over-reliance,
- assess Joule integration with BTP AI services: whether Joule's underlying AI foundation model access is governed per the SAP AI Core resource group model, and whether Joule's grounding pipeline respects data classification and residency requirements.

## When not to use

- When the request requires accessing a live Joule session, executing Joule skills in a connected SAP system, or reviewing actual Joule interaction logs — this skill reviews configuration documentation, governance policies, training materials, and written descriptions only.
- When the request is about SAP AI Core or Generative AI Hub governance for custom AI pipelines built by the organization rather than the Joule copilot product — use `sap-ai-core-generative-ai-hub-governance`.
- When the request is about the underlying SAP S/4HANA or SuccessFactors authorization model in isolation, not in the context of Joule data access boundaries — use `sap-security-iam-grc-sod-review` or `sap-successfactors-hr-process-risk-review`.
- When the request is about BTP account-level governance unrelated to Joule capability activation — use `sap-btp-governance-review`.

## Does not touch live systems

This skill operates on user-provided Joule configuration documentation, skill activation lists, data access boundary descriptions, audit log configuration summaries, acceptable-use policy documents, training material descriptions, and written governance posture descriptions. It does not access any live Joule session, execute Joule skills, query Joule interaction logs, or connect to any underlying SAP system (S/4HANA, SuccessFactors, Ariba, BTP). All live inspection is out of scope.

## Lean operating rules

- Classify governance findings by domain before recommending. Every finding must be assigned to a governance domain (Scope Grounding / Data Access Boundary / Auditability / Role-Aware Configuration / Hallucination and Over-Trust Risk / Change Management) before remediation is proposed.
- Write-back and mutation skills require explicit confirmation steps. Any Joule skill or action that can create, update, or delete records in an underlying SAP system (purchase orders, employee data, financial postings) without an explicit user confirmation step is a `high` governance finding. Automated mutation without confirmation creates audit trail gaps and irreversible state changes.
- Joule must not bypass the underlying authorization model. If Joule can surface data that the user is not authorized to see in the underlying SAP application, this is a `critical` data access boundary finding. Joule's responses must be constrained by the user's authorization context in the connected system.
- Cross-application data access requires explicit governance. Joule capabilities that aggregate data across multiple connected SAP systems (e.g., surfacing both HR and Finance data in a single response) must be explicitly authorized per the user's cross-system access rights. Unreviewed cross-application data aggregation is a `high` finding.
- Audit logging of Joule interactions is required for compliance-sensitive deployments. Deployments of Joule in regulated business processes (financial close, HR decisions, procurement approvals) without Joule interaction logging are a `high` auditability finding.
- Hallucination risk in regulated workflows requires compensating controls. Joule use in financial, legal, or HR decision workflows without documented human verification requirements and an acceptable-use policy is a `high` over-trust risk finding.
- Acceptable-use policy is a prerequisite for production rollout. Deploying Joule to end users without a documented acceptable-use policy covering output validation, prohibited uses, and escalation paths is a `medium` change management finding.
- Evidence from official SAP Joule documentation, SAP BTP Trust Center, or user-provided governance artifacts takes precedence over inference.

## Evidence rules

Label all claims with one of:

- `documentation-based` — grounded in official SAP Joule, SAP BTP, or SAP AI documentation (help.sap.com or SAP Trust Center)
- `user-provided evidence` — Joule configuration documentation, skill activation lists, data access boundary descriptions, audit log configuration summaries, acceptable-use policy documents, or written governance posture descriptions provided by the user
- `inference` — derived reasoning not directly confirmed by official docs or user evidence; must always be labeled as such

## Live-environment rules

**This skill does not touch live systems.** There is no Joule session access, Joule skill execution, Joule interaction log query, or connection to any underlying SAP system (S/4HANA, SuccessFactors, Ariba, BTP) in this skill's execution path. Users must supply configuration documentation, governance policy descriptions, or written assessments of the Joule deployment for this skill to review.

## References

Load only when needed:

- [Workflow and output contract](references/workflow-and-output.md) — Joule governance domain taxonomy, risk classification, output format.
- [Safety checklist](references/safety-checklist.md) — non-negotiables, common Joule governance mistakes, when to push back.
- [Official sources](references/official-sources.md) — SAP Joule product documentation, SAP BTP AI governance, and SAP Trust Center references.

## Response minimum

Return, at minimum:

- **Problem classification**: governance domain(s) in scope (Scope Grounding / Data Access Boundary / Auditability / Role-Aware Configuration / Hallucination and Over-Trust Risk / Change Management) and specific finding(s) per domain.
- **Evidence used**: documentation-based / user-provided evidence / inference.
- **Risk level**: critical (Joule bypasses underlying authorization, confirmed data boundary breach) / high (write-back without confirmation, cross-application unreviewed aggregation, no audit log for regulated workflow, over-trust in regulated decision flow) / medium (missing acceptable-use policy, untested role-aware configuration, no feedback mechanism) / low (best practice deviation in adoption materials or monitoring).
- **Recommended action**: specific governance control per finding (restrict skill scope, enforce confirmation step for mutation actions, enable interaction logging, define role-aware context configuration, implement human verification requirement for regulated workflows, create acceptable-use policy, etc.).
- **Refusal / escalation triggers**: if live Joule session access, interaction log review, or skill execution is required to complete the governance assessment, state that clearly and do not proceed. If a data access boundary breach is confirmed, escalate to the security and data protection team before continuing.
- **Business impact**: regulatory exposure (GDPR, SOX, local data protection), audit trail gap, financial or HR decision error risk from hallucinated outputs, reputational risk from misuse of AI-generated content.
- **Next verification step**: validate governance control recommendations against current SAP Joule configuration documentation and the organization's AI governance policy before deploying changes to the Joule rollout.
