---
name: sap-release-change-collision-review
description: "Review SAP release and change management for collision risk: transport sequencing and dependency analysis, overtake and overwrite risk between parallel tracks, parallel project conflict detection, retrofit strategy and dual landscape governance, downgrade protection assessment, and ChaRM and SAP Cloud ALM change governance completeness. Advisory only — does not import transports, modify transport requests, or mutate change management configuration."
allowed-tools: Read Grep Glob WebSearch WebFetch
metadata:
  author: "github: VincentChuWaiChow"
  version: "0.1.0"
  updated: "2026-06-19"
  category: delivery
  lifecycle: experimental
---

# SAP Release and Change Collision Review

## Purpose

Assess release and change management practices in SAP landscapes for collision, overwrite, and sequencing risk. Review transport sequencing and dependency analysis: whether transport requests are correctly ordered across the landscape route, whether object-level dependencies between transports are documented, and whether sequencing gaps create overwrite risk at target system import time. Evaluate overtake and overwrite risk between parallel transport tracks: whether transports from independent workstreams touch the same repository objects, whether import queue sequencing is controlled, and whether last-write-wins object collisions are identified and mitigated before reaching production. Assess parallel project conflict risk: whether concurrent SAP projects share development or quality systems, whether object ownership is enforced across parallel tracks, and whether a collision detection process exists to surface conflicts before production import. Review retrofit strategy and dual landscape governance: whether changes released to production in a maintenance landscape are retrofitted correctly to the development landscape of the next release, whether the retrofit transport chain is documented and sequenced, and whether dual landscape object integrity is maintained. Assess downgrade protection: whether transport downgrade protection is configured at the system and client level, whether transports moving against the import direction are detected and blocked, and whether emergency correction transports bypass downgrade protection. Evaluate ChaRM (Change and Release Management in SAP Solution Manager) and SAP Cloud ALM change governance completeness: change record coverage, approval workflow configuration, transport-to-change-record traceability, urgent correction and emergency change procedures. This is an advisory collision and governance review. It does not import transports, modify transport requests, change import queue order, or mutate ChaRM or Cloud ALM change management configuration.

## When to use

Use this skill when the user asks to:

- review transport sequencing plans across a multi-track SAP landscape (development → quality → production, or dual landscape with maintenance and main development lines) and identify dependency and overtake risk,
- assess whether transport requests from parallel SAP projects or workstreams share repository objects and create overwrite collision risk at quality or production import,
- evaluate the collision detection process: whether object-level transport analysis is performed before production import, whether STMS transport logs and TADIR object usage are reviewed, and whether conflicts are surfaced and resolved before release,
- review the retrofit strategy in a dual landscape (parallel maintenance and main development landscape): whether retrofit transports are created, sequenced, and imported correctly, and whether object integrity between the two landscape lines is maintained,
- assess downgrade protection configuration: whether the transport management system is configured with downgrade protection at system and client level, whether any emergency or urgent correction transports bypass downgrade protection, and what the process is for approved bypasses,
- review ChaRM change governance: whether all transports are linked to approved change records, whether the approval workflow is configured and enforced, whether urgent correction and emergency change procedures include appropriate authorization controls and post-change review,
- evaluate SAP Cloud ALM change management configuration as a replacement or supplement to ChaRM: change record structure, approval workflow, transport integration, and audit trail completeness,
- identify release management practices that create unacceptable collision or overwrite risk for production system stability.

## When not to use

- When the user wants to import transports, modify import queue order, or execute change management operations in a live system — this skill does not assist with execution. A separate guarded live-execution agent would be required for transport operations.
- When the user needs a testing strategy and quality gate review — use `sap-testing-quality-gate-review`.
- When the user needs a data migration or cutover readiness assessment — use `sap-data-migration-cutover-readiness`.
- When the user needs go-live hypercare incident command governance review — use `sap-hypercare-incident-commander-review`.
- When the user needs SAP Cloud ALM operations monitoring or SRE incident governance — use `sap-cloud-alm-sre-incident-review`.

## Advisory-only boundary — explicitly stated

**This skill does not import transports, modify transport requests, alter import queue sequences, or mutate ChaRM or Cloud ALM change management configuration.** It does not:

- connect to any SAP system via RFC, STMS, or ABAP transport tools,
- view, approve, or execute transport import queues in any SAP landscape,
- create, modify, or close change records in ChaRM or SAP Cloud ALM change management,
- access transport logs, TADIR object tables, or system change options from live SAP systems,
- authorize or imply authorization for production transport releases or emergency change approvals.

All collision and governance assessment is based on transport sequencing documentation, change record descriptions, landscape architecture diagrams, import queue plans, and written descriptions of the change management process supplied by the user. Transport execution is a risk-critical mutating operation that requires a separate, explicitly guarded execution skill with approval gates and rollback controls.

## Lean operating rules

- Overtake risk requires object-level evidence. A claim that "transports are sequenced correctly" is not sufficient to dismiss overwrite risk. Object-level analysis confirming that no two parallel transports modify the same repository object is required to classify overtake risk as managed.
- Downgrade protection must be active in production. Any SAP production client without downgrade protection configured is a release governance gap regardless of the perceived need. Bypasses must be documented, authorized, and time-limited.
- Retrofit completeness is a dual landscape integrity requirement. In dual landscapes, every productive change must be retrofitted to the main development line before the next main release. A retrofit backlog is a collision-in-waiting for the next development cycle.
- Change record coverage must be 100% for production transports. Any transport imported into production without a linked, approved change record is an ungoverned change. Emergency changes are not exempt — they require post-change documentation and approval review.
- Parallel project collision detection must be proactive, not reactive. Discovering object conflicts at production import is too late. Collision detection must be embedded into the quality system import gate and resolved before production release.
- Do not fabricate transport object lists, import queue states, or ChaRM change record counts. Only classify findings the user has provided from their actual transport documentation or change management reports.
- Evidence from official SAP ChaRM, STMS, and Cloud ALM documentation takes precedence over memory or training data.

## Evidence rules

Label all claims with one of:

- `documentation-based` — grounded in SAP ChaRM documentation, STMS transport management documentation, SAP Cloud ALM change management documentation, or SAP Help Portal
- `user-provided evidence` — transport sequencing plans, import queue descriptions, collision analysis outputs, ChaRM change record summaries, landscape architecture diagrams, retrofit transport lists, or downgrade protection configuration descriptions provided by the user
- `inference` — derived reasoning not directly confirmed by official docs or user evidence; must always be labeled as such

## Live-environment rules

**This skill does not touch live systems.** There is no RFC connection, STMS access, ABAP transport tool execution, ChaRM API call, or Cloud ALM change management mutation in this skill's execution path. Users must supply transport sequencing plans, import queue documentation, collision analysis outputs, ChaRM or Cloud ALM change governance descriptions, and landscape architecture documentation for this skill to review.

## References

Load only when needed:

- [Workflow and output contract](references/workflow-and-output.md) — collision risk taxonomy, release governance assessment dimensions, retrofit assessment criteria, downgrade protection classification, output format.
- [Safety checklist](references/safety-checklist.md) — non-negotiables, common change collision review mistakes, advisory boundary enforcement, when to push back.
- [Official sources](references/official-sources.md) — SAP STMS transport management, ChaRM change governance, SAP Cloud ALM change management, retrofit procedure, downgrade protection configuration.

## Response minimum

Return, at minimum:

- **Problem classification**: landscape topology (single vs. dual landscape, number of parallel tracks), change governance tool (ChaRM / Cloud ALM change management / no formal tool), and collision risk surface (transport overtake / object overwrite / retrofit gap / downgrade protection gap / change record coverage gap).
- **Evidence used**: documentation-based / user-provided evidence / inference.
- **Risk level**: production-blocking / high / medium / low per collision risk dimension.
- **Recommended action**: specific collision mitigation or governance control per finding (implement object-level transport analysis before production import, configure downgrade protection, establish retrofit governance cadence, enforce change record coverage, define parallel track collision detection gate, etc.), grounded in SAP STMS, ChaRM, or Cloud ALM documentation.
- **Refusal / escalation triggers**: refuse to assess production import safety if no transport sequencing documentation or object-level collision analysis has been provided; do not provide transport execution guidance under any circumstances; escalate immediately if a production transport without a linked change record is identified.
- **Business impact**: production system instability risk from object overwrites, audit and compliance risk from ungoverned production changes, dual landscape integrity risk from retrofit backlog, release velocity risk from unresolved collision blocking cycles.
- **Next verification step**: which transport sequencing evidence, object-level analysis output, or change governance documentation must be provided before the collision risk can be assessed as managed.
