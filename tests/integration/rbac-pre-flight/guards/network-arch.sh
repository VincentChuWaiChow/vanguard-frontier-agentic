#!/usr/bin/env bash
# guards/network-arch.sh — RBAC pre-flight for kubernetes-live-network-architecture-mutation-guard
# Source lib/common.sh before running this file.

run_guard_network_arch() {
  begin_guard "network-architecture-mutation"

  local SA="system:serviceaccount:vanguard-system:vanguard-network-arch-guard"

  # -------------------------------------------------------------------------
  # Universal must-not-be-yes
  # -------------------------------------------------------------------------
  run_universal_must_not "$SA"

  # -------------------------------------------------------------------------
  # Domain-specific must-not-be-yes
  # (from skills/kubernetes/kubernetes-live-network-architecture-mutation-guard/references/rbac-pre-flight.md)
  # -------------------------------------------------------------------------
  printf '%b  -- domain-specific must-not-be-yes --%b\n' "$_CYAN" "$_RESET"

  # Namespace destruction
  assert_cannot delete namespaces/kube-system "--as=$SA"
  assert_cannot delete namespaces/cilium "--as=$SA"
  assert_cannot delete namespaces/istio-system "--as=$SA"

  # kube-system daemonset patches
  assert_cannot patch daemonsets/cilium -n kube-system "--as=$SA"
  assert_cannot patch daemonsets/kube-proxy -n kube-system "--as=$SA"

  # Broad secret access
  assert_cannot list secrets --all-namespaces "--as=$SA"

  # Privilege escalation
  assert_cannot create clusterroles "--as=$SA"
  assert_cannot escalate roles "--as=$SA"
  assert_cannot bind roles "--as=$SA"
  assert_cannot impersonate users "--as=$SA"
  assert_cannot impersonate groups "--as=$SA"
  assert_cannot impersonate serviceaccounts "--as=$SA"

  # Node lifecycle
  assert_cannot delete nodes "--as=$SA"
  assert_cannot patch nodes "--as=$SA"
  assert_cannot update nodes "--as=$SA"
  assert_cannot create pods --subresource=eviction "--as=$SA"
  assert_cannot get nodes --subresource=proxy "--as=$SA"
  assert_cannot create nodes --subresource=proxy "--as=$SA"

  # Lease objects
  assert_cannot patch leases.coordination.k8s.io -n kube-node-lease "--as=$SA"

  # Admission webhook configs (additional verbs beyond universal)
  assert_cannot patch mutatingwebhookconfigurations.admissionregistration.k8s.io "--as=$SA"
  assert_cannot patch validatingwebhookconfigurations.admissionregistration.k8s.io "--as=$SA"
  assert_cannot delete validatingwebhookconfigurations.admissionregistration.k8s.io "--as=$SA"

  # APIService aggregation
  assert_cannot patch apiservices.apiregistration.k8s.io "--as=$SA"
  assert_cannot delete apiservices.apiregistration.k8s.io "--as=$SA"

  # Pod subresources
  assert_cannot create pods --subresource=proxy --all-namespaces "--as=$SA"
  assert_cannot create pods --subresource=binding --all-namespaces "--as=$SA"

  # CSR / token minting
  assert_cannot create certificatesigningrequests.certificates.k8s.io "--as=$SA"

  # Manual Endpoints / EndpointSlices writes
  assert_cannot create endpoints --all-namespaces "--as=$SA"
  assert_cannot patch endpoints --all-namespaces "--as=$SA"
  assert_cannot create endpointslices.discovery.k8s.io --all-namespaces "--as=$SA"
  assert_cannot patch endpointslices.discovery.k8s.io --all-namespaces "--as=$SA"

  # kube-system ConfigMap writes outside the resourceName-locked allowlist
  assert_cannot patch configmaps/cilium-config -n kube-system "--as=$SA"
  assert_cannot patch configmaps/kube-proxy -n kube-system "--as=$SA"
  assert_cannot patch configmaps/kubelet-config -n kube-system "--as=$SA"
  assert_cannot patch configmaps/cluster-info -n kube-public "--as=$SA"

  # PriorityClass / IngressClass / StorageClass
  assert_cannot patch priorityclasses.scheduling.k8s.io "--as=$SA"
  assert_cannot delete ingressclasses.networking.k8s.io "--as=$SA"
  assert_cannot patch ingressclasses.networking.k8s.io "--as=$SA"
  assert_cannot patch storageclasses.storage.k8s.io "--as=$SA"

  # Finalizer-stripping paths.  A CRD has no `finalize` subresource (only
  # `status`); stripping its finalizers is an update to the object itself.
  assert_cannot update customresourcedefinitions "--as=$SA"

  # resourceName negative tests — same configmap name in wrong namespace
  assert_cannot patch configmaps/coredns -n default "--as=$SA"
  assert_cannot patch configmaps/coredns -n kube-public "--as=$SA"
  assert_cannot patch configmaps/extension-apiserver-authentication -n kube-system "--as=$SA"

  # -------------------------------------------------------------------------
  # Domain-specific must-be-yes
  # -------------------------------------------------------------------------
  printf '%b  -- domain-specific must-be-yes --%b\n' "$_CYAN" "$_RESET"

  assert_can get services --all-namespaces "--as=$SA"
  assert_can list services --all-namespaces "--as=$SA"
  assert_can patch services --all-namespaces "--as=$SA"
  assert_can get endpointslices --all-namespaces "--as=$SA"
  assert_can get nodes "--as=$SA"
  assert_can get configmaps -n kube-system "--as=$SA"

  # CoreDNS Corefile (resourceName-locked — positive side of resourceName test)
  assert_can patch configmaps/coredns -n kube-system "--as=$SA"
  assert_can get configmaps/coredns -n kube-system "--as=$SA"

  # Gateway API resources — CRDs not present in vanilla kind; skip not fail
  printf '%b  -- Gateway API checks (SKIP if CRDs absent) --%b\n' "$_CYAN" "$_RESET"
  assert_can_or_skip create gateways.gateway.networking.k8s.io --all-namespaces "--as=$SA"
  assert_can_or_skip patch gateways.gateway.networking.k8s.io --all-namespaces "--as=$SA"
  assert_can_or_skip create httproutes.gateway.networking.k8s.io --all-namespaces "--as=$SA"
  assert_can_or_skip create grpcroutes.gateway.networking.k8s.io --all-namespaces "--as=$SA"
  assert_can_or_skip create referencegrants.gateway.networking.k8s.io --all-namespaces "--as=$SA"

  report_guard "network-architecture-mutation"
}
