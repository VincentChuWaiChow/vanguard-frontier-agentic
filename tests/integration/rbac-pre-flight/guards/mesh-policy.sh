#!/usr/bin/env bash
# guards/mesh-policy.sh — RBAC pre-flight for kubernetes-live-mesh-policy-guard-agent
# Source lib/common.sh before running this file.

run_guard_mesh_policy() {
  begin_guard "mesh-policy"

  local SA="system:serviceaccount:vanguard-system:vanguard-mesh-policy-guard"

  # -------------------------------------------------------------------------
  # Universal must-not-be-yes
  # -------------------------------------------------------------------------
  run_universal_must_not "$SA"

  # -------------------------------------------------------------------------
  # Domain-specific must-not-be-yes
  # (from agents/kubernetes/kubernetes-live-mesh-policy-guard-agent/references/rbac-pre-flight.md)
  # -------------------------------------------------------------------------
  printf '%b  -- domain-specific must-not-be-yes --%b\n' "$_CYAN" "$_RESET"

  # Delete on policies — rollback is via apply -f baseline, not delete
  # Istio CRDs may not exist in vanilla kind
  assert_cannot_or_skip delete authorizationpolicies.security.istio.io --all-namespaces "--as=$SA"
  assert_cannot_or_skip delete peerauthentications.security.istio.io --all-namespaces "--as=$SA"
  assert_cannot_or_skip delete requestauthentications.security.istio.io --all-namespaces "--as=$SA"
  assert_cannot_or_skip delete virtualservices.networking.istio.io --all-namespaces "--as=$SA"
  assert_cannot_or_skip delete destinationrules.networking.istio.io --all-namespaces "--as=$SA"

  # istio-system control plane — core resources always present even without Istio CRDs
  assert_cannot patch deployments -n istio-system "--as=$SA"
  assert_cannot patch configmaps -n istio-system "--as=$SA"

  # Istio Gateway resources — delegated to network-architecture guard
  assert_cannot_or_skip create gateways.networking.istio.io --all-namespaces "--as=$SA"
  assert_cannot_or_skip patch gateways.networking.istio.io --all-namespaces "--as=$SA"

  # -------------------------------------------------------------------------
  # Domain-specific must-be-yes
  # Istio CRDs — skip not fail if absent in vanilla kind
  # -------------------------------------------------------------------------
  printf '%b  -- domain-specific must-be-yes (Istio CRDs, SKIP if absent) --%b\n' "$_CYAN" "$_RESET"

  assert_can_or_skip create authorizationpolicies.security.istio.io --all-namespaces "--as=$SA"
  assert_can_or_skip patch authorizationpolicies.security.istio.io --all-namespaces "--as=$SA"
  assert_can_or_skip create peerauthentications.security.istio.io --all-namespaces "--as=$SA"
  assert_can_or_skip patch peerauthentications.security.istio.io --all-namespaces "--as=$SA"
  assert_can_or_skip create requestauthentications.security.istio.io --all-namespaces "--as=$SA"
  assert_can_or_skip patch requestauthentications.security.istio.io --all-namespaces "--as=$SA"
  assert_can_or_skip create virtualservices.networking.istio.io --all-namespaces "--as=$SA"
  assert_can_or_skip patch virtualservices.networking.istio.io --all-namespaces "--as=$SA"
  assert_can_or_skip create destinationrules.networking.istio.io --all-namespaces "--as=$SA"
  assert_can_or_skip patch destinationrules.networking.istio.io --all-namespaces "--as=$SA"

  report_guard "mesh-policy"
}
