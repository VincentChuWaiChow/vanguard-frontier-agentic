# Result 02: Native/prerequisite source review
## Summary
Existing UI callback asserts actual state after two clicks; native bootstrap observes actual backend handle, scene and Linux renderer, with finite child timeout and new evidence files. No proven API/compiler defect from source review alone.
## Evidence
Desktop tests/ui.rs lines 9-32; src/lib.rs lines 25-30; tests/native_smoke.rs lines 38-46, 83-126, 207-228, 254.
## Handoff
Build exact graph before UI/native runs; absent renderer/host cannot pass. Source cache was absent during sidecar review; exact minimal package closure was UNVERIFIED by the agent.
## Files changed
None by agent.
## Decisions
Parent restored exact cache and verified package requirements; exclude optional WebKit/ALSA. Upstream broad tool list is not the minimal consumer graph.
## Risks
Software renderer startup may exceed 10s and honestly fail; probe is not compositor pixels/AT/IME/clipboard or descendant ownership proof.
## Verification run
Read-only source and package presence inspection only; no tests/Cargo/install/network.
## Open questions
Actual compilation/UI/backend results pending parent execution.
