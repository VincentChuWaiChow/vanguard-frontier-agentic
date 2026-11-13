# Synthetic support example: an invoice export contains headings but no rows

> **EXPLICITLY SYNTHETIC.** The customer, product behavior, screenshots, records, and policy described below are fictional teaching inputs. They are not real evidence or actual customer data. No system was inspected, no settings were changed, no message was sent, and all proposed verification is **NOT RUN**.

## Fictional mini brief

The following hypothetical materials are supplied to a support agent:

- **S1 — Customer report:** “I need the March invoice report for our monthly review, but the download contains only the column headings.” No deadline or service commitment is supplied.
- **S2 — Mock screenshot:** The reporting screen has April selected and shows zero invoice rows. The screenshot identifies a single synthetic account, but no actual product version or environment is provided.
- **S3 — Mock record summary:** Three invoice records for that same synthetic account have March issue dates. The supplied summary lists no April invoices. It is a limited fixture, not an inventory of a real customer's data.
- **S4 — Fictional product guidance:** The invoice export contains the rows selected by the reporting screen's current date filter, using invoice issue dates. The export is restricted to the signed-in account. A filter with no matches produces column headings and no data rows.

These inputs suggest that the selected month explains the empty file. They do not establish that a real customer changed the filter, received the expected export, or confirmed resolution.

## Ticket: help the customer obtain the intended monthly invoice report

- **Type:** Support case
- **Readiness assessment (not Jira status):** Within this hypothetical scenario, **Ready for triage**. The available materials support a next support step; resolution is unverified. This is not Ready for implementation.
- **Owner:** TBD
- **Customer / target:** One synthetic account. Real identity, product version, environment, and case identifier: Unknown.
- **Service SLA / response commitment:** Unknown — none supplied.

### Customer need and impact

The customer needs the March invoice rows for a monthly business review. They report that the downloaded file is empty apart from headings.

- **Reported symptom:** An export contains no invoice rows, according to S1.
- **Supported by the fictional materials:** S2 shows April selected with zero rows; S3 lists March records; S4 says the export follows the selected filter.
- **Expected result:** The customer expects March invoices. The product contract says the downloaded rows should match the selected date filter and signed-in account. The selected month currently differs from the customer's stated need.
- **Reported business impact:** The customer cannot yet prepare the intended report.
- **Confirmed wider impact:** None established. There is no evidence of missing invoices, a service outage, another affected account, or financial loss.
- **Urgency / priority:** Unknown. A monthly review is a purpose, not proof of an urgent deadline or a severity level.

### Evidence, classification, and unknowns

| Source | What it supports | What it does not prove |
| --- | --- | --- |
| S1 | The requested month and reported difficulty | Root cause, scope, or successful resolution |
| S2 | A different month is selected in the supplied mock view | What the customer does after receiving guidance |
| S3 | The fictional fixture has records for the requested month | A complete real account inventory |
| S4 | The fictional export follows the current filter and account boundary | That the actual export implementation has been tested |

**Working classification:** Configuration / filter selection is the supported explanation within this fixture. No product defect has been established. Customer validation remains pending.

**Material unknowns:** The real target and version, the result after selecting the intended month, whether the downloaded rows match the visible rows, and customer confirmation. Owners for follow-up: TBD.

### Proposed support step and safe verification

1. Ask the customer to select the intended March reporting period and check whether the expected invoices appear on the screen.
2. If the visible rows are correct, ask them to export and check whether the downloaded file contains the same rows for their account.
3. Record the result using row counts and sanitized observations. If it still fails, obtain the product version, safe reproduction steps, selected period, and a sanitized discrepancy description before deciding on an engineering escalation.

**Execution state:** Proposed only. The author has not changed the filter or run the export.

**Workaround evidence:** Choosing the intended filter follows S4 and may resolve the request. It has not been tested in this scenario, so it is proposed guidance, not a proven workaround.

### Customer update — DRAFT, NOT SENT

> The supplied screenshot shows April selected, while you need the March report. The export uses the reporting screen's selected dates, which may explain why the file contains only headings. Please select the intended March period and check whether the expected invoices appear before exporting again. Let us know whether the file then contains the same rows. If it does not, share the row counts and a sanitized description of the difference so we can investigate further.

No response deadline, fix date, or promise of resolution is invented in this draft.

### Acceptance and expected verification

| AC ID | Observable customer outcome | Verification method and expected evidence | Actual result |
| --- | --- | --- | --- |
| AC-01 | The customer can display the intended March invoice rows using the agreed reporting period. | Customer validation or a permitted safe reproduction records the selected period and matching row count. | NOT RUN |
| AC-02 | The downloaded file contains the expected rows from that view and only the intended account's data. | Compare the filtered view and export using sanitized counts and appropriate scoped record checks; record target/version if available. | NOT RUN |
| AC-03 | The customer confirms that the report meets the original need, or a documented applicable closure rule is met. | Record the actual confirmation. If relying on a closure rule, supply that rule and evidence of its conditions; none is currently known. | NOT RUN |

### Escalation decision and optional engineering handoff

**Current disposition:** Support guidance and customer validation are the next steps. A bug ticket is not warranted merely because the initial report uses the word “empty.” No linked bug exists in this example.

If the customer demonstrates that the correct invoice rows are visible but the export omits them, reassess the classification. A separately drafted bug should carry the confirmed target/version, sanitized reproduction, expected contract from S4, observed discrepancy, impact, and stable acceptance criteria. The support case stays linked to that investigation without claiming a defect is fixed or the customer's request is resolved.

### Current handoff

- **Next owner:** TBD.
- **Delivered:** A support case draft and an unsent customer update.
- **Resolution evidence:** None yet; all verification is NOT RUN.
- **Customer confirmation:** Pending.
- **Case closure:** Not justified by the supplied materials. No closure exception or service policy was supplied.
- **Live actions:** None performed.
