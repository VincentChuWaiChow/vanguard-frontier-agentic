# help

Read this command only; do not preload other command or feature files. Show the
available commands and selectable feature IDs, then stop. No source scan, AWS
identity check, audit, benchmark or feature change is required for help.

```bash
python3 scripts/select_mode.py help
```

Commands: select, design, refactor, backup, restore, audit, performance-test,
feature-plan. Backup/restore default to --method pitr and pitr feature scope;
use --method on-demand for native named backups. Audit supports
`--mode offline` (default) or `--mode live`. Features and positive plan caps are
documented by each command's `--help`; the JSON help lists the feature IDs.

Recognize `/aws-dynamodb-patterns <command>` as the skill invocation form when
host discovery supports it; `$aws-dynamodb-patterns` or plain-language invocation
may be used by other hosts. Do not claim a host command was registered here.
The real selector also accepts the slash prefix as an explicit argument:

```bash
python3 scripts/select_mode.py /aws-dynamodb-patterns help
```

Unknown commands/features fail closed. No command means help; never fall through
from help into environment inspection. Use [the index](README.md) only for
navigation when requested, not to load every route.
