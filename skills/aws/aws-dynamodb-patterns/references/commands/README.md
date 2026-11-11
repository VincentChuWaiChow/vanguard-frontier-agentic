# Command routes

Use `/aws-dynamodb-patterns <command>` when the host exposes the skill, or the
real local selector: `python3 scripts/select_mode.py <command>`. Host discovery is
separate; this package does not install/register a slash command.

| Command | Read only this route |
|---|---|
| help | [Help](help.md) |
| select | [Selected features](select.md) |
| design | [Table/access-pattern design](design.md) |
| refactor | [Existing-model/code updates](refactor.md) |
| backup | [Recovery protection](backup.md), shared target gate and chosen method only |
| restore | [New-table restore](restore.md), shared target gate and chosen method only |
| audit | [Audit](audit.md), then only [offline](audit/offline.md) or [live](audit/live.md) |
| performance-test | [Performance testing](performance-test.md) |
| feature-plan | [Feature change plan](feature-plan.md) |

The index is navigation, not a combined workflow. Do not read sibling commands.
`audit` defaults to offline; `--mode live` selects authorized metadata collection.
Old `--evidence repo` / `--evidence live-metadata` remain compatible alternatives;
do not mix --mode and --evidence. Live mode is not approval or a completed audit.

AWS command companions are separate from agent workflow commands:
[Core AWS CLI](aws-cli.md), [embedding integrations CLI](embedding.md).
Load them only when an actual selected operation needs their command examples.

The selector prints selected paths only, performs no file/cloud work and persists
no activation state. Resolve paths from the skill directory. Performance/feature
plans retain their own execution, budget and authorization gates.
Backup/restore default to PITR and the pitr feature route; --method on-demand is
explicit. Both remain nonexecuting selectors, not backup jobs or restore results.
