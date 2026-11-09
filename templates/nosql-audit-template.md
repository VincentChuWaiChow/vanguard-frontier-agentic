# NoSQL audit template

Every NoSQL skill in `tests/validate-nosql-operations.py` ships a lazy-loaded audit
in three files. `tests/validate-nosql-audit.py` enforces the shape and the command
safety rules. The audit checks a deployment against the skill's operations
reference; it does not restate that reference.

| File | Loaded | Purpose |
| --- | --- | --- |
| `references/commands/audit.md` | from `SKILL.md`, only when an audit is asked for | router: scope, mode choice, finding format |
| `references/commands/audit/offline.md` | from the router, only for offline mode | checks on repository and sanitized evidence, local commands only |
| `references/commands/audit/live.md` | from the router, only for live mode | read-only control-plane commands, run by an authorized operator |

`SKILL.md` links the router and never a variant:

```markdown
- [Audit](references/commands/audit.md) — offline or live audit against the operations reference; load only when an audit is requested.
```

## Router: `references/commands/audit.md`

```markdown
# audit

Load only for an audit request. Offline is the default. After reading this file,
load **only** the selected variant: [offline](audit/offline.md) or
[live](audit/live.md). Do not read both.

## Scope

- Name the service (and API or mode), the resources and the selected sections of
  [the operations reference](../operations.md). Unselected sections are out of scope.
- An audit is a technical risk review, not a compliance attestation.

## Findings

For each finding give: severity, section, evidence (file:line, or sanitized command
output with its retrieval time), the expectation from the operations reference with
its evidence ID, impact, the smallest proposed fix and how to verify it.

- PASS only for a check actually performed. Missing evidence is UNVERIFIED.
- An absent repository setting does not prove the live setting is off.
- Propose fixes; never apply them as part of the audit.
```

## Offline: `references/commands/audit/offline.md`

No cloud CLI, network call, credential or customer data. Inputs are the repository
(IaC, application config, client code) and sanitized exports supplied by the user.
Commands are local only: `rg`, `grep`, `jq`, `find`, `git`, and `terraform show` on
a saved plan.

~~~markdown
# audit --mode offline

<inputs and the no-network rule>

## Backup and restore

- <what to check in IaC/config, with the attribute or property names>

```bash
rg -n '<attribute>' --glob '*.tf' --glob '*.json' --glob '*.bicep'
```

## Disaster recovery and replication
...the remaining operations headings, same order...
## Audit logging
~~~

## Live: `references/commands/audit/live.md`

Commands are examples for an operator holding read-only authorization; the skill
does not run them. Each section has a command block that cites a VERIFIED ledger
row for the CLI reference page, or a `**No live command:**` line giving the reason
(for example, billing data lives outside the resource API).

~~~markdown
# audit --mode live

## Before any command

- Confirm the approved principal, account/subscription/project, Region and exact
  resource. Stop on any mismatch; never switch target to make a command work.
- Use a read-only role. Write output to a private directory (`umask 077`). Never
  print keys, connection strings or customer data.

```bash
<identity check, e.g. aws sts get-caller-identity>
```

## Backup and restore

```bash
<read-only describe/get/list/show command> [PREFIX-E9xx]
```

## Disaster recovery and replication
...the remaining operations headings, same order...
## Audit logging
~~~

Command rules the gate enforces in live files:

- every cloud CLI call uses a read verb (`describe`, `get`, `list`, `show`, ...) and
  no mutating verb (`create`, `update`, `delete`, `modify`, `restore`, `export`, ...);
- no secret-reading words (`keys`, `secret`, `password`, `credentials`, `token`) and
  no `--force` or `--yes`;
- `curl` is GET only, with no request body;
- no data-plane shells (`mongosh`, `cqlsh`, ...) in either variant.

Ledger rows for audit commands use IDs `<PREFIX>-E901` upward and cite the
provider's official CLI or API reference page with an exact quote.
