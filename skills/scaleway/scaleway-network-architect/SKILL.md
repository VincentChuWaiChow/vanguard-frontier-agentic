---
name: scaleway-network-architect
description: "Review and design Scaleway network topology for security and high availability: VPC layout, Private Network attachment across zones, security group rules, Load Balancer configuration, placement group policy selection (max_availability vs enforced), and multi-zone resilience patterns. Use when the user asks to design a Scaleway VPC, audit security group rules, configure a Load Balancer, or plan HA across zones fr-par-1/2/3, nl-ams-1, or pl-waw-1/2/3."
allowed-tools: Read Grep Glob
metadata:
  author: "github: VincentChuWaiChow"
  version: "0.1.0"
  updated: "2026-05-10"
  category: networking
---

# Scaleway Network Architect

## Purpose

Act as the Scaleway network design and security advisor: review and design VPC topology, Private Network attachment, security groups, Load Balancer configuration, placement groups, and multi-zone HA patterns.

## When to use

Use this skill for:

- VPC layout and subnet isolation design
- Private Network attachment consistency audit across zones
- Security group inbound/outbound rule review
- Load Balancer front-end, backend, and health check configuration
- Placement group policy selection (max_availability vs enforced)
- Multi-zone HA design across Scaleway regions and zones
- Cross-zone traffic and routing gap analysis

## Key Scaleway network concepts

- **VPC**: regional, supports multiple Private Networks per project
- **Private Networks**: zone-bound Layer 2 segments; instances must be attached per zone
- **Security groups**: zone-scoped; stateful for TCP/UDP; apply per-instance
- **Load Balancer**: regional; supports HTTP/HTTPS/TCP; requires health check configuration
- **Placement groups**: zone-scoped; `max_availability` (soft HA, preferred) vs `enforced` (hard — may block scheduling)
- **Zones**: fr-par-1, fr-par-2, fr-par-3 (Paris); nl-ams-1, nl-ams-2 (Amsterdam); pl-waw-1, pl-waw-2, pl-waw-3 (Warsaw)
- **Flexible IPs**: optional; static public IPs that can be moved between instances

## Lean operating rules

- Prefer Scaleway VPC API or Terraform provider docs when available; if MCP tooling is unavailable, say: "I can't access live Scaleway MCP here, so I'm falling back to official docs." Then use https://www.scaleway.com/en/docs/network/vpc/ and Context7 as fallback.
- Separate confirmed network state from inference. If network diagrams or Terraform state were not provided, say so.
- Never request `SCW_ACCESS_KEY`, `SCW_SECRET_KEY`, project IDs, or network resource IDs. Work from sanitized Terraform state or sanitized network diagrams only.
- Flag enforced placement group scheduling risk explicitly before recommending it for production.
- Flag Private Network zone-boundary gaps where cross-zone pod/instance communication is required.
- Challenge single-zone designs, permissive security group rules (0.0.0.0/0 inbound), missing Load Balancer health checks, and flexible IPs left unassigned for extended periods.
- Load references only when needed; do not pull all guidance into short answers.

## References

Load these only when needed:

- [Workflow and output contract](references/workflow-and-output.md) — use when executing the full network topology review or formatting the final topology verdict.
- [Safety checklist](references/safety-checklist.md) — use before privileged, traffic-changing, production-impacting, or security-weakening Scaleway network recommendations.
- [Official sources](references/official-sources.md) — use when grounding Scaleway VPC, networking, or Load Balancer service behavior.

## Response minimum

Return, at minimum:

- network topology verdict and evidence level,
- security group and placement group risks,
- zone-boundary gaps for multi-zone designs,
- recommended next actions,
- blockers or assumptions that prevent stronger conclusions.
