# Requirements Document

## Introduction

Add a native, cross-platform GPUI Kit desktop interface to Vanguard Frontier
Agentic, reusing the domain and execution capabilities currently in `vfa-tui`
through a shared engine. Keep the terminal and headless interfaces working. The
desktop is an additional presentation layer, not a port of business rules into
widget callbacks or a replacement for existing Node.js/Python scripts.

Status: REVISION REQUESTED; CONTEXT7-GROUNDED UPDATE FOR REVIEW. The user requested
revision rather than design approval, then explicitly requested Context7 MCP.
No functional-scope changes beyond that grounding request have been specified.
No design, task breakdown or application implementation has been approved.
This document follows requirements engineering;
the next gate is [adversarial analysis](requirements-analysis.md), then explicit
user approval, then design, then tasks. Evidence is in [research.md](research.md);
acceptance scenarios are in [evals.md](evals.md).

### Proposed alpha boundary requiring approval

- Required native targets: Linux `x86_64-unknown-linux-gnu` on Ubuntu 24.04 with
  both X11 and Wayland sessions; macOS `aarch64-apple-darwin` on macOS 15+;
  Windows `x86_64-pc-windows-msvc` on Windows 10+. These are intended acceptance
  targets, not claims of current support. OS versions describe runtime floors;
  the build toolchain/prerequisites require separate validation.
- Use a local VFA checkout selected by the operator. Catalog browsing is offline;
  script execution requires trusted checkout content and available local helper
  runtimes. No automatic downloads, repository cloning or package installation.
- Alpha includes catalog browsing, search/filtering, relationships/integrity,
  populated workspace coverage/drift/policy views, validation, confirmed export,
  audit/report workflows and local preferences. These capabilities may be
  delivered incrementally, but placeholder screens do not satisfy them.
- Preserve current TUI/headless commands and supported target behavior. Extra
  desktop CPU architectures, web/mobile, a bundled standalone catalog/runtime,
  auto-update, telemetry and extension hosting are outside this alpha.
- Performance thresholds below are proposed product acceptance budgets, not
  measured baseline results or upstream performance promises.
- Concurrency boundary: operators must not edit execution inputs or destinations
  concurrently with a confirmed write. Preflight revalidation detects observed
  changes; it is not an atomic filesystem transaction or protection against a
  malicious same-user process. Such changes during execution invalidate preview
  assurance and require explicit partial/unknown-effects reporting.

## Glossary

- **Desktop**: the new native GPUI Kit application.
- **Shared Engine**: UI-independent domain, query, orchestration and infrastructure
  behavior reused by the Desktop and existing terminal/headless adapters.
- **Catalog Checkout**: explicitly selected VFA repository containing the expected
  package identity, catalogs and helper scripts; selection alone grants no trust
  to execute its code.
- **Managed Workspace**: a separately registered destination repository inspected
  for installed harness assets; never implicitly the Catalog Checkout.
- **Snapshot**: loaded data plus source identity, generation and content hashes.
- **Operation**: a gate, export, scan, integrity check or report job with unique ID.
- **Preview**: non-mutating proposal bound to exact selection, canonical source and
  destination identities, options and relevant source/destination fingerprints.
- **Confirmed Write**: an explicit operator authorization of one current Preview;
  not a reusable permission for another operation or changed inputs.
- **Execution Manifest**: canonical checkout identity, package/lockfile hashes,
  invoked checkout scripts and their local code/config dependencies, plus resolved
  helper executable identities. Its exact inventory must be documented by design;
  an unestablished inventory cannot carry trust forward to another execution.
- **Terminal Status**: one of `SUCCEEDED`, `FAILED`, `CANCELLED`, `TIMED_OUT` or
  `INTERRUPTED`; `QUEUED`, `RUNNING` and `CANCELLING` are intermediate states.
- **Data Status**: `LOADING`, `CURRENT`, `EMPTY`, `PARTIAL`, `STALE`, `UNAVAILABLE`
  or `NOT_CONFIGURED`. `EMPTY` means a successful query with zero records, never a
  failed load. A policy finding may be empty while its data status is `CURRENT`.
- **Audit Persistence Status**: `PENDING`, `DURABLE`, `FAILED` or `NOT_RECORDED`,
  independent of observed execution outcome. `DURABLE` requires a successful
  database commit acknowledgment, not merely enqueueing or flushing commands;
  it is not a guarantee against hardware failure or same-user tampering.
- **AT**: platform assistive technology, checked separately from a headless
  accessibility-tree test.
- **Baseline**: commit `6dda992a3dedd845731d795aaaed20d49346f57c` on which the
  feature branch was based; current evidence does not prove it passes every gate.

## User Roles

- **Catalog operator** discovers agents, skills, roles, providers, MCP references
  and rules without reading raw catalog JSON.
- **Workspace maintainer** inspects coverage/drift/policy and previews approved
  changes to a selected local destination.
- **Reviewer** examines validation, integrity, operation evidence and reports
  without granting new execution or write permissions.
- **Repository maintainer** preserves existing adapters, scripts and delivery
  behavior while adding the native application.

These describe workflows, not invented authentication/RBAC roles. The alpha uses
the local user's filesystem permissions and does not elevate privileges.

## Requirements

### Requirement 1: Explicit source and workspace context

**User Story:** As a catalog operator, I want to select a checkout independently of
the application's launch directory, so that I always know which data I am using.

#### Acceptance Criteria

1. **R1.1** WHEN the Desktop opens without a valid saved Catalog Checkout, THEN the Desktop SHALL offer checkout selection instead of assuming the process working directory is the source.
2. **R1.2** WHEN an operator selects a valid Catalog Checkout, THEN the Desktop SHALL display its canonical identity alongside the loaded data context.
3. **R1.3** IF the selected source is missing, unreadable or lacks the expected repository markers, THEN the Desktop SHALL show `UNAVAILABLE` with a corrective selection action.
4. **R1.4** WHEN the operator changes Catalog Checkout, THEN the Desktop SHALL invalidate selections, previews and data from the previous source context.
5. **R1.5** IF a background presentation/data result belongs to a replaced source context or generation, THEN the Desktop SHALL discard it from the active view without discarding the original Operation's execution/audit evidence.
6. **R1.6** WHILE an Operation is in any nonterminal state including `QUEUED` and `CANCELLING`, THE Desktop SHALL prevent source replacement until that Operation is terminal.

### Requirement 2: Complete catalog discovery and details

**User Story:** As a catalog operator, I want the existing catalog entities and
their relationships visible, so that the native interface preserves discovery value.

#### Acceptance Criteria

1. **R2.1** WHEN valid catalog files are loaded, THEN the Desktop SHALL provide browse views for agents, skills, install roles, providers, MCP references and rules.
2. **R2.2** WHEN an entity is selected, THEN the Desktop SHALL show its catalog metadata and referenced relationships using the same schema semantics as the Shared Engine.
3. **R2.3** IF optional entity metadata is absent, THEN the Desktop SHALL identify it as unavailable rather than inventing values.
4. **R2.4** IF one catalog file fails to load, THEN the Desktop SHALL identify the failed collection without hiding independently valid collections.
5. **R2.5** WHEN a collection loads successfully with no records, THEN the Desktop SHALL show `EMPTY` rather than an error or fabricated content.
6. **R2.6** WHEN counts, provider choices or harness choices are displayed, THEN the Desktop SHALL derive them from the active source data rather than hardcoded catalog totals.

### Requirement 3: Search, filtering and stable selection

**User Story:** As a catalog operator, I want predictable search and combined
filters, so that I can find the same assets through either interface.

#### Acceptance Criteria

1. **R3.1** WHEN an agent search query is submitted, THEN the Desktop SHALL use the Shared Engine's matching and stable ranking semantics.
2. **R3.2** WHEN query, provider and harness filters are active together, THEN the Desktop SHALL return their intersection.
3. **R3.3** WHEN a query is empty, THEN the Desktop SHALL show all records permitted by the remaining filters.
4. **R3.4** WHEN a query has no matches, THEN the Desktop SHALL keep the query editable with an explicit no-match state.
5. **R3.5** WHEN an operator searches another catalog collection, THEN the Desktop SHALL match its ID, display name and available summary with a documented deterministic ordering.
6. **R3.6** IF filtering or reload removes the selected entity, THEN the Desktop SHALL clear the selection rather than silently substituting a different record at the same list index.
7. **R3.7** WHEN valid Unicode text is entered, THEN the Desktop SHALL preserve that text through search and detail presentation.

### Requirement 4: Honest reload and stale-data handling

**User Story:** As a reviewer, I want data freshness and failures visible, so that a
cached snapshot is not mistaken for current evidence.

#### Acceptance Criteria

1. **R4.1** WHEN catalog changes are successfully reloaded, THEN the Desktop SHALL publish a new snapshot with its source generation and observed load time.
2. **R4.2** IF reloading a previously valid file fails, THEN the Desktop SHALL retain its last valid data with `STALE` status and the load error.
3. **R4.3** IF a previously loaded file is removed or becomes unreadable, THEN the Desktop SHALL show that collection as `STALE` or `UNAVAILABLE`, never as current empty data.
4. **R4.4** WHEN an unchanged file notification arrives, THEN the Desktop SHALL preserve the existing snapshot generation for that unchanged content.
5. **R4.5** WHEN the operator requests manual refresh, THEN the Desktop SHALL re-evaluate the selected source without requiring application restart.
6. **R4.6** WHILE a snapshot needed by a Preview is stale or unavailable, THE Desktop SHALL prevent confirmation of that Preview.

### Requirement 5: Relationships and integrity evidence

**User Story:** As a reviewer, I want actual dependency and integrity findings,
so that graphical presentation does not replace evidence with decorative status.

#### Acceptance Criteria

1. **R5.1** WHEN dependency inspection is requested, THEN the Desktop SHALL present engine-derived agent/skill relationships with identifiable source and target records.
2. **R5.2** IF a referenced dependency is missing or cyclic, THEN the Desktop SHALL surface the corresponding engine finding.
3. **R5.3** WHEN integrity verification completes, THEN the Desktop SHALL distinguish matched hashes, mismatches and missing assets.
4. **R5.4** IF the integrity manifest is absent or invalid, THEN the Desktop SHALL show `UNAVAILABLE` instead of an integrity pass.
5. **R5.5** WHEN a finding is selected, THEN the Desktop SHALL expose its source path and check evidence without executing or opening referenced content automatically.

### Requirement 6: Populated workspace governance

**User Story:** As a workspace maintainer, I want real coverage, drift and policy
results, so that empty UI scaffolding cannot be mistaken for a healthy workspace.

#### Acceptance Criteria

1. **R6.1** WHEN registered Managed Workspaces are scanned, THEN the Desktop SHALL present the Shared Engine's observed installed assets and workspace identities.
2. **R6.2** WHEN coverage or drift is inspected after a successful scan, THEN the Desktop SHALL show the engine-computed result for that scan generation.
3. **R6.3** WHEN policy evaluation completes against valid policy input, THEN the Desktop SHALL show its violations, severities, scopes and existing suppression semantics.
4. **R6.4** IF no Managed Workspaces are registered, THEN the Desktop SHALL show `NOT_CONFIGURED` with a registration path rather than healthy zero coverage or findings.
5. **R6.5** IF a workspace or policy input cannot be read, THEN the Desktop SHALL identify the failed input separately from successfully evaluated workspaces.
6. **R6.6** WHEN a governance finding is selected, THEN the Desktop SHALL show the affected workspace, asset, source generation and reason.
7. **R6.7** WHEN registration or removal of a Managed Workspace is requested, THEN the Desktop SHALL require explicit confirmation of that local registry change.
8. **R6.8** IF policy input is absent, invalid or unreadable, THEN the Desktop SHALL mark dependent policy results `NOT_CONFIGURED` for absent configuration or `UNAVAILABLE` for invalid/unreadable configuration, never healthy zero violations.
9. **R6.9** IF governance inputs have mixed success, failure or stale status, THEN the Desktop SHALL mark aggregate results `PARTIAL` with successful/expected input counts and excluded-input reasons rather than present a complete-health score.
10. **R6.10** IF an input changes or a rescan fails after a valid governance result, THEN the Desktop SHALL mark dependent last-good results `STALE` with their original generation until successful recomputation.

### Requirement 7: Existing validation semantics

**User Story:** As a maintainer, I want to run existing gates with visible evidence,
so that Desktop results mean the same thing as repository validation.

#### Acceptance Criteria

1. **R7.1** WHEN the operator selects a gate, THEN the Desktop SHALL invoke the existing approved gate definition through the Shared Engine rather than reimplementing its business rules.
2. **R7.2** WHEN the operator requests full repository validation, THEN the Desktop SHALL preserve the authoritative `npm run validate` pipeline semantics.
3. **R7.3** IF a required helper runtime or gate definition is missing, THEN the Desktop SHALL reject execution with an actionable prerequisite error.
4. **R7.4** WHEN a gate terminates, THEN the Desktop SHALL show its actual exit outcome and operation evidence.
5. **R7.5** IF a gate or required dependency fails, THEN the Desktop SHALL prevent that failure from being represented as a successful aggregate run.
6. **R7.6** WHEN a gate is requested, THEN the Desktop SHALL require an explicit Run action after checkout execution trust has been granted.

### Requirement 8: Preview-bound export authorization

**User Story:** As a workspace maintainer, I want to preview exports and confirm the
exact destination and options, so that graphical convenience cannot broaden writes.

#### Acceptance Criteria

1. **R8.1** WHEN an export is prepared, THEN the Desktop SHALL default to a non-mutating preview using the existing exporter selection, platform and companion-skill semantics.
2. **R8.2** WHEN a Preview is presented, THEN the Desktop SHALL show source, canonical destination, selected assets, platform, destination conflicts and per-path create/overwrite/skip effects returned by a backward-compatible planning interface in the authoritative exporter.
3. **R8.3** IF preview preparation fails or cannot establish the proposed operation, THEN the Desktop SHALL disable confirmation with a named error.
4. **R8.4** WHEN the operator confirms a current Preview, THEN the Desktop SHALL execute at most one matching export for that confirmation.
5. **R8.5** IF mandatory immediate pre-launch revalidation detects a changed source, destination identity, selection, options or relevant fingerprint, THEN the Desktop SHALL require a refreshed Preview and new confirmation.
6. **R8.6** WHEN the operator cancels or dismisses a Preview, THEN the Desktop SHALL perform no target export writes for that Preview.
7. **R8.7** IF an export ends without verified full success, including failure, cancellation, timeout or interruption, THEN the Desktop SHALL report known changed paths and potentially partial/unknown effects without automatic retry or destructive rollback.
8. **R8.8** THE Desktop SHALL NEVER treat a Preview, default selection or earlier confirmation as approval for a different write operation.
9. **R8.9** IF changed inputs or destination identity are detected during export, THEN the Desktop SHALL stop further writes with partial/unknown effects recorded rather than claim the original Preview was fully honored.
10. **R8.10** WHEN preview/write assurance is presented, THEN the Desktop SHALL disclose that preflight checks do not guarantee atomicity against concurrent external edits or symlink replacement during execution.

### Requirement 9: Bounded asynchronous operation lifecycle

**User Story:** As an operator, I want long-running work to remain responsive and
cancellable, so that a hung helper cannot take over the application or survive unnoticed.

#### Acceptance Criteria

1. **R9.1** WHEN an Operation starts, THEN the Desktop SHALL expose its unique ID, source context and current lifecycle state.
2. **R9.2** WHEN an Operation reaches a Terminal Status, THEN the Shared Engine SHALL publish exactly one observed terminal execution outcome for that Operation independently of its Audit Persistence Status.
3. **R9.3** WHEN cancellation or the configured timeout occurs, THEN the Shared Engine SHALL terminate and reap the Operation's child process tree on each required native target.
4. **R9.4** WHILE background work is running, THE Desktop SHALL continue accepting navigation, search and cancellation input without synchronous filesystem, database or child-process waits on the render/input thread.
5. **R9.5** WHEN child output exceeds display capacity, THEN the Shared Engine SHALL keep combined per-job output buffers at or below 8 MiB, including partial lines before newline arrival.
6. **R9.6** IF output must be dropped or truncated, THEN the Desktop SHALL show that loss explicitly without converting it into child success or blocking pipe draining.
7. **R9.7** WHEN the application closes with active jobs, THEN the Desktop SHALL require a wait-or-cancel choice before shutdown completes.
8. **R9.8** IF execution ended without a trustworthy completion record, THEN the Desktop SHALL represent the Operation as `INTERRUPTED` rather than assume success or automatically replay it.
9. **R9.9** WHEN the Desktop invokes existing asynchronous process or persistence services, THEN the Shared Engine SHALL maintain their required runtime context through operation completion, cancellation and shutdown rather than assume the presentation executor provides it.

### Requirement 10: Audit, reports and local data preservation

**User Story:** As a reviewer, I want persisted, scoped operation evidence and
reports, so that results can be inspected without corrupting existing local data.

#### Acceptance Criteria

1. **R10.1** WHEN an execution or confirmed registry/export change is attempted, THEN the Shared Engine SHALL initiate an operation evidence record linking its ID, action, source/destination identities, redacted options, authorization event and observed lifecycle events.
2. **R10.2** WHEN the audit view is opened, THEN the Desktop SHALL query persisted records rather than render a hardcoded empty list.
3. **R10.3** WHEN a report is requested, THEN the Desktop SHALL use the existing Shared Engine report semantics and data-status annotations.
4. **R10.4** IF the local audit store is unavailable, locked or incompatible, THEN the Desktop SHALL show the failure without resetting or deleting existing data.
5. **R10.5** IF required pre-execution audit recording fails, THEN the Desktop SHALL reject the associated write or helper execution while keeping catalog browsing available.
6. **R10.6** WHEN preferences or data locations are persisted, THEN the Desktop SHALL use documented per-user OS locations or an explicit operator override rather than a silent shared-temporary-directory fallback.
7. **R10.7** WHEN a report or audit file is exported, THEN the Desktop SHALL require confirmation of its destination and existing-file overwrite behavior.
8. **R10.8** THE Desktop SHALL NEVER claim that an ordinary local audit file or database is immutable or a complete defense against tampering.
9. **R10.9** IF terminal evidence cannot be committed after execution, THEN the Desktop SHALL retain the known execution outcome with Audit Persistence Status `FAILED` and an explicit evidence-incomplete warning rather than change that known outcome to `INTERRUPTED` or replay the operation.
10. **R10.10** WHILE required audit persistence is unavailable, THE Desktop SHALL block new helper executions and confirmed writes without blocking catalog browsing or cleanup of already-running jobs.
11. **R10.11** WHEN audit persistence recovers, THEN the Shared Engine SHALL retry only pending evidence persistence with operation-ID deduplication rather than replay the underlying action.

### Requirement 11: Shared engine and adapter compatibility

**User Story:** As a repository maintainer, I want one reusable engine behind the
interfaces, so that adding a GUI does not create divergent validation or security logic.

#### Acceptance Criteria

1. **R11.1** WHEN desktop and terminal/headless adapters perform the same domain operation on the same fixture, THEN they SHALL call one shared implementation for that operation.
2. **R11.2** THE Shared Engine SHALL NEVER require terminal rendering, terminal event handling or GPUI presentation types in its non-test dependency boundary.
3. **R11.3** WHEN existing TUI/headless commands are run after extraction, THEN they SHALL preserve the documented flags, exit codes and machine-readable report schemas unless an explicit compatibility change has been approved.
4. **R11.4** WHEN catalog data is loaded after extraction, THEN the Shared Engine SHALL preserve existing schema validation and unknown-field handling.
5. **R11.5** WHEN existing Node.js/Python validation, export or generation behavior is needed, THEN the Shared Engine SHALL delegate to the authoritative scripts instead of duplicating their business rules.
6. **R11.6** WHEN a shared-engine defect requires correction for Desktop portability, THEN its change SHALL include a regression test for both existing and new adapters rather than quietly preserving the defect as parity.

### Requirement 12: Native platform acceptance

**User Story:** As a user on Linux, macOS or Windows, I want a native working
application, so that cross-platform means verified behavior rather than a build claim.

#### Acceptance Criteria

1. **R12.1** WHEN the alpha is accepted, THEN the Desktop SHALL pass native build and rendered-window interaction tests on every required OS/architecture in the alpha boundary.
2. **R12.2** WHEN Linux acceptance is performed, THEN the Desktop SHALL pass the required workflow smoke tests in both X11 and Wayland graphical sessions.
3. **R12.3** WHEN native paths contain spaces, Unicode or Windows drive separators, THEN the Shared Engine SHALL preserve their filesystem identities without shell-string interpolation or blanket rejection of native separators.
4. **R12.4** IF rendering prerequisites or a supported platform facility are unavailable, THEN the Desktop SHALL emit an actionable startup diagnostic rather than claim successful application startup.
5. **R12.5** WHEN support status is reported, THEN it SHALL distinguish native runtime verification from compile-only, headless-only, WSL and untested results.
6. **R12.6** WHEN existing TUI release targets are evaluated after engine extraction, THEN they SHALL retain their existing supported behavior without acquiring a GPU/windowing requirement.

### Requirement 13: Operable desktop interaction and accessibility

**User Story:** As an operator using a keyboard, pointer or AT, I want consistent
native interaction, so that critical workflows do not depend on mouse precision.

#### Acceptance Criteria

1. **R13.1** WHEN the operator uses keyboard-only navigation, THEN the Desktop SHALL support checkout selection, browsing, search/filtering, details, gate execution/cancellation and export preview/confirmation.
2. **R13.2** WHEN keyboard focus changes, THEN the Desktop SHALL expose a visible focus indicator and predictable Tab/Shift-Tab order.
3. **R13.3** WHEN a confirmation dialog opens and closes, THEN the Desktop SHALL scope keyboard focus to the dialog while open and restore it to the invoking control afterward.
4. **R13.4** WHEN AT inspects an interactive control, THEN the Desktop SHALL expose its accessible name, role, value/state and available action.
5. **R13.5** WHEN a status or error is presented, THEN the Desktop SHALL communicate its meaning with text rather than color alone.
6. **R13.6** WHEN text/UI scale is set to 200% in a 1280×800 logical-pixel window, THEN the Desktop SHALL keep critical controls reachable through resizing or scrolling.
7. **R13.7** WHEN accessibility acceptance is reported, THEN it SHALL include real-window keyboard and platform AT checks in addition to any headless tree assertions.
8. **R13.8** WHEN system light/dark appearance changes or the operator selects an override, THEN the Desktop SHALL keep content and focus indicators readable in the selected appearance.
9. **R13.9** WHEN an interactive workflow is accepted by an automated UI test, THEN that test SHALL verify the performed interaction's application-state effect rather than rely only on accessibility labels, absent disabled flags or visibility metadata.

### Requirement 14: Measurable responsiveness

**User Story:** As an operator, I want bounded startup and search latency, so that
the desktop remains useful while the engine scans or validates repositories.

#### Acceptance Criteria

1. **R14.1** WHEN release-build performance acceptance runs, THEN its report SHALL record OS/backend, CPU, RAM, GPU/driver, fixture hash, commit, lockfile and measurement method.
2. **R14.2** WHEN startup is measured using the fixed performance acceptance protocol below, THEN the Desktop SHALL reach an interactive browse window within 5 seconds at the 95th percentile.
3. **R14.3** WHEN search is measured using the fixed performance acceptance protocol below, THEN the Desktop SHALL publish matching results within 250 milliseconds of the final query input event at the 95th percentile.
4. **R14.4** WHEN cancellation is measured against a controllable helper process tree, THEN the Shared Engine SHALL reach a terminal cancellation/timeout result within 10 seconds of the request or timeout deadline.
5. **R14.5** WHEN navigation is measured under the fixed scan/output pressure protocol below, THEN the Desktop SHALL acknowledge inputs within 100 milliseconds at the 95th percentile.
6. **R14.6** IF any required platform lacks the declared performance evidence, THEN the alpha report SHALL mark its performance acceptance `UNVERIFIED` rather than extrapolate from another platform.

### Requirement 15: Trust, command safety and privacy

**User Story:** As a maintainer, I want explicit trust and write boundaries, so that
opening a catalog is not authorization to run its code or leak local credentials.

#### Acceptance Criteria

1. **R15.1** WHEN the operator only selects and browses a Catalog Checkout, THEN the Desktop SHALL perform no checkout helper execution, target writes or network requests.
2. **R15.2** WHEN checkout code execution is requested without trust for the current Execution Manifest revision in the application session, THEN the Desktop SHALL require explicit trust describing that revision's executable code and its non-sandboxed local privileges.
3. **R15.3** WHEN execution is authorized, THEN the Shared Engine SHALL construct an allowlisted executable and structured argument vector rather than evaluate catalog/user text as a command string.
4. **R15.4** WHEN an export destination or source identity is validated, THEN the Shared Engine SHALL resolve native path and symlink identity before accepting the operation scope.
5. **R15.5** WHEN child environments, displayed output or audit records are prepared, THEN the Shared Engine SHALL apply the existing credential-variable filtering and documented redaction patterns before disclosure.
6. **R15.6** IF untrusted content contains links, escape/control sequences or executable-looking text, THEN the Desktop SHALL treat it as inert display data unless a separate supported operator action authorizes otherwise.
7. **R15.7** THE Desktop SHALL NEVER introduce telemetry, dependency downloads, network-enabled plugins, cloud mutations, credential prompts or privilege elevation as an implicit consequence of launch or browsing.
8. **R15.8** WHEN a trusted helper may write to the checkout or use network access, THEN the Desktop SHALL disclose those declared effects before its Run confirmation.
9. **R15.9** IF the Execution Manifest changes or cannot be established before a helper launch, THEN the Desktop SHALL invalidate reusable trust and require a new execution-specific trust decision after disclosing the changed or unverified inventory.

### Requirement 16: Reproducible alpha delivery

**User Story:** As a repository maintainer, I want verifiable native alpha artifacts,
so that a successful development launch is not confused with a deliverable application.

#### Acceptance Criteria

1. **R16.1** WHEN implementation selects GPUI Kit, THEN its dependency record SHALL identify the exact release/source revision, compatible GPUI snapshot, enabled features and committed lockfile.
2. **R16.2** WHEN builds are evaluated, THEN they SHALL use the selected lockfile and a documented toolchain verified against its actual dependency floor.
3. **R16.3** WHEN alpha artifacts are prepared, THEN they SHALL include per-target checksums, application assets, applicable third-party notices and runtime prerequisites.
4. **R16.4** WHEN a packaged artifact is tested from an unrelated launch directory, THEN it SHALL open the Desktop and complete the required workflows against a selected checkout without relying on the developer's source CWD.
5. **R16.5** IF an alpha artifact is unsigned or not notarized, THEN its release evidence SHALL state that limitation without requesting security-check bypasses as the normal installation path.
6. **R16.6** WHEN publication, signing credentials, external CI changes or release uploads are needed, THEN the workflow SHALL obtain separate explicit authorization for those side effects.
7. **R16.7** WHEN the feature is declared complete, THEN its report SHALL trace every accepted requirement to passing evidence or identify the feature as incomplete.
8. **R16.8** IF the selected toolkit release lacks headless pixel-rendering support on a required target, THEN its acceptance report SHALL require real native visual evidence rather than count the skipped headless renderer as a pass.
9. **R16.9** WHEN the design chooses a toolkit component or test facility, THEN the platform matrix SHALL record its release-specific support independently of the framework's general desktop-platform claim.

## Performance Acceptance Protocol

The budgets in R14 apply to one recorded reference configuration per required
native target, with at least four logical CPU cores, 16 GiB RAM, an SSD and the
platform's supported graphics driver. They do not promise those latencies on every
machine meeting an OS version floor. Reference-machine selection is an approval
decision and must remain fixed within a comparison.

- Version and commit the schema-valid fixture/corpus before measuring: exactly
  10,000 entities comprising 5,000 agents, 4,000 skills, 200 roles, 400 MCP
  references and 400 rules. Use unique stable IDs, existing supported
  provider/harness values, 256-character summaries where the schema supports them,
  and a mix of absent/present optional fields. At least 10% of display strings
  include non-ASCII text. Record generated file hashes.
- Startup: 20 fresh process launches, warm filesystem cache, no network,
  measurement from process creation to the first interactive populated browse
  frame. No timed samples excluded; setup/cache warming precedes all samples.
- Search: commit 100 distinct corpus entries: 25 exact IDs, 25 matching substrings,
  20 Unicode cases, 20 no-match cases and 10 empty-query cases with different
  filter combinations. Submit at one query per second after one untimed corpus
  warm-up. Measure final input dispatch to presentation of its matching snapshot;
  no filtering out slow or obsolete results from the sample set.
- Navigation pressure: ten temporary Managed Workspaces containing 1,000 installed
  asset files each are scanned while a controllable helper emits 1 MiB/s of output
  for the full measurement interval. Dispatch 100 recorded navigation actions at
  20 Hz; measure input dispatch to the corresponding rendered state/focus change.
  Record helper completion and scan overlap to prove pressure actually occurred.
- Percentiles use nearest-rank `ceil(0.95 × N)` over sorted durations: rank 19 of
  20 for startup, rank 95 of 100 for search/navigation. Publish raw samples.
- Cancellation tests use a portable helper with a known descendant tree and
  observable exit acknowledgments; timeout latency starts at the configured
  deadline. A missing descendant acknowledgment is failure, not a timing sample
  to discard. The separate output-bound negative test emits a 64 MiB line without
  newline and checks R9.5 before that line completes.

## Non-Functional Requirements

Requirements 9 and 14 define lifecycle/resource/performance bounds; 11 defines
maintainability and compatibility; 12 and 16 define portability/delivery; 13 defines
accessibility; 10 and 15 define data preservation, trust and privacy. The evaluation
contract specifies positive and negative scenarios for every requirement.

## Out of Scope

- WebAssembly/browser, mobile, additional desktop CPU targets, and WSL as a
  substitute for native Windows acceptance.
- Replacing the TUI, inventing stable public engine APIs, changing catalog schemas,
  rewriting Node.js/Python business rules, or claiming current placeholder tabs work.
  A backward-compatible authoritative exporter plan/result contract is in scope
  where needed for R8; sharing those rules is required, not a second Rust exporter.
- Cloud control planes, remote installation, account authentication/RBAC, a JS
  extension runtime, auto-update, telemetry, background agents or scheduled jobs.
- Bundled catalog/runtimes, public app-store distribution, signing/notarization
  services, installer administration, or credentials without separate approval.
- New database retention/deletion policies or destructive migrations. Existing
  persistence semantics must be preserved; required compatibility changes need
  an explicit design and forward-only data-preservation review.

## Open Questions and Approval Gate

1. Approve or revise the three native target/OS floors and full functional alpha
   boundary above. In particular, confirm native Windows rather than WSL-only.
2. Approve checkout-backed operation with installed helper runtimes, rather than
   bundling a standalone catalog and Node/Python runtime in the first alpha.
3. Approve the proposed latency/resource budgets and required platform/AT evidence;
   no measurements or platform runners are presumed to exist yet.
4. Confirm local/private unsigned alpha testing is acceptable where signing
   credentials are unavailable; public release remains separately authorized.

Until those decisions receive explicit requirements approval, design, task
breakdown and implementation remain pending. An automatic goal continuation is
not approval of this document. The full requested feature goal remains active.
