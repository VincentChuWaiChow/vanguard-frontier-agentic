# RBAC Pre-flight Integration Tests

Regression harness for the least-privilege RBAC bindings of all 7 Vanguard
Kubernetes live-guard agents. The suite creates a real kind cluster, applies
each guard's `least-privilege-rbac.yaml`, and then asserts every row in the
`rbac-pre-flight.md` matrices — both the universal must-not-be-yes block and
each guard's domain-specific checks.

Running this suite after any change to an RBAC manifest catches privilege
creep and under-scoping before the change reaches a production cluster.

---

## Purpose

Kubernetes RBAC semantics evolve across minor versions. An RBAC manifest that
is correctly scoped on 1.28 may silently acquire broader rights on 1.30 if a
new built-in ClusterRole or defaulting behavior changes. This suite pins the
expected can-i matrix for every guard so that changes to Kubernetes itself,
or accidental edits to the manifests, are caught immediately in CI.

---

## Requirements

| Tool   | Minimum version |
|--------|----------------|
| kind   | 0.22            |
| k3d    | 5.6 (alternative to kind) |
| kubectl | 1.28           |
| bash   | 4.0+            |

The tests do not require Docker Desktop — rootless Docker or Podman work as
long as kind can reach them.

---

## Running locally

```bash
# Full run: creates a kind cluster, tests all guards, destroys the cluster
cd tests/integration/rbac-pre-flight
./run-all.sh

# Use a specific Kubernetes version (default: v1.30.6)
KIND_K8S_VERSION=v1.29.10 ./run-all.sh

# Skip cluster creation and run against your current kubeconfig context
./run-all.sh --skip-cluster-create

# Run only one guard
./run-all.sh --guard=rbac-mutation

# Combine flags
./run-all.sh --skip-cluster-create --guard=network-arch
```

A timestamped log is always written to `/tmp/rbac-preflight-<timestamp>.log`.

---

## Exit codes

| Code | Meaning |
|------|---------|
| 0    | All assertions passed (SKIP rows do not count as failures) |
| 1    | One or more assertions failed |

---

## How CI works

The GitHub Actions workflow is at `.github/workflows/rbac-pre-flight.yml`. It triggers
on any change to:

- `agents/**/references/least-privilege-rbac.yaml`
- `skills/**/references/least-privilege-rbac.yaml`
- `agents/**/references/rbac-pre-flight.md`
- `skills/**/references/rbac-pre-flight.md`
- `tests/integration/rbac-pre-flight/**`

The workflow runs `run-all.sh` in a matrix across four Kubernetes versions
(1.28, 1.29, 1.30, 1.31) using `fail-fast: false` so all matrix legs
complete even when one fails. On failure, the log file is uploaded as a
GitHub Actions artifact.

---

## Understanding SKIP rows

Several domain-specific checks target CRDs that are not installed in a
vanilla kind cluster:

| CRD group | Example guard | Pre-install URL |
|-----------|--------------|-----------------|
| `gateway.networking.k8s.io` | network-arch | https://github.com/kubernetes-sigs/gateway-api/releases/download/v1.2.0/standard-install.yaml |
| `cilium.io` | network-policy | https://docs.cilium.io/en/stable/installation/k8s-install-helm/ |
| `security.istio.io`, `networking.istio.io` | mesh-policy | https://istio.io/latest/docs/setup/install/ |
| `kyverno.io` | admission-policy | https://kyverno.io/docs/installation/ |
| `argoproj.io` | argocd-sync | https://argo-cd.readthedocs.io/en/stable/getting_started/ |
| `velero.io` | velero-restore | https://velero.io/docs/latest/basic-install/ |

SKIP rows are informational — the binding cannot be checked without the CRD
present. To validate those rows, pre-apply the CRDs before running the suite:

```bash
# Example: test Gateway API rows
kubectl apply -f https://github.com/kubernetes-sigs/gateway-api/releases/download/v1.2.0/standard-install.yaml
./run-all.sh --skip-cluster-create
```

---

## Note on impersonation

`kubectl auth can-i --as=<serviceaccount>` requires the requesting principal
to have `impersonate` rights. In a kind cluster where you start as
cluster-admin this works without additional configuration. The manifests
themselves grant no impersonation rights to the guard ServiceAccounts.

If you are running `--skip-cluster-create` against a hardened cluster, ensure
your kubeconfig principal has `impersonate` on `users`, `groups`, and
`serviceaccounts`.

---

## Adding a new guard

1. Copy an existing file in `guards/` and rename it.
2. Update the `SA` variable to the new ServiceAccount name.
3. Call `run_universal_must_not "$SA"` at the top.
4. Add domain-specific `assert_can` / `assert_cannot` calls extracted from
   the guard's `references/rbac-pre-flight.md`.
5. Call `report_guard "<guard-name>"` at the end.
6. Register the new guard in `run-all.sh`:
   - Add an entry to the `GUARD_FN` associative array.
   - Add the guard name to `GUARD_ORDER`.
   - Source the new file with `source "$SCRIPT_DIR/guards/<name>.sh"`.

---

## File layout

```
tests/integration/rbac-pre-flight/
  README.md                  — this file
  run-all.sh                 — main entrypoint
  lib/
    common.sh                — assert_can / assert_cannot helpers
  guards/
    network-arch.sh          — network-architecture-mutation guard
    network-policy.sh        — network-policy guard
    mesh-policy.sh           — mesh-policy guard
    admission-policy.sh      — admission-policy guard
    argocd-sync.sh           — argocd-sync guard
    rbac-mutation.sh         — rbac-mutation guard
    velero-restore.sh        — velero-restore guard
```

The GitHub Actions workflow lives at `.github/workflows/rbac-pre-flight.yml`.
It used to sit in a `ci/` directory here, where GitHub never reads workflows, so
it had never run.
