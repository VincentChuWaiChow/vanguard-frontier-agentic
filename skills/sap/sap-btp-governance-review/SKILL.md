---
name: sap-btp-governance-review
description: "Review SAP BTP account model governance: global account, directories, subaccounts, entitlements, quotas, environments (Cloud Foundry, Kyma), role collections, and trust configuration. Flags entitlement sprawl, over-provisioning, missing guardrails, and trust misconfiguration. Does not touch live systems."
allowed-tools: Read Grep Glob WebSearch WebFetch
metadata:
  author: "github: VincentChuWaiChow"
  version: "0.1.0"
  updated: "2026-06-19"
  category: platform
  lifecycle: experimental
---

# SAP BTP Governance Review

## Purpose

Assess the governance posture of an SAP Business Technology Platform (BTP) account model. Evaluate the structural soundness of the global account hierarchy, directory layout, subaccount design, entitlement and quota allocation, environment provisioning (Cloud Foundry, Kyma, multi-environment), role collection design, trust configuration, and platform access controls. Surface entitlement sprawl, over-provisioning risks, missing administrative guardrails, and identity trust misconfigurations. Does not connect to or mutate any live BTP system.

## When to use

Use this skill when the user asks to:

- review or audit the BTP global account structure, directory hierarchy, or subaccount layout for governance compliance,
- assess entitlement and quota allocations across directories and subaccounts for sprawl or over-provisioning,
- evaluate Cloud Foundry org/space design or Kyma cluster provisioning decisions against BTP governance best practices,
- review role collection assignments, platform user access, and identity provider trust configuration for a BTP account,
- flag missing administrative guardrails such as absent emergency access procedures, unreviewed trust configurations, or uncontrolled quota consumption,
- plan a BTP landing zone or account model for a new project or migration,
- evaluate BTP entitlement management practices against the principle of least privilege.

## When not to use

- When the user needs live inspection of a BTP cockpit, running service instances, or deployed applications — this skill accepts only user-provided descriptions, exports, or architecture documents.
- When the request concerns application-level security within a service deployed on BTP — use `sap-security-iam-grc-sod-review` for identity and access management within services.
- When the request is specifically about SAP Integration Suite topology — use `sap-integration-suite-review`.
- When the request is about ABAP extensibility or clean core — use `sap-clean-core-debt-review`.

## Does not touch live systems

This skill operates on user-provided descriptions, architecture diagrams, exported account structure documents, BTP cockpit screenshots, or configuration summaries. It does not connect to the BTP cockpit, issue BTP REST APIs, invoke the SAP BTP CLI, read from any BTP global account, or access Cloud Foundry or Kyma APIs. All live inspection is out of scope.

## Lean operating rules

- Classify governance issues before recommending. Every finding must be classified by governance domain (account structure, entitlements, environments, roles, trust) before remediation is proposed.
- Apply least-privilege to entitlements. Entitlements should be assigned at the lowest scope that satisfies the business need — global-account-level entitlements are not automatically correct.
- Directories are optional but powerful. BTP directories can carry entitlements, role collections, and managed subaccount lifecycle; the absence of a directory structure in a large BTP estate is itself a governance risk.
- Subaccounts are the isolation boundary. Each subaccount has its own environment instances, trust configurations, and service bindings. Cross-subaccount blast radius is a key risk dimension.
- Role collections must be mapped to identity provider groups. Direct user assignments to role collections (instead of IdP group mappings) are an operational risk — they bypass centralized access lifecycle management.
- Cloud Foundry quotas are separate from BTP entitlements. CF org and space quotas must be reviewed alongside BTP-level entitlements to identify over-provisioning.
- Trust to external IdPs requires explicit review. Every trust configuration (to SAP Identity Authentication Service, Azure AD, Okta, etc.) that allows login to BTP subaccounts is a governance boundary. Unused or unconfigured default trusts are risk vectors.
- Evidence from user-provided artifacts or official SAP BTP documentation takes precedence over inference.
- Load only the reference needed for the component under review.

## Evidence rules

Label all claims with one of:

- `documentation-based` — grounded in SAP BTP Help Portal documentation, BTP account model guides, or official SAP entitlement docs
- `user-provided evidence` — BTP cockpit exports, architecture documents, screenshots, or configuration summaries provided by the user
- `inference` — derived reasoning not directly confirmed by official docs or user evidence

## Live-environment rules

**This skill does not touch live systems.** There is no BTP API call, BTP CLI invocation, cockpit session, Cloud Foundry CLI command, or Kyma kubectl command in this skill's execution path. Users must supply account structure exports, role collection lists, entitlement summaries, or written descriptions of their BTP account model for this skill to review.

## References

Load only when needed:

- [Workflow and output contract](references/workflow-and-output.md) — governance classification taxonomy, finding severity, output format.
- [Safety checklist](references/safety-checklist.md) — non-negotiables, common mistakes, when to push back.
- [Official sources](references/official-sources.md) — SAP BTP account model, entitlements, environments, role collections, trust configuration docs.

## Response minimum

Return, at minimum:

- **Problem classification**: governance domain(s) affected (account structure / entitlements / environments / roles / trust) and specific finding(s).
- **Evidence used**: documentation-based / user-provided evidence / inference.
- **Risk level**: critical (security or compliance boundary violation) / high (operational risk, sprawl, or over-provisioning) / medium (governance gap) / low (best practice deviation).
- **Recommended action**: specific governance remediation per finding (restructure, entitlement reduction, role collection clean-up, trust reconfiguration, or guardrail addition).
- **Refusal / escalation triggers**: if live BTP cockpit access or BTP API calls are required to complete the review, state that clearly and do not proceed.
- **Business impact**: data isolation risk, cost overrun, compliance gap, or operational incident blast radius.
- **Next verification step**: confirm the recommended changes against the current BTP account model before applying.
