---
name: sap-security-iam-grc-sod-review
description: "Review SAP identity and access management posture: Cloud Identity Services (IAS/IPS), Authorization and Trust Management (XSUAA), role collections, GRC Access Control, and Segregation of Duties. Flags SoD conflicts, excessive privilege, orphaned accounts, and trust misconfigurations. Does not touch live systems."
allowed-tools: Read Grep Glob WebSearch WebFetch
metadata:
  author: "github: VincentChuWaiChow"
  version: "0.1.0"
  updated: "2026-06-19"
  category: security
  lifecycle: experimental
---

# SAP Security IAM GRC and SoD Review

## Purpose

Assess the identity and access management posture of SAP landscapes spanning cloud and on-premise systems. Review SAP Identity Authentication Service (IAS) and Identity Provisioning Service (IPS) configuration for provisioning correctness, group mapping, and lifecycle management. Review Authorization and Trust Management (XSUAA) scopes, role templates, and role collection assignments for privilege excess and least-privilege compliance. Assess GRC Access Control ruleset quality, SoD conflict classification, and mitigation control effectiveness. Flag SoD conflicts, excessive or accumulated privilege, orphaned accounts, misconfigured identity provider trust, and access lifecycle gaps. Does not connect to or mutate any live SAP system.

## When to use

Use this skill when the user asks to:

- review SAP Identity Authentication Service (IAS) configuration for application assignments, corporate identity provider federation, risk-based authentication policies, and MFA enforcement,
- assess SAP Identity Provisioning Service (IPS) connector configuration for user and group synchronization correctness, provisioning rule quality, and transformation script safety,
- evaluate XSUAA (Authorization and Trust Management Service) application security descriptor (xs-security.json) design: scope definitions, role template construction, role collection granularity, and least-privilege compliance,
- audit role collection assignments for a BTP subaccount — identifying over-permissive built-in role collections, direct user assignments versus IdP group mappings, and privilege accumulation across multiple role collections,
- review SAP GRC Access Control ruleset configuration: SoD function and permission entries, risk classification (critical/high/medium/low), and whether the ruleset covers key business processes (FI, MM, SD, HR),
- assess SoD conflict findings from GRC Access Control, SAP Access Violation Management by Pathlock, or equivalent tooling — classify conflicts, evaluate mitigating controls, and flag unmitigated critical conflicts,
- identify excessive privilege patterns: access to incompatible transaction pairs, accumulated authorizations from multiple roles, or overly broad authorization object values (e.g., `*` for company code or plant),
- review identity trust misconfigurations: IAS corporate IdP federation without MFA enforcement, XSUAA trust to external IdPs with insufficient attribute mapping, or IPS provisioning to production systems without approval workflows.

## When not to use

- When the user needs live inspection of a GRC Access Control system, BTP XSUAA tenant, or IAS admin console — this skill accepts only user-provided configuration exports, SoD conflict reports, role lists, or written descriptions.
- When the request is about BTP account-level structural governance (entitlements, directories, environments) rather than IAM — use `sap-btp-governance-review`.
- When the request is about ABAP authorization objects directly in an on-premise SAP system without a GRC or cloud IAM layer — this skill focuses on cloud IAM (IAS/IPS/XSUAA) and GRC; direct ABAP auth object review is out of scope.
- When the request is about integration security (iFlow OAuth, API proxy policies) — use `sap-integration-suite-review`.

## Does not touch live systems

This skill operates on user-provided configuration exports, SoD conflict reports from GRC or equivalent tools, xs-security.json files, role collection exports, IAS application exports, IPS connector configuration descriptions, or written descriptions of the IAM landscape. It does not connect to any IAS tenant, IPS admin console, XSUAA service instance, GRC system, BTP subaccount, or on-premise SAP system. All live inspection is out of scope.

## Lean operating rules

- Classify IAM findings by domain before recommending. Every finding must be assigned to a domain (IAS / IPS / XSUAA / GRC / SoD) before a remediation path is proposed.
- Least privilege is the baseline. Any role, scope, or role collection that grants access beyond what the business function requires is a finding. Broad authorization object values (`*` for organizational levels) are always flagged.
- SoD conflicts must be classified by risk level. Not all SoD conflicts carry the same risk. Critical SoD conflicts (e.g., Create Vendor + Approve Payment, Create PO + Approve PO) must be treated as `critical` findings requiring either role redesign or compensating controls with documented approval.
- Unmitigated critical SoD conflicts block sign-off. An environment with unmitigated critical SoD conflicts (no approved compensating control on record) is not compliant with SOX, ISO 27001, or SAP GRC best practices. Escalate before proceeding.
- IPS provisioning to production requires approval workflow. Any IPS connector that provisions users or group memberships to a production target system without an approval step in the provisioning flow is a `high` finding.
- IAS MFA enforcement is required for privileged users. Administrator and privileged application users authenticated via IAS without MFA enforcement (risk-based authentication minimum) are a `high` finding.
- XSUAA trust to external IdPs must carry attribute validation. XSUAA application trust configured to accept tokens from an external IdP without validating custom attributes (e.g., groups, costCenter) is a privilege escalation vector.
- Orphaned accounts are a `high` finding. Accounts in IAS, IPS target systems, or BTP role collection assignments that belong to users who have left the organization or changed roles are `high` IAM lifecycle findings.
- Evidence from user-provided artifacts or official SAP security documentation takes precedence over inference.
- Load only the reference needed for the IAM domain under review.

## Evidence rules

Label all claims with one of:

- `documentation-based` — grounded in SAP IAS, IPS, XSUAA, GRC Access Control, or SAP security best practice documentation
- `user-provided evidence` — SoD conflict reports, role lists, xs-security.json files, IAS exports, IPS connector configurations, or written descriptions provided by the user
- `inference` — derived reasoning not directly confirmed by official docs or user evidence

## Live-environment rules

**This skill does not touch live systems.** There is no IAS API call, IPS REST API invocation, XSUAA service binding access, GRC system RFC call, BTP cockpit session, or on-premise SAP system connection in this skill's execution path. Users must supply configuration exports, SoD conflict reports, role collection lists, or written descriptions of their IAM landscape for this skill to review.

## References

Load only when needed:

- [Workflow and output contract](references/workflow-and-output.md) — IAM finding taxonomy, SoD conflict classification, severity assignment, output format.
- [Safety checklist](references/safety-checklist.md) — non-negotiables, common IAM review mistakes, when to push back.
- [Official sources](references/official-sources.md) — SAP IAS, IPS, XSUAA, GRC Access Control, and SoD documentation.

## Response minimum

Return, at minimum:

- **Problem classification**: IAM domain(s) affected (IAS / IPS / XSUAA / GRC / SoD) and specific finding(s) per domain.
- **Evidence used**: documentation-based / user-provided evidence / inference.
- **Risk level**: critical (SoD conflict, privilege escalation path, unmitigated access violation) / high (excessive privilege, orphaned account, missing MFA, unapproved provisioning) / medium (governance gap, missing lifecycle review, weak attribute mapping) / low (best practice deviation).
- **Recommended action**: specific remediation per finding (role redesign, compensating control documentation, IPS approval workflow addition, IAS MFA policy enforcement, XSUAA scope reduction, etc.).
- **Refusal / escalation triggers**: if unmitigated critical SoD conflicts are found, escalate to GRC / audit team before any further access approval or system change is authorized.
- **Business impact**: compliance risk (SOX, ISO 27001, internal audit), fraud risk, data breach risk, or access lifecycle debt.
- **Next verification step**: validate remediation recommendations against current role assignments and GRC ruleset before implementing access changes.
