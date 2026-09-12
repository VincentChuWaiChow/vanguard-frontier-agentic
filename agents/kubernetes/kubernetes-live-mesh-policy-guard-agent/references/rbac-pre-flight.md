# RBAC pre-flight self-check — Kubernetes Live Mesh Policy Guard

This is the mandatory first action of every session. The agent runs this matrix before reading any user-supplied YAML, before formulating any mutation, before producing any output other than the matrix result.

The matrix is grounded against `kubernetes.io/docs/concepts/security/rbac-good-practices` and `kubernetes.io/docs/reference/kubectl/generated/kubectl_auth/kubectl_auth_can-i`. The canonical authoring contract is `docs/least-privilege-rbac.md`.

If any **must-not-be-yes** check returns `yes`, or any **must-be-yes** check returns `no`, the agent refuses to act and tells the user the binding is over- or under-scoped.

---

## Required RBAC manifest

Apply `references/least-privilege-rbac.yaml` (shipped with this agent) before invoking it. The manifest creates `ServiceAccount/vanguard-mesh-policy-guard` in namespace `vanguard-system`.

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
SA="system:serviceaccount:vanguard-system:vanguard-mesh-policy-guard"

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

## Domain-specific must-not-be-yes (Kubernetes Live Mesh Policy Guard)

```bash
# Delete on policies — rollback is via apply -f baseline, not delete
kubectl auth can-i delete authorizationpolicies.security.istio.io --all-namespaces --as=$SA
kubectl auth can-i delete peerauthentications.security.istio.io --all-namespaces --as=$SA
kubectl auth can-i delete requestauthentications.security.istio.io --all-namespaces --as=$SA
kubectl auth can-i delete virtualservices.networking.istio.io --all-namespaces --as=$SA
kubectl auth can-i delete destinationrules.networking.istio.io --all-namespaces --as=$SA
# istio-system control plane
kubectl auth can-i patch deployments -n istio-system --as=$SA
kubectl auth can-i patch configmaps -n istio-system --as=$SA
# Istio Gateway resources are out of scope for mesh-policy-guard (delegated to network-architecture)
kubectl auth can-i create gateways.networking.istio.io --all-namespaces --as=$SA
kubectl auth can-i patch gateways.networking.istio.io --all-namespaces --as=$SA
```

## Domain-specific must-be-yes (Kubernetes Live Mesh Policy Guard)

```bash
kubectl auth can-i create authorizationpolicies.security.istio.io --all-namespaces --as=$SA
kubectl auth can-i patch authorizationpolicies.security.istio.io --all-namespaces --as=$SA
kubectl auth can-i create peerauthentications.security.istio.io --all-namespaces --as=$SA
kubectl auth can-i patch peerauthentications.security.istio.io --all-namespaces --as=$SA
kubectl auth can-i create requestauthentications.security.istio.io --all-namespaces --as=$SA
kubectl auth can-i patch requestauthentications.security.istio.io --all-namespaces --as=$SA
kubectl auth can-i create virtualservices.networking.istio.io --all-namespaces --as=$SA
kubectl auth can-i patch virtualservices.networking.istio.io --all-namespaces --as=$SA
kubectl auth can-i create destinationrules.networking.istio.io --all-namespaces --as=$SA
kubectl auth can-i patch destinationrules.networking.istio.io --all-namespaces --as=$SA
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
