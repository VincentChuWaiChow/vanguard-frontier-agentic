# Evidence pack worksheet

Records each coverage or impact claim with the numbers behind it, so a reader can see what the evidence supports and what it cannot. The assurance analyst fills one row per claim from supplied evidence only; a claim without a population is not recorded.

## How to use

- Fill every cell from supplied evidence; never paste secrets, tokens, request payloads or customer identifiers.
- Always give the population source and the caller-supplied window and as_of; do not infer them.
- Write `Unknown` for any cell the evidence does not settle, with the smallest item that would settle it.
- Keep blocked requests out of object denominators; list excepted objects separately.
- Keep the decision ID and name the evidence owner as a role.

## Claims

Decision ID: `<KYV-DEC-nnnn>`.

| Claim | Numerator | Denominator | Population source | Window / as_of | Evidence lineage | Blind spots | What it cannot support |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `<claim>` | `<count and rule>` | `<count and rule>` | `<inventory and capture time>` | `<window> / <as_of>` | `<EV-n per number>` | `<cause or Unknown>` | `<claims not supported>` |
| 100 of 120 invoice-api objects have a fresh result | 100 objects with a result timestamp inside the window | 120 in-scope objects | inventory export, 2026-11-20 | 7 days / 2026-11-20T00:00:00Z | EV-1 (inventory), EV-2 (reports) | 20 objects with no fresh result, cause Unknown; 8 excepted objects | a compliance percentage; enforcement; the estate-wide state |

## Counted separately

| Item | Count | Why not in the denominator | Evidence |
| --- | --- | --- | --- |
| `<excepted objects / blocked creates / excluded>` | `<n>` | `<reason>` | `<EV-n>` |
| Blocked creates | 6 | denied requests never became objects | EV-4 runtime-metric |

## Evidence status

| Item | Type | Locator | Captured at | Omits |
| --- | --- | --- | --- | --- |
| `<EV-n>` | `<population-report / runtime-metric / installed-config>` | `<export or query>` | `<timestamp>` | `<what it does not show>` |
| EV-2 | population-report | PolicyReports, 7-day window | 2026-11-20T00:00:00Z | per-result breakdown for 92 objects; the 20 objects |
