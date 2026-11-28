#!/usr/bin/env bash
# guards/admission-policy.sh — RBAC pre-flight for kubernetes-live-admission-policy-guard-agent
# Source lib/common.sh before running this file.

run_guard_admission_policy() {
  begin_guard "admission-policy"

  local SA="system:serviceaccount:vanguard-system:vanguard-admission-policy-guard"

  # -------------------------------------------------------------------------
  # Universal must-not-be-yes
  # -------------------------------------------------------------------------
  run_universal_must_not "$SA"

  # -------------------------------------------------------------------------
  # Domain-specific must-not-be-yes
  # (from agents/kubernetes/kubernetes-live-admission-policy-guard-agent/references/rbac-pre-flight.md)
  # -------------------------------------------------------------------------
  printf '%b  -- domain-specific must-not-be-yes --%b\n' "$_CYAN" "$_RESET"

  # Webhook configurations — operator install only
  # Note: create mutatingwebhookconfigurations already covered by universal block;
  # these additional verbs are domain-specific.
  assert_cannot patch mutatingwebhookconfigurations.admissionregistration.k8s.io "--as=$SA"
  assert_cannot patch validatingwebhookconfigurations.admissionregistration.k8s.io "--as=$SA"

  # Delete on policies — rollback via apply -f baseline
  # Kyverno CRDs may not exist in vanilla kind
  assert_cannot_or_skip delete clusterpolicies.kyverno.io "--as=$SA"
  assert_cannot delete validatingadmissionpolicies.admissionregistration.k8s.io "--as=$SA"
  assert_cannot_or_skip delete validatingpolicies.policies.kyverno.io "--as=$SA"
  assert_cannot_or_skip delete policyexceptions.policies.kyverno.io --all-namespaces "--as=$SA"

  # Kyverno CEL policies whose controller writes or deletes other objects
  assert_cannot_or_skip create mutatingpolicies.policies.kyverno.io "--as=$SA"
  assert_cannot_or_skip create generatingpolicies.policies.kyverno.io "--as=$SA"
  assert_cannot_or_skip create deletingpolicies.policies.kyverno.io "--as=$SA"
  assert_cannot_or_skip create namespacedgeneratingpolicies.policies.kyverno.io --all-namespaces "--as=$SA"

  # Policy reports are evidence — read only
  assert_cannot_or_skip create policyreports.wgpolicyk8s.io --all-namespaces "--as=$SA"
  assert_cannot_or_skip delete clusterpolicyreports.wgpolicyk8s.io "--as=$SA"

  # Kyverno control plane (namespace always exists once kyverno is installed;
  # patch deployments/configmaps check works even if kyverno is absent — returns no)
  assert_cannot patch deployments -n kyverno "--as=$SA"
  assert_cannot patch configmaps -n kyverno "--as=$SA"

  # -------------------------------------------------------------------------
  # Domain-specific must-be-yes
  # -------------------------------------------------------------------------
  printf '%b  -- domain-specific must-be-yes --%b\n' "$_CYAN" "$_RESET"

  # Kyverno CRDs — skip not fail if absent
  assert_can_or_skip create clusterpolicies.kyverno.io "--as=$SA"
  assert_can_or_skip patch clusterpolicies.kyverno.io "--as=$SA"
  assert_can_or_skip create policies.kyverno.io --all-namespaces "--as=$SA"
  assert_can_or_skip create policyexceptions.kyverno.io --all-namespaces "--as=$SA"
  assert_can_or_skip list clusterpolicies.kyverno.io "--as=$SA"

  # Kyverno CEL policies (policies.kyverno.io, Kyverno 1.20+)
  assert_can_or_skip create validatingpolicies.policies.kyverno.io "--as=$SA"
  assert_can_or_skip patch validatingpolicies.policies.kyverno.io "--as=$SA"
  assert_can_or_skip patch imagevalidatingpolicies.policies.kyverno.io "--as=$SA"
  assert_can_or_skip create namespacedvalidatingpolicies.policies.kyverno.io --all-namespaces "--as=$SA"
  assert_can_or_skip create policyexceptions.policies.kyverno.io --all-namespaces "--as=$SA"
  assert_can_or_skip list generatingpolicies.policies.kyverno.io "--as=$SA"

  # Policy reports (wgpolicyk8s.io/v1alpha2 at Kyverno v1.20.0)
  assert_can_or_skip list policyreports.wgpolicyk8s.io --all-namespaces "--as=$SA"
  assert_can_or_skip list clusterpolicyreports.wgpolicyk8s.io "--as=$SA"

  # ValidatingAdmissionPolicy is GA in 1.30+, available in 1.28+ as beta
  assert_can create validatingadmissionpolicies.admissionregistration.k8s.io "--as=$SA"

  report_guard "admission-policy"
}
