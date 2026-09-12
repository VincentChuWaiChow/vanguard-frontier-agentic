# Persistence, extensions and rollback

## UPGRADE-05: storage versus historical persistence

Distinguish `spec.versions[*].storage` from `status.storedVersions`. Changing the storage-write version does not rewrite existing stored objects. Before removing a served/storage version, require migration/conversion evidence and compatibility with the rollback control plane. Never delete/reinstall CRDs to repair an ownership problem without an explicit data-preservation procedure.

## UPGRADE-06: extension and shared-resource boundary

Inventory EnvoyFilter, WasmPlugin, custom authorization providers, CNI integrations, telemetry and managed gateway configuration. Pin artifacts, inspect supported APIs and identify what shares CRDs, webhooks or trust material across revisions. A canary control plane does not isolate all cluster-wide changes.

## Rollback plan

Preserve exact chart/configuration values, resource baseline, supported rollback binaries, credential/trust dependencies and required old APIs. Define abort measurements for allowed/denied traffic, latency, errors, saturation and control-plane health. Distinguish reversible workload revision selection from shared-resource or stored-data changes.

A rollback command is not a tested rollback. Require observed rollback in an isolated representative environment before labeling rollback validated. Review long-lived connections and partially rolled nodes/pods. Leave unknown conversion/extension behavior as a readiness blocker or unresolved requirement according to evidence.

Sources: CRD, HELM, MIGRATE. Tests: UPGRADE-05, UPGRADE-06.
