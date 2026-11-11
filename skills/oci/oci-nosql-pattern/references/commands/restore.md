# Restore dispatcher — new destination only

Select `restore/migrator.md` OR `restore/sdk.md`. Review exact source export and NEW
region/compartment/table; never drop/reset/overwrite the source or an existing
production table. Run the offline manifest checker only if a sanitized manifest is
provided; its pass is caller-evidence consistency, not cloud verification. Require
separate import/new-table/cutover approval, IAM/key access and validation. There is
no `--method pitr` alias or recovered point before an available verified export.
