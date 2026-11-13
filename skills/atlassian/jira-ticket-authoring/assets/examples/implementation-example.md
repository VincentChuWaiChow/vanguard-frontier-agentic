# Fictional worked example: checklist status filter

This example is an authoring fixture. The application, data, and brief below are fictional. No repository, Jira project, or running implementation has been inspected.

## Synthetic input brief

Add a status filter to a checklist that currently displays all items. Offer **All**, **Open**, and **Completed**. The initial selection is **All**. Open shows incomplete items; Completed shows completed items. Preserve the original relative order of visible items. Switching filters must not change an item's text, completion flag, or stored position. When a selected filter has no matches, show “No matching items.” Persisting the filter selection and changing how items are completed are outside this request.

Use this supplied fixture in its listed order:

| ID | Text | Completed |
|---|---|---|
| item-1 | Pack charger | false |
| item-2 | Book train | true |
| item-3 | Pack coat | false |

## Completed ticket draft

### Add a status filter to the checklist

**Content pattern:** Implementation.
**Configured Jira work type:** Unresolved; no Jira project was supplied.
**Authoring readiness:** Needs clarification — the behavior specification is complete, but the actual repository and execution target are unresolved.
**Jira workflow status:** Unspecified.

### Outcome and reason

Let checklist users view all, open, or completed items so they can focus on the items relevant to them. Filtering changes which items are visible while preserving the checklist data.

### Facts and open decisions

- **Verified source facts:** The synthetic input brief specifies the three options, initial selection, membership rules, order preservation, nonmutation requirement, and empty-state copy. These are verified requirements of this fixture, not observations of an implemented application.
- **Reported facts:** The brief says the current checklist displays all items; no running application was inspected.
- **Assumptions:** None needed for the specified filtering behavior.
- **Questions:** The target repository, existing interface conventions, and configured Jira work type are not supplied. They must be established before taking action in a real project.

### Scope

- **In scope:** Filter selection, visible membership, relative ordering, unchanged underlying item data, and the stated empty state.
- **Out of scope:** Persisting the selected filter, changing completion behavior, and reordering stored items.
- **Implementation context:** No repository or existing test command is asserted. Apply the actual project's conventions when that context is provided.

### Acceptance criteria

1. **AC-01 — Initial view:** With the supplied fixture, the initial selected option is All, and the visible IDs are item-1, item-2, item-3 in that order.
2. **AC-02 — Membership and order:** Selecting Open shows item-1 and item-3 in that order. Selecting Completed shows only item-2. Selecting All again restores all three visible items in their original order.
3. **AC-03 — Negative case: no data mutation:** After switching from All to Open to Completed to All, each item's text, completion flag, and stored position equal their values before the switches.
4. **AC-04 — Empty match set:** With a fixture containing only item-1 and item-3, selecting Completed shows “No matching items.” and no item rows. Switching to All shows both items in their original order.

### Criterion-to-verification mapping

These are planned checks. They do not assert that tests exist or have passed.

| Criterion | Planned check and fixture | Pass condition | Evidence to retain | Evidence status |
|---|---|---|---|---|
| AC-01 | Initialize the checklist with the three-item fixture; inspect selection and visible IDs. | All selected; visible IDs exactly item-1, item-2, item-3. | Observed selection and ordered IDs. | Not run |
| AC-02 | Select Open, Completed, and All in sequence using the three-item fixture. | Each visible ID sequence matches AC-02 exactly. | Selection-to-visible-ID results. | Not run |
| AC-03 | Snapshot the three items before the selection sequence and compare the underlying data afterward. | Text, completion flags, and stored positions are unchanged for every item. | Before/after comparison result. | Not run |
| AC-04 | Initialize with item-1 and item-3 only; select Completed, then All. | Exact empty-state copy with zero rows, followed by item-1 and item-3 in order. | Empty-state observation and restored visible IDs. | Not run |

### Dependencies and handoff

No cross-ticket dependency is specified in the synthetic brief. Before real execution, identify the target repository and inspect its interface and verification conventions. This example does not identify or create a parent issue, dependency link, or Jira status transition.

### Completion evidence

- **Changes delivered:** None; this is a ticket draft.
- **Verification results:** Not run for AC-01 through AC-04.
- **Implementation artifacts:** None.
- **Remaining uncertainty:** Real project context is absent. No claim is made about implementation, test success, deployment, or external publication.
