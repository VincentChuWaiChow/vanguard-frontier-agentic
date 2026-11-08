# backup

Plan/inspect recovery protection or create a separately approved native backup.
PITR is the default; selecting this command performs no AWS operation. Load the
[shared target gate](recovery-target.md), selected feature references and **only**
the chosen method. Do not load the restore sibling or both methods.

| Method | Read only this route | Purpose |
|---|---|---|
| pitr (default) | [PITR](backup/pitr.md) | Inspect actual window/status; propose reviewed retention/enablement |
| on-demand | [Native backup](backup/on-demand.md) | Approved asynchronous backup; verify exact ARN/source/readiness |

```text
/aws-dynamodb-patterns backup --method pitr
/aws-dynamodb-patterns backup --method on-demand --features pitr,encryption
```

Real local selector: `python3 scripts/select_mode.py backup --method pitr`.
Both backup/restore default to pitr scope, not core. Their selected method and
shared target gate supply recovery guidance without loading replica procedures.
Native on-demand backup does not require PITR enabled; the default pitr feature
label selects recovery scope, not a service prerequisite for that method.

Return protection intent vs observed status/window, business RPO/RTO gaps, backup
identity/readiness and next approved recovery drill. A backup is not sufficient
BCDR evidence until restoration and application acceptance are demonstrated.
No backup deletion, retention reset, vault policy or recovery test is implied.
