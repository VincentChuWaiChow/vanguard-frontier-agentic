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
| 0    | Every check ran and passed |
| 1    | One or more assertions failed — a manifest is over- or under-scoped |
| 2    | INCOMPLETE — some checks never ran, so nothing was proven |
| 3    | HARNESS ERRORS — checks could not obtain a verdict from `kubectl` |

A skipped check is an assertion that was never made. This suite is the
privilege-creep gate for the shipped RBAC manifests, so "could not check"
never renders as "passed": exit 2 and exit 3 are both non-zero, and they are
kept apart so an expected environment limitation is never confused with a
broken cluster.

---

## How CI works

The GitHub Actions workflow is at `.github/workflows/rbac-pre-flight.yml`. It triggers
on any change to:

- `agents/**/references/least-privilege-rbac.yaml`
- `skills/**/references/least-privilege-rbac.yaml`
- `agents/**/references/rbac-pre-flight.md`
- `skills/**/references/rbac-pre-flight.md`
- `tests/integration/rbac-pre-flight/**`

Two jobs run. `harness-self-test` runs `self-test.sh` with no cluster, in
seconds. `rbac-pre-flight` runs `run-all.sh` in a matrix across four
Kubernetes versions (1.28, 1.29, 1.30, 1.31) with `fail-fast: false` so every
leg completes even when one fails. On failure the log is uploaded as an
artifact.

On 1.28 and 1.29 the cluster is created with `ValidatingAdmissionPolicy` and
`admissionregistration.k8s.io/v1beta1` enabled. That API went GA in 1.30; on
the two earlier legs it is beta and off by default, so without the patch
`validatingadmissionpolicies` is absent from discovery and the admission-policy
guard cannot assert anything about it. The patch is scoped to those versions
because from 1.30 the gate is GA-locked.

---

## CRDs, and why SKIP rows are now rare

Several domain-specific checks target CRDs no vanilla cluster installs —
Gateway API, Cilium, Istio, Kyverno, Argo CD, Velero. The suite installs
minimal stand-ins for them from `fixtures/crds.yaml` before running, so those
assertions execute for real.

Stubs are enough because only discovery matters here, and they are necessary
because without them `kubectl` answers the *wrong* question. `kubectl auth
can-i` resolves its resource argument through discovery first; when the
resource is missing it warns and falls back to the whole dotted argument as
the resource name with an empty API group, so the access review asks about
`{group: "", resource: "schedules.velero.io"}` — something no RBAC rule can
match.

Measured against a live `kube-apiserver` v1.30.6, for a ServiceAccount
explicitly granted `velero.io/schedules` `create`:

| CRD present? | verdict | ground truth |
|--------------|---------|--------------|
| no  | `no`  | wrong |
| yes | `yes` | correct |

So an absent CRD is not merely an unavailable answer, it is a wrong one in
both directions: every must-not check would pass vacuously and every
must-be-able check would fail spuriously. Nothing in `fixtures/crds.yaml` is
ever instantiated and no controller reconciles it; only `group`,
`names.plural`, `names.kind` and `scope` are load-bearing.

Keeping the stubs in-tree rather than fetching upstream bundles keeps the gate
deterministic: no network at test time, no version drift, same behaviour on
every Kubernetes version in the matrix.

A SKIP therefore now means something genuinely unexpected, and it keeps the
run INCOMPLETE rather than passing.

---

## Subresources

Write subresource checks with `--subresource=`, never as `resource/subresource`:

```bash
assert_cannot create pods --subresource=exec -n kube-system "--as=$SA"   # correct
assert_cannot create pods/exec -n kube-system "--as=$SA"                 # WRONG
```

`kubectl auth can-i create pods/exec` parses `pods/exec` as resource `pods`
with the *name* `exec` — the access review goes out as
`{"verb":"create","resource":"pods","name":"exec"}`. Measured on v1.30.6, a
ServiceAccount explicitly granted `pods/exec` `create` still answers `no` to
that form, and `yes` only to `--subresource=exec`. Every such check was
therefore incapable of detecting the escalation it named.

`resource/name` remains correct and intentional for the resourceName tests
(`configmaps/coredns`, `namespaces/kube-system`). `self-test.sh` lints for the
difference, flagging only names that are real subresources.

---

## Harness self-test

`./self-test.sh` exercises the harness itself with a stub `kubectl` — no
cluster, no network, no clock. It covers stream handling, skip-vs-error
classification, exit codes, and the subresource lint, and runs as its own CI
job in seconds.

Run it after any change to `lib/common.sh` or the guards.

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
  self-test.sh               — tests the harness itself; needs no cluster
  fixtures/
    crds.yaml                — minimal stand-ins for third-party CRDs
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
