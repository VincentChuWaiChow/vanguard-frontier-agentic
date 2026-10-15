# Packet 03: integrated refutation

Owner: independent read-only native agent, requested gpt-6.1-sol medium.
Inputs: approved requirements/design/tasks; full tracked diff from 4f5278f6;
new models/mod_catalog.rs; current implementation tests and generated catalog.

Goal: find actionable correctness/security/coverage defects against R18.1–20,
R8.7, R16.3, R17.5 and pending tasks. Inspect actual production loading,
validation, hashing/reload retention, primary App route, all metadata display,
removed-ID behavior, read-only scope, generator/validator and P3 workload.

Read-only. No edits, Cargo/Node/gates, network, Git writes, secrets or subagents.
Ignore unrelated .kiro/kiro-agents/. Parent owns gates and task-status updates.
Return ranked actionable findings with file:line and a minimal reproduction;
if none, say so and distinguish unverified behavior. Review is evidence, not
completion certification. Scope ends after one integrated pass.
