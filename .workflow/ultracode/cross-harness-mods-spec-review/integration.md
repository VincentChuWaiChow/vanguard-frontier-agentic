# Integration

## Accepted

Both discovery reports: remove the invented discriminator, distinguish actual
absence from failed reads, validate all strings on reload, bind hashes to adopted
bytes, cover every Rust store literal, and replace insufficient test claims.

## Rejected

The excerpt's blanket claim that Context7 confirmed everything. A line-relative
column is not an absolute byte offset, and null prototypes alone do not break
JSON serialization. Do not substitute `anything()` for a JSON-only domain.

## Conflicts

R18 promised display and footprint while design excluded them. User explicitly
retained the full browser; do not narrow the goal to load/store plumbing.

## Decisions

Generate footprint arrays from the canonical manifest; accept legacy absence as
unknown metadata. Browser is read-only through the actual primary Catalog tab.
Missing-at-startup is optional; missing-during-reload retains previous state.
Serialize shared writers; final verification follows every T9 change.

## Final changes

Amended R8.7, R16.3, R17.5, R18, P3 and P8; added ADR-7, test/traceability mapping,
P3/README follow-ups and UI/generator tasks. All runtime tasks remain unchecked;
9.0 records only the completed scope decision. No runtime/catalog/skill-source
edits were made.

## Verification still needed

Repository validation completed with exit 0 outside the sandbox; focused probes,
spelling and full Markdown lint also passed. Final lint is repeated after this
evidence text settles. Commit/publication verification belongs to the parent's
final handoff, not to the read-only worker reports.

## Remaining risks

Manual analysis cannot attest Kiro's SMT solver. Original seven Kiro questions
were not supplied; the seven grouped findings here were reconstructed from source.
Historical requirements outside the amendment boundary remain governed by the
approved deviations rather than newly audited or reapproved.
