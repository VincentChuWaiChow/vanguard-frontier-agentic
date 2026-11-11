# <Service> operations

Operational coverage for <exact service name and API/mode>. Every section uses the
same heading and states one status. Cite evidence rows from
[the evidence ledger](evidence-ledger.md) as `[PREFIX-Exx]`.
`tests/validate-nosql-operations.py` enforces this structure.

Statuses:

- `VERIFIED`: the provider documents the capability; every statement below cites a
  VERIFIED row.
- `PARTIAL`: some of it is documented and some is not, or it is limited (by tier,
  mode, Region or API); say which part is which.
- `UNSUPPORTED`: provider documentation states the capability does not exist. Cite it.
- `UNVERIFIED`: no official page settled it in this pass. Say what was searched.
- `NOT APPLICABLE`: the capability does not apply to this service. Say why.

## Backup and restore

**Status:** <STATUS>

## Disaster recovery and replication

**Status:** <STATUS>

## Cost

**Status:** <STATUS>

Billing dimensions only (what is billed and how it scales). Never write unit prices
or currency amounts: they change by Region and over time. Link the official pricing
page and calculator.

## Monitoring

**Status:** <STATUS>

## Performance

**Status:** <STATUS>

## IAM

**Status:** <STATUS>

## Tagging

**Status:** <STATUS>

## Encryption

**Status:** <STATUS>

## Network isolation

**Status:** <STATUS>

## Audit logging

**Status:** <STATUS>
