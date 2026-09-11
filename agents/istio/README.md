# 🕸️ Istio Agents

<p align="center">
  <span style="font-size:3.5em">🕸️</span>
</p>

Istio agent catalog for this marketplace.

## 🧱 Agent tiers

| Tier | Purpose | Default access | Live cluster mutation |
|---|---|---|---|
| Review agents | Audit ambient enrollment, authorization, traffic resilience, Gateway API, upgrades, and dataplane evidence | read-only | not allowed |
| Guarded live operators | Apply AuthorizationPolicy, PeerAuthentication mutations on live clusters | workspace-write | approval-gated and target-confirmed only |

## 📋 Mesh review agents

| Agent | Primary use | Default live posture | Must refuse when |
|---|---|---|---|
| `istio-ambient-mesh-review-agent` | Resolve enrollment, traffic paths, and ztunnel versus waypoint enforcement | read-only | live evidence collection or mutation |
| `istio-authorization-policy-review-agent` | Evaluate the complete authentication and authorization policy set | read-only | live evidence collection or mutation |
| `istio-dataplane-diagnostics-agent` | Rank connectivity and response-code hypotheses from supplied evidence | read-only | live logs, probes, or debug changes |
| `istio-gateway-api-review-agent` | Resolve listeners, route attachment, references, and condition freshness | read-only | Secret retrieval or Gateway mutation |
| `istio-traffic-resilience-review-agent` | Review routes, relative weights, retries, mirroring, and failure amplification | read-only | fault injection, mirroring, or mutation |
| `istio-upgrade-readiness-agent` | Assess a specified source-to-target transition and rollback readiness | read-only | installing or upgrading Istio |
| `istio-maestro-agent` | Route multi-domain reviews while preserving blockers and unknowns | read-only | auto-dispatching the live guard |

## 🔒 Live-guard operators (never auto-dispatched)

Live-guard agents for Istio are housed in `agents/kubernetes/` because they operate at the Kubernetes API layer:

| Agent | Primary use |
|---|---|
| `kubernetes-live-mesh-policy-guard-agent` | Guard an independently approved, target-bound Istio policy mutation through `istio-live-policy-change` |

## 🛡️ Operating note

- Review agents stay read-only — they never write to the cluster
- Resolve the actual traffic path and policy target before judging missing-waypoint
  behavior; L7 attributes at a receiving ztunnel have a documented fail-safe denial case
- `PERMISSIVE` peer authentication accepts plaintext and mTLS; evaluate the effective
  policy and path rather than claiming that all mTLS is disabled
- Any `AuthorizationPolicy` with `action: DENY` on a wide selector can cause unintended traffic black-holes
- All live-guard agents produce a structured verdict response — see [`docs/evidence-output-spec.md`](../../docs/evidence-output-spec.md)

## 📦 Install

```bash
# Install all Istio review agents and their companions
npx vfa-export-agents --platform claude-code --provider istio --all --repo .

# Install all Kubernetes network agents (includes live-guard)
npx vfa-export-agents --platform claude-code --role kubernetes-network-engineer --repo .
```
