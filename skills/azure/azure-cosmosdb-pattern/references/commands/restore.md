# Restore dispatcher

Read [recovery target](recovery-target.md) and **only** the selected procedure:

| Method | Procedure | Destination boundary |
|---|---|---|
| continuous-new-account (default) | [Continuous new account](restore/continuous-new-account.md) | New account; no overwrite or adoption |
| continuous-deleted-resource | [Deleted resource](restore/continuous-deleted-resource.md) | Existing continuous account, deleted DB/container only; target name absent |
| periodic-support | [Periodic support request](restore/periodic-support.md) | Support-assisted new account; no fake self-service PITR |

No SDK/CLI operation is authorized by selection. Require exact source/time/region/
destination approval, encrypted recovery dependencies and validation before any
separately approved cutover. A successful long-running operation isn't acceptance.
