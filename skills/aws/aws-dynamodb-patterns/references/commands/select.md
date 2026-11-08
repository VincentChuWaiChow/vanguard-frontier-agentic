# select

Read this command and only the selected feature references returned by the helper.
Do not load audit, performance or feature-change procedures merely to answer a
focused design/explanation question.

```text
/aws-dynamodb-patterns select --features encryption,vectors
```

```bash
python3 scripts/select_mode.py select --features encryption,vectors
```

Selection is per invocation, not an AWS toggle or persisted host preference.
Default is core only. The helper rejects unknown/empty IDs and deduplicates shared
guides. Get feature IDs through help; do not expand an unknown selection to all.

Explain/design only the requested feature scope using existing application
language/SDK and the entrypoint safety rules. Load a linked CLI companion only
when command generation is actually requested. Report material dependencies or
missing evidence without silently expanding scope or enabling anything.
