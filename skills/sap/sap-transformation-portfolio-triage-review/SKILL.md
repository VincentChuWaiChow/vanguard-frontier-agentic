---
name: sap-transformation-portfolio-triage-review
description: "Advisory triage of an SAP transformation program portfolio: workstream prioritization, dependency and risk mapping, value vs effort classification, readiness gating across S/4HANA, BTP, integration, and security workstreams, and RAID log assessment. Does not touch or mutate any live system."
allowed-tools: Read Grep Glob WebSearch WebFetch
metadata:
  author: "github: VincentChuWaiChow"
  version: "0.1.0"
  updated: "2026-06-19"
  category: architecture
  lifecycle: experimental
---

# SAP Transformation Portfolio Triage Review

## Purpose

Assess the portfolio-level posture of an SAP transformation program. Evaluate workstream prioritization, cross-workstream dependency mapping, risk and risk interdependency exposure, value vs effort classification, readiness gating criteria across S/4HANA conversion or greenfield, BTP platform adoption, integration modernization, and security workstreams, and the completeness of the RAID log (Risks, Assumptions, Issues, Decisions). Surface workstreams that are underplanned, sequenced incorrectly, missing readiness gates, or carrying unresolved cross-workstream dependencies that threaten the program. Does not connect to or mutate any live system, project tool, or SAP environment.

## When to use

Use this skill when the user asks to:

- triage an SAP transformation program portfolio to identify which workstreams are highest risk or blocking others,
- review workstream sequencing and dependency ordering across S/4HANA, BTP, integration, and security tracks,
- classify workstreams by value vs effort to support reprioritization or executive escalation,
- assess readiness gating criteria across program workstreams and flag workstreams entering execution without sufficient readiness evidence,
- review a RAID log for completeness, unresolved risks, unowned issues, or stale assumptions,
- identify dependency chains that create single points of failure in the transformation timeline,
- evaluate whether the transformation program structure aligns with SAP Activate methodology phases (Discover, Prepare, Explore, Realize, Deploy, Run),
- support a program health check, steering committee review, or transformation audit preparation.

## When not to use

- When the request requires live access to a project management tool, SAP Solution Manager, SAP Cloud ALM, or Jira — this skill accepts only user-provided descriptions, exports, or architecture documents.
- When the request concerns technical clean core compliance of specific ABAP objects — use `sap-clean-core-debt-review`.
- When the request is specifically about BTP account model governance — use `sap-btp-governance-review`.
- When the request concerns RISE with SAP contract obligations, SLA commitments, or vendor risk — use `sap-rise-sla-vendor-risk-review`.
- When the request is about SAP licensing cost optimization — use `sap-license-btp-consumption-finops-review`.

## Does not touch live systems

This skill operates on user-provided transformation program artifacts: project plans, workstream charters, RAID logs, dependency maps, architecture decision records, steering committee decks, or written descriptions of the program structure. It does not connect to SAP Cloud ALM, SAP Solution Manager, any SAP backend system, any cloud project management platform, or any live environment. All live inspection is out of scope.

## Lean operating rules

- Classify workstreams before prioritizing. Every workstream must be classified by type (S/4HANA conversion, BTP platform, integration modernization, data migration, security and compliance, change management, infrastructure) before sequencing recommendations are made.
- Use SAP Activate as the reference methodology. Workstream readiness gates correspond to SAP Activate phase exits (Prepare exit, Explore exit, Realize exit). Workstreams that enter a phase without completing the prior phase exit checklist carry structural program risk.
- Dependency direction matters. Distinguish predecessor dependencies (A must complete before B starts) from shared-outcome dependencies (A and B must reach a milestone together). Both create timeline risk but through different mechanisms.
- Value vs effort classification uses the program's own evidence. Do not invent value scores. Derive from the user's provided program artifacts — business case, OKRs, or value driver trees.
- RAID items must be owned and dated. Unowned risks, undated assumptions, and issues without a resolution path are program governance deficiencies regardless of their content.
- Readiness gating is a binary gate, not a score. A workstream is either ready to proceed to the next phase or it is not. Partial readiness is not readiness.
- S/4HANA conversion sequencing is non-negotiable. Fit-to-standard workshops must precede solution design which must precede configuration. Gap analysis must precede extensibility decisions. Reversing this order is a rework risk.
- Integration workstreams are cross-cutting. Integration dependencies affect every other workstream. Flag integration workstreams that are underplanned relative to the S/4HANA and BTP workstreams they serve.
- Evidence from user-provided artifacts or official SAP methodology documentation takes precedence over inference.
- Load only the reference needed for the workstream domain under review.

## Evidence rules

Label all claims with one of:

- `documentation-based` — grounded in SAP Activate methodology documentation, SAP Transformation Navigator guidance, or official SAP program management resources
- `user-provided evidence` — project plans, RAID logs, workstream charters, dependency maps, or program decks provided by the user
- `inference` — derived reasoning not directly confirmed by official documentation or user evidence

## Live-environment rules

**This skill does not touch live systems.** There is no API call, project tool connection, SAP system access, SAP Cloud ALM query, or Jira integration in this skill's execution path. Users must supply program artifacts, workstream descriptions, RAID log exports, or written descriptions of the transformation program structure for this skill to review.

## References

Load only when needed:

- [Workflow and output contract](references/workflow-and-output.md) — workstream classification taxonomy, risk severity, RAID assessment criteria, output format.
- [Safety checklist](references/safety-checklist.md) — non-negotiables, common sequencing mistakes, when to push back.
- [Official sources](references/official-sources.md) — SAP Activate methodology, SAP Transformation Navigator, SAP Cloud ALM program management documentation.

## Response minimum

Return, at minimum:

- **Problem classification**: workstream domain(s) affected (S/4HANA conversion / BTP platform / integration / security / data migration / change management) and specific triage finding(s).
- **Evidence used**: documentation-based / user-provided evidence / inference.
- **Risk level**: critical (dependency or sequencing failure that threatens the program delivery date or regulatory commitment) / high (workstream without readiness gate or with unresolved blocking dependency) / medium (RAID gap or workstream prioritization imbalance) / low (best practice deviation in program structure).
- **Recommended action**: specific triage remediation per finding (resequence, add readiness gate, escalate unowned RAID item, restructure dependency, deprioritize low-value workstream).
- **Refusal / escalation triggers**: if live project tool access or SAP Cloud ALM data is required to complete the triage, state that clearly and do not proceed.
- **Business impact**: program delivery risk, regulatory deadline exposure, budget overrun risk, or scope creep from underplanned workstreams.
- **Next verification step**: validate workstream sequencing and dependency map against the program's current project plan before communicating to stakeholders.
