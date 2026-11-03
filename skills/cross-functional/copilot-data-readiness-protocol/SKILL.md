---
name: copilot-data-readiness-protocol
description: "Use this skill before enabling Microsoft 365 Copilot for any user population. Runs an oversharing assessment, applies sensitivity labels and DLP controls, validates permissions baseline, and confirms a secure data foundation exists. Orchestrates m365-copilot-readiness-governance-agent as primary, m365-identity-zero-trust-agent for identity-layer controls, and copilot-governance-maestro-agent for cross-pillar sign-off. Hard refusal: Microsoft 365 Copilot must not be enabled for any user until the oversharing baseline is established and remediated. Never requests credentials, tenant IDs, or customer data. Production-impacting steps escalate to the relevant data owner or compliance team."
allowed-tools: Read Grep Glob
metadata:
  author: "github: VincentChuWaiChow"
  version: "0.1.0"
  updated: "2026-06-16"
  category: ai
  lifecycle: experimental
---

# Copilot Data Readiness Protocol

## Purpose
Microsoft 365 Copilot surfaces content that users already have permission to
access. If that content is over-shared, unlabelled, or poorly governed, Copilot
amplifies the exposure. This skill defines the mandatory readiness sequence that
must complete before Copilot is enabled: oversharing baseline, sensitivity
labelling, DLP guardrails, and a permissions baseline sign-off. It exists so
organisations never enable Copilot into an ungoverned data estate. It does not
configure production systems; it produces structured recommendations that data
owners and security teams confirm.

## When to use
- An organisation is planning to enable Microsoft 365 Copilot for the first time.
- Copilot is expanding to new user groups and the data estate for those users
  has not been assessed.
- A periodic data readiness review is due for an active Copilot deployment.
- An oversharing alert has been raised by Microsoft Purview DSPM and must be
  assessed before Copilot access continues.

## When NOT to use
- Copilot is already enabled and this is a routine operational review with no
  new user groups — use the periodic governance review workflow instead.
- The matter is a data breach or security incident — route to the incident
  response protocol.
- The organisation has not yet deployed Microsoft Purview or SharePoint Advanced
  Management — prerequisites are not met; this protocol cannot proceed.
- You need live tenant configuration changes — escalate to the data owner;
  this protocol is recommendation-only.

## Participating agents
- `m365-copilot-readiness-governance-agent` (primary — oversharing assessment, sensitivity labels, DLP, permissions baseline)
- `m365-identity-zero-trust-agent` (secondary — Conditional Access, identity-layer least privilege)
- `copilot-governance-maestro-agent` (cross-pillar sign-off and final readiness gate)

## Inputs required
- Tenant-level SharePoint sharing settings and EEEU (Everyone except external
  users) status
- Microsoft Purview DSPM Data Risk Assessment output (or permission to run one)
- SharePoint Advanced Management (SAM) Content Management Assessment output
- List of user groups proposed for Copilot enablement
- Existing sensitivity label taxonomy and publishing scope
- DLP policy inventory for Microsoft 365 workloads

## Evidence required
- Microsoft Purview is deployed and DSPM is active
- SharePoint Advanced Management is licensed and configured
- Sensitivity labels are published to users in scope
- Purview Audit is enabled for Copilot interaction activity
- A data owner is identified for each high-risk site

## Workflow

1. **Oversharing baseline** — Run (or review) the Microsoft Purview DSPM Data
   Risk Assessment and SAM Content Management Assessment. Identify sites with
   EEEU access, broken permission inheritance, ownerless sites, inactive sites,
   and sensitive content exposed broadly.
2. **Interim protections** — Before remediating, apply SAM Restricted Content
   Discovery (RCD) to exclude high-risk sites from Copilot discovery. Apply
   Purview DLP policies scoped to the Copilot location to exclude sensitive
   content from grounding.
3. **Sensitivity label review** — Confirm that sensitivity labels are published
   to all users in scope. Identify content lacking labels on high-risk sites.
   Draft labelling recommendations for data owners.
4. **Permissions remediation** — For sites flagged as high-risk: remove
   excessive access and company-wide sharing links, correct broken inheritance,
   rescope sharing to approved users or groups, confirm site ownership.
5. **Gate 1 — Oversharing remediation sign-off** — Confirm that all
   critical-risk sites have been remediated or have accepted interim protections.
   Do not proceed to Copilot enablement without this sign-off from the data
   owner and copilot-governance-maestro-agent.
6. **Identity layer check** — Invoke m365-identity-zero-trust-agent to confirm
   that Conditional Access policies for Copilot users enforce MFA and device
   compliance. Flag any gaps.
7. **DLP guardrails** — Confirm DLP policies cover the Copilot location and
   are actively monitoring sensitive content interactions. Validate Insider Risk
   Management (IRM) adaptive protection is configured for high-risk users.
8. **Gate 2 — Permissions baseline** — Produce a permissions baseline document:
   confirmed label coverage rate, EEEU status, high-risk site count (open vs.
   remediated), DLP policy coverage. This is the minimum viable baseline
   before Copilot enablement.
9. **Copilot readiness recommendation** — copilot-governance-maestro-agent
   produces a go / conditional-go / no-go recommendation per user group.
   Conditional-go requires time-bound remediation commitments.
10. **Human confirmation** — Route the readiness recommendation to the data
    owner, security team, and Copilot programme owner for final sign-off.
    This protocol never enables Copilot autonomously.

## Decision gates

| Gate | Condition | Action |
|---|---|---|
| Oversharing baseline | Purview DSPM or SAM assessment not run | Block enablement; run assessment first |
| Critical-risk sites unmediated | High-risk sites without interim protection or remediation | Block enablement; apply RCD and DLP interim controls |
| EEEU active at tenant level | Everyone except external users enabled tenant-wide | Escalate to SharePoint admin; recommend disabling before enablement |
| Sensitivity label gap | >20% of content on high-risk sites lacks labels | Require labelling sprint before go-live |
| Identity layer gap | MFA or device compliance not enforced for Copilot users | Invoke m365-identity-zero-trust-agent; hold enablement |

## Refusal triggers
- Hard refusal: do not recommend Copilot enablement without an oversharing
  baseline. This is unconditional.
- Stop if the Purview DSPM assessment has not been run — do not substitute
  manual estimates for a real assessment.
- Stop if credentials, tenant IDs, or customer PII are requested to perform
  this assessment — refuse and escalate.
- Stop if the data owner cannot be identified for a critical-risk site — flag
  as a blocker; do not proceed past Gate 1.

## Handoff rules
- All handoffs carry: tenant_scope, skill_id, skill_version, invoked_by,
  oversharing_risk_level, label_coverage_rate, gate_status, open_questions,
  do_not_do_list.
- Gate 1 sign-off must include the data owner name/role, the date, and a
  statement of residual risk accepted.
- copilot-governance-maestro-agent's readiness recommendation is the final
  cross-pillar artefact; it must not be bypassed.

## KPIs
- Oversharing risk sites: critical / high / medium count (pre- vs. post-remediation)
- Sensitivity label coverage rate (% of files on high-risk sites labelled)
- EEEU status: tenant-level and site-level
- DLP policy coverage for Copilot location
- Time from assessment to enablement (days)

## References
- https://learn.microsoft.com/microsoft-365/copilot/secure-govern-copilot-foundational-deployment-guidance
- https://learn.microsoft.com/microsoft-365/copilot/configure-secure-governed-data-foundation-microsoft-365-copilot
- https://learn.microsoft.com/sharepoint/get-ready-copilot-sharepoint-advanced-management
- https://learn.microsoft.com/purview/data-security-posture-management-learn-about
- https://learn.microsoft.com/security/zero-trust/copilots/zero-trust-microsoft-365-copilot
