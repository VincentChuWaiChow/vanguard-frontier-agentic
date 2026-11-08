# Cosmos DB platform operations

## What people get wrong

- They treat account-level multi-region settings as proof the application fails over safely.
- They ignore SDK preferred-region configuration, private DNS, and remaining-region throughput when reviewing regional resiliency.
- They change consistency, failover priority, or throughput during stress without knowing the blast radius.
- They view backup as an availability feature instead of a data-recovery feature.
- They approve partitioning from a diagram instead of measured access patterns and distribution.

## Officially grounded service shape

Microsoft Learn describes Cosmos DB as an account containing databases and containers, with containers as logical units of distribution and scalability. Reliability features include zone redundancy, multi-region replication, several consistency levels, service-managed or customer-managed failover, SDK resiliency behavior, and continuous or periodic backup. The reliability guide also warns against control-plane changes on affected regions during outage scenarios and calls out private endpoint DNS considerations after failover.

## Non-negotiable design rules

1. Identify API type, write model, regions, consistency, backup, and throughput mode before giving a platform verdict.
2. Treat partitioning and throughput as workload-coupled, not platform-only decisions.
3. Verify SDK region preference and retry behavior for multi-region availability.
4. For private endpoints, design regional endpoints and DNS resolution deliberately.
5. Size remaining regions for failover load before claiming regional resilience.
6. Use backup/restore for data corruption/deletion scenarios; use multi-region and failover for availability scenarios.
7. Require diagnostics, Resource Health, Service Health, alerts, and DR runbooks.

## Minimal safe implementation flow

1. Inventory account topology: API, regions, write model, consistency, backup, throughput, network access, and private endpoints.
2. Review workload shape: partition key, hot partition risk, RU headroom, critical operations, and consistency needs.
3. Review reliability: zone redundancy, failover mode, preferred regions, capacity under region loss, and DR drills.
4. Review network: private endpoints, DNS zones, hub/spoke forwarding, firewall, and client routing.
5. Review operations: metrics, logs, alerts, backup restore testing, and owner/runbook evidence.
6. Rank blockers and reversible remediations.
7. Require explicit approval before any control-plane mutation.

## High-risk assumptions to kill

- Multi-region account configuration does not prove application failover unless SDK preferred regions, retries, conflict handling, and remaining-region capacity are verified.
- Private endpoints can break regional resilience if DNS zones, VNet links, forwarding, and local regional endpoint paths are not designed deliberately.
- Strong, bounded staleness, session, consistent prefix, and eventual consistency (all five levels; see `consistency-capacity-security.md`) have different availability, RPO, and throughput implications; do not treat consistency as a cosmetic setting.
- Control-plane changes during an affected-region outage can delay recovery; do not mutate write region, consistency, throughput, networking, or failover priorities casually.
- Backup protects data recovery scenarios; it is not a substitute for live availability, failover routing, or tested application DR.

## Safe command/code verification targets

- Inspect IaC for account regions, zone redundancy, failover priorities, write-region model, consistency, backup mode, throughput mode, and private endpoint resources.
- Review application configuration for SDK preferred regions, excluded regions, retry diagnostics, conflict resolution, and multi-write support where applicable.
- Check network templates for one-region-per-DNS-zone patterns, private DNS links, forwarding rules, firewall dependencies, and client resolution paths.
- Verify monitoring code or dashboards cover normalized RU, 429s, region availability, Resource Health, Service Health, backup status, and restore-test evidence.
- Confirm every proposed account mutation has an approval gate, rollback plan, and explicit outage-state caveat.

## Safe verification targets

- Account regions, write region or multi-write state, failover priorities, and service-managed/forced failover process.
- Consistency level and documented RPO/availability implications.
- Throughput mode and normalized RU headroom by partition key range.
- Private endpoint count, DNS zones, VNet links, forwarding, and client resolution path.
- Backup mode, retention, restore test, and data-corruption response plan.
- SDK preferred regions, retry, partition-level circuit breaker or equivalent availability settings.

## When to push back

Push back on production signoff without failover evidence, private endpoint DNS proof, capacity-under-failure math, SDK routing proof, backup restore tests, or partition distribution evidence.
