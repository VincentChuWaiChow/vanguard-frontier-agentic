# RBAC pre-flight self-check — Kubernetes Live Admission Policy Guard

This is the mandatory first action of every session. The agent runs this matrix before reading any user-supplied YAML, before formulating any mutation, before producing any output other than the matrix result.

The matrix is grounded against `kubernetes.io/docs/concepts/security/rbac-good-practices` and `kubernetes.io/docs/reference/kubectl/generated/kubectl_auth/kubectl_auth_can-i`. The canonical authoring contract is `docs/least-privilege-rbac.md`.

If any **must-not-be-yes** check returns `yes`, or any **must-be-yes** check returns `no`, the agent refuses to act and tells the user the binding is over- or under-scoped.

---

## Required RBAC manifest

Apply `references/least-privilege-rbac.yaml` (shipped with this agent) before invoking it. The manifest creates `ServiceAccount/vanguard-admission-policy-guard` in namespace `vanguard-system`.

---

## Operator principal check (run first)

```bash
# If yes: operator is in system:masters or has cluster-admin. Refuse.
kubectl auth can-i '*' '*' --all-namespaces
```

Per upstream `kubernetes.io/docs/concepts/security/rbac-good-practices`:

> *Administrators should avoid using `cluster-admin` accounts and instead provide low-privileged accounts with impersonation rights.*
>
> *Do not add users to the `system:masters` group, as this bypasses all RBAC checks.*

---

## Universal must-not-be-yes (every live-guard)

```bash
SA="system:serviceaccount:vanguard-system:vanguard-admission-policy-guard"

kubectl auth can-i '*' '*' --all-namespaces --as=$SA
kubectl auth can-i delete namespaces --as=$SA
kubectl auth can-i delete pods -n kube-system --as=$SA
kubectl auth can-i create pods/exec -n kube-system --as=$SA
kubectl auth can-i create pods/portforward --all-namespaces --as=$SA
kubectl auth can-i delete daemonsets -n kube-system --as=$SA
kubectl auth can-i delete deployments -n kube-system --as=$SA
kubectl auth can-i create customresourcedefinitions --as=$SA
kubectl auth can-i delete customresourcedefinitions --as=$SA
kubectl auth can-i get secrets --all-namespaces --as=$SA
kubectl auth can-i create clusterrolebindings --as=$SA
kubectl auth can-i create mutatingwebhookconfigurations.admissionregistration.k8s.io --as=$SA
kubectl auth can-i delete mutatingwebhookconfigurations.admissionregistration.k8s.io --as=$SA
kubectl auth can-i create validatingwebhookconfigurations.admissionregistration.k8s.io --as=$SA
kubectl auth can-i create apiservices.apiregistration.k8s.io --as=$SA
kubectl auth can-i update certificatesigningrequests.certificates.k8s.io --subresource=approval --as=$SA
kubectl auth can-i create serviceaccounts/token --all-namespaces --as=$SA
kubectl auth can-i delete priorityclasses.scheduling.k8s.io --as=$SA
kubectl auth can-i delete ingressclasses.networking.k8s.io --as=$SA
kubectl auth can-i delete leases.coordination.k8s.io -n kube-node-lease --as=$SA
kubectl auth can-i update namespaces/finalize --as=$SA
```

## Domain-specific must-not-be-yes (Kubernetes Live Admission Policy Guard)

```bash
# Webhook configurations — operator install only
kubectl auth can-i create mutatingwebhookconfigurations.admissionregistration.k8s.io --as=$SA
kubectl auth can-i patch mutatingwebhookconfigurations.admissionregistration.k8s.io --as=$SA
kubectl auth can-i create validatingwebhookconfigurations.admissionregistration.k8s.io --as=$SA
kubectl auth can-i patch validatingwebhookconfigurations.admissionregistration.k8s.io --as=$SA
# Delete on policies — rollback is via apply -f baseline
kubectl auth can-i delete clusterpolicies.kyverno.io --as=$SA
kubectl auth can-i delete validatingadmissionpolicies.admissionregistration.k8s.io --as=$SA
kubectl auth can-i delete validatingpolicies.policies.kyverno.io --as=$SA
kubectl auth can-i delete policyexceptions.policies.kyverno.io --all-namespaces --as=$SA
# Kyverno CEL policies whose controller writes or deletes other objects — never written by this guard
kubectl auth can-i create mutatingpolicies.policies.kyverno.io --as=$SA
kubectl auth can-i create generatingpolicies.policies.kyverno.io --as=$SA
kubectl auth can-i create deletingpolicies.policies.kyverno.io --as=$SA
kubectl auth can-i create namespacedgeneratingpolicies.policies.kyverno.io --all-namespaces --as=$SA
# Policy reports are evidence — read only, never written or deleted by this guard
kubectl auth can-i create policyreports.wgpolicyk8s.io --all-namespaces --as=$SA
kubectl auth can-i delete clusterpolicyreports.wgpolicyk8s.io --as=$SA
# Kyverno control plane
kubectl auth can-i patch deployments -n kyverno --as=$SA
kubectl auth can-i patch configmaps -n kyverno --as=$SA
```

## Domain-specific must-be-yes (Kubernetes Live Admission Policy Guard)

```bash
kubectl auth can-i create clusterpolicies.kyverno.io --as=$SA
kubectl auth can-i patch clusterpolicies.kyverno.io --as=$SA
kubectl auth can-i create policies.kyverno.io --all-namespaces --as=$SA
kubectl auth can-i create policyexceptions.kyverno.io --all-namespaces --as=$SA
kubectl auth can-i create validatingadmissionpolicies.admissionregistration.k8s.io --as=$SA
kubectl auth can-i list clusterpolicies.kyverno.io --as=$SA
# Kyverno CEL policies (policies.kyverno.io, Kyverno 1.20+)
kubectl auth can-i create validatingpolicies.policies.kyverno.io --as=$SA
kubectl auth can-i patch validatingpolicies.policies.kyverno.io --as=$SA
kubectl auth can-i patch imagevalidatingpolicies.policies.kyverno.io --as=$SA
kubectl auth can-i create namespacedvalidatingpolicies.policies.kyverno.io --all-namespaces --as=$SA
kubectl auth can-i create policyexceptions.policies.kyverno.io --all-namespaces --as=$SA
kubectl auth can-i list generatingpolicies.policies.kyverno.io --as=$SA
# Policy reports (wgpolicyk8s.io/v1alpha2 at Kyverno v1.20.0)
kubectl auth can-i list policyreports.wgpolicyk8s.io --all-namespaces --as=$SA
kubectl auth can-i list clusterpolicyreports.wgpolicyk8s.io --as=$SA
```

Every must-not row must print `no`. Every must-be row must print `yes`. Any deviation: refuse and tell the operator which line failed.

---

## resourceName-scoped binding verification (positive AND negative)

Where the manifest uses `resourceNames`, test BOTH the allowed name and at least one denied adjacent name. `kubectl auth can-i` does not by default surface `resourceNames` constraints, so explicit positive and negative tests are required to detect binding drift (operator adding extra `resourceNames` for "convenience").

---

## Refusal posture

If pre-flight fails:

```
Pre-flight: FAIL
Failing check: <verb> <resource> <namespace>
Expected: no | Actual: yes (binding over-scoped on the bound ServiceAccount)
Action: refusing to proceed. Re-apply references/least-privilege-rbac.yaml or scope down the existing binding.
```

No exceptions. The pre-flight is the gate.
