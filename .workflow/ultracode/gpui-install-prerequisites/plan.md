# GPUI installation prerequisite closure
## Goal
Install missing local Linux GPUI build prerequisites and execute installation-blocked bootstrap/consumer checks from tasks 1.4/1.6. No backlog-wide implementation.
## Success criteria
Missing packages installed or concrete approval failure captured; locked Desktop builds/UI/engine contracts executed when possible; native Linux evidence run only on available actual backends; full-task gaps explicit.
## Current context
Ubuntu 24.04 WSL2, Rust stable bits 1.99.1 match numeric pin; numeric alias absent. HEAD recorded in state. Pre-existing untracked .kiro/kiro-agents/ preserved. Missing fontconfig/xkbcommon/wayland development metadata. Existing task 1.4 prior sudo denial is historical, not current permission evidence.
## Constraints
Parent owns package manager/Cargo shared state; no dependency upgrades or lock edits, secrets, Git staging/commit/push, remote CI, deployment or unrelated code work. Children behaviorally read-only, not independently sandbox-isolated. CLI version/auth only attest CLI, not IDE serving identity/profile/model/effort (UNVERIFIED). Native spawn available, inherited model/effort without overrides.
## Risk level
Medium: local system package installation and native build execution; no production systems.
## Approval gates
User explicitly requests installation-related tasks. Outside-workspace/system writes and network use require native escalation. Do not bypass denial or sudo credentials. Exact apt package list determined from locked source and distro metadata. No unattended service/account/repository configuration.
## Mode
Delegated. Ultracode sole workflow owner; Codex Agent Teams routing. Two read-only sidecars; integration stays parent-owned.
## Work packets
01-scope: task/dependency completion and verification command review, read-only.
02-native-review: existing native/UI graders and installed-source prerequisites review, read-only.
03-install-verify: parent-owned installation, serialized build/checks, evidence.
## Eval contract
- Outcome: remove actual missing local prerequisites and execute blocked checks without claiming unavailable targets.
- Shared surfaces: system packages and Cargo cache serialized by parent; manifests/locks frozen.
- Required checks: dpkg/pkg-config; locked check/UI/engine tests; existing bootstrap/engine boundary graders; supported native smoke or explicit failure.
- Blocking conditions: approval denial, sudo unavailable, unsafe package plan, mismatched source/compiler, build defect beyond scoped repair, missing renderer/host.
- Handoff evidence: exact command/exit/logs, package inventory, source/lock hashes, backend observations, diff review.
## Integration policy
Accept evidence only after parent inspection; update task 1.4/1.6 evidence rather than check incomplete tasks. No unsupported platform/AT/performance claims. A minimal build repair only after concrete failure and scope review.
## Verification plan
Review apt simulation before installation; locked/offline-first Cargo on installed exact compiler; run narrow checks before native probes; no remote CI. Tests use caller-selected disposable evidence dirs.
## Completion criteria
Report PASS/MISS/UNVERIFIED, changes, unrun gates and remaining blockers. Installation alone is not feature completion.

## Approved bounded replan
The first repair was the compiler-proven UI trait import. The user explicitly approved one additional native-test repair after the parent stopped on the benchmark-only API defect. Total repair limit is now two, both consumed. No further repair or feature/dependency expansion is authorized. Native scope includes actual host display access via scoped escalation; all failures remain recorded.
